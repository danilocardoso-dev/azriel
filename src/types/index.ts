export type ModuleId = "command" | "daily" | "projects" | "knowledge" | "stark" | "education" | "research" | "system" | "settings";
export type AzrielState = "idle" | "processing" | "alert" | "offline";
export type ProjectStatus = "active" | "research" | "paused" | "planned" | "completed";
export type Priority = "critical" | "high" | "medium" | "low";
export type EducationKind = "graduation" | "postgraduate" | "masters" | "doctorate" | "course" | "certification";
export type EducationStatus = "completed" | "in_progress" | "planned";

export interface Project {
  id: string; name: string; category: string; description: string; status: ProjectStatus;
  knowledgeAreaIds: string[]; objective: string; progress: number; nextStep: string;
  createdAt: string; updatedAt: string;
}
export type ProjectInput = Omit<Project, "createdAt" | "updatedAt">;

export interface KnowledgeArea {
  id: string; name: string; category: string; description: string; coverage: number; depth: number;
  priority: Priority; projectIds: string[]; createdAt: string; updatedAt: string;
}
export type KnowledgeInput = Omit<KnowledgeArea, "projectIds" | "createdAt" | "updatedAt">;

export interface KnowledgeHistory { id: number; knowledgeId: string; coverage: number; depth: number; recordedAt: string; reason: string }
export interface MetricsInput { knowledgeId: string; coverage: number; depth: number; reason: string }

export interface EducationItem {
  id: string; name: string; kind: EducationKind; institution: string; status: EducationStatus;
  startDate: string | null; expectedEndDate: string | null; completedAt: string | null;
  description: string; period: string; domains: string[]; createdAt: string; updatedAt: string;
}
export type EducationInput = Omit<EducationItem, "createdAt" | "updatedAt">;
export interface DatabaseInfo { path: string; schemaVersion: number; integrationValue: number }

export type TaskStatus = "inbox" | "pending" | "in_progress" | "completed" | "cancelled";
export type TaskPriority = "low" | "medium" | "high" | "critical";
export type DailyView = "today" | "inbox" | "upcoming" | "completed" | "notes";
export interface Task {
  id: string; title: string; description: string; status: TaskStatus; priority: TaskPriority;
  dueDate: string | null; projectId: string | null; knowledgeAreaId: string | null;
  createdAt: string; updatedAt: string; completedAt: string | null;
}
export type TaskInput = Omit<Task, "createdAt" | "updatedAt" | "completedAt">;
export type NoteStatus = "active" | "archived";
export interface Note {
  id: string; title: string | null; content: string; status: NoteStatus;
  projectId: string | null; knowledgeAreaId: string | null; createdAt: string; updatedAt: string;
}
export type NoteInput = Omit<Note, "createdAt" | "updatedAt">;
export interface DailyCounters { pending: number; today: number; overdue: number; priority: number; notes: number }

export interface ResearchItem {
  id: string; title: string; domain: string; objective: string;
  status: "project" | "study" | "queue"; impact: string;
}
