import { invokeDatabase } from "./tauri";
import type { ImportMarketDatasetInput, MarketAgentDefinition, MarketAiDecisionLog, MarketAiRuntimeMetric, MarketAiStatus, MarketDataset, MarketExperimentInput, MarketExperimentResult, MarketExperimentSummary, MarketRiskProfile, MarketValidationInput, MarketValidationResult, MarketValidationSummary, UpdateMarketAiConfigInput } from "../types";

export const marketLabRepository = {
  listDatasets: () => invokeDatabase<MarketDataset[]>("list_market_datasets"),
  importDataset: (input: ImportMarketDatasetInput) => invokeDatabase<MarketDataset>("import_market_dataset", { input }),
  listAgents: () => invokeDatabase<MarketAgentDefinition[]>("list_market_agents"),
  listRiskProfiles: () => invokeDatabase<MarketRiskProfile[]>("list_market_risk_profiles"),
  listExperiments: () => invokeDatabase<MarketExperimentSummary[]>("list_market_experiments"),
  getExperiment: (id: string) => invokeDatabase<MarketExperimentResult>("get_market_experiment", { id }),
  runExperiment: (input: MarketExperimentInput) => invokeDatabase<MarketExperimentResult>("run_market_experiment", { input }),
  rerunExperiment: (id: string) => invokeDatabase<MarketExperimentResult>("rerun_market_experiment", { id }),
  activateKillSwitch: () => invokeDatabase<boolean>("activate_market_kill_switch"),
  runValidation: (input: MarketValidationInput) => invokeDatabase<MarketValidationResult>("run_market_validation", { input }),
  listValidations: () => invokeDatabase<MarketValidationSummary[]>("list_market_validations"),
  getValidation: (id: string) => invokeDatabase<MarketValidationResult>("get_market_validation", { id }),
  getAiStatus: () => invokeDatabase<MarketAiStatus>("get_market_ai_status"),
  updateAiConfig: (input: UpdateMarketAiConfigInput) => invokeDatabase<MarketAiStatus>("update_market_ai_config", { input }),
  listAiRuntime: (experimentId: string) => invokeDatabase<MarketAiRuntimeMetric[]>("list_market_ai_runtime", { experimentId }),
  listValidationAiRuntime: (validationId: string) => invokeDatabase<MarketAiRuntimeMetric[]>("list_market_validation_ai_runtime", { validationId }),
  listAiDecisions: (experimentId: string) => invokeDatabase<MarketAiDecisionLog[]>("list_market_ai_decisions", { experimentId }),
};
