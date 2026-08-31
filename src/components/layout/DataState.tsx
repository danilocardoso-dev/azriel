interface DataStateProps { loading: boolean; error: string | null; empty?: boolean; onRetry: () => void }
export function DataState({ loading, error, empty, onRetry }: DataStateProps) {
  if (loading) return <div className="data-state"><span className="live-dot" /><strong>CARREGANDO BANCO LOCAL</strong><p>Sincronizando o HUD com o SQLite.</p></div>;
  if (error) return <div className="data-state data-state--error"><strong>FALHA NO BANCO LOCAL</strong><p>{error}</p><button onClick={onRetry}>TENTAR NOVAMENTE</button></div>;
  if (empty) return <div className="data-state"><strong>NENHUM REGISTRO</strong><p>Esta área ainda não possui dados persistidos.</p></div>;
  return null;
}
