import { createContext } from "react";
import type { DatabaseInfo, EducationInput, EducationItem, KnowledgeArea, KnowledgeHistory, KnowledgeInput, MetricsInput, Project, ProjectInput } from "../types";

export interface DataContextValue {
  projects: Project[]; knowledgeAreas: KnowledgeArea[]; education: EducationItem[];
  databaseInfo: DatabaseInfo | null; loading: boolean; error: string | null; reload: () => Promise<void>;
  updateMetrics: (input: MetricsInput) => Promise<KnowledgeArea>;
  loadHistory: (knowledgeId: string) => Promise<KnowledgeHistory[]>;
  saveKnowledge: (input: KnowledgeInput) => Promise<void>; deleteKnowledge: (id: string) => Promise<void>;
  saveProject: (input: ProjectInput) => Promise<void>; deleteProject: (id: string) => Promise<void>;
  saveEducation: (input: EducationInput) => Promise<void>; deleteEducation: (id: string) => Promise<void>;
}

export const DataContext = createContext<DataContextValue | null>(null);
