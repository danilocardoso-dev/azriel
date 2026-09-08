import { invokeDatabase } from "./tauri";
import type { ImportMarketDatasetInput, MarketAgentDefinition, MarketDataset, MarketExperimentInput, MarketExperimentResult, MarketExperimentSummary, MarketRiskProfile, MarketValidationInput, MarketValidationResult, MarketValidationSummary } from "../types";

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
};
