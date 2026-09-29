# AZRIEL STUDY LAB v0.2 — Knowledge Workspace
## Prompt para Codex

## Contexto
A v0.1 — Study Foundation está concluída. O fluxo atual é:
`ROADMAP → TÓPICO/ATIVIDADE → STUDY SESSION → POMODORO → TEMPO/PROGRESSO → HISTÓRICO`.

A v0.2 adiciona a camada `APRENDER → ESTUDAR → REGISTRAR`.

O AZRIEL permanece enxuto. O Mapa Stark foi descontinuado. O Market Lab é separado. Study Lab não deve virar Notion/Obsidian/OneNote.

## Objetivo
Implementar `STUDY LAB v0.2 — KNOWLEDGE WORKSPACE` com:
- Cadernos;
- Notas;
- Markdown;
- blocos de código;
- vínculos opcionais com Roadmap/Stage/Topic/Activity/StudySession;
- busca textual simples;
- notas recentes;
- integração contextual com Study Sessions.

Fluxo:
`ROADMAP → ATIVIDADE → STUDY SESSION → NOTA → CADERNO`

## 1. Inspeção obrigatória
Antes de codificar, revisar implementação real da v0.1, StudySession, Roadmaps, migrations/SQLite, padrões Rust/Tauri, frontend/state management, design system e qualquer suporte Markdown/editor já existente. O repositório é a fonte de verdade. Não criar arquitetura paralela.

## 2. Preservar v0.1
Não quebrar HOJE, ROADMAPS, HISTÓRICO, Pomodoro, recovery, métricas, CONTINUAR ESTUDANDO ou dados existentes. v0.2 é aditiva.

## 3. Navegação
Evoluir Estudos para:
- HOJE
- ROADMAPS
- CADERNOS
- HISTÓRICO

Sem placeholders para Biblioteca, Cards ou Tutor.

## 4. StudyNotebook
Criar entidade conceitual `StudyNotebook`:
- id
- title
- description nullable
- status
- created_at
- updated_at

Status simples: ACTIVE / ARCHIVED. Sem árvore infinita, tags ou propriedades customizadas.

## 5. StudyNote
Criar `StudyNote`:
- id
- notebook_id
- title
- content
- content_format (`MARKDOWN`)
- roadmap_id nullable
- stage_id nullable
- topic_id nullable
- activity_id nullable
- study_session_id nullable
- created_at
- updated_at

Usar IDs reais para relações.

Conceitos permanecem separados:
- Roadmap = o que estudar
- Notebook = onde organizar conhecimento
- StudySession = quando/quanto estudou
- Note = o que registrou

## 6. Cadernos UI
Estado vazio funcional e ação `+ NOVO CADERNO`.

Lista deve mostrar título, descrição curta, número de notas, última atualização e status. Operações: CREATE, EDIT/RENAME, ARCHIVE e RESTORE quando simples. Preferir archive à exclusão destrutiva.

Dentro do caderno, listar notas por `updated_at DESC`, com título, contexto quando houver, updated_at e preview curto.

## 7. Editor
Editor Markdown simples e confiável:
- título
- conteúdo Markdown
- preview/renderização
- save
- estado SALVANDO / SALVO / ERRO

Não criar WYSIWYG complexo.

Suportar headings, parágrafos, bold/italic, listas, blockquote, inline code, fenced code blocks, links e horizontal rule.

Reutilizar parser existente. Se não houver, usar dependência pequena e consolidada somente se necessário.

Não renderizar HTML/script arbitrário inseguro.

## 8. Code blocks
Fenced code blocks devem preservar whitespace, linguagem informada e scroll horizontal. Syntax highlighting é opcional; não adicionar dependência pesada apenas para isso.

## 9. Persistência / Autosave
Preferência: autosave com debounce e estado visual, adaptado ao padrão real do projeto. Não gravar SQLite a cada tecla.

Não perder conteúdo silenciosamente ao navegar. Em falha, mostrar erro; nunca exibir SALVO sem persistência confirmada.

Definir claramente conteúdo local, persisted note e save status.

## 10. Nota contextual
Criar nota a partir de Roadmap/Topic/Activity deve pré-associar IDs daquele contexto sem copiar automaticamente o conteúdo do roadmap.

Durante StudySession ativa, oferecer `ABRIR / CRIAR NOTA`. Nota criada nesse fluxo associa `study_session_id` e herda contexto disponível.

Criar/editar nota não pode interromper Pomodoro.

## 11. Activity View
Adicionar `NOTAS RELACIONADAS` na visão da atividade, com abrir/criar nota. Não carregar caderno inteiro.

No Roadmap/Topic, permitir acesso contextual às notas sem poluir a árvore principal.

## 12. Busca
Busca textual simples em título/conteúdo/caderno.

Resultado:
- note title
- notebook
- trecho curto
- contexto Roadmap quando houver
- updated_at

Clique abre nota.

Sem embeddings, semantic search, vector DB ou RAG. SQLite LIKE/FTS5 somente se integrar de forma limpa.

## 13. Dashboard e Histórico
HOJE: adicionar pequena seção `NOTAS RECENTES`, limitada a poucos itens.

Ao concluir StudySession, se houver notas vinculadas, pode mostrar `N notas atualizadas`.

Histórico pode indicar existência de notas vinculadas e permitir acesso, sem duplicar conteúdo.

## 14. SQLite
Migrations incrementais e compatíveis. Preservar Roadmaps e StudySessions.

Foreign keys/índices somente conforme consultas reais, especialmente notebook_id, roadmap/topic/activity/session IDs e updated_at.

Notebook ARCHIVED preserva notas.

Se delete de nota existir, exigir confirmação e preservar integridade. Não criar lixeira/soft-delete complexo se o projeto não possui esse padrão.

## 15. Backend
Seguir arquitetura real. Operações conceituais, adaptadas ao padrão:
Notebooks: create/update/archive/list/get.
Notes: create/update/delete/get/list/search/list_for_activity/list_recent.

Não criar CRUD genérico se o projeto não usa esse estilo.

## 16. Performance
Listagens devem buscar metadata/preview, não conteúdo integral de todas as notas. Conteúdo completo só ao abrir. Busca deve ter limite razoável.

## 17. Visual
Preservar identidade AZRIEL: fundo escuro, cyan, bordas, tipografia, labels técnicos, sidebar/header, densidade e espaçamentos existentes.

Não imitar Notion, Obsidian, LMS ou SaaS colorido.

## 18. Fora do escopo
Não implementar:
- Tutor/Ollama
- Explain/Summarize/Quiz
- Study Cards
- Biblioteca/PDF/image manager
- embeddings/RAG/vector DB
- Knowledge Graph
- Mastery
- versionamento completo de notas
- tags hierárquicas
- Mapa Stark

v0.2 é um workspace confiável, não um sistema de IA.

## 19. Testes
Cobrir conforme infraestrutura:
- notebook create/update/archive/list
- note create/update/delete
- relações notebook/roadmap/activity/session
- foreign IDs inválidos
- search
- recent notes
- Markdown/code block/security
- autosave/error/reload
- migration preserva v0.1
- integração Roadmap → Activity → StudySession → Note
- Pomodoro continua ao editar
- histórico/vínculos

Não adicionar framework novo só para checklist.

## 20. Fluxo principal de validação
Validar:
`Roadmap → Activity → Start StudySession → Create Note → Edit → Pomodoro continua → Complete Session → Note vinculada → History → Reopen Note`.

## 21. Critérios de aceite
v0.2 concluída quando:
1. CADERNOS aparece na navegação;
2. v0.1 permanece funcional;
3. StudyNotebook/StudyNote existem e persistem;
4. migrations são compatíveis;
5. criar/editar/arquivar caderno funciona;
6. criar/editar/excluir nota com semântica segura funciona;
7. Markdown e fenced code blocks renderizam;
8. HTML/script arbitrário não executa;
9. save status é confiável;
10. navegação não perde conteúdo silenciosamente;
11. notas vinculam Roadmap/Topic/Activity/StudySession;
12. criar/editar nota não interrompe Pomodoro;
13. Activity View mostra notas relacionadas;
14. Dashboard mostra notas recentes;
15. Histórico reconhece vínculos;
16. busca textual funciona;
17. listas não carregam conteúdo integral desnecessariamente;
18. archive preserva notas;
19. design segue AZRIEL;
20. nenhuma IA/Library/Cards/RAG/Mapa Stark é introduzida;
21. testes/build passam;
22. Tauri inicia;
23. documentação existe.

## 22. Documentação
Criar `docs/study-lab/v0.2.md` ou caminho equivalente, documentando arquitetura, entidades, relações, schema/migration, Markdown, save semantics, search, integração com StudySession/Activity, segurança, testes e limitações.

Roadmap apenas como direção:
- v0.1 Study Foundation — COMPLETED
- v0.2 Knowledge Workspace — CURRENT
- v0.3 Review / Active Recall — FUTURE
- v0.4 AI Study Tools — FUTURE
- v0.5 Study Library — FUTURE

Não criar módulos futuros agora.

## 23. Entrega Codex
Ao concluir, resumir arquivos criados/alterados, migrations, entidades, commands/API, telas/componentes, save semantics, busca, integrações, testes, build/Tauri, limitações e itens futuros.

## Resultado esperado
Antes:
`ROADMAP → STUDY SESSION → POMODORO → HISTÓRICO`

Depois:
`ROADMAP → ATIVIDADE → STUDY SESSION → (POMODORO + NOTA) → CADERNO/HISTÓRICO`

A v0.2 deve permitir estudar e registrar conhecimento sem sair do fluxo do AZRIEL. Construir isso bem e parar; não antecipar as próximas versões.
