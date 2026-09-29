# AZRIEL STUDY LAB v0.4 — AI Study Tools
## Contextual Local Tutor — Prompt para Codex

## Contexto
v0.1 Study Foundation, v0.2 Knowledge Workspace e v0.3 Review & Active Recall estão concluídas. O fluxo atual é:
`ROADMAP → ATIVIDADE → STUDY SESSION → (POMODORO + NOTE) → STUDY CARD → REVIEW → HISTORY`.

A v0.4 introduz IA local controlada e contextual. O AZRIEL já possui AI Core/Ollama: **não criar segundo subsistema de IA**.

Objetivo: IA como ferramenta de estudo, não chatbot genérico.

## 1. Objetivo
Implementar `STUDY LAB v0.4 — AI STUDY TOOLS`:
- EXPLICAR
- RESUMIR
- ME TESTAR
- GERAR STUDY CARDS
- AVALIAR RESPOSTA

Fluxo:
`CONTEÚDO → CONTEXT BUILDER → AI ACTION → AI CORE/OLLAMA → STRUCTURED RESULT → USUÁRIO REVISA → AÇÃO EXPLÍCITA`.

IA propõe; usuário decide o que persiste.

## 2. Inspeção obrigatória
Antes de codificar, revisar v0.1–v0.3 reais, AI Core, provider/interface Ollama, model config, timeout/retry, logger, prompts, StudyNote/Card/Review, frontend/state e design system. Repositório é fonte de verdade.

Não duplicar provider nem criar cliente HTTP Ollama paralelo.

## 3. Preservação
Não quebrar Roadmaps, Pomodoro, StudySession, Cadernos, Notes, Markdown, busca, Study Cards, Review Scheduler/Sessions ou Histórico. v0.4 é aditiva.

## 4. Não criar chat genérico
A UI deve privilegiar ações contextuais:
`EXPLICAR | RESUMIR | ME TESTAR | GERAR CARDS`.

Pergunta livre contextual pode existir se encaixar bem, mas não substituir as ferramentas.

## 5. StudyAIContext
Criar estrutura conceitual contendo somente o disponível/necessário:
- roadmap/stage/topic/activity;
- notebook;
- note title/content;
- selected text;
- study session context;
- related card;
- review question;
- user answer.

## 6. StudyAIContextBuilder
Responsável por selecionar apenas contexto necessário, impor limites, serializar deterministicamente e evitar dump do banco.

Prioridade:
- seleção de texto > contexto mínimo relacionado;
- ação sobre nota > note content/title + metadata mínima;
- review > question/expected answer/user answer + contexto somente se necessário.

Criar limite configurável. Se excedido, truncar explicitamente quando seguro ou bloquear com mensagem. Nunca esconder truncamento. Registrar `context_truncated=true`.

Sem RAG.

## 7. StudyAIAction / Request / Result
Actions:
- EXPLAIN
- SUMMARIZE
- QUIZ
- GENERATE_CARDS
- EVALUATE_ANSWER
- FREE_QUESTION apenas se realmente útil.

Request: action, context, user_instruction nullable, model/config, prompt_version.

Result: action, status, structured payload/content, model, prompt_version, latency_ms, context_truncated, error.

Status: SUCCESS / INVALID_OUTPUT / TIMEOUT / PROVIDER_ERROR / CANCELLED.

## 8. Prompts versionados
Criar prompts separados seguindo padrão do AZRIEL:
- STUDY_AI_EXPLAIN_V1
- STUDY_AI_SUMMARIZE_V1
- STUDY_AI_QUIZ_V1
- STUDY_AI_GENERATE_CARDS_V1
- STUDY_AI_EVALUATE_ANSWER_V1

Não usar prompt universal gigante.

## 9. Grounding
Para ações baseadas em Note/Activity/seleção, instruir modelo a:
- usar material fornecido como base;
- preservar terminologia;
- não afirmar que algo estava no material quando não estava;
- sinalizar inferência/conhecimento adicional;
- admitir contexto insuficiente.

## 10. EXPLICAR
Entrada: selected text ou Note/Topic/Activity.

UI apresenta resultado com:
- COPIAR
- INSERIR NA NOTA
- DESCARTAR

Inserção é explícita. Não sobrescrever nota automaticamente.

## 11. RESUMIR
Entrada: Note, selected text ou Activity content.

Resultado não substitui original. Permitir copiar/inserir/criar nova nota somente mediante ação explícita.

## 12. ME TESTAR / QUIZ
Gerar temporariamente 3/5/10 perguntas, com structured output:
`[{question, expected_answer}]`.

Fluxo: QUESTION → USER ANSWER → EVALUATE_ANSWER → FEEDBACK → NEXT.

Quiz temporário não cria StudyCards nem altera Review Scheduler automaticamente.

## 13. EVALUATE_ANSWER
Recebe question, expected_answer, user_answer e contexto mínimo.

Structured result:
- assessment: CORRECT / PARTIALLY_CORRECT / INCORRECT / INSUFFICIENT_CONTEXT
- explanation
- missing_points
- strengths quando aplicável
- suggested_answer

Sem score 0–100. Avaliação é assistência local, não verdade acadêmica absoluta.

## 14. GERAR STUDY CARDS
Entrada: Note, selected text ou Topic/Activity.

Saída validada: `cards:[{front,back}]`.

Limite inicial: 3/5/10.

Gerados entram em `AI GENERATED CARD DRAFTS`, nunca diretamente em StudyCard.

Usuário pode editar, selecionar/desmarcar, descartar e `SALVAR SELECIONADOS`. Só então persistir StudyCards.

Preservar source trace (note_id/activity_id/topic_id) quando aplicável.

## 15. Alteração de Notes
Qualquer escrita de IA exige ação explícita. Se editor suportar com segurança: INSERIR NO FINAL e eventualmente SUBSTITUIR SELEÇÃO. Caso contrário, implementar somente operação segura.

Nunca sobrescrever nota inteira silenciosamente.

## 16. AI Tool Panel
Na Note/Activity, painel discreto:
`AZRIEL AI — EXPLICAR | RESUMIR | ME TESTAR | GERAR CARDS`.

Pode ser lateral/modal/expansível conforme design real. Conteúdo continua prioridade visual.

## 17. Pomodoro e Review
Usar IA durante StudySession não pausa/reinicia Pomodoro nem cria nova StudySession.

Review v0.3 continua determinístico. Não substituir ERREI/DIFÍCIL/BOM/FÁCIL por LLM.

Pode existir `EXPLICAR RESPOSTA` após revelar card, sem alterar scheduler automaticamente.

## 18. Modelo/provider
Reutilizar configuração/model selection do AI Core. Não hardcode qwen se já existe config.

Não adicionar API cloud. Se AI Core suporta providers múltiplos, usar somente provider explicitamente configurado; nunca trocar silenciosamente.

## 19. Timeout / erro
Reutilizar timeout/retry do AI Core.

Tratar UI:
- loading
- timeout
- Ollama offline
- modelo indisponível
- invalid output
- cancelamento quando suportado

Nunca travar Study Lab indefinidamente.

## 20. Structured Output
QUIZ, GENERATE_CARDS e EVALUATE_ANSWER devem usar payload estruturado e validado. Evitar parsing frágil.

Invalid output: no máximo reparo/retry simples conforme padrão existente; caso contrário erro. Nunca persistir payload parcial.

## 21. Logging e privacidade
Registrar metadata técnica: action/model/prompt version/latency/status/error/context_truncated.

Evitar logar conteúdo integral de Notes/respostas sem necessidade.

Não criar banco gigante de conversas. Conteúdo gerado persiste apenas quando usuário o transforma explicitamente em Note/Card existente.

## 22. Sem autonomia
Não criar tutor autônomo, background agent, planner AI, agente que altera Roadmap, cria cards sozinho ou agenda reviews.

Toda ação é iniciada pelo usuário.

## 23. Fora do escopo
Não implementar:
- RAG/embeddings/vector DB
- Study Library/PDF/image ingestion
- Mastery artificial
- cloud AI nova
- Mapa Stark

v0.5 será Study Library.

## 24. Performance
Chamadas assíncronas. Não bloquear UI. Contexto mínimo. Não manter múltiplos modelos Ollama carregados por causa do Study Lab.

## 25. Testes Context Builder
Testar selected text, Note, Activity, Topic, Review, missing context, truncation, limits, deterministic serialization e ausência de dump indevido.

## 26. Testes AI
Com provider mock/fake:
- EXPLAIN/SUMMARIZE requests
- prompt version
- success/timeout/offline/invalid
- inserção requer ação explícita

Não depender de Ollama real em unit tests.

## 27. Testes Quiz
Testar 3/5/10, structured validation, temporary quiz, evaluation, navigation/completion e ausência de mutação no Review Scheduler.

## 28. Testes Generated Cards
Drafts, edit, deselect, discard, save selected, source relation, duplicate-submit protection e invalid output sem persistência.

## 29. Integração principal
`Roadmap → Activity → StudySession → Note → Select text → EXPLAIN → Insert explicitly → GENERATE_CARDS → Review drafts → Save selected → Review queue`.

Outro:
`Note → ME TESTAR → answer → AI evaluation → finish → scheduler unchanged`.

## 30. Migration
Evitar novas tabelas se AI metadata não precisar persistir. Se necessária, migration incremental preservando v0.1–v0.3. Não criar tabela genérica de chat.

## 31. Critérios de aceite
v0.4 concluída quando:
1. AI Core existente é reutilizado;
2. não existe segundo Ollama client;
3. StudyAIContext/ContextBuilder existem;
4. limits/truncation explícitos existem;
5. actions e prompts versionados existem;
6. EXPLAIN/SUMMARIZE/QUIZ/EVALUATE_ANSWER/GENERATE_CARDS funcionam;
7. structured outputs são validados;
8. generated cards são drafts revisáveis;
9. usuário confirma qualquer persistência;
10. source trace existe quando aplicável;
11. IA não quebra Pomodoro;
12. IA não altera Review Scheduler;
13. offline/timeout/invalid output são tratados;
14. UI permanece responsiva;
15. logging técnico existe sem conteúdo excessivo;
16. nenhuma cloud API/RAG/autonomous agent/Library/Mastery é introduzida;
17. v0.1–v0.3 permanecem funcionais;
18. testes/build passam;
19. Tauri inicia;
20. documentação existe.

## 32. Documentação
Criar `docs/study-lab/v0.4.md` (ou padrão real), documentando arquitetura de integração, ContextBuilder, context limits, actions/prompts, structured outputs, persistence rules, errors, privacy, testes e limitações.

Roadmap:
- v0.1 Study Foundation — COMPLETED
- v0.2 Knowledge Workspace — COMPLETED
- v0.3 Review & Active Recall — COMPLETED
- v0.4 AI Study Tools — CURRENT
- v0.5 Study Library — FUTURE

## 33. Entrega Codex
Resumir arquivos/migrations, integração com AI Core, ContextBuilder, prompts, actions, UI, structured contracts, error handling, testes, build/Tauri, limitações e itens futuros.

## Resultado esperado
Antes:
`ESTUDAR → REGISTRAR → REVISAR`

Depois:
`ESTUDAR → REGISTRAR → [AZRIEL AI CONTEXTUAL] → TESTAR/EXPLICAR/RESUMIR/GERAR DRAFTS → USUÁRIO REVISA → PERSISTE → REVISAR`

A IA deve reduzir fricção sem tomar controle do Study Lab. Construir ferramentas contextuais confiáveis e parar antes de RAG/Library/autonomia.
