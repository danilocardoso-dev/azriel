import { useEffect, useRef, type FormEvent, type ReactNode } from "react";
import { createPortal } from "react-dom";

interface RecordEditorDialogProps {
  eyebrow: string;
  title: string;
  busy: boolean;
  error: string | null;
  onCancel: () => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  children: ReactNode;
}

export function RecordEditorDialog({ eyebrow, title, busy, error, onCancel, onSubmit, children }: RecordEditorDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    dialogRef.current?.querySelector<HTMLElement>("input, select, textarea")?.focus();
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape" && !busy) onCancel();
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [busy, onCancel]);

  return createPortal(
    <div className="record-editor__backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget && !busy) onCancel(); }}>
      <div className="record-editor" role="dialog" aria-modal="true" aria-labelledby="record-editor-title" ref={dialogRef}>
        <form onSubmit={onSubmit}>
          <header><span>{eyebrow}</span><button type="button" aria-label="Fechar editor" onClick={onCancel} disabled={busy}>×</button></header>
          <div className="record-editor__title"><h2 id="record-editor-title">{title}</h2><small>DADOS PERSISTIDOS NO SQLITE LOCAL</small></div>
          <div className="record-editor__fields">{children}</div>
          {error && <p className="form-error" role="alert">{error}</p>}
          <footer><button type="button" onClick={onCancel} disabled={busy}>CANCELAR</button><button type="submit" disabled={busy}>{busy ? "SALVANDO..." : "SALVAR REGISTRO"}</button></footer>
        </form>
      </div>
    </div>,
    document.body,
  );
}
