import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { analyzeModel, prepareModelRoot } from "./modelMath";
import { ComponentService } from "./componentService";
import { ExplosionService } from "./explosionService";
import type { ModelFormat, ModelMetadata, ModelNode } from "./types";
import type { ModelCoreState } from "./types";

export interface LoadedEngineeringModel {
  identity: string;
  byteSize: number;
  root: THREE.Group;
  metadata: ModelMetadata;
  nodes: ModelNode[];
  components: ComponentService;
  explosion: ExplosionService;
}

const extensionPattern = /\.([^.]+)$/;

export type ModelCoreEvent = "load_started" | "load_succeeded" | "load_failed" | "load_cancelled" | "unloaded";

export function resolveModelCoreState(event: ModelCoreEvent, hasLoadedModel: boolean): ModelCoreState {
  if (event === "load_started") return "loading";
  if (event === "load_succeeded") return "ready";
  if (event === "load_failed") return "error";
  if (event === "load_cancelled") return hasLoadedModel ? "ready" : "empty";
  return "empty";
}

export function modelFormatFromName(name: string): ModelFormat {
  const extension = extensionPattern.exec(name.trim())?.[1]?.toLowerCase();
  if (extension === "glb") return "GLB";
  if (extension === "gltf") return "GLTF";
  throw new Error("Formato inválido. Selecione um arquivo GLB ou GLTF.");
}

export function externalGltfResources(json: unknown): string[] {
  if (!json || typeof json !== "object") return [];
  const document = json as { buffers?: Array<{ uri?: unknown }>; images?: Array<{ uri?: unknown }> };
  return [...(document.buffers ?? []), ...(document.images ?? [])]
    .map((resource) => resource.uri)
    .filter((uri): uri is string => typeof uri === "string" && !uri.startsWith("data:"));
}

export function disposeObjectResources(root: THREE.Object3D): void {
  const materials = new Set<THREE.Material>();
  const textures = new Set<THREE.Texture>();
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    object.geometry?.dispose();
    const meshMaterials = Array.isArray(object.material) ? object.material : [object.material];
    meshMaterials.forEach((material) => materials.add(material));
  });
  materials.forEach((material) => {
    for (const value of Object.values(material)) if (value instanceof THREE.Texture) textures.add(value);
    material.dispose();
  });
  textures.forEach((texture) => texture.dispose());
}

async function parseModel(bytes: Uint8Array, format: ModelFormat): Promise<THREE.Object3D> {
  const loader = new GLTFLoader();
  if (format === "GLB") {
    const buffer = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
    return (await loader.parseAsync(buffer, "")).scene;
  }
  const source = new TextDecoder().decode(bytes);
  let document: unknown;
  try { document = JSON.parse(source); }
  catch { throw new Error("O arquivo GLTF não contém JSON válido."); }
  if (externalGltfResources(document).length) {
    throw new Error("Este GLTF depende de arquivos .bin ou texturas externas. Use GLB ou GLTF com recursos incorporados.");
  }
  return (await loader.parseAsync(source, "")).scene;
}

export async function loadEngineeringModelBytes(bytes: Uint8Array, fileName: string): Promise<LoadedEngineeringModel> {
  const format = modelFormatFromName(fileName);
  const source = await parseModel(bytes, format);
  const analysis = analyzeModel(source);
  if (!analysis.metadata.meshes || !analysis.metadata.dimensions.largest) {
    disposeObjectResources(source);
    throw new Error("O modelo não contém geometria 3D utilizável.");
  }
  const components = new ComponentService(source);
  const explosion = new ExplosionService(components);
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes.slice().buffer);
  const identity = [...new Uint8Array(digest)].map((value) => value.toString(16).padStart(2, "0")).join("");
  return {
    identity,
    byteSize: bytes.byteLength,
    root: prepareModelRoot(source, analysis.center, analysis.normalizationScale),
    metadata: { name: fileName, format, ...analysis.metadata },
    nodes: analysis.nodes,
    components,
    explosion,
  };
}

export class ModelService {
  async selectAndLoad(): Promise<LoadedEngineeringModel | null> {
    const selection = await open({
      multiple: false,
      directory: false,
      title: "Carregar modelo no Engineering Core",
      filters: [{ name: "Modelo 3D", extensions: ["glb", "gltf"] }],
    });
    if (typeof selection !== "string") return null;
    return this.loadFromPath(selection);
  }

  async loadFromPath(path: string): Promise<LoadedEngineeringModel> {
    const fileName = path.split(/[\\/]/).pop() || "MODEL";
    const payload = await invoke<ArrayBuffer | Uint8Array>("read_engineering_model", { path });
    const bytes = payload instanceof Uint8Array ? payload : new Uint8Array(payload);
    return loadEngineeringModelBytes(bytes, fileName);
  }

  unload(model: LoadedEngineeringModel | null): void {
    if (model) disposeObjectResources(model.root);
  }
}
