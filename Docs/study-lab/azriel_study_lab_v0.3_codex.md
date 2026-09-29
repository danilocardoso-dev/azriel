# AZRIEL STUDY LAB v0.3 — Review & Active Recall
## Prompt para Codex

## Contexto
v0.1 Study Foundation e v0.2 Knowledge Workspace estão concluídas. O fluxo atual é:
`ROADMAP → ATIVIDADE → STUDY SESSION → (POMODORO + NOTA) → CADERNO/HISTÓRICO`.

A v0.3 adiciona `APRENDER → REGISTRAR → TESTAR → REVISAR`.

Não construir Anki dentro do AZRIEL. Criar revisão simples, auditável e integrada ao contexto real de estudo. Sem Mapa Stark.

## Objetivo
Implementar `STUDY LAB v0.3 — REVIEW & ACTIVE RECALL`:
- Study Cards manuais;
- sessões de revisão;
- avaliação ERREI/DIFÍCIL/BOM/FÁCIL;
- histórico;
- scheduler determinístico simples;
- integração Roadmap/Activity/Note/StudySession;
- visão de revisão no Dashboard.

Sem IA nesta versão.

## 1. Inspeção
Antes de codificar, revisar v0.1/v0.2 reais, StudySession, StudyNote/Notebook, Roadmaps, migrations, backend Rust/Tauri, frontend/state, design system e utilities de data/hora. Repositório é fonte de verdade. Não criar arquitetura paralela.

## 2. Preservação e navegação
Não quebrar HOJE, ROADMAPS, CADERNOS, HISTÓRICO, Pomodoro, Notes ou busca.

Navegação:
- HOJE
- ROADMAPS
- CADERNOS
- REVISÃO
- HISTÓRICO

REVISÃO deve ser funcional, não placeholder.

## 3. StudyCard
Criar entidade conceitual:
- id
- deck_id nullable somente se realmente necessário
- front
- back
- status
- roadmap_id/stage_id/topic_id/activity_id nullable
- notebook_id/note_id nullable
- created_at/updated_at

Status: ACTIVE / SUSPENDED / ARCHIVED.

Front = pergunta/prompt. Back = resposta/referência.

Reutilizar Markdown/code renderer da v0.2. Sem cloze, image occlusion ou card types complexos.

## 4. Deck
Antes de criar StudyDeck, verificar se Notebook/Roadmap já agrupam adequadamente. Evitar entidade redundante. Se necessário, criar deck mínimo (id/title/description/status/timestamps), sem nesting. Documentar decisão.

## 5. Criação manual e contexto
Criar cards:
- em REVISÃO;
- a partir de Note;
- a partir de Activity/Topic quando útil.

Note → `CRIAR STUDY CARD`: pré-associar note_id e contexto disponível, mas usuário escreve front/back. Não gerar conteúdo automaticamente.

Activity View: `STUDY CARDS RELACIONADOS`, com visualizar/criar, sem poluir árvore do Roadmap.

## 6. ReviewState
Criar estado persistido por card:
- card_id
- due_at
- last_reviewed_at nullable
- review_count
- correct_count
- incorrect_count
- current_interval
- learning/status quando necessário
- scheduler_version
- updated_at

Evitar métricas redundantes quando facilmente calculáveis.

## 7. ReviewEvent
Toda avaliação gera evento auditável:
- id
- card_id
- review_session_id
- reviewed_at
- result
- response_time_ms nullable
- previous_due_at
- next_due_at
- previous_interval
- next_interval
- scheduler_version

Result estável internamente:
- AGAIN
- HARD
- GOOD
- EASY

UI:
- ERREI
- DIFÍCIL
- BOM
- FÁCIL

## 8. Scheduler V1
Criar `STUDY_REVIEW_SCHEDULER_V1`, determinístico, simples e versionado.

Sem FSRS completo nesta versão.

Semântica:
- AGAIN → revisão próxima
- HARD → intervalo curto
- GOOD → intervalo cresce
- EASY → intervalo cresce mais

Centralizar parâmetros/config. Documentar fórmula/intervalos e garantir testes com clock controlado.

Não otimizar parâmetros usando desempenho do usuário nesta versão.

## 9. Sem Mastery
Não mostrar `Mastery 78%`.

Mostrar apenas fatos:
- reviews;
- correct/incorrect;
- historical success rate;
- due;
- last reviewed.

Success rate não significa domínio.

## 10. StudyReviewSession
Criar:
- id
- started_at
- ended_at nullable
- status
- planned_cards nullable
- reviewed_cards
- study_session_id nullable quando útil
- created_at

Status: ACTIVE / COMPLETED / CANCELLED.

StudySession = tempo geral de foco.
StudyReviewSession = sequência de cards.
Podem se relacionar, mas não duplicar tempo.

## 11. Fila de revisão
Prioridade determinística:
1. overdue;
2. due today;
3. new cards quando permitido.

Ordenação interna estável. Sem randomização default e sem IA.

Sessão: 10 / 20 / todos vencidos, ou equivalente simples.

## 12. Tela REVISÃO
Dashboard enxuto:
- VENCIDOS
- HOJE
- NOVOS
- `INICIAR REVISÃO`
- última revisão

Estado vazio claro e sem números fictícios.

## 13. Review Player
Fluxo:
`PERGUNTA → MOSTRAR RESPOSTA → ERREI/DIFÍCIL/BOM/FÁCIL → persistir → próximo card`.

Usuário se autoavalia. Não comparar resposta semanticamente e não usar LLM.

Código Markdown é renderizado, nunca executado.

## 14. Atomicidade e duplo submit
`submit_review_result` deve, idealmente numa transação:
1. inserir ReviewEvent;
2. atualizar ReviewState;
3. atualizar ReviewSession;
4. commit.

Falha não pode deixar metade persistida.

Evitar duplo clique gerando dois eventos. Implementar proteção simples coerente com arquitetura.

## 15. Review Summary
Ao concluir, mostrar dados reais:
- cards revisados;
- AGAIN/HARD/GOOD/EASY;
- duração;
- próxima revisão quando calculável.

Sem XP, score arbitrário ou gamificação.

## 16. Dashboard HOJE
Adicionar seção pequena `REVISÃO`:
- due cards;
- reviewed today;
- `REVISAR AGORA`.

Não sobrecarregar.

## 17. Histórico
Integrar ReviewSessions ao histórico ou filtro apropriado, preservando diferença de StudySession.

Exemplo: `REVIEW · Cybersecurity · 18 cards · 12 min`.

## 18. Card Browser
Permitir:
- listar;
- buscar;
- editar;
- suspender/reativar;
- arquivar;
- excluir com confirmação quando seguro.

Filtros mínimos: due/new/suspended e contexto quando útil.

SUSPENDED não entra na fila e preserva histórico. ARCHIVED preserva histórico e não entra em revisão normal.

Editar front/back não apaga histórico.

## 19. Busca
Busca textual simples em front/back, reutilizando padrões de Notes. Sem embeddings/semantic search.

## 20. SQLite / Backend
Migrations incrementais preservando v0.1/v0.2.

Índices somente conforme consultas reais: due_at, card_id, review_session_id, note_id, roadmap_id, status.

Operações conceituais, adaptadas ao projeto:

Cards:
- create/update/suspend/archive/get/list/search

Review:
- get_review_queue
- start_review_session
- submit_review_result
- complete/cancel_review_session
- get_review_summary
- get_review_dashboard_summary

Não criar API CRUD genérica desnecessária.

## 21. Tempo e reproducibilidade
Usar convenção temporal existente (UTC/local) de forma explícita. Não misturar silenciosamente.

Com mesmo estado do banco e mesma hora de referência, fila deve ser reproduzível.

Histórico detalhado sob demanda; não carregar todos ReviewEvents para montar fila.

## 22. Visual
Preservar AZRIEL: dark/cyan, bordas, tipografia, labels técnicos, sidebar/header e densidade atuais.

Review Player focado e sem ruído.

Proibido: confetti, XP, níveis, moedas, streak flame e estética de app gamificado.

## 23. Fora do escopo
Não implementar:
- Ollama/Tutor
- geração automática de cards
- correção de resposta por IA
- Library/PDF/image manager
- embeddings/RAG
- Mastery Engine
- Mapa Stark

v0.4 será AI Study Tools.

## 24. Testes Scheduler
Testar deterministicamente:
- new card
- AGAIN/HARD/GOOD/EASY
- progressão de intervalos
- due_at
- scheduler version
- timezone/reference clock

Sem esperar tempo real.

## 25. Testes Review
Cobrir:
- create/edit card
- suspend/reactivate/archive
- queue
- overdue priority
- due today
- new
- start session
- submit
- event/state/session persistence
- complete/cancel
- duplicate submission
- transaction rollback
- search

## 26. Teste integrado principal
`Roadmap → Activity → StudySession → Note → Create Card → Review → GOOD → next_due_at → History → Reopen Card`.

Também:
`Note → multiple cards → Review queue → Complete ReviewSession`.

## 27. Migration Safety
Banco v0.2 deve migrar preservando Roadmaps, StudySessions, Notebooks e Notes. Banco novo deve inicializar. Foreign keys válidas.

## 28. Critérios de aceite
v0.3 concluída quando:
1. REVISÃO existe;
2. v0.1/v0.2 continuam funcionando;
3. StudyCard/ReviewState/ReviewEvent/ReviewSession existem;
4. scheduler V1 versionado existe;
5. cards manuais funcionam;
6. Note→Card e Activity→Card funcionam;
7. Markdown/code funcionam;
8. fila overdue/today/new é determinística;
9. Review Player funciona;
10. reveal + quatro avaliações funcionam;
11. submit é atômico;
12. duplo submit não duplica;
13. next_due_at funciona;
14. Summary funciona;
15. HOJE mostra revisão;
16. Histórico integra ReviewSession;
17. Browser/search/suspend/archive funcionam;
18. edição preserva histórico;
19. timezone é consistente;
20. nenhuma IA/Mastery/Library/RAG/Mapa Stark entra;
21. migrations preservam dados;
22. testes/build passam;
23. Tauri inicia;
24. documentação existe.

## 29. Documentação
Criar `docs/study-lab/v0.3.md` ou padrão equivalente, documentando entidades, Scheduler V1, intervalos, queue semantics, timezone, atomicidade, integrações, testes e limitações.

Roadmap apenas como direção:
- v0.1 Study Foundation — COMPLETED
- v0.2 Knowledge Workspace — COMPLETED
- v0.3 Review & Active Recall — CURRENT
- v0.4 AI Study Tools — FUTURE
- v0.5 Study Library — FUTURE

Não criar módulos futuros agora.

## 30. Entrega Codex
Resumir arquivos/migrations, entidades, scheduler, commands/API, telas, integrações, testes, build/Tauri, limitações e itens futuros.

## Resultado esperado
Antes:
`ROADMAP → STUDY → NOTE → HISTORY`

Depois:
`ROADMAP → STUDY → NOTE → STUDY CARD → REVIEW → REVIEW EVENT → NEXT DUE → HISTORY`

A v0.3 deve criar um ciclo de Active Recall útil e confiável. Construir isso bem e parar antes de adicionar IA.
