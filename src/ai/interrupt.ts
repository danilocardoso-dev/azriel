import type { AgentStep } from "../types";

/** Interrupção pelo Senhor (ESC ou botão Parar), como no Claude. */
export class InterruptedError extends Error {
  /** Texto devolvido ao campo quando nada foi feito ainda; null quando a mensagem ficou na conversa. */
  constructor(readonly restoredText: string | null = null, readonly toolsUsed: string[] = [], readonly steps: AgentStep[] = []) {
    super("Interrompido pelo Senhor.");
    this.name = "InterruptedError";
  }
}

export const isInterrupted = (error: unknown): error is InterruptedError => error instanceof InterruptedError;

export function throwIfAborted(signal: AbortSignal | undefined, toolsUsed: string[] = []) {
  if (signal?.aborted) throw new InterruptedError(null, toolsUsed);
}

/**
 * Corre a promessa contra o sinal: ao parar, a espera acaba na hora. O pedido ao modelo pode até
 * terminar no backend, mas o resultado é descartado.
 */
export function abortable<T>(promise: Promise<T>, signal: AbortSignal | undefined, toolsUsed: string[] = []): Promise<T> {
  if (!signal) return promise;
  if (signal.aborted) return Promise.reject(new InterruptedError(null, toolsUsed));
  return new Promise<T>((resolve, reject) => {
    const onAbort = () => reject(new InterruptedError(null, toolsUsed));
    signal.addEventListener("abort", onAbort, { once: true });
    promise.then(
      (value) => { signal.removeEventListener("abort", onAbort); resolve(value); },
      (error) => { signal.removeEventListener("abort", onAbort); reject(error); },
    );
  });
}
