export function safeStudyMarkdownUrl(url: string): string {
  const value = url.trim();
  if (value.startsWith("#") || /^(https?:|mailto:)/i.test(value)) return value;
  return "";
}
