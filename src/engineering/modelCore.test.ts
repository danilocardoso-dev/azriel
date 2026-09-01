import { describe, expect, it, vi } from "vitest";
import * as THREE from "three";
import { analyzeModel, buildModelHierarchy, dimensionsFromBox, normalizationScale, prepareModelRoot } from "./modelMath";
import { disposeObjectResources, externalGltfResources, loadEngineeringModelBytes, modelFormatFromName, resolveModelCoreState } from "./modelService";

function testAssembly() {
  const root = new THREE.Group();
  root.name = "MotorAssembly";
  const housing = new THREE.Group();
  housing.name = "Housing";
  const material = new THREE.MeshStandardMaterial();
  const mesh = new THREE.Mesh(new THREE.BoxGeometry(2, 4, 6), material);
  mesh.name = "Body";
  housing.add(mesh);
  root.add(housing);
  return { root, mesh, material };
}

describe("Engineering Core Model Core", () => {
  it("aceita GLB e GLTF sem diferenciar maiúsculas", () => {
    expect(modelFormatFromName("motor.glb")).toBe("GLB");
    expect(modelFormatFromName("assembly.GLTF")).toBe("GLTF");
  });

  it("rejeita extensões fora do Model Core", () => {
    expect(() => modelFormatFromName("motor.obj")).toThrow(/GLB ou GLTF/);
    expect(() => modelFormatFromName("sem-extensao")).toThrow(/Formato inválido/);
  });

  it("calcula bounding box, centro, dimensões e escala normalizada", () => {
    const { root } = testAssembly();
    const box = new THREE.Box3().setFromObject(root);
    expect(dimensionsFromBox(box)).toEqual({ x: 2, y: 4, z: 6, largest: 6 });
    expect(normalizationScale(dimensionsFromBox(box))).toBeCloseTo(0.4);
    const analysis = analyzeModel(root);
    expect(analysis.center.toArray()).toEqual([0, 0, 0]);
    expect(analysis.metadata.dimensions.largest).toBe(6);
    expect(analysis.metadata.meshes).toBe(1);
    expect(analysis.metadata.materials).toBe(1);
    expect(analysis.metadata.triangles).toBe(12);
  });

  it("normaliza por wrapper sem alterar a transformação original", () => {
    const { root } = testAssembly();
    root.position.set(10, 2, -4);
    const analysis = analyzeModel(root);
    const originalPosition = root.position.clone();
    const prepared = prepareModelRoot(root, analysis.center, analysis.normalizationScale);
    const normalizedSize = new THREE.Box3().setFromObject(prepared).getSize(new THREE.Vector3());
    expect(root.position).toEqual(originalPosition);
    expect(Math.max(normalizedSize.x, normalizedSize.y, normalizedSize.z)).toBeCloseTo(2.4);
  });

  it("gera hierarquia estável com nomes técnicos para nós anônimos", () => {
    const { root } = testAssembly();
    root.add(new THREE.Group());
    const first = buildModelHierarchy(root);
    const second = buildModelHierarchy(root);
    expect(first).toEqual(second);
    expect(first[0].name).toBe("MotorAssembly");
    expect(first.some((node) => node.name.startsWith("GROUP-"))).toBe(true);
    expect(first[0].children).toHaveLength(2);
  });

  it("identifica dependências externas de GLTF e aceita data URI", () => {
    expect(externalGltfResources({ buffers: [{ uri: "model.bin" }], images: [{ uri: "data:image/png;base64,AA==" }] })).toEqual(["model.bin"]);
    expect(externalGltfResources({ buffers: [{ uri: "data:application/octet-stream;base64,AA==" }] })).toEqual([]);
  });

  it("carrega um fixture GLTF incorporado e produz metadata", async () => {
    vi.stubGlobal("ProgressEvent", class {
      readonly lengthComputable: boolean;
      readonly loaded: number;
      readonly total: number;
      constructor(readonly type: string, init: ProgressEventInit = {}) {
        this.lengthComputable = init.lengthComputable ?? false;
        this.loaded = init.loaded ?? 0;
        this.total = init.total ?? 0;
      }
    });
    const positions = new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]);
    const bytes = new Uint8Array(positions.buffer);
    const binary = btoa(String.fromCharCode(...bytes));
    const fixture = JSON.stringify({
      asset: { version: "2.0" },
      scene: 0,
      scenes: [{ nodes: [0] }],
      nodes: [{ name: "Triangle", mesh: 0 }],
      meshes: [{ primitives: [{ attributes: { POSITION: 0 } }] }],
      buffers: [{ byteLength: bytes.byteLength, uri: `data:application/octet-stream;base64,${binary}` }],
      bufferViews: [{ buffer: 0, byteOffset: 0, byteLength: bytes.byteLength }],
      accessors: [{ bufferView: 0, componentType: 5126, count: 3, type: "VEC3", min: [0, 0, 0], max: [1, 1, 0] }],
    });
    try {
      const model = await loadEngineeringModelBytes(new TextEncoder().encode(fixture), "fixture.gltf");
      expect(model.metadata.name).toBe("fixture.gltf");
      expect(model.metadata.meshes).toBe(1);
      expect(model.metadata.triangles).toBe(1);
      expect(model.nodes.some((node) => node.name === "Triangle")).toBe(true);
      disposeObjectResources(model.root);
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it("libera geometria, material e textura uma única vez", () => {
    const { root, mesh, material } = testAssembly();
    const texture = new THREE.Texture();
    material.map = texture;
    const geometryDispose = vi.spyOn(mesh.geometry, "dispose");
    const materialDispose = vi.spyOn(material, "dispose");
    const textureDispose = vi.spyOn(texture, "dispose");
    disposeObjectResources(root);
    expect(geometryDispose).toHaveBeenCalledOnce();
    expect(materialDispose).toHaveBeenCalledOnce();
    expect(textureDispose).toHaveBeenCalledOnce();
  });

  it("mantém transições explícitas dos estados do Model Core", () => {
    expect(resolveModelCoreState("load_started", false)).toBe("loading");
    expect(resolveModelCoreState("load_succeeded", true)).toBe("ready");
    expect(resolveModelCoreState("load_failed", false)).toBe("error");
    expect(resolveModelCoreState("load_cancelled", true)).toBe("ready");
    expect(resolveModelCoreState("load_cancelled", false)).toBe("empty");
    expect(resolveModelCoreState("unloaded", true)).toBe("empty");
  });
});
