export type ModuleId = "command" | "ai" | "daily" | "settings";
export type AzrielState = "idle" | "processing" | "tool" | "executing" | "engineering" | "routine" | "alert" | "offline";
export type AIToolPermission = "read" | "safe_write" | "confirm_write" | "visual_action";
export interface DatabaseInfo { path: string; schemaVersion: number }

export type TaskStatus = "inbox" | "pending" | "in_progress" | "completed" | "cancelled";
export type TaskPriority = "low" | "medium" | "high" | "critical";
export type DailyView = "pending" | "today" | "overdue" | "priority" | "inbox" | "upcoming" | "completed" | "notes" | "archived_notes";
export interface Task { id:string; title:string; description:string; status:TaskStatus; priority:TaskPriority; dueDate:string|null; createdAt:string; updatedAt:string; completedAt:string|null }
export type TaskInput = Omit<Task,"createdAt"|"updatedAt"|"completedAt">;
export type NoteStatus = "active" | "archived";
export interface Note { id:string; title:string|null; content:string; status:NoteStatus; createdAt:string; updatedAt:string }
export type NoteInput = Omit<Note,"createdAt"|"updatedAt">;
export interface DailyCounters { pending:number; today:number; overdue:number; priority:number; notes:number; completed:number }

export interface AISettings { provider:"ollama"; endpoint:string; model:string; contextMessageLimit:number; timeoutSeconds:number; updatedAt:string }
export type AISettingsInput=Pick<AISettings,"endpoint"|"model"|"contextMessageLimit"|"timeoutSeconds">;
export interface Conversation { id:string; title:string; createdAt:string; updatedAt:string }
export type ConversationRole="user"|"assistant"|"system";
export interface ConversationMessage { id:string; conversationId:string; role:ConversationRole; content:string; createdAt:string }
export interface ConversationMessageInput { id:string; conversationId:string; role:ConversationRole; content:string }
export type AIGenerationProfile="standard"|"repetition-retry";
export interface AIRequest { model:string; messages:Array<{role:ConversationRole;content:string}>; timeoutSeconds:number; generationProfile?:AIGenerationProfile; structuredOutputSchema?:Record<string,unknown>|null; requestMetadata?:unknown }
export interface AIResponse { content:string; model:string; truncated:boolean }
export interface OllamaStatus { available:boolean; models:string[]; error:string|null }
export type LocalAiServerStatus="ONLINE"|"OFFLINE"|"UNKNOWN";
export type LocalAiModelState="UNLOADED"|"LOADING"|"LOADED"|"UNLOADING"|"NOT_AVAILABLE"|"ERROR"|"UNKNOWN";
export interface LocalAiModelStatus { provider:"OLLAMA"; model:string; serverStatus:LocalAiServerStatus; modelStatus:LocalAiModelState; loaded:boolean; expiresAt:string|null; lastCheckedAt:string; error:string|null }

export interface SystemDetails { osName:string|null; osVersion:string|null; kernelVersion:string|null; architecture:string; hostname:string|null; logicalCores:number; physicalCores:number|null; uptimeSeconds:number }
export interface CpuSnapshot { usagePercent:number; cores:number[] }
export interface MemorySnapshot { totalBytes:number; usedBytes:number; availableBytes:number; swapTotalBytes:number; swapUsedBytes:number }
export interface StorageSnapshot { name:string; mountPoint:string; fileSystem:string; totalBytes:number; availableBytes:number; removable:boolean }
export interface NetworkSnapshot { interfaceName:string; receivedBytes:number; transmittedBytes:number; receivedBytesTotal:number; transmittedBytesTotal:number }
export interface ProcessSnapshot { pid:number; name:string; cpuPercent:number; memoryBytes:number }
export interface SystemSnapshot { collectedAt:number; details:SystemDetails; cpu:CpuSnapshot; memory:MemorySnapshot; storage:StorageSnapshot[]; network:NetworkSnapshot[]; errors:string[] }

export type AIToolName="get_today_tasks"|"get_overdue_tasks"|"get_upcoming_tasks"|"get_critical_tasks"|"get_priority_tasks"|"complete_task"|"get_recent_notes"|"get_daily_operations_summary"|"get_system_status"|"get_cpu_status"|"get_memory_status"|"get_storage_status"|"get_network_status"|"get_process_summary"|"get_ollama_status"|"get_azriel_status"|"get_azriel_version";
export interface AIToolInput { query:string; term?:string; conversationId?:string; referencePosition?:number; factor?:number; delta?:number }
export interface AIToolResult { name:AIToolName; domain:string; data:unknown; empty:boolean }
export interface RoutedIntent { intent:string; scope:"azriel"|"general"; tools:AIToolName[]; term?:string; referencePosition?:number; factor?:number; delta?:number }
