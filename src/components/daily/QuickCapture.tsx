import { useEffect, useRef, useState, type FormEvent } from "react";
import { useDailyOperations } from "../../contexts/useDailyOperations";

export function QuickCapture({ initialKind, autoFocus = false }: { initialKind?: "task" | "note"; autoFocus?: boolean }) {
  const { createQuickTask, createQuickNote, setView } = useDailyOperations();
  const [kind, setKind] = useState<"task" | "note">(initialKind ?? "task");
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const shortcut = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.key.toLowerCase() === "n") { event.preventDefault(); inputRef.current?.focus(); }
    };
    window.addEventListener("keydown", shortcut);
    return () => window.removeEventListener("keydown", shortcut);
  }, []);

  useEffect(() => { if (autoFocus) inputRef.current?.focus(); }, [autoFocus]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!value.trim()) { setMessage("Digite uma tarefa ou nota."); return; }
    setBusy(true); setMessage(null);
    try {
      if (kind === "task") { await createQuickTask(value); setView("inbox"); }
      else { await createQuickNote(value); setView("notes"); }
      setValue(""); setMessage(kind === "task" ? "Tarefa capturada." : "Nota registrada."); inputRef.current?.focus();
    } catch (error) { setMessage(error instanceof Error ? error.message : String(error)); }
    finally { setBusy(false); }
  }

  return <form className="quick-capture" onSubmit={submit}>
    <label htmlFor="daily-capture">ENTRADA RÁPIDA <kbd>CTRL + N</kbd></label>
    <input id="daily-capture" ref={inputRef} value={value} onChange={(event) => setValue(event.target.value)} placeholder="Adicionar tarefa ou nota..." maxLength={240} />
    <div><button type="button" className={kind === "task" ? "active" : ""} onClick={() => setKind("task")}>TAREFA</button><button type="button" className={kind === "note" ? "active" : ""} onClick={() => setKind("note")}>NOTA</button></div>
    <button className="quick-capture__save" disabled={busy}>{busy ? "SALVANDO..." : "CAPTURAR"}</button>
    {message && <p role="status">{message}</p>}
  </form>;
}
