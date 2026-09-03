import type { AssemblyIntelligenceSnapshot, ComponentRelationship, ComponentRelationshipType, ComponentSemantic, EngineeringCalibration, EngineeringCalibrationInput, EngineeringSubsystem, ModelComponent, ModelFormat } from "../engineering/types";
import { invokeDatabase } from "./tauri";

export const engineeringRepository = {
  getCalibration: () => invokeDatabase<EngineeringCalibration>("get_engineering_calibration"),
  updateCalibration: (input: EngineeringCalibrationInput) => invokeDatabase<EngineeringCalibration>("update_engineering_calibration", { input }),
  resetCalibration: () => invokeDatabase<EngineeringCalibration>("reset_engineering_calibration"),
  registerModel: (input: { modelIdentity: string; fileName: string; format: ModelFormat; byteSize: number; components: Array<{ componentIdentity: string; originalName: string; structuralPath: string; componentType: string; selectable: boolean }> }) => invokeDatabase<AssemblyIntelligenceSnapshot>("register_engineering_model", { input }),
  getAssemblyIntelligence: (modelIdentity: string) => invokeDatabase<AssemblyIntelligenceSnapshot>("get_assembly_intelligence", { modelIdentity }),
  saveSemantic: (input: Omit<ComponentSemantic, "createdAt" | "updatedAt" | "originalName" | "structuralPath" | "componentType">) => invokeDatabase<ComponentSemantic>("save_component_semantic", { input }),
  saveSubsystem: (input: Omit<EngineeringSubsystem, "createdAt" | "updatedAt">) => invokeDatabase<EngineeringSubsystem>("save_engineering_subsystem", { input }),
  deleteSubsystem: (modelIdentity: string, id: string) => invokeDatabase<void>("delete_engineering_subsystem", { modelIdentity, id }),
  saveRelationship: (input: { id: string; modelIdentity: string; sourceComponentIdentity: string; targetComponentIdentity: string; relationshipType: ComponentRelationshipType; description: string }) => invokeDatabase<ComponentRelationship>("save_component_relationship", { input }),
  deleteRelationship: (modelIdentity: string, id: string) => invokeDatabase<void>("delete_component_relationship", { modelIdentity, id }),
};

export function componentRegistryInput(components: ModelComponent[]) {
  return components.map((component) => ({ componentIdentity: component.persistentIdentity, originalName: component.originalName, structuralPath: component.structuralPath, componentType: component.type, selectable: component.selectable }));
}
