export type ModuleId =
  | "command"
  | "projects"
  | "knowledge"
  | "stark"
  | "education"
  | "research"
  | "system"
  | "settings";

export type AzrielState = "idle" | "processing" | "alert" | "offline";
export type ProjectStatus = "active" | "research" | "paused" | "planned";
export type Priority = "critical" | "high" | "medium" | "low";

export interface Project {
  id: string;
  name: string;
  category: string;
  description: string;
  status: ProjectStatus;
  knowledgeAreas: string[];
  objective: string;
  progress: number;
  nextStep: string;
}

export interface KnowledgeArea {
  id: string;
  name: string;
  group: string;
  coverage: number;
  depth: number;
  priority: Priority;
  projectIds: string[];
}

export interface EducationItem {
  id: string;
  name: string;
  kind: "base" | "postgraduate" | "graduation" | "future";
  period: string;
  status: "completed" | "in-progress" | "planned";
  description: string;
  domains: string[];
}

export interface ResearchItem {
  id: string;
  title: string;
  domain: string;
  objective: string;
  status: "project" | "study" | "queue";
  impact: string;
}
