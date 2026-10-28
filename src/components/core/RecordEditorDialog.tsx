import { useEffect, useRef, type FormEvent, type ReactNode } from "react";
import { createPortal } from "react-dom";
import "../../styles/record-editor.css";

interface RecordEditorDialogProps {
  eyebrow: string;
  title: string;
  busy: boolean;
  error: string | null;
  onCancel: () => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  children: ReactNode;
  /** Linha de metadados abaixo do título (datas de criação, conclusão...). */
  meta?: ReactNode;
  /** Ações do registro à esquerda do rodapé (concluir, arquivar, excluir). */
  actions?: ReactNode;
  submitLabel?: string;
}

export function RecordEditorDialog({ eyebrow, title, busy, error, onCancel, onSubmit, children, meta, actions, submitLabel = "SALVAR" }: RecordEditorDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    dialogRef.current?.querySelector<HTMLElement>("input, select, textarea")?.focus();
  }, []);

  useEffect(() => {
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
          <header><span>{eyebrow}</span><button type="button" aria-label="Fechar" onClick={onCancel} disabled={busy}>×</button></header>
          <div className="record-editor__title"><h2 id="record-editor-title">{title}</h2>{meta && <small>{meta}</small>}</div>
          <div className="record-editor__fields">{children}</div>
          {error && <p className="record-editor__error" role="alert">{error}</p>}
          <footer>
            <div className="record-editor__actions">{actions}</div>
            <button type="button" onClick={onCancel} disabled={busy}>FECHAR</button>
            <button type="submit" className="record-editor__primary" disabled={busy}>{busy ? "SALVANDO..." : submitLabel}</button>
          </footer>
        </form>
      </div>
    </div>,
    document.body,
  );
}
