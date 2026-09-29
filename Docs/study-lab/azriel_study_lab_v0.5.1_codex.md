# AZRIEL STUDY LAB v0.5.1 — Roadmap Learning Model
## Prompt para Codex

## Contexto
O Study Lab v0.5 está concluído. Os Roadmaps atuais funcionam como `Roadmap → Stage → Topic → Activity`, mas Activity ainda descreve principalmente “o que fazer”.

A v0.5.1 deve acrescentar “por que aprender, como estudar, com quais recursos, o que produzir, como demonstrar aprendizado e o que refletir”, sem criar novo módulo.

## 1. Objetivo
Expandir Activity com:
- learningObjective
- instructions
- completionCriteria
- deliverable
- estimatedMinutes
- learningMethod
- resources[]
- isValidation
- reflectionPrompt
- integração opcional com Study Library

Ciclo suportado: `APRENDER → PRATICAR → DEMONSTRAR → REVISAR → INTEGRAR`.

## 2. Inspeção e compatibilidade
Antes de codificar, revisar Roadmap/Topic/Activity reais, schema, import/export JSON, validators, UI de edição, Activity View, Study Library e migrations.

Não reescrever Roadmaps. JSONs antigos devem continuar válidos. Novos campos são opcionais com defaults/null/[]/false adequados. Export→Import deve preservar dados.

## 3. Novos campos de Activity
`learningObjective`: competência que a atividade pretende desenvolver.

`instructions`: como executar, podendo usar Markdown se renderer existente permitir.

`completionCriteria`: evidência/condição que orienta o usuário a decidir se concluiu. O sistema não aprova automaticamente.

`deliverable`: resultado concreto esperado (PCAP, script, relatório, nota, protótipo, entrevista, hipótese validada/refutada etc.). Não exigir upload nesta versão.

`estimatedMinutes`: inteiro positivo opcional. Mostrar separado do tempo real de StudySession.

`isValidation`: boolean default false. Quando true, representa checkpoint de demonstração.

`reflectionPrompt`: pergunta opcional de reflexão pós-atividade.

## 4. Learning Method
Estrutura conceitual:
```json
{"type":"ACTIVE_RECALL","instructions":"Feche o material e explique sem consultar as notas."}
```

Tipos iniciais:
ACTIVE_RECALL, FEYNMAN, SHADOWING, SPACED_REVIEW, HANDS_ON, PROBLEM_SOLVING, CASE_STUDY, BUILD, OBSERVE, EXPERIMENT, REFLECTION, RESEARCH, OTHER.

Adaptar ao padrão do projeto.

Não confundir:
- activityType = natureza (LESSON, PROJECT, EXERCISE...)
- learningMethod = forma de aprender (FEYNMAN, SHADOWING, HANDS_ON...)

## 5. Resources
Adicionar `resources[]`:
```json
{
  "id":"resource-tcp-video",
  "type":"VIDEO",
  "title":"TCP Handshake",
  "url":"https://...",
  "provider":"YouTube",
  "language":"en",
  "required":true,
  "studyMaterialId":null
}
```

Resource IDs únicos.

Tipos: VIDEO, ARTICLE, DOCUMENTATION, COURSE, BOOK, LAB, TOOL, PODCAST, DATASET, WEBSITE, OTHER.

`language`: código consistente (`pt-BR`, `en`, etc.).
`required`: principal vs aprofundamento; não bloquear conclusão automaticamente.
URL clicável e aberta com segurança. Sem scraping/download automático.

## 6. Study Library
Resource é referência do Roadmap; StudyMaterial é item da Biblioteca. Não são a mesma entidade.

`studyMaterialId` opcional.

UI pode oferecer `ADICIONAR À BIBLIOTECA`; depois persistir relação sem duplicar material existente.

URL sozinha não dá à IA acesso ao conteúdo da página.

## 7. Validation e Topic State
Activity `isValidation=true` recebe indicador discreto CHECKPOINT.

Preservar Topic states:
NOT_STARTED, EXPOSED, UNDERSTOOD, PRACTICED, APPLIED, MASTERED.

Não marcar MASTERED automaticamente só porque atividades terminaram. Semântica:
- EXPOSED: contato
- UNDERSTOOD: consegue explicar
- PRACTICED: praticou
- APPLIED: aplicou em situação real/nova
- MASTERED: demonstrou domínio consistentemente

Sem Mastery Score ou LLM judge.

## 8. Prerequisites
Preservar `prerequisiteTopicIds`. Podem orientar dependências, mas não bloquear rigidamente o usuário de abrir tópicos posteriores.

## 9. Activity View
Mostrar somente se presentes, em boa hierarquia:
1. título/status
2. OBJETIVO
3. MÉTODO
4. INSTRUÇÕES
5. RECURSOS
6. ENTREGA
7. CRITÉRIO DE CONCLUSÃO
8. REFLEXÃO
9. ESTIMATIVA

Não mostrar seções vazias.

Resource UI conceitual:
`[VIDEO] TCP Handshake · YouTube · EN · OBRIGATÓRIO  [ABRIR] [+ BIBLIOTECA]`

Manter design AZRIEL.

## 10. Create/Edit Activity
Atualizar formulário para novos campos, todos opcionais salvo regras estruturais existentes. Não tornar criação simples burocrática.

## 11. Import JSON
Expandir Activity:
```json
{
  "id":"activity-analisar-tcp-handshake",
  "title":"Analisar um TCP Handshake real",
  "description":"Capturar e interpretar uma conexão TCP.",
  "activityType":"EXPERIMENT",
  "learningObjective":"Compreender o estabelecimento de uma conexão TCP.",
  "instructions":"Capture uma conexão TCP no Wireshark e identifique o handshake.",
  "completionCriteria":"Identificar e explicar o handshake em captura desconhecida.",
  "deliverable":"PCAP e nota técnica.",
  "estimatedMinutes":60,
  "learningMethod":{"type":"HANDS_ON","instructions":"Execute primeiro sem copiar roteiro completo."},
  "isValidation":true,
  "reflectionPrompt":"O que você não identificou de primeira e por quê?",
  "resources":[{
    "id":"resource-tcp-video",
    "type":"VIDEO",
    "title":"TCP Handshake",
    "url":"https://...",
    "provider":"YouTube",
    "language":"en",
    "required":true,
    "studyMaterialId":null
  }],
  "status":"pending",
  "completedAt":null,
  "order":1,
  "primaryKnowledgeNodeId":"knowledge-redes",
  "secondaryKnowledgeNodeIds":[],
  "projectId":null,
  "researchId":null
}
```

JSON antigo sem campos novos continua válido.

## 12. Export / validation
Exportar novos campos quando presentes. Testar round-trip.

Validar resource id/type/title/url/language/studyMaterialId. Erros devem indicar caminho do JSON quando possível.

Preservar activityTypes atuais: READING, LESSON, QUIZ, EXERCISE, SIMULATION, EXPERIMENT, PROJECT, DOCUMENTATION, RESEARCH, OTHER.

## 13. StudySession / Notes / Review / AI
StudySession: mostrar estimatedMinutes; pode sugerir duração, nunca alterar timer automaticamente.

Notes: quando deliverable/reflection existirem, pode oferecer `CRIAR NOTA DE ENTREGA` / `REGISTRAR REFLEXÃO`.

Review: não gerar cards automaticamente.

AI Context: quando ação partir de Activity, pode incluir objective/instructions/criteria/method e somente resources explicitamente selecionados. Sem scraping.

## 14. Casos que a arquitetura deve suportar
Inglês:
`VIDEO + language=en + SHADOWING + instruções + critério de compreensão/produção`.

Empreendedorismo:
`EXPERIMENT + hipótese + entrevistas + deliverable de evidências + critério de sustentar/refutar hipótese + reflectionPrompt`.

Técnico:
labs, código, investigação, documentação, projetos, checkpoints e recursos.

Não hardcode comportamento específico por área.

## 15. Persistência
Migration incremental, escolhendo colunas/tabelas/JSON conforme padrões reais do projeto. Resources provavelmente merecem estrutura própria se houver consultas/relações. Preservar dados existentes.

Evitar N+1; listas de Roadmap não precisam carregar detalhes completos de todos resources.

## 16. Segurança
URLs devem ser validadas e abertas de forma segura. Não executar conteúdo externo. Não fazer scraping, download ou chamada de IA automática.

## 17. Testes
Cobrir:
- migration de Roadmaps existentes
- JSON antigo
- JSON novo completo
- JSON novo parcial
- import/export round-trip
- resource IDs duplicados
- invalid resource URL/type/language/material relation
- learningMethod
- isValidation
- estimatedMinutes
- Activity View sem campos
- Activity View completa
- Library relation
- StudySession estimate
- Notes actions
- AI Context explícito
- Topic states preservados
- nenhum MASTERED automático

## 18. Critérios de aceite
1. Roadmaps antigos continuam funcionando;
2. import antigo permanece válido;
3. novos campos existem e são opcionais;
4. Activity View exibe apenas dados presentes;
5. learningObjective/instructions/criteria/deliverable funcionam;
6. estimatedMinutes funciona sem substituir tempo real;
7. learningMethod funciona;
8. resources múltiplos funcionam;
9. links são clicáveis/seguros;
10. resource language/required funcionam;
11. Resource↔StudyMaterial funciona opcionalmente;
12. `+ BIBLIOTECA` não duplica material;
13. isValidation/checkpoint funciona;
14. reflectionPrompt funciona;
15. Topic states permanecem;
16. MASTERED não é inferido apenas por conclusão;
17. prerequisites permanecem;
18. Create/Edit Activity suporta campos novos;
19. import/export round-trip preserva dados;
20. StudySession/Notes/AI integram sem automação indevida;
21. nenhum scoring/XP/LLM judge é criado;
22. nenhuma nova página/módulo é criada;
23. migrations/testes/build passam;
24. Tauri inicia;
25. documentação existe.

## 19. Documentação
Criar `docs/study-lab/v0.5.1.md` ou padrão equivalente. Documentar novos campos, learning methods, resources, Library relation, validation checkpoints, Topic state semantics, JSON schema, backward compatibility, segurança, testes e limitações.

## 20. Entrega Codex
Resumir arquivos/migrations, schema, import/export, Activity UI, Resources/Library, integrações, testes, build/Tauri e limitações.

## Resultado esperado
Antes:
`TÓPICO → ATIVIDADE → CONCLUIR`

Depois:
`OBJETIVO → MÉTODO → RECURSOS → EXECUÇÃO → ENTREGA → CRITÉRIO → REFLEXÃO → INTEGRAÇÃO`

A v0.5.1 deve permitir construir roadmaps que realmente ensinam, sem transformar o Roadmap em LMS complexo.
