export type ModuleId = "command" | "ai" | "daily" | "projects" | "studies" | "market" | "automation" | "settings";
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
export type AIGenerationProfile = "standard" | "repetition-retry" | "study" | "study-structured";
export interface AIRequestMetadata {
  domain: "study";
  action: string;
  promptVersion: string;
  contextTruncated: boolean;
  sourceMaterialCount?: number;
}
export interface AIRequest {
  model: string;
  messages: ProviderMessage[];
  timeoutSeconds: number;
  generationProfile?: AIGenerationProfile;
  structuredOutputSchema?: Record<string, unknown> | null;
  requestMetadata?: AIRequestMetadata | null;
}
export interface AIResponse { content: string; model: string; truncated: boolean }
export interface OllamaStatus { available: boolean; models: string[]; error: string | null }
export type LocalAiServerStatus = "ONLINE" | "OFFLINE" | "UNKNOWN";
export type LocalAiModelState = "UNLOADED" | "LOADING" | "LOADED" | "UNLOADING" | "NOT_AVAILABLE" | "ERROR" | "UNKNOWN";
export interface LocalAiModelStatus {
  provider: "OLLAMA";
  model: string;
  serverStatus: LocalAiServerStatus;
  modelStatus: LocalAiModelState;
  loaded: boolean;
  expiresAt: string | null;
  lastCheckedAt: string;
  error: string | null;
}
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
  | "get_today_tasks" | "get_overdue_tasks" | "get_upcoming_tasks" | "get_critical_tasks" | "get_priority_tasks" | "complete_task" | "get_recent_notes" | "get_daily_operations_summary"
  | "list_projects" | "get_project" | "get_project_tasks" | "get_project_knowledge"
  | "list_knowledge_areas" | "get_knowledge_area" | "get_knowledge_gaps" | "get_stark_map" | "get_knowledge_history"
  | "list_study_roadmaps" | "get_study_roadmap" | "list_research_items" | "get_knowledge_origin"
  | "get_learning_progress" | "get_topic_mastery" | "get_knowledge_evidence" | "get_recent_knowledge_events" | "explain_knowledge_level" | "get_roadmap_learning_status" | "get_current_study_position"
  | "get_system_status" | "get_cpu_status" | "get_memory_status" | "get_storage_status" | "get_network_status" | "get_process_summary"
  | "list_workspaces" | "get_workspace_status" | "get_git_status" | "get_recent_commits" | "get_ollama_status"
  | "get_azriel_status" | "get_azriel_version"
  | "get_loaded_model" | "get_model_summary" | "list_components" | "find_component" | "get_component_details" | "get_selected_component" | "get_explosion_state"
  | "get_component_semantics" | "get_subsystems" | "get_subsystem_components" | "get_component_relationships" | "get_unclassified_components" | "get_semantic_coverage" | "get_assembly_graph_summary"
  | "select_component" | "focus_component" | "isolate_component" | "show_all_components" | "hide_component" | "show_component"
  | "set_explosion_factor" | "explode_all" | "explode_component" | "reassemble" | "reset_model_view"
  | "list_routines" | "run_routine" | AutomationActionId;
export interface AIToolInput { query: string; term?: string; entityId?: string; workspaceId?: string; conversationId?: string; referencePosition?: number; factor?: number; delta?: number }
export interface AIToolResult { name: AIToolName; domain: string; data: unknown; empty: boolean }
export interface RoutedIntent { intent: string; scope: "azriel" | "general"; tools: AIToolName[]; term?: string; referencePosition?: number; factor?: number; delta?: number }

export type KnowledgeNodeType = "area" | "discipline" | "topic" | "competency";
export interface KnowledgeBaseline { knowledgeAreaId: string; coverage: number; depth: number; recordedAt: string }
export type KnowledgeEventSource = "baseline" | "roadmap" | "project" | "research" | "manual" | "education";
export interface KnowledgeEvent { id: string; knowledgeNodeId: string; sourceType: KnowledgeEventSource; sourceId: string | null; eventType: string; coverageDelta: number; depthDelta: number; integrationDelta: number; description: string; createdAt: string; activityType: RoadmapActivityType | null; roadmapId: string | null; topicId: string | null; evidenceCycle: number; formulaVersion: string; coverageImpact: number; depthImpact: number; integrationImpact: number; metadataJson: string; reversalOfEventId: string | null }

export type RoadmapStatus = "planned" | "active" | "paused" | "completed";
export type RoadmapTopicState = "NOT_STARTED" | "EXPOSED" | "UNDERSTOOD" | "PRACTICED" | "APPLIED" | "MASTERED";
export type RoadmapActivityType = "READING" | "LESSON" | "QUIZ" | "EXERCISE" | "SIMULATION" | "EXPERIMENT" | "PROJECT" | "DOCUMENTATION" | "RESEARCH" | "OTHER";
export type RoadmapActivityStatus = "pending" | "in_progress" | "completed";
export interface RoadmapActivity { id: string; title: string; description: string; activityType: RoadmapActivityType; status: RoadmapActivityStatus; completedAt: string | null; order: number; primaryKnowledgeNodeId?: string | null; secondaryKnowledgeNodeIds?: string[]; projectId?: string | null; researchId?: string | null }
export interface RoadmapTopic { id: string; name: string; description: string; knowledgeNodeId: string | null; state: RoadmapTopicState; order: number; prerequisiteTopicIds?: string[]; activities: RoadmapActivity[] }
export interface RoadmapStage { id: string; name: string; description: string; order: number; topics: RoadmapTopic[] }
export interface StudyRoadmap { id: string; name: string; description: string; status: RoadmapStatus; completedActivities: number; totalActivities: number; progress: number; stages: RoadmapStage[]; createdAt: string; updatedAt: string }
export type StudyRoadmapInput = Pick<StudyRoadmap, "id" | "name" | "description" | "status" | "stages">;

export type ResearchKind = "project" | "study" | "research";
export type ResearchStatus = "planned" | "active" | "paused" | "completed";
export interface ResearchItem { id: string; title: string; domain: string; objective: string; description: string; kind: ResearchKind; status: ResearchStatus; impact: string; knowledgeNodeId: string | null; roadmapId: string | null; roadmapTopicId: string | null; projectId: string | null; createdAt: string; updatedAt: string }
export type ResearchInput = Omit<ResearchItem, "createdAt" | "updatedAt">;
export interface StarkSummary { roadmapCount: number; activeRoadmapCount: number; researchCount: number; activeResearchCount: number; baselineCount: number; eventCount: number }
export interface LearningMutation { createdEvents: KnowledgeEvent[]; affectedKnowledgeIds: string[]; integration: number }
export interface RoadmapSaveResult { roadmaps: StudyRoadmap[]; learning: LearningMutation }
export interface RoadmapActivityStatusInput { activityId: string; status: RoadmapActivityStatus }

export type StudySessionStatus = "ACTIVE" | "PAUSED" | "COMPLETED" | "CANCELLED";
export interface StudySession {
  id: string;
  roadmapId: string | null;
  stageId: string | null;
  topicId: string | null;
  activityId: string | null;
  roadmapName: string;
  stageName: string;
  topicName: string;
  activityTitle: string;
  plannedFocusMinutes: number;
  actualFocusSeconds: number;
  currentFocusSeconds: number;
  linkedNoteCount: number;
  observedAt: string;
  breakSeconds: number;
  status: StudySessionStatus;
  startedAt: string;
  runningSince: string | null;
  pausedAt: string | null;
  endedAt: string | null;
  createdAt: string;
  updatedAt: string;
}
export interface StartStudySessionInput { id: string; roadmapId: string | null; activityId: string | null; plannedFocusMinutes: number }
export interface StudySessionListInput { roadmapId?: string | null; dateFrom?: string | null; dateTo?: string | null; limit?: number; offset?: number }
export interface StudyRoadmapFocus { roadmapId: string | null; roadmapName: string; focusSeconds: number }
export interface StudyTodaySummary {
  focusSeconds: number;
  completedSessions: number;
  completedActivities: number;
  mostStudiedRoadmap: StudyRoadmapFocus | null;
  activeSession: StudySession | null;
  lastSession: StudySession | null;
}
export interface StudySettings { focusMinutes: number; shortBreakMinutes: number; updatedAt: string }
export interface StudySettingsInput { focusMinutes: number; shortBreakMinutes: number }
export type StudyNotebookStatus = "ACTIVE" | "ARCHIVED";
export interface StudyNotebook {
  id: string;
  title: string;
  description: string | null;
  status: StudyNotebookStatus;
  noteCount: number;
  lastNoteUpdatedAt: string | null;
  createdAt: string;
  updatedAt: string;
}
export interface StudyNotebookInput { id: string; title: string; description: string | null }
export interface StudyNoteContext {
  roadmapId: string | null;
  stageId: string | null;
  topicId: string | null;
  activityId: string | null;
  studySessionId: string | null;
}
export interface StudyNote extends StudyNoteContext {
  id: string;
  notebookId: string;
  notebookTitle: string;
  title: string;
  content: string;
  contentFormat: "MARKDOWN";
  roadmapName: string | null;
  stageName: string | null;
  topicName: string | null;
  activityTitle: string | null;
  createdAt: string;
  updatedAt: string;
}
export interface StudyNoteSummary {
  id: string;
  notebookId: string;
  notebookTitle: string;
  title: string;
  preview: string;
  roadmapId: string | null;
  roadmapName: string | null;
  activityId: string | null;
  activityTitle: string | null;
  studySessionId: string | null;
  createdAt: string;
  updatedAt: string;
}
export interface StudyNoteInput extends StudyNoteContext { id: string; notebookId: string; title: string; content: string }
export interface StudyNoteListInput { notebookId?: string | null; activityId?: string | null; studySessionId?: string | null; limit?: number; offset?: number }
export interface StudyNoteSearchInput { query: string; limit?: number }
export type StudyCardStatus = "ACTIVE" | "SUSPENDED" | "ARCHIVED";
export type StudyCardFilter = "ALL" | "DUE" | "NEW" | "SUSPENDED" | "ARCHIVED";
export type StudyReviewResultValue = "AGAIN" | "HARD" | "GOOD" | "EASY";
export type StudyReviewQueueMode = "TEN" | "TWENTY" | "ALL_OVERDUE";
export interface StudyCard {
  id: string;
  front: string;
  back: string;
  status: StudyCardStatus;
  roadmapId: string | null;
  roadmapName: string | null;
  stageId: string | null;
  stageName: string | null;
  topicId: string | null;
  topicName: string | null;
  activityId: string | null;
  activityTitle: string | null;
  notebookId: string | null;
  notebookTitle: string | null;
  noteId: string | null;
  noteTitle: string | null;
  dueAt: string;
  lastReviewedAt: string | null;
  reviewCount: number;
  correctCount: number;
  incorrectCount: number;
  currentIntervalSeconds: number;
  schedulerVersion: string;
  createdAt: string;
  updatedAt: string;
}
export interface StudyCardContext {
  roadmapId: string | null;
  stageId: string | null;
  topicId: string | null;
  activityId: string | null;
  notebookId: string | null;
  noteId: string | null;
}
export interface StudyCardInput extends StudyCardContext { id: string; front: string; back: string; sourceMaterialIds?: string[] }
export interface StudyCardListInput {
  query?: string | null;
  status?: StudyCardStatus | null;
  filter?: StudyCardFilter | null;
  roadmapId?: string | null;
  activityId?: string | null;
  noteId?: string | null;
  limit?: number;
  offset?: number;
}
export interface StudyReviewDashboardSummary {
  overdue: number;
  dueToday: number;
  newCards: number;
  reviewedToday: number;
  lastReviewedAt: string | null;
}
export interface StudyReviewQueueItem { itemId: string | null; order: number; category: "OVERDUE" | "TODAY" | "NEW"; card: StudyCard }
export interface StartStudyReviewSessionInput { id: string; mode: StudyReviewQueueMode; studySessionId: string | null }
export interface StudyReviewSession {
  id: string;
  startedAt: string;
  endedAt: string | null;
  status: "ACTIVE" | "COMPLETED" | "CANCELLED";
  plannedCards: number;
  reviewedCards: number;
  studySessionId: string | null;
  contextLabel: string;
  currentItem: StudyReviewQueueItem | null;
  createdAt: string;
}
export interface SubmitStudyReviewResultInput {
  id: string;
  reviewSessionId: string;
  sessionItemId: string;
  result: StudyReviewResultValue;
  responseTimeMs: number | null;
}
export interface StudyReviewResult { session: StudyReviewSession; nextDueAt: string; nextIntervalSeconds: number }
export interface StudyReviewSummary {
  sessionId: string;
  status: "ACTIVE" | "COMPLETED" | "CANCELLED";
  reviewedCards: number;
  again: number;
  hard: number;
  good: number;
  easy: number;
  durationSeconds: number;
  nextReviewAt: string | null;
}
export interface StudyReviewSessionListInput { limit?: number; offset?: number }
export type StudyMaterialType = "PDF" | "IMAGE" | "TEXT" | "MARKDOWN" | "LINK" | "OTHER";
export type StudyMaterialStorageKind = "MANAGED_COPY" | "LINKED_LOCAL_FILE" | "EXTERNAL_URL";
export type StudyMaterialStatus = "ACTIVE" | "ARCHIVED" | "MISSING" | "ERROR";
export type MaterialTextStatus = "NOT_ATTEMPTED" | "AVAILABLE" | "TEXT_UNAVAILABLE" | "FAILED";
export type StudyMaterialRelationType = "ROADMAP" | "STAGE" | "TOPIC" | "ACTIVITY" | "NOTEBOOK" | "NOTE" | "CARD";
export interface StudyMaterialRelation { id: string; relationType: StudyMaterialRelationType; relationId: string; label: string; createdAt: string }
export interface StudyMaterialRelationInput { relationType: StudyMaterialRelationType; relationId: string }
export interface MaterialTextContent { materialId: string; status: MaterialTextStatus; text: string | null; pages: string[]; extractedAt: string | null; extractorVersion: string | null; charCount: number; isStale: boolean; errorCode: string | null }
export interface StudyMaterial {
  id: string;
  title: string;
  materialType: StudyMaterialType;
  storageKind: StudyMaterialStorageKind;
  externalUrl: string | null;
  originalFilename: string | null;
  mimeType: string | null;
  fileSize: number | null;
  checksumSha256: string | null;
  sourceAuthor: string | null;
  sourceTitle: string | null;
  sourceYear: number | null;
  description: string | null;
  pageCount: number | null;
  status: StudyMaterialStatus;
  removed: boolean;
  managedFileAvailable: boolean;
  relations: StudyMaterialRelation[];
  textContent: MaterialTextContent;
  createdAt: string;
  updatedAt: string;
}
export interface StudyMaterialSummary { id: string; title: string; materialType: StudyMaterialType; storageKind: StudyMaterialStorageKind; originalFilename: string | null; sourceAuthor: string | null; status: StudyMaterialStatus; extractionStatus: MaterialTextStatus; extractionStale: boolean; relationLabels: string[]; updatedAt: string }
export interface ImportStudyMaterialInput { id: string; title: string; sourcePath: string; storageKind: Exclude<StudyMaterialStorageKind, "EXTERNAL_URL">; sourceAuthor: string | null; sourceTitle: string | null; sourceYear: number | null; description: string | null; relations: StudyMaterialRelationInput[] }
export interface AddLinkStudyMaterialInput { id: string; title: string; externalUrl: string; description: string | null; sourceAuthor: string | null; sourceTitle: string | null; sourceYear: number | null; relations: StudyMaterialRelationInput[] }
export interface UpdateStudyMaterialInput { id: string; title: string; description: string | null; sourceAuthor: string | null; sourceTitle: string | null; sourceYear: number | null }
export interface StudyMaterialListInput { query?: string | null; materialType?: StudyMaterialType | null; status?: StudyMaterialStatus | null; relationType?: StudyMaterialRelationType | null; relationId?: string | null; limit?: number; offset?: number }
export interface ImportStudyMaterialResult { material: StudyMaterial; duplicate: boolean }
export type StudyAIAction = "EXPLAIN" | "SUMMARIZE" | "QUIZ" | "GENERATE_CARDS" | "EVALUATE_ANSWER";
export type StudyAIStatus = "SUCCESS" | "INVALID_OUTPUT" | "TIMEOUT" | "PROVIDER_ERROR" | "CANCELLED";
export type StudyAIAssessment = "CORRECT" | "PARTIALLY_CORRECT" | "INCORRECT" | "INSUFFICIENT_CONTEXT";
export interface StudyAIEntityContext { id: string | null; name: string | null }
export interface StudyAIActivityContext extends StudyAIEntityContext { description: string | null; activityType: string | null }
export interface StudyAINoteContext { id: string; title: string; content: string; notebookId: string | null; notebookTitle: string | null }
export interface StudyAISessionContext { id: string; status: string; plannedFocusMinutes: number | null }
export interface StudyAICardContext { id: string | null; front: string; back: string }
export interface StudyAIReviewContext { question: string; expectedAnswer: string; userAnswer: string | null }
export interface StudyAIMaterialContext { id: string; title: string; materialType: StudyMaterialType; selectedText: string; pageRange: { from: number; to: number } | null }
export interface StudyAIContext {
  selectedText?: string | null;
  roadmap?: StudyAIEntityContext | null;
  stage?: StudyAIEntityContext | null;
  topic?: StudyAIEntityContext | null;
  activity?: StudyAIActivityContext | null;
  note?: StudyAINoteContext | null;
  studySession?: StudyAISessionContext | null;
  relatedCard?: StudyAICardContext | null;
  review?: StudyAIReviewContext | null;
  materials?: StudyAIMaterialContext[];
}
export interface StudyAIBuiltContext {
  serialized: string;
  contextTruncated: boolean;
  source: "SELECTED_TEXT" | "MATERIAL" | "NOTE" | "ACTIVITY" | "TOPIC" | "REVIEW" | "CARD";
  materialIds: string[];
}
export interface StudyAIQuizQuestion { question: string; expectedAnswer: string }
export interface StudyAIGeneratedCard { front: string; back: string }
export interface StudyAIEvaluation {
  assessment: StudyAIAssessment;
  explanation: string;
  missingPoints: string[];
  strengths: string[];
  suggestedAnswer: string;
}
export interface StudyAIResult {
  action: StudyAIAction;
  status: StudyAIStatus;
  content: string | null;
  quiz: StudyAIQuizQuestion[] | null;
  cards: StudyAIGeneratedCard[] | null;
  evaluation: StudyAIEvaluation | null;
  model: string;
  promptVersion: string;
  latencyMs: number;
  contextTruncated: boolean;
  materialIds: string[];
  error: string | null;
}
export interface CurrentStudyPosition { roadmapId: string; stageId: string | null; topicId: string | null; activityId: string | null }
export interface LearningEngineStatus { formulaVersion: string; integrationBaseline: number; currentIntegration: number; eventCount: number; lastRecalculatedAt: string | null; status: "ready" | "recalculating" | "error"; lastError: string | null }

export type MarketTimeframe = "1D" | "15M";
export interface MarketDataset { id: string; name: string; asset: string; timeframe: MarketTimeframe; currency: string; market: string; timezone: string; sessionType: string; startAt: string; endAt: string; candleCount: number; sessionCount: number; expectedGapCount: number; unexpectedGapCount: number; fingerprint: string; sourcePath: string; importedAt: string }
export interface MarketAgentDefinition { id: string; name: string; strategyType: string; strategyVersion: string; defaultConfigJson: string; enabled: boolean }
export interface MarketAiStatus { agentId: string; configured: boolean; available: boolean; provider: string; model: string; promptVersion: string; decisionInterval: number; timeoutMs: number; maxRetries: number; error: string | null }
export interface UpdateMarketAiConfigInput { agentId: string; decisionInterval: number; timeoutMs: number; maxRetries: number }
export interface MarketAiRuntimeMetric { agentId: string; promptVersion: string; callCount: number; successfulCallCount: number; invalidResponseCount: number; timeoutCount: number; retryCount: number; fallbackCount: number; averageLatencyMs: number; maxLatencyMs: number; totalLatencyMs: number; inputTokens: number | null; outputTokens: number | null; buyCount: number; sellCount: number; llmHoldCount: number; noLlmCallCount: number; averageConfidence: number | null; averageBuyConfidence: number | null; averageSellConfidence: number | null; averageHoldConfidence: number | null; minConfidence: number | null; maxConfidence: number | null; medianConfidence: number | null; confidenceDistribution: number[]; firstPassValidCount: number; retryRecoveredCount: number; finalValidCount: number; finalInvalidCount: number; systemFallbackCount: number; firstAttemptAverageLatencyMs: number; retryAverageLatencyMs: number; p50LatencyMs: number; p95LatencyMs: number; slowCallCount: number; firstPassValidRatePct: number; retryRecoveryRatePct: number; finalValidRatePct: number; fallbackRatePct: number }
export interface MarketAiValidationStageLog { stage: string; success: boolean; errorCode: string | null; message: string | null }
export interface MarketAiAttemptLog { attemptNumber: number; rawResponse: string | null; status: "VALID" | "INVALID" | "PROVIDER_ERROR"; errorType: string | null; errorField: string | null; errorValue: string | null; errorMessage: string | null; normalizedFromWrappedJson: boolean; latencyMs: number; stages: MarketAiValidationStageLog[] }
export interface MarketRiskRuleEvaluation { rule: string; status: "PASSED" | "REJECTED" | "NOT_APPLICABLE"; reason: string | null }
export interface MarketRiskEvaluation { result: "APPROVED" | "MODIFIED" | "REJECTED"; reason: string; approvedPositionPct: number | null; classification: "RISK_INCREASING" | "RISK_REDUCING" | "RISK_NEUTRAL"; currentExposurePct: number; targetExposurePct: number; exposureDeltaPct: number; policyVersion: string; rules: MarketRiskRuleEvaluation[] }
export interface MarketAiDecisionLog { decisionId: number; agentId: string; timestamp: string; action: "BUY" | "SELL" | "HOLD"; lifecycleAction: "ENTER_LONG" | "HOLD_POSITION" | "INCREASE_LONG" | "REDUCE_LONG" | "EXIT_LONG" | "STAY_FLAT" | null; intent: "ENTER" | "HOLD" | "REDUCE" | "EXIT" | null; generatedTargetExposurePct: number | null; positionSizingVersion: string | null; contextVersion: string | null; triggerVersion: string | null; contextBytes: number | null; temperature: number | null; randomSeed: number | null; seedSupported: boolean | null; desiredPositionPct: number | null; confidence: number | null; reason: string; reasonCode: string | null; validationCode: string | null; riskResult: "APPROVED" | "MODIFIED" | "REJECTED"; riskReason: string; approvedPositionPct: number | null; executionPrice: number | null; callStatus: "VALID" | "INVALID" | "TIMEOUT" | "OFFLINE" | "NO_LLM_CALL"; provider: string; model: string; promptVersion: string; latencyMs: number; attempts: number; fallbackUsed: boolean; inputSnapshot: Record<string, unknown> | null; forwardReturn1: number | null; forwardReturn5: number | null; forwardReturn10: number | null; signalDisagreement: boolean; firstFailureType: string | null; fallbackReason: string | null; riskTrace: MarketRiskEvaluation; outputAttempts: MarketAiAttemptLog[] }
export interface MarketAiExperimentComparison { experimentId: string; experimentName: string; datasetId: string; datasetName: string; riskProfileId: string; initialCapital: number; randomSeed: number; feePct: number; slippagePct: number; decisionInterval: number; agentId: string; promptVersion: string; callCount: number; successfulCallCount: number; invalidResponseCount: number; timeoutCount: number; buyCount: number; sellCount: number; llmHoldCount: number; tradeCount: number; holdRatePct: number; averageConfidence: number | null; totalReturnPct: number; maxDrawdownPct: number; averageExposurePct: number; averageLatencyMs: number; maxLatencyMs: number; p95LatencyMs: number; firstPassValidCount: number; retryRecoveredCount: number; finalValidCount: number; finalInvalidCount: number; systemFallbackCount: number; enterCount: number; intentHoldCount: number; reduceCount: number; exitCount: number; averageGeneratedTargetExposurePct: number | null; riskIncreasingCount: number; riskReducingCount: number; riskNeutralCount: number; maxOperationsRejectionCount: number; maxOperationsNotApplicableCount: number; reductionsExecutedCount: number; exitsExecutedCount: number }
export interface MarketSignalMatrixRow { label: string; buyCount: number; sellCount: number; holdCount: number; averageForward5: number | null }
export interface MarketSignalDiagnostics { signalEngineVersion: string; signalConfigVersion: string; bullishCount: number; bearishCount: number; neutralCount: number; strongCount: number; weakCount: number; lowConflictCount: number; mediumConflictCount: number; highConflictCount: number; actionCollapse: boolean; collapsedAction: string | null; confidenceCollapse: boolean; collapsedConfidence: number | null; disagreementCount: number; disagreementRatePct: number; strongBullishBuyRatePct: number | null; strongBullishHoldRatePct: number | null; strongBearishSellRatePct: number | null; strongBearishHoldRatePct: number | null; signalActionMatrix: MarketSignalMatrixRow[]; conflictActionMatrix: MarketSignalMatrixRow[]; reasonCodes: { reasonCode: string; count: number }[] }
export type HoldQuality = "GOOD_HOLD" | "POTENTIAL_MISSED_OPPORTUNITY" | "POTENTIAL_LATE_REDUCTION" | "INCONCLUSIVE";
export interface HoldDiagnosticsConfig { missedForward5Pct: number; missedMfe5Pct: number; adverseForward5Pct: number; adverseMae5Pct: number; concentrationWarningPct: number; candidateMinimumCount: number; candidateQualityRatePct: number; candidateEffectPct: number }
export interface HoldPostDecisionOutcome { forward1: number | null; forward5: number | null; forward10: number | null; mfe1: number | null; mfe5: number | null; mfe10: number | null; mae1: number | null; mae5: number | null; mae10: number | null }
export interface HoldDiagnosticItem { decisionId: number; candleIndex: number; timestamp: string; sessionId: string; sessionPhase: string; positionState: "FLAT" | "LONG"; currentExposurePct: number; confidence: number | null; confidenceBucket: string; reasonCode: string; reason: string; price: number; ema9: number | null; ema21: number | null; vwap: number | null; rsi14: number | null; atr14: number | null; relativeVolume: number | null; distanceFromVwapAtr: number | null; emaSpreadAtr: number | null; trend: string; momentum: string; volatility: string; location: string; vwapPosition: string; triggerReason: string; conflictSignature: string | null; rsiBucket: string; relativeVolumeBucket: string; vwapDistanceBucket: string; emaSpreadBucket: string; outcome: HoldPostDecisionOutcome; quality: HoldQuality; qualityContext: string }
export interface HoldAggregateRow { key: string; count: number; averageConfidence: number | null; averageForward1: number | null; averageForward5: number | null; averageForward10: number | null; averageMfe5: number | null; averageMae5: number | null; goodHoldCount: number; missedOpportunityCount: number; lateReductionCount: number; inconclusiveCount: number; goodHoldRatePct: number; missedOpportunityRatePct: number; lateReductionRatePct: number }
export interface CalibrationCandidate { reasonCode: string; count: number; issue: "MISSED_OPPORTUNITY" | "LATE_REDUCTION"; issueRatePct: number; averageForward5: number | null }
export interface HoldTriggerAnalysisRow { trigger: string; callCount: number; holdCount: number; enterCount: number; holdRatePct: number; averageForward5AfterHold: number | null; missedOpportunityRatePct: number }
export interface HoldDiagnosticsReport { runId: string; experimentId: string; experimentName: string; datasetId: string; datasetName: string; asset: string; timeframe: string; agentId: string; engineVersion: string; status: string; createdAt: string; completedAt: string | null; totalAiCalls: number; totalHolds: number; holdRatePct: number; holdConcentrationWarning: boolean; flatCount: number; longCount: number; quality: { total: number; goodHoldCount: number; missedOpportunityCount: number; lateReductionCount: number; inconclusiveCount: number }; qualityMatrix: HoldAggregateRow[]; reasons: HoldAggregateRow[]; confidence: HoldAggregateRow[]; sessionPhases: HoldAggregateRow[]; triggers: HoldAggregateRow[]; triggerAnalysis: HoldTriggerAnalysisRow[]; featureBuckets: HoldAggregateRow[]; conflicts: HoldAggregateRow[]; calibrationCandidates: CalibrationCandidate[]; config: HoldDiagnosticsConfig; items: HoldDiagnosticItem[] }
export interface HoldDiagnosticsComparison { developmentExperimentId: string; outOfSampleExperimentId: string; asset: string; timeframe: string; metrics: { label: string; developmentValue: number; outOfSampleValue: number }[]; developmentTopReason: string | null; outOfSampleTopReason: string | null; developmentTopCandidate: string | null; outOfSampleTopCandidate: string | null; candidateEvidence: { reasonCode: string; issue: string; development: CalibrationCandidate | null; outOfSample: CalibrationCandidate | null; evidence: "INSUFFICIENT" | "WEAK" | "MODERATE" | "STRONG" }[] }
export type LifecycleState = "FLAT" | "OPENING" | "LONG" | "REDUCING" | "CLOSING" | "CLOSED";
export type DeteriorationLevel = "NONE" | "LOW" | "MODERATE" | "HIGH" | "CRITICAL";
export type PositionHealth = "STRONG" | "HEALTHY" | "WEAKENING" | "DETERIORATING" | "CRITICAL";
export interface LifecycleSummary { lifecycleId: string; asset: string; timeframe: string; entryDecisionTimestamp: string | null; entryExecutionTimestamp: string; entryPrice: number; initialExposurePct: number; maxExposurePct: number; exitDecisionTimestamp: string | null; exitExecutionTimestamp: string | null; exitPrice: number | null; durationCandles: number; durationMarketMinutes: number; overnight: boolean; status: string; realizedPnl: number; realizedPnlPct: number; mfePct: number; maePct: number; givebackPct: number; exitEfficiencyPct: number | null; entryReason: string | null; exitReason: string | null; entryAtr: number | null; entryVwap: number | null; entryEma9: number | null; entryEma21: number | null; entryRsi: number | null; entryRelativeVolume: number | null; entryMarketState: unknown | null; entrySessionPhase: string | null; firstLowHealthTimestamp: string | null; firstModerateDeteriorationTimestamp: string | null; firstHighDeteriorationTimestamp: string | null; responseDelayCandles: number | null; responseDelayMarketMinutes: number | null; responseCensored: boolean }
export interface LifecycleEventView { id: number; lifecycleId: string; decisionId: number | null; executionId: number | null; timestamp: string; candleIndex: number; eventType: "OPEN" | "INCREASE" | "HOLD" | "REDUCE" | "EXIT" | "FORCED_EXIT"; stateBefore: LifecycleState; stateAfter: LifecycleState; positionBefore: string; positionAfter: string; exposureBeforePct: number; exposureAfterPct: number; marketPrice: number; unrealizedPnlPct: number; mfeSoFarPct: number; maeSoFarPct: number; aiIntent: string | null; confidence: number | null; reasonCode: string | null; triggerReason: string | null; riskResult: string; executionResult: string }
export interface LifecycleHealthTrace { lifecycleId: string; candleIndex: number; timestamp: string; positionAgeCandles: number; positionAgeMarketMinutes: number; marketPrice: number; exposurePct: number; returnSinceEntryPct: number; mfeSinceEntryPct: number; maeSinceEntryPct: number; distanceFromEntryAtr: number | null; distanceFromMfePct: number; givebackFromMfePct: number; givebackRelativePct: number | null; vwapChangeSinceEntry: number | null; emaSpreadChangeSinceEntry: number | null; rsiChangeSinceEntry: number | null; trendChanged: boolean; momentumChanged: boolean; trendComponent: number; momentumComponent: number; givebackComponent: number; vwapComponent: number; volatilityComponent: number; timeComponent: number; deteriorationScore: number; deteriorationLevel: DeteriorationLevel; positionHealth: PositionHealth; availableAtT: true }
export interface LifecycleOutcome { lifecycleId: string; candleIndex: number; decisionId: number | null; holdQuality: HoldQuality | null; forwardReturn1: number | null; forwardReturn5: number | null; mfe5: number | null; mae5: number | null; missedReductionWindow: boolean; postDecisionOnly: true }
export interface LifecycleOutcomeAggregate { label: string; count: number; averageForward1: number; averageForward5: number; averageMfe5: number; averageMae5: number }
export interface ExitReasonAggregate { reasonCode: string; count: number; averagePnlPct: number; averageMfePct: number; averageMaePct: number; averageGivebackPct: number; averageDurationMinutes: number }
export interface PositionDeteriorationConfig { trendWeight: number; momentumWeight: number; givebackWeight: number; vwapWeight: number; volatilityWeight: number; timeWeight: number; lowThreshold: number; moderateThreshold: number; highThreshold: number; criticalThreshold: number }
export interface LifecycleIntelligenceReport { runId: string; experimentId: string; experimentName: string; datasetId: string; datasetName: string; asset: string; timeframe: string; agentId: string; lifecycleEngineVersion: string; deteriorationEngineVersion: string; status: string; createdAt: string; completedAt: string | null; summary: { totalLifecycles: number; openLifecycles: number; closedLifecycles: number; averageDurationMinutes: number; averagePnlPct: number; averageMfePct: number; averageMaePct: number; averageGivebackPct: number; lateReductionEvents: number; averageResponseDelayMinutes: number }; lateReduction: { totalHoldWhileLong: number; potentialLateReductions: number; ratePct: number; averageForward5: number; averageMae5: number; averageResponseDelayMinutes: number; topPositionHealth: string | null; topReasonCode: string | null }; config: PositionDeteriorationConfig; lifecycles: LifecycleSummary[]; events: LifecycleEventView[]; healthTrace: LifecycleHealthTrace[]; outcomes: LifecycleOutcome[]; healthOutcomes: LifecycleOutcomeAggregate[]; deteriorationOutcomes: LifecycleOutcomeAggregate[]; exitReasons: ExitReasonAggregate[] }
export interface LifecycleIntelligenceComparison { developmentExperimentId: string; outOfSampleExperimentId: string; asset: string; timeframe: string; metrics: { label: string; developmentValue: number; outOfSampleValue: number }[]; deteriorationDistribution: { level: DeteriorationLevel; developmentCount: number; outOfSampleCount: number }[] }
export type LifecycleValidationSourceRole = "DEVELOPMENT" | "OOS" | "HOLDOUT" | "ADDITIONAL_VALIDATION";
export interface LifecycleValidationPeriodInput { experimentId: string; sourceRole: LifecycleValidationSourceRole }
export interface LifecycleValidationBatchInput { name: string; periods: LifecycleValidationPeriodInput[] }
export interface LifecycleValidationBatch { batchId: string; name: string; status: "CREATED" | "VALIDATING" | "COMPLETED" | "FAILED" | "PARTIAL"; asset: string; timeframe: string; agentId: string; agentVersion: string; lifecycleConfigVersion: string; deteriorationConfigVersion: string; holdDiagnosticsVersion: string; executionModelVersion: string; validationEngineVersion: string; datasetCount: number; experimentCount: number; lifecycleCount: number; createdAt: string; startedAt: string | null; completedAt: string | null }
export interface ValidationDistribution { average: number; median: number; p25: number | null; p75: number | null; p90: number | null; minimum: number; maximum: number }
export interface LifecycleValidationPeriod { periodId: string; experimentId: string; experimentName: string; datasetId: string; datasetName: string; sourceRole: LifecycleValidationSourceRole; startAt: string; endAt: string; candleCount: number; sessionCount: number; lifecycleCount: number; lifecycleRunId: string; closedCount: number; openCount: number; duration: ValidationDistribution; pnl: ValidationDistribution; mfe: ValidationDistribution; mae: ValidationDistribution; giveback: ValidationDistribution; lateReductionEvents: number; lateReductionLifecycles: number; eligibleLifecycles: number; eventLateReductionRatePct: number; lifecycleLateReductionRatePct: number; averageForward5: number | null; averageMae5: number | null; responseDelay: ValidationDistribution; unresolvedResponseCount: number; overlapStatus: "NON_OVERLAPPING" | "TOUCHING_BOUNDARY" | "OVERLAPPING" | "DUPLICATE_RANGE"; context: { periodReturnPct: number; realizedVolatilityPct: number; averageAtrPct: number | null; trendProxy: string; averageRelativeVolume: number | null } }
export interface LifecycleComponentSnapshot { trend: number; momentum: number; giveback: number; vwap: number; volatility: number; time: number }
export interface LifecycleValidationLifecycle { periodId: string; sourceRole: LifecycleValidationSourceRole; experimentId: string; lifecycleId: string; entryAt: string; exitAt: string | null; status: "OPEN" | "CLOSED"; durationMinutes: number; pnlPct: number | null; mfePct: number; maePct: number; givebackPct: number; worstHealth: PositionHealth; dominantHealth: PositionHealth; healthDistribution: { strongPct: number; healthyPct: number; weakeningPct: number; deterioratingPct: number; criticalPct: number }; maxDeteriorationScore: number; maxDeteriorationLevel: DeteriorationLevel; firstTimestampAtMaxLevel: string | null; lateReductionEvents: number; hasLateReduction: boolean; responseDelayMinutes: number | null; responseStatus: "RESOLVED" | "UNRESOLVED" | "NOT_APPLICABLE"; responseFromFirstDeteriorationMinutes: number | null; responseFromHighMinutes: number | null; responseFromCriticalMinutes: number | null; overnight: boolean; censoredAtDatasetEnd: boolean; averageComponents: LifecycleComponentSnapshot; maximumComponents: LifecycleComponentSnapshot; componentAtFirstHigh: LifecycleComponentSnapshot | null; componentAtFirstCritical: LifecycleComponentSnapshot | null; dominantDeteriorationComponent: string }
export interface LifecycleDatasetQuality { datasetId: string; datasetName: string; status: "PASS" | "PASS_WITH_WARNINGS" | "FAIL"; candleCount: number; sessionCount: number; firstAt: string; lastAt: string; nullValueCount: number; duplicateTimestampCount: number; outOfOrderCount: number; nonpositivePriceCount: number; negativeVolumeCount: number; invalidHighCount: number; invalidLowCount: number; expectedGapCount: number; unexpectedGapCount: number; market: string; timezone: string; sessionType: string; sourceFileAvailable: boolean; warnings: string[]; errors: string[] }
export interface LifecycleValidationAudit { reusedExperiments: number; rebuiltArtifacts: number; newLlmRuns: number; failedPeriods: number; skippedPeriods: number; durationMs: number; model: string; promptVersion: string; contextVersion: string; triggerVersion: string; positionSizingVersion: string; riskPolicyVersion: string; executionModelVersion: string; feePct: number; slippagePct: number }
export interface MultiPeriodLifecycleValidationReport { batch: LifecycleValidationBatch; sample: { datasets: number; periods: number; experiments: number; lifecycles: number; closed: number; open: number; censored: number; dateCoverageStart: string; dateCoverageEnd: string; lifecycleLateReductionRatePct: number; eventLateReductionRatePct: number; medianResponseDelayMinutes: number | null; scoreStability: string; sampleStatus: string }; periods: LifecycleValidationPeriod[]; lifecycles: LifecycleValidationLifecycle[]; deteriorationByPeriod: { periodId: string; level: DeteriorationLevel; eventCount: number; eventPct: number; lifecycleCount: number; lifecycleReachPct: number }[]; deteriorationOutcomes: { label: string; lifecycleCount: number; averagePnlPct: number | null; medianPnlPct: number | null; averageMfePct: number; averageMaePct: number; averageGivebackPct: number; lateReductionRatePct: number }[]; healthOutcomes: { label: string; lifecycleCount: number; averagePnlPct: number | null; medianPnlPct: number | null; averageMfePct: number; averageMaePct: number; averageGivebackPct: number; lateReductionRatePct: number }[]; components: { periodId: string | null; component: string; lifecycleCount: number; averageValue: number; maximumValue: number; averagePnlPct: number | null; averageMaePct: number; averageGivebackPct: number; lateReductionRatePct: number }[]; consistency: { label: string; values: number[]; median: number; minimum: number; maximum: number; dispersion: number }[]; evidence: { sampleSize: string; periodCoverage: string; lateReduction: string; deteriorationStability: string; healthOutcomeRelation: string; componentConsistency: string; givebackConsistency: string }; outlierSensitivity: { metric: string; lifecycleId: string | null; fullAverage: number; withoutLargestOutlierAverage: number; delta: number }[]; quality: LifecycleDatasetQuality[]; audit: LifecycleValidationAudit; warnings: string[]; postDecisionOnly: true }
export interface MarketRiskProfile { id: string; name: string; maxPositionPct: number; maxTotalExposurePct: number; maxDailyLossPct: number; maxDrawdownPct: number; maxTradesPerDay: number | null; allowLeverage: boolean; allowShort: boolean; allowedAssets: string[] }
export interface ImportMarketDatasetInput { path: string; name: string; asset: string; timeframe: MarketTimeframe; currency?: string; market?: string; timezone?: string; sessionType?: string }
export interface MarketExperimentInput { name: string; datasetId: string; riskProfileId: string; agentIds: string[]; initialCapital: number; randomSeed: number; feePct: number; slippagePct: number }
export interface MarketRepeatabilityInput { sourceExperimentId: string; repetitions: number }
export interface MarketRepeatabilityMetric { agentId: string; agentName: string; runCount: number; returnMeanPct: number; returnMinPct: number; returnMaxPct: number; returnStdPct: number; drawdownMeanPct: number; drawdownMinPct: number; drawdownMaxPct: number; callCountMean: number; enterCount: number; holdCount: number; reduceCount: number; exitCount: number }
export interface MarketRepeatabilityReport { groupId: string; name: string; sourceExperimentId: string; repetitions: number; status: "running" | "completed" | "failed" | "aborted"; experimentIds: string[]; metrics: MarketRepeatabilityMetric[]; createdAt: string; completedAt: string | null }
export interface MarketAgentMetric { agentId: string; agentName: string; status: string; finalEquity: number; totalReturnPct: number; maxDrawdownPct: number; decisionCount: number; holdCount: number; tradeCount: number; winRatePct: number; profitFactor: number | null; realizedPnl: number; unrealizedPnl: number; averageExposurePct: number; maxExposurePct: number }
export interface MarketEquityPoint { timestamp: string; agentId: string; equity: number; exposurePct: number }
export interface MarketDecisionLog { id: number; timestamp: string; agentId: string; action: "BUY" | "SELL" | "HOLD"; observedPrice: number; desiredPositionPct: number | null; reasoning: string; riskResult: "APPROVED" | "MODIFIED" | "REJECTED"; riskReason: string; approvedPositionPct: number | null; executionPrice: number | null; quantity: number | null; fees: number | null }
export interface MarketExperimentSummary { id: string; name: string; datasetId: string; datasetName: string; asset: string; timeframe: MarketTimeframe; currency: string; market: string; timezone: string; sessionType: string; annualizationFactor: number; executionModelVersion: string; triggerEngineVersion: string | null; initialCapital: number; riskProfileId: string; randomSeed: number; feePct: number; slippagePct: number; agentIds: string[]; status: "pending" | "running" | "completed" | "failed" | "aborted"; error: string | null; startedAt: string | null; completedAt: string | null; createdAt: string }
export interface MarketBehaviorMetric { agentId: string; buyCount: number; sellCount: number; holdCount: number; holdRatePct: number; tradeFrequencyPct: number; averageExposurePct: number; maxExposurePct: number; averagePositionSizePct: number; maxPositionSizePct: number; averageHoldingCandles: number; medianHoldingCandles: number; maxHoldingCandles: number; turnoverPct: number; timeInMarketPct: number; timeInCashPct: number; entryCount: number; exitCount: number; riskRejectionCount: number; riskModificationCount: number; riskRejectionRatePct: number; riskModificationRatePct: number; drawdownTriggerCount: number; dailyLossTriggerCount: number; formulaVersion: string }
export interface MarketPositionEpisode { agentId: string; episodeIndex: number; openedCandleIndex: number; openedAt: string; closedCandleIndex: number | null; closedAt: string | null; durationCandles: number; maxExposurePct: number; status: "open" | "closed" }
export interface MarketAgentCorrelation { agentAId: string; agentBId: string; equityReturnCorrelation: number | null; decisionSimilarity: number; highSimilarity: boolean; similarityThreshold: number }
export interface MarketBenchmarkComparison { agentId: string; cashReturnPct: number | null; buyHoldReturnPct: number | null; excessVsCashPct: number; excessVsBuyHoldPct: number | null }
export interface MarketObservatory { behavior: MarketBehaviorMetric[]; episodes: MarketPositionEpisode[]; correlations: MarketAgentCorrelation[]; benchmarks: MarketBenchmarkComparison[]; similarityThreshold: number }
export interface MarketTradeLifecycle { id: string; agentId: string; asset: string; lifecycleIndex: number; openedAt: string; closedAt: string | null; entryPrice: number; averageEntryPrice: number; exitPrice: number | null; initialExposurePct: number; maxExposurePct: number; holdingCandles: number; entrySessionId: string | null; exitSessionId: string | null; holdingMarketMinutes: number; overnight: boolean; realizedPnl: number; realizedPnlPct: number; mfePct: number; maePct: number; exitEfficiencyPct: number | null; profitGivebackPct: number; entryReasonCode: string | null; exitReasonCode: string | null; status: "OPEN" | "CLOSED" | "ABORTED"; reentry: boolean }
export interface MarketPositionEvent { id: number; lifecycleId: string; decisionId: number | null; executionId: number | null; timestamp: string; action: "ENTER_LONG" | "HOLD_POSITION" | "INCREASE_LONG" | "REDUCE_LONG" | "EXIT_LONG"; previousExposurePct: number; targetExposurePct: number; newExposurePct: number; signalBias: string | null; confidence: number | null; reasonCode: string | null; riskResult: string }
export interface MarketPositionMetric { agentId: string; closedTrades: number; winningTrades: number; losingTrades: number; averageHoldingCandles: number; medianHoldingCandles: number; averageMfePct: number; averageMaePct: number; averageExitEfficiencyPct: number | null; averageProfitGivebackPct: number; rapidReentryCount: number; rapidExitCount: number; reentryCount: number; averageEntryExposurePct: number; averageMaxExposurePct: number; invalidPositionActionCount: number; flatSellAttemptCount: number; redundantExitCount: number; engineVersion: string }
export interface MarketPositionLifecycleReport { lifecycles: MarketTradeLifecycle[]; events: MarketPositionEvent[]; metrics: MarketPositionMetric[] }
export interface MarketDecisionTriggerAudit { id: number; agentId: string; candleIndex: number; timestampUtc: string; sessionId: string; shouldEvaluate: boolean; triggerReason: string | null; skipReason: "NO_TRIGGER" | "COOLDOWN" | "COMPUTE_BUDGET" | "CADENCE" | null; cooldownRemaining: number; callIndex: number | null }
export interface MarketSessionMetric { agentId: string; sessionId: string; startEquity: number; endEquity: number; returnPct: number; maxDrawdownPct: number; tradeCount: number; aiCallCount: number; triggerCount: number; noCallCount: number }
export interface MarketIntradayFeatureTrace { candleIndex: number; timestampUtc: string; sessionId: string; sessionPhase: string; sessionProgress: number; ready: boolean; features: Record<string, unknown>; engineVersion: string }
export interface MarketIntradayStrategyDecision { decisionId: number; candleIndex: number; timestamp: string; agentId: string; style: string; intent: "ENTER" | "HOLD" | "REDUCE" | "EXIT"; reasonCode: string; reason: string; indicators: Record<string, unknown>; thresholds: Record<string, unknown>; positionBefore: string; generatedTargetExposurePct: number; riskResult: string; executionPrice: number | null }
export interface MarketIntradayStrategyMetric { agentId: string; style: string; trendEntries: number; emaCrossEntries: number; averageTrendDurationMinutes: number; breakoutAttempts: number; breakoutEntries: number; failedBreakouts: number; vwapDeviationEvents: number; meanReversionEntries: number; successfulReversions: number }
export interface MarketSessionPhasePerformance { agentId: string; sessionPhase: string; trades: number; returnPct: number; winRatePct: number; averagePnl: number }
export interface MarketHoldingTimeBucket { agentId: string; bucket: string; tradeCount: number }
export interface MarketIntradayAgentOverlap { agentAId: string; agentBId: string; sameDirectionDecisionRate: number; sameEntryWindowCount: number; sameExitWindowCount: number }
export interface MarketIntradayStrategyReport { featureEngineVersion: string; featureTraces: MarketIntradayFeatureTrace[]; decisions: MarketIntradayStrategyDecision[]; metrics: MarketIntradayStrategyMetric[]; phasePerformance: MarketSessionPhasePerformance[]; holdingDistribution: MarketHoldingTimeBucket[]; overlaps: MarketIntradayAgentOverlap[] }
export interface MarketExperimentResult { experiment: MarketExperimentSummary; metrics: MarketAgentMetric[]; equity: MarketEquityPoint[]; decisions: MarketDecisionLog[]; observatory: MarketObservatory; positionLifecycle: MarketPositionLifecycleReport; triggerAudits: MarketDecisionTriggerAudit[]; sessionMetrics: MarketSessionMetric[]; intradayStrategy: MarketIntradayStrategyReport }
export interface ValidationSplitConfig { inSamplePct: number; validationPct: number; outOfSamplePct: number }
export interface WalkForwardConfig { trainWindowSize: number; testWindowSize: number; stepSize: number }
export interface MarketRegimeConfig { trendWindow: number; volatilityWindow: number; bullThresholdPct: number; bearThresholdPct: number; highVolatilityThresholdPct: number; lowVolatilityThresholdPct: number; minimumSampleCandles: number; minimumSampleTrades: number }
export interface MarketValidationInput { name: string; datasetId: string; riskProfileId: string; agentIds: string[]; initialCapital: number; randomSeed: number; feePct: number; slippagePct: number; splitConfig: ValidationSplitConfig; walkForwardConfig: WalkForwardConfig; regimeConfig: MarketRegimeConfig; rollingWindow: number; annualizationFactor: number }
export interface MarketValidationSummary { id: string; name: string; datasetId: string; datasetName: string; datasetHash: string; riskProfileId: string; agentIds: string[]; status: "draft" | "running" | "completed" | "failed" | "aborted"; error: string | null; startedAt: string | null; completedAt: string | null; createdAt: string }
export interface MarketValidationWindow { id: number; windowIndex: number; windowType: "in_sample" | "validation" | "out_of_sample" | "walk_forward"; startIndex: number; endIndex: number; startAt: string; endAt: string; trainStartIndex: number | null; trainEndIndex: number | null; trainStartAt: string | null; trainEndAt: string | null }
export interface MarketWindowMetric { windowId: number; windowIndex: number; windowType: MarketValidationWindow["windowType"]; agentId: string; agentName: string; totalReturnPct: number; maxDrawdownPct: number; sharpe: number | null; sortino: number | null; calmar: number | null; tradeCount: number; holdRatePct: number; exposurePct: number; benchmarkCashExcessPct: number; benchmarkBuyHoldExcessPct: number }
export interface MarketRegimeMetric { agentId: string; regimeType: "trend" | "volatility"; regime: string; candleCount: number; totalReturnPct: number; maxDrawdownPct: number; tradeCount: number; holdRatePct: number; exposurePct: number; profitFactor: number | null; winRatePct: number | null; lowSampleSize: boolean }
export interface MarketRollingMetric { agentId: string; candleIndex: number; timestamp: string; rollingReturnPct: number | null; rollingVolatilityPct: number | null; rollingSharpe: number | null; rollingDrawdownPct: number | null }
export interface MarketRobustnessReport { agentId: string; agentName: string; inSampleReturnPct: number; validationReturnPct: number; outOfSampleReturnPct: number; outOfSampleDrawdownPct: number; positiveWindowRatioPct: number; averageWindowReturnPct: number; medianWindowReturnPct: number; bestWindowReturnPct: number; worstWindowReturnPct: number; returnStdAcrossWindows: number; drawdownStdAcrossWindows: number; benchmarkExcessPct: number; overfittingGapPct: number; possibleOverfitting: boolean; robustnessStatus: "insufficient_data" | "unstable" | "mixed" | "robust_candidate" }
export interface MarketValidationAudit { frozenConfigJson: string; splitConfigJson: string; walkForwardConfigJson: string; regimeConfigJson: string; rollingWindow: number; annualizationFactor: number; validationEngineVersion: string; metricFormulaVersion: string; regimeEngineVersion: string }
export interface MarketValidationResult { validation: MarketValidationSummary; windows: MarketValidationWindow[]; metrics: MarketWindowMetric[]; regimeMetrics: MarketRegimeMetric[]; rollingMetrics: MarketRollingMetric[]; reports: MarketRobustnessReport[]; audit: MarketValidationAudit }
