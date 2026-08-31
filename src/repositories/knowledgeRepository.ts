import type { KnowledgeArea, KnowledgeHistory, KnowledgeInput, MetricsInput } from "../types";
import { invokeDatabase } from "./tauri";
export const knowledgeRepository = {
  list: () => invokeDatabase<KnowledgeArea[]>("list_knowledge"),
  get: (id: string) => invokeDatabase<KnowledgeArea | null>("get_knowledge", { id }),
  save: (input: KnowledgeInput) => invokeDatabase<KnowledgeArea[]>("save_knowledge", { input }),
  remove: (id: string) => invokeDatabase<KnowledgeArea[]>("delete_knowledge", { id }),
  updateMetrics: (input: MetricsInput) => invokeDatabase<KnowledgeArea>("update_knowledge_metrics", { input }),
  history: (knowledgeId: string) => invokeDatabase<KnowledgeHistory[]>("list_knowledge_history", { knowledgeId }),
};
