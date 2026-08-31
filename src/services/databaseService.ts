import { invokeDatabase } from "../repositories/tauri";
import type { DatabaseInfo } from "../types";
export const getDatabaseInfo = () => invokeDatabase<DatabaseInfo>("database_info");
