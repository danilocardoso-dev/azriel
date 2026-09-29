# AZRIEL STUDY LAB v0.1 — Study Foundation
## Prompt para Codex

## Contexto
O AZRIEL passou por uma simplificação deliberada. O Mapa Stark foi descontinuado e não deve reaparecer direta ou indiretamente. A navegação atual é enxuta: Command Center, AI Core, Operações Diárias, Projetos, Estudos, Market Lab, Automação e Configurações.

O Market Lab continua sendo o laboratório experimental. **Estudos deve ser pragmático:** uma ferramenta diária para formação e evolução profissional.

Hoje Estudos é centrado em Roadmaps. A v0.1 deve evoluir essa base sem substituí-la.

## Objetivo
Implementar `STUDY LAB v0.1 — STUDY FOUNDATION`.

Fluxo:
`ROADMAP → TÓPICO/ATIVIDADE → STUDY SESSION → POMODORO → TEMPO/PROGRESSO → HISTÓRICO`

A v0.1 deve responder:
- o que estou estudando;
- qual roadmap está ativo;
- qual atividade devo continuar;
- quanto estudei hoje;
- quantas sessões concluí;
- quanto tempo dediquei a cada roadmap;
- o que concluí recentemente.

Não implementar mastery, recomendações por IA, spaced repetition, RAG ou analytics avançado.

## 1. Inspeção obrigatória antes de codificar
Antes de modificar:
1. analisar arquitetura atual;
2. localizar Estudos/Roadmaps;
3. localizar schema/migrations;
4. identificar padrões Rust/Tauri/SQLite;
5. identificar frontend, stores/hooks/services;
6. identificar design system/tokens/componentes;
7. registrar um plano curto de integração.

Não reescrever Roadmaps funcionais. Não criar arquitetura paralela ou abstrações desnecessárias.

## 2. Preservar Roadmaps
Preservar criação, etapas, tópicos, atividades, progresso, estados, persistência e interação atuais. Migrations devem ser compatíveis com dados existentes.

Roadmaps são o eixo do Study Lab.

## 3. Navegação
Na v0.1:
- `HOJE`
- `ROADMAPS`
- `HISTÓRICO`

Pomodoro é ferramenta contextual dentro de Estudos, não um grande módulo separado.

Não criar placeholders vazios para Cadernos, Biblioteca, Cards ou Tutor.

## 4. Dashboard — HOJE
Mostrar apenas dados úteis.

### Foco atual
- roadmap;
- etapa/tópico;
- atividade;
- progresso existente;
- `CONTINUAR ESTUDANDO`.

### Hoje
- minutos de foco;
- sessões concluídas;
- atividades concluídas;
- roadmap mais estudado, quando houver.

### Roadmaps ativos
Para poucos itens prioritários:
- título;
- progresso;
- próxima atividade;
- último estudo.

### Atividade recente
Últimas sessões/conclusões relevantes.

Sem dashboard analítico excessivo.

### Estado vazio
Quando não houver roadmap:
`Nenhum roadmap ativo. Crie um roadmap para iniciar seu primeiro caminho de estudo.`
`[ + NOVO ROADMAP ]`

Não mostrar métricas falsas.

## 5. Foco atual
Criar conceito simples para recuperar contexto atual. Antes de nova tabela, verificar se schema existente atende.

Quando necessário, representar:
- active_roadmap_id;
- active_stage_id;
- active_topic_id;
- active_activity_id;
- updated_at.

Não criar sistema complexo de prioridades.

## 6. StudySession
Criar entidade central `StudySession`.

Campos mínimos conceituais:
- id;
- roadmap_id (nullable quando necessário);
- stage_id;
- topic_id;
- activity_id;
- started_at;
- ended_at;
- planned_focus_minutes;
- actual_focus_seconds;
- break_seconds;
- status;
- created_at;
- updated_at.

Adaptar naming ao projeto real.

Status:
- ACTIVE
- PAUSED
- COMPLETED
- CANCELLED

Default: somente uma sessão ACTIVE/PAUSED por vez.

Uma sessão pode estar vinculada a `Roadmap → Stage → Topic → Activity`.

## 7. Pomodoro
Operações:
- START
- PAUSE
- RESUME
- COMPLETE
- CANCEL/RESET

Configuração inicial simples:
- focus duration;
- short break duration.

Não implementar gamificação, streaks, achievements, long-break engine complexo, cloud sync ou notificações avançadas.

### Timer confiável
Não depender apenas de decremento por `setInterval`.

Usar timestamps/tempo acumulado suficientes para reconstrução correta. A UI pode atualizar por tick, mas a fonte temporal deve ser derivada do relógio.

Pausa não acumula foco.

### Navegação/recovery
A sessão deve sobreviver à navegação interna de Estudos sem duplicação.

Quando coerente com arquitetura existente, recuperar sessão após reabrir o app. Não inventar tempo estudado.

Definir uma única fonte de verdade para estado persistido/derivado/visual.

## 8. Pomodoro contextual
Ao iniciar de uma atividade, mostrar:
- Roadmap;
- tópico;
- atividade;
- tempo de foco.

O timer deve parecer parte do Study Lab, não cronômetro genérico.

Ao completar:
`SESSÃO CONCLUÍDA`
`25 min de foco`
+ contexto estudado.

**Concluir StudySession NÃO conclui automaticamente a atividade.** Conclusão acadêmica continua explícita.

## 9. Histórico
Criar `HISTÓRICO` com:
- data;
- roadmap;
- tópico/atividade;
- duração;
- status.

Agrupar por dia quando simples.

Filtros mínimos: período e roadmap, somente se úteis.

Usar paginação/limite; não carregar histórico inteiro sem necessidade.

## 10. Métricas determinísticas
Calcular:
- focus_minutes_today;
- completed_sessions_today;
- completed_activities_today;
- focus_minutes_by_roadmap;
- last_study_at.

Não persistir agregados desnecessários se SQL simples resolver.

Progresso do Roadmap e tempo estudado são métricas diferentes.

## 11. CONTINUAR ESTUDANDO
Comportamento determinístico:
1. atividade ativa;
2. próxima atividade não concluída;
3. tópico atual;
4. roadmap.

Não usar IA para escolher.

## 12. Activity View
Se encaixar naturalmente, criar visão contextual simples:
- breadcrumb;
- título;
- status;
- descrição/conteúdo existente;
- concluir/reabrir conforme suporte;
- `INICIAR SESSÃO`.

Sem editor de notas na v0.1.

## 13. Visual
Preservar identidade atual do AZRIEL:
- fundo escuro;
- cyan;
- bordas;
- tipografia;
- labels técnicos;
- sidebar/header;
- espaçamento e botões existentes.

Não imitar Notion, Duolingo, LMS ou SaaS colorido.

Manter estética técnica, mas com boa hierarquia e leitura. Poucas métricas por card.

## 14. SQLite / Rust / Frontend
Inspecionar e seguir padrões reais do projeto.

SQLite:
- migration incremental;
- não apagar/recriar Roadmaps;
- banco existente e banco novo devem funcionar;
- índices apenas para consultas reais (sessões por data/roadmap, sessão ativa, recentes).

Rust/backend:
- reutilizar models/repositories/services/commands conforme arquitetura existente;
- não criar camadas que o projeto não usa.

Frontend:
- reutilizar components/hooks/stores/services/styles/tokens;
- sem framework UI novo;
- sem state manager novo;
- sem chart library na v0.1.

## 15. Operações necessárias
Implementar equivalente arquitetural a:
- start_study_session
- pause_study_session
- resume_study_session
- complete_study_session
- cancel_study_session
- get_active_study_session
- list_study_sessions
- get_study_today_summary

Adaptar nomes ao padrão real. Não criar API genérica prematura.

`StudyTodaySummary` deve conter somente campos usados pela UI, por exemplo:
- focus_seconds;
- completed_sessions;
- completed_activities;
- active_roadmap;
- last_session.

## 16. Recovery e consistência
Testar:
1. iniciar;
2. pausar;
3. navegar;
4. retornar;
5. fechar/reabrir quando suportado;
6. recuperar.

Não duplicar sessão. Não contar pausa como foco.

Falha de persistência deve aparecer na UI; não fingir sucesso.

## 17. Fora do escopo
Explicitamente NÃO implementar:
- Mapa Stark;
- Cadernos/Notas;
- Biblioteca/PDF;
- Study Cards;
- Tutor/Ollama;
- Quiz AI;
- embeddings;
- RAG;
- Knowledge Graph;
- Mastery score;
- adaptive learning;
- gamificação.

O AZRIEL possui AI Core/Ollama, mas **Study Lab v0.1 não usa IA**. Primeiro criar base funcional e dados reais de uso.

## 18. Testes
Adicionar testes compatíveis com infraestrutura existente para:
- migration preserva Roadmaps;
- create/start session;
- uma sessão ativa;
- pause/resume;
- complete/cancel;
- cálculo de foco;
- pausa excluída;
- today summary;
- history;
- vínculo com roadmap;
- IDs inválidos;
- recovery;
- navegação não perde timer;
- estado vazio;
- continuar estudando.

Não adicionar framework de testes novo apenas para cumprir checklist. Não esperar minutos reais em testes do timer.

## 19. Critérios de aceite
v0.1 concluída quando:
1. Estudos possui HOJE / ROADMAPS / HISTÓRICO;
2. Roadmaps continuam funcionando e dados existentes são preservados;
3. Dashboard existe e estado vazio funciona;
4. foco atual e CONTINUAR ESTUDANDO funcionam deterministicamente;
5. StudySession existe e persiste em SQLite;
6. apenas uma sessão ativa/pausada existe por padrão;
7. START/PAUSE/RESUME/COMPLETE/CANCEL funcionam;
8. pausa não conta como foco;
9. timer usa tempo real/timestamps, não apenas ticks;
10. navegação não perde sessão;
11. recovery funciona conforme arquitetura suportada;
12. sessão vincula-se ao contexto de Roadmap;
13. sessão não conclui atividade automaticamente;
14. Histórico existe;
15. métricas de Hoje funcionam;
16. progresso do Roadmap não é confundido com tempo;
17. UI segue design do AZRIEL;
18. nenhuma IA/RAG/Notes/Cards/Library é introduzida;
19. nenhuma referência ao Mapa Stark é recriada;
20. migrations/testes/build passam;
21. Tauri inicia normalmente;
22. documentação é atualizada.

## 20. Documentação
Criar `docs/study-lab/v0.1.md` (ou caminho equivalente ao padrão do repo) contendo:
- arquitetura encontrada;
- integração escolhida;
- StudySession semantics;
- timer semantics;
- recovery;
- schema/migration;
- comandos/API;
- telas;
- testes;
- limitações;
- decisões arquiteturais.

Atualizar roadmap/changelog existente sem inventar novo sistema de documentação.

## 21. Entrega final do Codex
Ao terminar, responder resumidamente:
- arquivos criados;
- arquivos alterados;
- migrations;
- decisões arquiteturais;
- funcionalidades concluídas;
- testes executados/resultados;
- build/Tauri;
- limitações;
- itens explicitamente deixados para versões futuras.

## Resultado esperado
Antes:
`ESTUDOS → ROADMAPS`

Depois:
`ESTUDOS → HOJE / ROADMAPS / HISTÓRICO`

e:

`ROADMAP → ATIVIDADE → STUDY SESSION → POMODORO → HISTÓRICO`

A v0.1 deve ser pequena o suficiente para ser confiável e útil todos os dias.

Não antecipar features. Não transformar Study Lab em outro projeto experimental. Construir a fundação e parar.
