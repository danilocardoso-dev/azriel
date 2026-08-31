import type { AIToolResult, RoutedIntent } from "../../types";
import { ToolRegistry } from "../tools/toolRegistry";

export interface BuiltContext { intent: RoutedIntent; results: AIToolResult[]; text: string; empty: boolean }

export class ContextBuilder {
  constructor(private readonly registry: ToolRegistry, private readonly maximumCharacters = 14_000) {}

  async build(query: string, intent: RoutedIntent, onTool?: (domain: string, permission: "read" | "safe_write") => void): Promise<BuiltContext> {
    const results: AIToolResult[] = [];
    for (const name of intent.tools) {
      const tool = this.registry.get(name);
      onTool?.(tool.domain, tool.permission === "safe_write" ? "safe_write" : "read");
      results.push(await this.registry.execute(name, { query, term: intent.term }));
    }
    const payload = JSON.stringify({ intent: intent.intent, searchTerm: intent.term ?? null, tools: results.map(({ name, domain, data }) => ({ name, domain, data })) }, null, 2);
    const text = payload.length > this.maximumCharacters ? `${payload.slice(0, this.maximumCharacters)}\n[CONTEXTO LIMITADO PELO SISTEMA]` : payload;
    return { intent, results, text, empty: results.length === 0 || results.every((result) => result.empty) };
  }
}
