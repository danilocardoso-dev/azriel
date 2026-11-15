import { memoryRepository } from "../repositories/memoryRepository";
import type { AzrielIdentity, Memory } from "../types";
import type { ToolRegistry } from "./tools/toolRegistry";

/** Teto das memórias no contexto: as mais recentes entram; o resto ele acha com search_memories. */
export const PERSONA_MEMORY_CHARACTERS = 6_000;

const CATEGORY_LABEL: Record<string, string> = { operador: "sobre o Senhor", preferencia: "preferência", projeto: "projeto", pessoa: "pessoa", decisao: "decisão", fato: "fato" };

const MEMORY_RULES = `COMO USO MINHA MEMÓRIA
- As memórias abaixo são o que sei do Senhor e da nossa história. Uso-as naturalmente, sem anunciar que estou "consultando a memória".
- Quando o Senhor me conta algo duradouro (quem ele é, preferências, projetos, pessoas, datas, decisões, metas), registro com a ferramenta remember, sem pedir licença, e confirmo em poucas palavras.
- Quando algo muda ou uma memória está errada, corrijo com update_memory usando o id dela. Não crio duplicatas.
- Não registro trivialidades, conversas passageiras nem o conteúdo de buscas na web.
- Se o assunto pode estar em memórias que não aparecem aqui, procuro com search_memories antes de dizer que não sei.
- Apagar memórias cabe ao Senhor, na tela de Configurações.`;

/** Idade em anos completos a partir de "dd/mm/aaaa" no texto da identidade, para ele não depender de um número fixo. */
export function ageFrom(identity: string, today: Date): number | null {
  const match = identity.match(/nascid[oa] em (\d{2})\/(\d{2})\/(\d{4})/i);
  if (!match) return null;
  const [day, month, year] = [Number(match[1]), Number(match[2]), Number(match[3])];
  let age = today.getFullYear() - year;
  if (today.getMonth() + 1 < month || (today.getMonth() + 1 === month && today.getDate() < day)) age -= 1;
  return age >= 0 && age < 150 ? age : null;
}

export function formatMemories(memories: Memory[], budget = PERSONA_MEMORY_CHARACTERS): { text: string; shown: number } {
  const lines: string[] = [];
  let used = 0;
  for (const memory of memories) {
    const line = `- [${memory.id}] (${CATEGORY_LABEL[memory.category] ?? memory.category}) ${memory.content}`;
    if (used + line.length > budget) break;
    lines.push(line);
    used += line.length + 1;
  }
  return { text: lines.join("\n"), shown: lines.length };
}

export interface PersonaInput { identity: string; memories: Memory[]; capabilities: string[]; now: Date }

/** Texto de sistema que abre toda conversa (chat e voz): quem ele é, o que sabe fazer e o que lembra. */
export function buildPersona({ identity, memories, capabilities, now }: PersonaInput): string {
  const age = ageFrom(identity, now);
  const { text, shown } = formatMemories(memories);
  const hidden = memories.length - shown;
  const parts = [
    identity.trim(),
    age != null ? `(Hoje o Senhor tem ${age} anos.)` : "",
    capabilities.length ? `O QUE POSSO FAZER AGORA\n${capabilities.map((item) => `- ${item}`).join("\n")}\nSe me pedirem algo fora disso, digo com clareza o que falta e sugiro como resolver; não digo que "não fui projetado" para algo que está nesta lista.` : "",
    MEMORY_RULES,
    `MINHAS MEMÓRIAS (${memories.length})\n${text || "Ainda não tenho memórias registradas."}${hidden > 0 ? `\n(+${hidden} memórias mais antigas: use search_memories.)` : ""}`,
  ];
  return parts.filter(Boolean).join("\n\n");
}

/** Capacidades reais, tiradas das ferramentas que o agente pode usar (agrupadas por área). */
export function capabilitiesOf(registry: ToolRegistry): string[] {
  const groups = new Map<string, string[]>();
  for (const tool of registry.list()) {
    if (!registry.agentCanRun(tool.name)) continue;
    groups.set(tool.domain, [...(groups.get(tool.domain) ?? []), tool.description.split(":")[0].split(". ")[0]]);
  }
  return [...groups.entries()].map(([domain, items]) => `${domain}: ${items.join("; ")}`);
}

export interface PersonaSource {
  identity(): Promise<AzrielIdentity>;
  memories(): Promise<Memory[]>;
}

/** Monta a persona com os dados do banco. Se o banco falhar, segue sem ela em vez de derrubar a conversa. */
export async function loadPersona(registry: ToolRegistry, source: PersonaSource = { identity: memoryRepository.identity, memories: memoryRepository.list }): Promise<string> {
  const [identity, memories] = await Promise.all([source.identity().catch(() => null), source.memories().catch(() => [] as Memory[])]);
  if (!identity) return "";
  return buildPersona({ identity: identity.content, memories, capabilities: capabilitiesOf(registry), now: new Date() });
}
