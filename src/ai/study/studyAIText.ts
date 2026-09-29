export function appendStudyAIContent(original: string, generated: string): string {
  const addition = generated.trim();
  if (!addition) return original;
  const base = original.trimEnd();
  return `${base}${base ? "\n\n" : ""}${addition}\n`;
}
