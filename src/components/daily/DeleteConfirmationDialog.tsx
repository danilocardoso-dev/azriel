import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";

type DeleteConfirmationDialogProps = {
  kind: "tarefa" | "anotação" | "conversa";
  title: string;
  description?: string;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
};

export function DeleteConfirmationDialog({
  kind,
  title,
  description,
  busy,
  onCancel,
  onConfirm,
}: DeleteConfirmationDialogProps) {
  const cancelButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    cancelButtonRef.current?.focus();

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape" && !busy) onCancel();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [busy, onCancel]);

  return createPortal(
    <div
      className="delete-dialog__backdrop"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !busy) onCancel();
      }}
    >
      <section
        className="delete-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="delete-dialog-title"
        aria-describedby="delete-dialog-description"
      >
        <header>
          <span>PROTOCOLO DE EXCLUSÃO</span>
          <strong>IRREVERSÍVEL</strong>
        </header>
        <div className="delete-dialog__body">
          <span className="delete-dialog__icon" aria-hidden="true">!</span>
          <div>
            <h2 id="delete-dialog-title">Excluir {kind}?</h2>
            <p id="delete-dialog-description">
              {description ?? "Esta ação removerá definitivamente o registro do banco local e não poderá ser desfeita."}
            </p>
          </div>
        </div>
        <div className="delete-dialog__target">
          <small>REGISTRO SELECIONADO</small>
          <strong>{title}</strong>
        </div>
        <footer>
          <button ref={cancelButtonRef} type="button" onClick={onCancel} disabled={busy}>
            CANCELAR
          </button>
          <button type="button" className="danger" onClick={onConfirm} disabled={busy}>
            {busy ? "EXCLUINDO..." : "EXCLUIR DEFINITIVAMENTE"}
          </button>
        </footer>
      </section>
    </div>,
    document.body,
  );
}
