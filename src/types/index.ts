export type ModuleId = "command" | "ai" | "daily" | "music" | "money" | "settings";
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
export interface RemoteAISettings { provider:"gemini"; enabled:boolean; model:string; apiKeyConfigured:boolean; updatedAt:string }
export interface RemoteAISettingsInput { enabled:boolean; model:string; apiKey?:string|null }
export type AIEngine="ollama"|"gemini"|"deepseek";
export interface DeepSeekSettings { enabled:boolean; model:string; apiKeyConfigured:boolean; updatedAt:string }
export interface DeepSeekSettingsInput { enabled:boolean; model:string; apiKey?:string|null }
export interface NowPlaying { title:string; artist:string; album:string; source:string; playing:boolean; positionSeconds:number|null; durationSeconds:number|null; artwork:string|null }
export type MediaAction="toggle"|"next"|"previous";
export interface WebSource { title:string; url:string }
export interface WebSearchResult { answer:string; sources:WebSource[]; queries:string[] }
export interface Conversation { id:string; title:string; createdAt:string; updatedAt:string }
export type ConversationRole="user"|"assistant"|"system";
/** Um passo do agente numa resposta (o que ele fez), como as linhas de ferramenta do Claude. */
export interface AgentStep { id:number; tool:string; label:string; target?:string; detail?:string; added?:number; removed?:number; status:"running"|"ok"|"error"|"denied" }
export interface ConversationMessage { id:string; conversationId:string; role:ConversationRole; content:string; createdAt:string; steps?:AgentStep[] }
export interface ConversationMessageInput { id:string; conversationId:string; role:ConversationRole; content:string; activity?:string|null }
export type AIGenerationProfile="standard"|"repetition-retry";
export interface AIRequest { model:string; messages:Array<{role:ConversationRole;content:string}>; timeoutSeconds:number; generationProfile?:AIGenerationProfile; structuredOutputSchema?:Record<string,unknown>|null; requestMetadata?:unknown }
export interface AIResponse { content:string; model:string; truncated:boolean }
// Formato nativo do Ollama para tool calling (snake_case no fio).
// id/extra_content vêm do Gemini (thought_signature) e precisam voltar intactos na rodada seguinte.
export interface AgentToolCall { function:{ name:string; arguments:Record<string,unknown> }; id?:string; extra_content?:unknown }
export interface AgentMessage { role:"system"|"user"|"assistant"|"tool"; content:string; tool_calls?:AgentToolCall[]; tool_name?:string }
export interface AgentToolSchema { type:"function"; function:{ name:string; description:string; parameters:Record<string,unknown> } }
export interface AgentRequest { model:string; messages:AgentMessage[]; tools:AgentToolSchema[]; timeoutSeconds:number }
export interface AgentResponse extends AIResponse { toolCalls:AgentToolCall[] }
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

export type AIToolName="get_today_tasks"|"get_overdue_tasks"|"get_upcoming_tasks"|"get_critical_tasks"|"get_priority_tasks"|"complete_task"|"get_recent_notes"|"get_daily_operations_summary"|"get_system_status"|"get_cpu_status"|"get_memory_status"|"get_storage_status"|"get_network_status"|"get_process_summary"|"get_ollama_status"|"get_azriel_status"|"get_azriel_version"|"search_web"|"remember"|"update_memory"|"search_memories"|"create_task"|"update_task"|"delete_task"|"create_note"|"update_note"|"archive_note"|"delete_note"|"get_finances"|"save_money_item"|"set_money_paid"|"delete_money_item"|"play_music"|"music_control"|"list_folder"|"read_file"|"search_code"|"write_file"|"edit_file"|"create_folder"|"delete_path"|"run_command";
export interface ConfirmationRequest { title:string; detail:string }
export interface AIToolInput { confirm?:(request:ConfirmationRequest)=>Promise<boolean>; query:string; args?:Record<string,unknown>; term?:string; conversationId?:string; referencePosition?:number; factor?:number; delta?:number }
export interface AIToolResult { name:AIToolName; domain:string; data:unknown; empty:boolean }
export interface RoutedIntent { intent:string; scope:"azriel"|"general"; tools:AIToolName[]; term?:string; referencePosition?:number; factor?:number; delta?:number }
export type MemoryCategory="operador"|"preferencia"|"projeto"|"pessoa"|"decisao"|"fato";
export type MemorySource="azriel"|"operador";
export interface AzrielIdentity { content:string; isDefault:boolean; updatedAt:string }
export interface Memory { id:string; content:string; category:MemoryCategory; source:MemorySource; conversationId:string|null; createdAt:string; updatedAt:string }
export interface MemoryInput { id:string; content:string; category:MemoryCategory; source:MemorySource; conversationId?:string|null }
