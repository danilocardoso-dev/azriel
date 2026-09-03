export type ModuleId = "command" | "engineering" | "ai" | "daily" | "projects" | "stark" | "education" | "system" | "automation" | "settings";
export type AzrielState = "idle" | "processing" | "tool" | "executing" | "engineering" | "routine" | "alert" | "offline";
export type AIToolPermission = "read" | "visual_action" | "safe_write" | "confirm_write";
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
  priority: Priority; nodeType: KnowledgeNodeType; parentId: string | null;
  projectIds: string[]; createdAt: string; updatedAt: string;
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
export type DailyView = "pending" | "today" | "overdue" | "priority" | "inbox" | "upcoming" | "completed" | "notes" | "archived_notes";
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
export interface DailyCounters { pending: number; today: number; overdue: number; priority: number; notes: number; completed: number }

export interface AISettings {
  provider: "ollama"; endpoint: string; model: string; contextMessageLimit: number;
  timeoutSeconds: number; updatedAt: string;
}
export type AISettingsInput = Pick<AISettings, "endpoint" | "model" | "contextMessageLimit" | "timeoutSeconds">;
export interface Conversation { id: string; title: string; createdAt: string; updatedAt: string }
export type ConversationRole = "user" | "assistant" | "system";
export interface ConversationMessage { id: string; conversationId: string; role: ConversationRole; content: string; createdAt: string }
export interface ConversationMessageInput { id: string; conversationId: string; role: ConversationRole; content: string }
export interface ProviderMessage { role: ConversationRole; content: string }
export type AIGenerationProfile = "standard" | "repetition-retry";
export interface AIRequest { model: string; messages: ProviderMessage[]; timeoutSeconds: number; generationProfile?: AIGenerationProfile }
export interface AIResponse { content: string; model: string; truncated: boolean }
export interface OllamaStatus { available: boolean; models: string[]; error: string | null }
export interface SystemDetails { osName: string | null; osVersion: string | null; kernelVersion: string | null; architecture: string; hostname: string | null; logicalCores: number; physicalCores: number | null; uptimeSeconds: number }
export interface CpuSnapshot { usagePercent: number; cores: number[] }
export interface MemorySnapshot { totalBytes: number; usedBytes: number; availableBytes: number; swapTotalBytes: number; swapUsedBytes: number }
export interface StorageSnapshot { name: string; mountPoint: string; fileSystem: string; totalBytes: number; availableBytes: number; removable: boolean }
export interface NetworkSnapshot { interfaceName: string; receivedBytes: number; transmittedBytes: number; receivedBytesTotal: number; transmittedBytesTotal: number }
export interface ProcessSnapshot { pid: number; name: string; cpuPercent: number; memoryBytes: number }
export interface SystemSnapshot { collectedAt: number; details: SystemDetails; cpu: CpuSnapshot; memory: MemorySnapshot; storage: StorageSnapshot[]; network: NetworkSnapshot[]; errors: string[] }
export interface Workspace { id: string; name: string; path: string; projectId: string | null; applicationId: string | null; enabled: boolean; createdAt: string; updatedAt: string }
export type WorkspaceInput = Omit<Workspace, "createdAt" | "updatedAt">;
export interface GitCommit { hash: string; shortHash: string; subject: string; date: string; author: string }
export interface GitStatus { available: boolean; repository: boolean; branch: string | null; clean: boolean; modified: string[]; added: string[]; removed: string[]; untracked: string[]; lastCommit: GitCommit | null; recentCommits: GitCommit[]; error: string | null }
export interface WorkspaceStatus { workspace: Workspace; pathAvailable: boolean; entryCount: number | null; git: GitStatus | null; error: string | null }
export type ActionPermission = "read" | "safe_write" | "confirm_write" | "blocked";
export type ActionSource = "user" | "ai" | "ui";
export type AutomationState = "offline" | "safe" | "validating" | "executing" | "waiting_confirmation" | "completed" | "cancelled" | "failed" | "blocked" | "error";
export interface RegisteredAction { id: AutomationActionId; name: string; description: string; permission: ActionPermission; targetType: "application" | "workspace" | "project" | "url" }
export type AutomationActionId = "open_application" | "open_workspace" | "open_project" | "reveal_workspace" | "open_registered_url";
export interface Application { id: string; name: string; path: string; enabled: boolean; createdAt: string; updatedAt: string }
export type ApplicationInput = Omit<Application, "createdAt" | "updatedAt">;
export interface RegisteredUrl { id: string; name: string; url: string; enabled: boolean; createdAt: string; updatedAt: string }
export type RegisteredUrlInput = Omit<RegisteredUrl, "createdAt" | "updatedAt">;
export interface ConfirmationRequest { actionId: string; targetName: string; description: string; impact: string }
export interface ActionRequest { actionId: AutomationActionId; source: ActionSource; targetId?: string }
export interface ActionResult { success: boolean; message: string; errorCode: string | null; actionId: string; targetName: string | null; historyId: number; confirmation: ConfirmationRequest | null }
export interface ActionHistory { id: number; actionId: string; source: ActionSource; targetType: string | null; targetId: string | null; targetName: string | null; permission: ActionPermission; confirmationRequired: boolean; confirmed: boolean; success: boolean | null; error: string | null; createdAt: string; completedAt: string | null }
export interface RoutineStep { id: string; order: number; actionId: AutomationActionId; targetType: RegisteredAction["targetType"]; targetId: string; delayMs: number; enabled: boolean }
export interface Routine { id: string; name: string; description: string; enabled: boolean; confirmationRequired: boolean; revision: number; steps: RoutineStep[]; createdAt: string; updatedAt: string }
export type RoutineInput = Omit<Routine, "revision" | "createdAt" | "updatedAt">;
export type RoutineStatus = "waiting_confirmation" | "executing" | "completed" | "cancelled" | "failed";
export interface RoutineHistory { id: number; routineId: string | null; routineName: string; routineRevision: number; source: ActionSource; status: RoutineStatus; confirmationRequired: boolean; confirmed: boolean; totalSteps: number; completedSteps: number; failedStep: number | null; error: string | null; startedAt: string; completedAt: string | null }
export interface RoutineActionSummary { order: number; actionId: AutomationActionId; actionName: string; targetName: string; delayMs: number }
export interface RoutineConfirmation { historyId: number; routineId: string; routineName: string; revision: number; actions: RoutineActionSummary[] }
export interface RoutineExecutionResult { success: boolean; status: RoutineStatus; routineId: string; routineName: string; historyId: number; completedSteps: number; failedStep: number | null; error: string | null; confirmation: RoutineConfirmation | null }
export interface RunRoutineRequest { routineId: string; source: ActionSource }
export type AIToolName =
  | "get_today_tasks" | "get_overdue_tasks" | "get_upcoming_tasks" | "get_recent_notes" | "get_daily_operations_summary"
  | "list_projects" | "get_project" | "get_project_tasks" | "get_project_knowledge"
  | "list_knowledge_areas" | "get_knowledge_area" | "get_knowledge_gaps" | "get_stark_map" | "get_knowledge_history"
  | "list_study_roadmaps" | "get_study_roadmap" | "list_research_items" | "get_knowledge_origin"
  | "get_education" | "get_current_education" | "get_planned_education"
  | "get_system_status" | "get_cpu_status" | "get_memory_status" | "get_storage_status" | "get_network_status" | "get_process_summary"
  | "list_workspaces" | "get_workspace_status" | "get_git_status" | "get_recent_commits" | "get_ollama_status"
  | "get_azriel_status" | "get_azriel_version"
  | "get_loaded_model" | "get_model_summary" | "list_components" | "find_component" | "get_component_details" | "get_selected_component" | "get_explosion_state"
  | "get_component_semantics" | "get_subsystems" | "get_subsystem_components" | "get_component_relationships" | "get_unclassified_components" | "get_semantic_coverage" | "get_assembly_graph_summary"
  | "select_component" | "focus_component" | "isolate_component" | "show_all_components" | "hide_component" | "show_component"
  | "set_explosion_factor" | "explode_all" | "explode_component" | "reassemble" | "reset_model_view"
  | "list_routines" | "run_routine" | AutomationActionId;
export interface AIToolInput { query: string; term?: string; entityId?: string; workspaceId?: string; factor?: number; delta?: number }
export interface AIToolResult { name: AIToolName; domain: string; data: unknown; empty: boolean }
export interface RoutedIntent { intent: string; scope: "azriel" | "general"; tools: AIToolName[]; term?: string; factor?: number; delta?: number }

export type KnowledgeNodeType = "area" | "discipline" | "topic" | "competency";
export interface KnowledgeBaseline { knowledgeAreaId: string; coverage: number; depth: number; recordedAt: string }
export type KnowledgeEventSource = "baseline" | "roadmap" | "project" | "research" | "manual" | "education";
export interface KnowledgeEvent { id: string; knowledgeNodeId: string; sourceType: KnowledgeEventSource; sourceId: string | null; eventType: string; coverageDelta: number; depthDelta: number; integrationDelta: number; description: string; createdAt: string }

export type RoadmapStatus = "planned" | "active" | "paused" | "completed";
export type RoadmapTopicState = "NOT_STARTED" | "EXPOSED" | "UNDERSTOOD" | "PRACTICED" | "APPLIED" | "MASTERED";
export type RoadmapActivityType = "READING" | "LESSON" | "QUIZ" | "EXERCISE" | "SIMULATION" | "EXPERIMENT" | "PROJECT" | "DOCUMENTATION" | "RESEARCH" | "OTHER";
export type RoadmapActivityStatus = "pending" | "in_progress" | "completed";
export interface RoadmapActivity { id: string; title: string; description: string; activityType: RoadmapActivityType; status: RoadmapActivityStatus; completedAt: string | null; order: number }
export interface RoadmapTopic { id: string; name: string; description: string; knowledgeNodeId: string | null; state: RoadmapTopicState; order: number; activities: RoadmapActivity[] }
export interface RoadmapStage { id: string; name: string; description: string; order: number; topics: RoadmapTopic[] }
export interface StudyRoadmap { id: string; name: string; description: string; status: RoadmapStatus; completedActivities: number; totalActivities: number; progress: number; stages: RoadmapStage[]; createdAt: string; updatedAt: string }
export type StudyRoadmapInput = Pick<StudyRoadmap, "id" | "name" | "description" | "status" | "stages">;

export type ResearchKind = "project" | "study" | "research";
export type ResearchStatus = "planned" | "active" | "paused" | "completed";
export interface ResearchItem { id: string; title: string; domain: string; objective: string; description: string; kind: ResearchKind; status: ResearchStatus; impact: string; knowledgeNodeId: string | null; roadmapId: string | null; roadmapTopicId: string | null; projectId: string | null; createdAt: string; updatedAt: string }
export type ResearchInput = Omit<ResearchItem, "createdAt" | "updatedAt">;
export interface StarkSummary { roadmapCount: number; activeRoadmapCount: number; researchCount: number; activeResearchCount: number; baselineCount: number; eventCount: number }
