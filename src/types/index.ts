export type ModuleId = "command" | "ai" | "daily" | "music" | "money" | "settings";
export type AzrielState = "idle" | "processing" | "tool" | "executing" | "engineering" | "routine" | "alert" | "offline";
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

export interface NowPlaying { title:string; artist:string; album:string; source:string; playing:boolean; positionSeconds:number|null; durationSeconds:number|null; artwork:string|null }
export type MediaAction="toggle"|"next"|"previous";
export interface Conversation { id:string; title:string; createdAt:string; updatedAt:string }
export type ConversationRole="user"|"assistant"|"system";
export interface ConversationMessage { id:string; conversationId:string; role:ConversationRole; content:string; createdAt:string }
export interface ConversationMessageInput { id:string; conversationId:string; role:ConversationRole; content:string }

export interface SystemDetails { osName:string|null; osVersion:string|null; kernelVersion:string|null; architecture:string; hostname:string|null; logicalCores:number; physicalCores:number|null; uptimeSeconds:number }
export interface CpuSnapshot { usagePercent:number; cores:number[] }
export interface MemorySnapshot { totalBytes:number; usedBytes:number; availableBytes:number; swapTotalBytes:number; swapUsedBytes:number }
export interface StorageSnapshot { name:string; mountPoint:string; fileSystem:string; totalBytes:number; availableBytes:number; removable:boolean }
export interface NetworkSnapshot { interfaceName:string; receivedBytes:number; transmittedBytes:number; receivedBytesTotal:number; transmittedBytesTotal:number }
export interface ProcessSnapshot { pid:number; name:string; cpuPercent:number; memoryBytes:number }
export interface SystemSnapshot { collectedAt:number; details:SystemDetails; cpu:CpuSnapshot; memory:MemorySnapshot; storage:StorageSnapshot[]; network:NetworkSnapshot[]; errors:string[] }

