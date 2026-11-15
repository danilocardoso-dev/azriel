import { useEffect, useState } from "react";
import { memoryRepository } from "../../repositories/memoryRepository";
import { formatTimestamp } from "../../services/dateService";
import type { AzrielIdentity } from "../../types";
import "../../styles/mind.css";

const messageOf = (reason: unknown) => reason instanceof Error ? reason.message : String(reason);

/** Núcleo de identidade: quem o Azriel é. Só o Senhor edita; vale para o chat e a voz. */
export function IdentityPanel() {
  const [identity, setIdentity] = useState<AzrielIdentity | null>(null);
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState<"save" | "reset" | null>(null);
  const [message, setMessage] = useState<{ kind: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    let active = true;
    void memoryRepository.identity()
      .then((loaded) => { if (active) { setIdentity(loaded); setDraft(loaded.content); } })
      .catch((reason) => { if (active) setMessage({ kind: "error", text: messageOf(reason) }); });
    return () => { active = false; };
  }, []);

  async function run(kind: "save" | "reset", action: () => Promise<AzrielIdentity>, done: string) {
    setBusy(kind); setMessage(null);
    try { const saved = await action(); setIdentity(saved); setDraft(saved.content); setMessage({ kind: "success", text: done }); }
    catch (reason) { setMessage({ kind: "error", text: messageOf(reason) }); }
    finally { setBusy(null); }
  }

  const dirty = identity !== null && draft.trim() !== identity.content.trim();

  return <section className="ai-settings mind-panel">
    <header><span>NÚCLEO DE IDENTIDADE // QUEM O AZRIEL É</span><i data-state={identity ? "online" : "offline"}>{!identity ? "CARREGANDO" : identity.isDefault ? "PADRÃO" : "PERSONALIZADA"}</i></header>
    <div className="mind-panel__body">
      <p className="mind-panel__hint">Vale para toda conversa, no chat e na voz. O Azriel não altera este texto sozinho. Escreva a data de nascimento como "nascido em dd/mm/aaaa" para ele calcular a sua idade.</p>
      <textarea className="mind-identity" value={draft} onChange={(event) => setDraft(event.target.value)} spellCheck={false} disabled={!identity || busy !== null} aria-label="Identidade do Azriel" />
      <div className="mind-panel__actions">
        <button type="button" disabled={!dirty || busy !== null} onClick={() => void run("save", () => memoryRepository.saveIdentity(draft), "Identidade salva. Vale a partir da próxima mensagem.")}>{busy === "save" ? "SALVANDO..." : "SALVAR IDENTIDADE"}</button>
        <button type="button" className="mind-button--ghost" disabled={!dirty || busy !== null} onClick={() => identity && setDraft(identity.content)}>DESCARTAR</button>
        <button type="button" className="mind-button--ghost" disabled={!identity || identity.isDefault || busy !== null} onClick={() => void run("reset", memoryRepository.resetIdentity, "Identidade padrão restaurada.")}>{busy === "reset" ? "RESTAURANDO..." : "RESTAURAR PADRÃO"}</button>
        {identity?.updatedAt && <small>Atualizada em {formatTimestamp(identity.updatedAt)}</small>}
      </div>
      {message && <p className="mind-panel__status" role="status" data-kind={message.kind}>{message.text}</p>}
    </div>
  </section>;
}
