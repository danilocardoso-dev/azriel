# AZRIEL v0.8.3 — Learning Engine / Evidence-Based Progression

A versão **v0.8.2 — Stark Knowledge System** foi concluída e validada.

O Mapa Stark agora centraliza:

* Visão Geral;
* Conhecimento;
* Roadmaps;
* Pesquisa;
* Evolução;
* Lacunas.

Também já existem:

* Knowledge Baselines;
* Knowledge Events;
* Roadmaps;
* Stages;
* Topics;
* Activities;
* Activity Types;
* relações entre Roadmap Topics e Knowledge Nodes.

Agora implemente:

# v0.8.3 — Learning Engine

---

# Objetivo

Transformar atividades concluídas em **evidências rastreáveis de aprendizado** capazes de atualizar automaticamente:

* Cobertura;
* Profundidade;
* Integração;
* nível de domínio dos tópicos;
* histórico de evolução;
* Mapa Stark.

A progressão deve ser:

* determinística;
* explicável;
* auditável;
* reversível;
* resistente a repetição artificial;
* baseada em evidências.

---

# Princípio central

O sistema NÃO deve funcionar como XP.

Não implementar:

```text
ler artigo = +3 pontos
fazer projeto = +10 pontos
```

de forma simplesmente cumulativa.

O Learning Engine deve considerar:

```text
TIPO DE EVIDÊNCIA
+
DIVERSIDADE
+
REPETIÇÃO
+
NÍVEL DO KNOWLEDGE NODE
+
BASELINE
+
RELAÇÕES ENTRE CONHECIMENTOS
```

O objetivo é representar evolução aproximada de conhecimento, não gamificação.

---

# Antes de modificar código

1. leia o README;
2. leia `docs/versions/v0.8.2.md`;
3. examine Knowledge Core;
4. examine Knowledge Baselines;
5. examine Knowledge Events;
6. examine Roadmaps;
7. examine Activities;
8. examine Knowledge History;
9. examine Gap Diagnostics;
10. examine AI Core;
11. execute migrations/testes/build existentes;
12. preserve todos os dados atuais.

Não modificar migrations antigas.

---

# Regra arquitetural fundamental

Nenhum módulo deve alterar diretamente:

```text
coverage
depth
integration
```

a partir desta versão.

Fluxo obrigatório:

```text
EVIDÊNCIA
   ↓
KNOWLEDGE EVENT
   ↓
LEARNING ENGINE
   ↓
RECALCULATION
   ↓
KNOWLEDGE STATE
```

---

# Fonte de verdade

A evolução automática deve poder ser reconstruída a partir de:

```text
BASELINE
+
KNOWLEDGE EVENTS VÁLIDOS
```

Isso significa que, se necessário:

```text
recalculateKnowledge()
```

deve conseguir reconstruir o estado.

---

# Activity Types

Utilizar os tipos preparados na v0.8.2:

```text
READING
LESSON
QUIZ
EXERCISE
SIMULATION
EXPERIMENT
PROJECT
DOCUMENTATION
RESEARCH
OTHER
```

---

# Evidence Profile

Cada tipo possui um perfil relativo.

Criar configuração central.

Exemplo conceitual:

```ts
interface EvidenceProfile {
  coverage: number;
  depth: number;
  integration: number;
}
```

Valores iniciais relativos sugeridos:

```text
READING
coverage      1.00
depth         0.10
integration   0.00

LESSON
coverage      0.90
depth         0.20
integration   0.00

QUIZ
coverage      0.60
depth         0.35
integration   0.00

EXERCISE
coverage      0.45
depth         0.70
integration   0.10

SIMULATION
coverage      0.35
depth         0.85
integration   0.30

EXPERIMENT
coverage      0.30
depth         1.00
integration   0.45

PROJECT
coverage      0.40
depth         1.00
integration   1.00

DOCUMENTATION
coverage      0.55
depth         0.60
integration   0.35

RESEARCH
coverage      0.70
depth         0.45
integration   0.30

OTHER
coverage      0.30
depth         0.30
integration   0.10
```

Esses valores NÃO são pontos percentuais.

São pesos relativos.

Documentar isso claramente.

---

# Diminishing Returns

Repetir indefinidamente o mesmo tipo de evidência não deve gerar crescimento linear.

Criar fator de retorno decrescente.

Exemplo conceitual:

```ts
effectiveWeight =
baseWeight * repetitionFactor
```

Uma função possível:

```text
1ª evidência    1.00
2ª              0.75
3ª              0.55
4ª              0.40
5ª              0.30
...
```

Pode utilizar função matemática contínua equivalente.

Centralizar configuração.

---

# Regra importante

Repetição deve ser considerada por:

```text
knowledgeNode
+
activityType
```

Não globalmente.

Exemplo:

10 leituras sobre genética não devem reduzir o valor da primeira leitura sobre eletrônica.

---

# Evidence Diversity

Diversidade de evidências deve ser recompensada.

Exemplo:

```text
READING
+
EXERCISE
+
EXPERIMENT
+
PROJECT
```

deve representar evidência mais forte do que:

```text
READING
+
READING
+
READING
+
READING
```

Criar:

```text
diversityFactor
```

com limites razoáveis.

Não permitir multiplicadores explosivos.

---

# Cobertura

Cobertura representa:

> Quanto daquele território foi explorado/compreendido.

Deve responder principalmente a:

* reading;
* lesson;
* research;
* quiz;
* documentação;
* exposição a tópicos diferentes.

Cobertura deve depender também da quantidade de Knowledge Nodes filhos explorados.

---

# Profundidade

Profundidade representa:

> O quanto o conhecimento consegue ser utilizado.

Deve responder principalmente a:

* exercises;
* simulations;
* experiments;
* projects;
* documentação técnica aplicada.

Leitura isolada deve ter efeito muito pequeno.

---

# Integração

Integração representa:

> Capacidade de combinar conhecimentos diferentes.

Não aumentar integração significativamente quando atividade possuir apenas um Knowledge Node.

PROJECT, EXPERIMENT ou atividade explicitamente associada a múltiplos conhecimentos podem contribuir.

Exemplo:

```text
Atividade:
Controlar motor com ESP32

Knowledge:
Eletrônica
IoT
Sistemas Embarcados
Controle
```

Isso representa integração.

---

# Activity ↔ Multiple Knowledge Nodes

Expandir relação se necessário para permitir:

```text
1 activity
↕
N knowledge nodes
```

Não limitar uma atividade prática complexa a apenas um conhecimento.

---

# Primary Knowledge

Permitir opcionalmente indicar:

```text
primaryKnowledgeNodeId
```

e conhecimentos secundários.

Isso poderá influenciar distribuição do impacto.

---

# Distribuição

Evitar que uma única atividade gere impacto máximo integral em cinco áreas simultaneamente.

Criar distribuição controlada.

Exemplo conceitual:

```text
primary      1.00
secondary    0.50
```

Valores devem ser configuráveis.

---

# Hierarquia

Eventos em um tópico podem contribuir parcialmente para ancestrais.

Exemplo:

```text
MOSFET
↓
Semicondutores
↓
Eletrônica
```

Uma atividade em MOSFET contribui:

* fortemente para MOSFET;
* parcialmente para Semicondutores;
* menos para Eletrônica.

Criar decay por nível.

Exemplo conceitual:

```text
node atual       1.00
parent           0.50
grandparent      0.25
```

Não propagar indefinidamente sem limite.

---

# Baseline

Preservar baseline da v0.8.2.

O baseline representa conhecimento anterior ao Learning Engine.

Exemplo:

```text
Eletrônica

Baseline
coverage 20
depth    10
```

O Learning Engine trabalha a partir disso.

---

# Saturação

Nenhuma métrica pode ultrapassar:

```text
100
```

Mas não utilizar simplesmente:

```ts
Math.min(100, baseline + sum)
```

como único mecanismo.

Quanto mais alto o conhecimento, mais difícil deve ser aumentar.

Criar saturation factor.

Conceitualmente:

```text
0 → 30       crescimento relativamente fácil
30 → 60      moderado
60 → 80      difícil
80 → 95      muito difícil
95 → 100     extremamente difícil
```

---

# 100%

Não tornar 100% facilmente alcançável.

Idealmente atividades normais devem tender assintoticamente ao limite.

Não bloquear matematicamente 100 se arquitetura exigir, mas tornar excepcional.

---

# Knowledge Event

Ao concluir uma atividade elegível:

criar:

```text
KnowledgeEvent
```

contendo pelo menos:

```text
sourceType = roadmap
sourceId = activityId
eventType = activity_completed
knowledgeNode
activityType
rawEvidenceProfile
effectiveImpact
createdAt
```

Adaptar ao schema existente.

---

# Event Metadata

Guardar informação suficiente para explicar posteriormente:

```text
por que este evento produziu este impacto?
```

Exemplo:

```json
{
  "activityType": "EXPERIMENT",
  "repetitionIndex": 1,
  "repetitionFactor": 1.0,
  "diversityFactor": 1.12,
  "hierarchyFactor": 1.0
}
```

Não é obrigatório usar exatamente esse JSON.

Mas o cálculo deve ser auditável.

---

# Idempotência

Concluir a mesma atividade duas vezes não pode gerar dois eventos válidos.

Criar constraint/regra.

```text
activityId
+
eventType
```

deve ser idempotente quando apropriado.

---

# Reopening Activity

Se:

```text
completed
→
pending
```

não deletar silenciosamente o Knowledge Event original.

Criar evento de reversão:

```text
activity_reopened
```

ou mecanismo equivalente auditável.

---

# Reversal

O evento de reversão deve neutralizar o impacto anterior durante recálculo.

Preservar histórico.

---

# Recomplete

Se atividade for concluída novamente depois de reaberta:

criar novo evento correspondente, mantendo sequência auditável.

Evitar duplicação incorreta.

---

# Manual Adjustments

Alterações manuais existentes devem continuar possíveis quando necessário.

Mas devem gerar:

```text
sourceType = manual
```

e Knowledge Event correspondente.

Evitar editar diretamente métricas sem registro.

---

# Topic Mastery

Automatizar estados:

```text
NOT_STARTED
EXPOSED
UNDERSTOOD
PRACTICED
APPLIED
MASTERED
```

---

# Regras iniciais

Não utilizar apenas percentual.

Usar composição de evidências.

Exemplo conceitual:

## NOT_STARTED

nenhuma evidência válida.

## EXPOSED

pelo menos uma:

```text
READING
LESSON
RESEARCH
```

## UNDERSTOOD

evidência conceitual + validação:

```text
QUIZ
ou
EXERCISE
```

## PRACTICED

pelo menos uma:

```text
SIMULATION
EXPERIMENT
```

## APPLIED

pelo menos:

```text
PROJECT
```

ou atividade equivalente explicitamente aplicada.

## MASTERED

Não definir apenas por uma atividade.

Exigir combinação forte de:

* cobertura;
* profundidade;
* diversidade;
* aplicação.

Manter threshold alto.

---

# Mastered

Sugestão:

```text
coverage >= 85
depth >= 80
evidence diversity >= threshold
PROJECT/APPLIED evidence exists
```

Adaptar após testes.

Documentar que `MASTERED` é uma heurística do Azriel, não certificação objetiva de domínio humano.

---

# Topic UI

Ao abrir tópico:

mostrar:

```text
MOSFET

STATE
PRACTICED

COVERAGE
63%

DEPTH
47%

EVIDENCE

✓ Reading
✓ Lesson
✓ Exercises
✓ Simulation
✓ Experiment
□ Project Application
```

---

# Activity Completion Feedback

Ao marcar atividade:

```text
✓ Montar circuito com MOSFET
```

mostrar feedback discreto:

```text
KNOWLEDGE EVENT CREATED

Eletrônica / MOSFET

Cobertura
+...

Profundidade
+...

Integração
+...
```

Não usar linguagem de XP.

---

# Impact Preview

Antes de concluir atividade, pode mostrar:

```text
EVIDENCE TYPE
EXPERIMENT

PRIMARY IMPACT
DEPTH

SECONDARY
INTEGRATION
```

Não precisa mostrar número exato se isso incentivar gamificação.

---

# Evolution

Atualizar:

```text
Mapa Stark → Evolução
```

Mostrar timeline real de Knowledge Events.

Exemplo:

```text
03 SET 2026

EXPERIMENT
Montar circuito MOSFET

Roadmap:
Controle e Automação

Knowledge:
MOSFET

Impact:
Cobertura ↑
Profundidade ↑↑
```

---

# Explainability

Adicionar:

```text
POR QUE ESTE NÍVEL?
```

Para cada Knowledge Node.

Mostrar:

```text
BASELINE
20 / 10

EVIDÊNCIAS

Reading
3

Exercise
8

Experiment
2

Project
1

Principais contribuições:
...
```

---

# Não expor matemática excessiva por padrão

Interface principal deve ser compreensível.

Criar detalhes avançados opcionalmente:

```text
VER CÁLCULO
```

para inspeção.

---

# Recalculation Engine

Criar:

```text
LearningEngine.recalculate(...)
```

Capaz de recalcular:

* node;
* ancestors;
* metrics;
* mastery.

---

# Rebuild

Criar função administrativa:

```text
RECALCULAR CONHECIMENTO
```

na área apropriada de Configurações/diagnóstico.

Ela deve:

```text
baseline
+
events
→
rebuild
```

Não criar eventos novos.

---

# Dry Run

Antes de aplicar rebuild completo:

se viável, mostrar preview/diff.

Não obrigatório se aumentar muito o escopo.

---

# Integrity

Criar validações para:

* event sem knowledge node;
* activity inexistente;
* baseline duplicado;
* reversal sem evento original;
* delta inválido;
* métricas fora de faixa.

---

# Research

Na v0.8.3, pesquisa pode gerar evidência SOMENTE quando existir ação explícita de conclusão/registro como atividade `RESEARCH`.

Não transformar automaticamente toda pesquisa cadastrada em conhecimento.

---

# Projects

Projetos podem contribuir quando uma Roadmap Activity do tipo:

```text
PROJECT
```

estiver relacionada ao projeto.

Não inferir automaticamente contribuição apenas porque o projeto existe.

---

# Operations Daily

Não transformar conclusão de qualquer tarefa diária em conhecimento.

Somente quando uma tarefa estiver explicitamente relacionada a:

```text
Roadmap Activity
```

ou evidência de aprendizado.

Evitar que tarefas administrativas alterem Mapa Stark.

---

# Education

Não automatizar evolução simplesmente porque uma formação está em andamento.

Conclusão de cursos/disciplinas poderá gerar evidências futuramente.

Fora do escopo automático desta versão.

---

# AI Core — READ

Adicionar/atualizar tools:

```text
get_learning_progress
get_topic_mastery
get_knowledge_evidence
get_recent_knowledge_events
explain_knowledge_level
get_roadmap_learning_status
```

---

# AI Core — perguntas

Azriel deve conseguir responder:

```text
Por que minha profundidade em Eletrônica aumentou?
```

```text
O que contribuiu para meu conhecimento em MOSFET?
```

```text
Qual é meu nível atual em PID?
```

```text
Quais conhecimentos evoluíram esta semana?
```

```text
Qual roadmap mais contribuiu para minha evolução?
```

Respostas devem vir de eventos reais.

---

# AI Core — WRITE

O LLM não pode criar Knowledge Events arbitrariamente.

Knowledge Events automáticos surgem de ações válidas do sistema.

Não permitir:

```text
AI decide que usuário aprendeu algo
→ aumenta conhecimento
```

Isso é proibido.

---

# Anti-inflation

Criar proteções:

1. diminishing returns;
2. saturation;
3. diversity;
4. idempotência;
5. hierarchy decay;
6. primary/secondary weighting;
7. event auditability.

---

# Configuração

Centralizar parâmetros do Learning Engine.

Exemplo:

```text
learningEngineConfig
```

Não espalhar números mágicos.

---

# Versionamento da fórmula

Isso é importante.

Criar:

```text
formulaVersion
```

Exemplo:

```text
LEARNING_ENGINE_V1
```

Knowledge Events devem registrar versão relevante quando necessário.

Isso permitirá mudar a fórmula futuramente.

---

# Recalcular com fórmula nova

Não implementar migração automática de fórmula agora.

Apenas preparar versionamento.

---

# Metrics History

Quando recálculo alterar métricas:

atualizar histórico de conhecimento existente de forma consistente.

Evitar snapshots duplicados em cada render.

---

# UI — Visão Geral

Mapa Stark deve atualizar automaticamente após eventos.

Exemplo:

```text
ATIVIDADE CONCLUÍDA
↓
EVENT
↓
RECALCULATION
↓
MAPA STARK
```

sem reiniciar aplicativo.

---

# UI — Roadmaps

Mostrar duas métricas distintas:

```text
ROADMAP PROGRESS
62%

KNOWLEDGE IMPACT
Eletrônica ↑
Controle ↑
IoT ↑
```

Nunca apresentar os dois como a mesma coisa.

---

# UI — Evolução

Adicionar filtros:

```text
PERÍODO
ROADMAP
KNOWLEDGE
EVENT TYPE
```

---

# UI — Knowledge

Mostrar:

```text
CURRENT
BASELINE
CHANGE SINCE BASELINE
```

Exemplo:

```text
ELETRÔNICA

Cobertura
20 → 31

Profundidade
10 → 19
```

---

# Lacunas

Gap Diagnostics deve usar métricas recalculadas.

Não manter cópia paralela.

---
# TESTES — continuação

Adicionar testes para:

1. evidence profiles;
2. reading impact;
3. experiment impact;
4. project impact;
5. repetition factor;
6. diminishing returns;
7. diversity factor;
8. saturation;
9. hierarchy propagation;
10. primary knowledge weighting;
11. secondary knowledge weighting;
12. baseline preservation;
13. event creation;
14. event idempotency;
15. activity reopen;
16. reversal;
17. recomplete;
18. manual event;
19. topic mastery;
20. NOT_STARTED;
21. EXPOSED;
22. UNDERSTOOD;
23. PRACTICED;
24. APPLIED;
25. MASTERED;
26. recalculation;
27. rebuild from baseline + events;
28. formula version;
29. event explainability;
30. relationship with roadmap;
31. relationship with multiple knowledge nodes;
32. research evidence;
33. project evidence;
34. Operations Daily sem vínculo não altera conhecimento;
35. activity ligada explicitamente ao roadmap altera conhecimento;
36. Gap Diagnostics utiliza métricas recalculadas;
37. AI read tools;
38. ausência de eventos;
39. event inválido;
40. reversal inválido;
41. métricas nunca ultrapassam 100;
42. rebuild não cria eventos duplicados.

---

# TESTE DE NÃO GAMIFICAÇÃO

Criar um Knowledge Node de teste.

Executar várias atividades do tipo:

```text
READING
READING
READING
READING
READING
READING
READING
READING
READING
READING
```

Confirmar que:

* cobertura cresce progressivamente;
* retorno diminui;
* profundidade permanece relativamente baixa;
* integração permanece próxima de zero;
* o conhecimento não chega artificialmente próximo de domínio apenas por leitura.

Depois adicionar:

```text
EXERCISE
SIMULATION
EXPERIMENT
PROJECT
```

Confirmar aumento significativamente mais consistente de profundidade/diversidade.

---

# TESTE DE DIVERSIDADE

Comparar dois cenários.

## Cenário A

```text
10 × READING
```

## Cenário B

```text
READING
LESSON
QUIZ
EXERCISE
SIMULATION
EXPERIMENT
PROJECT
DOCUMENTATION
```

O cenário B deve produzir uma representação mais equilibrada de conhecimento.

Especialmente em:

```text
depth
integration
mastery
```

---

# TESTE DE SATURAÇÃO

Criar Knowledge Node com baseline alto.

Exemplo:

```text
coverage = 90
depth = 85
```

Adicionar evidências.

Confirmar que crescimento é muito menor que em um node com:

```text
coverage = 20
depth = 10
```

Nenhuma métrica deve ultrapassar 100.

---

# TESTE DE HIERARQUIA

Criar:

```text
Eletrônica
└── Semicondutores
    └── MOSFET
```

Concluir atividade em:

```text
MOSFET
```

Confirmar impacto:

```text
MOSFET          alto
Semicondutores  parcial
Eletrônica      menor
```

Confirmar que decay ocorre corretamente.

---

# TESTE DE INTEGRAÇÃO

Criar atividade:

```text
Controlar motor com ESP32
```

Relacionar:

```text
PRIMARY
Controle

SECONDARY
Eletrônica
IoT
Sistemas Embarcados
```

Tipo:

```text
PROJECT
```

Confirmar:

* profundidade aumenta;
* integração aumenta;
* primary recebe impacto superior aos secondary;
* nenhuma área recebe impacto integral duplicado.

---

# TESTE DE IDEMPOTÊNCIA

Concluir atividade uma vez.

Confirmar:

```text
1 KnowledgeEvent válido
```

Executar a mesma operação novamente sem alteração de estado.

Confirmar:

```text
continua existindo apenas 1 evento válido
```

---

# TESTE DE REABERTURA

Sequência:

```text
pending
→ completed
→ pending
```

Confirmar histórico:

```text
activity_completed
activity_reopened
```

Recalcular.

Estado final deve refletir atividade não concluída.

O evento original permanece no histórico.

---

# TESTE DE RECONCLUSÃO

Sequência:

```text
pending
→ completed
→ pending
→ completed
```

Confirmar histórico auditável.

Recalcular corretamente sem duplicação indevida.

---

# MIGRATIONS

Se forem necessárias novas colunas/tabelas:

criar novas migrations.

Não modificar migrations antigas.

Preservar completamente:

* projetos;
* roadmaps;
* pesquisas;
* knowledge nodes;
* baselines;
* tasks;
* Automation Core;
* Engineering Core;
* configurações.

---

# BACKWARD COMPATIBILITY

Dados da v0.8.2 devem continuar funcionando.

Knowledge Events já existentes não podem desaparecer.

Se eventos antigos não possuírem campos novos:

criar valores/defaults compatíveis ou migration apropriada.

---

# OBSERVABILIDADE

Adicionar logging técnico apenas para:

* criação de Knowledge Event;
* recalculation failure;
* invalid event;
* reversal failure;
* formula version mismatch;
* rebuild.

Não gerar logs excessivos para cada renderização.

---

# DEBUG DO LEARNING ENGINE

Adicionar opção de diagnóstico em área apropriada.

Exemplo:

```text
LEARNING ENGINE

STATUS
ONLINE

FORMULA
V1

EVENTS
128

[ RECALCULAR CONHECIMENTO ]
```

Pode mostrar último recalculation.

---

# SEGURANÇA DE DADOS

Antes de executar rebuild:

usar transação quando apropriado.

Se recálculo falhar:

não deixar métricas parcialmente atualizadas.

Preferir:

```text
BEGIN
recalculate
validate
persist
COMMIT
```

ou equivalente suportado pela arquitetura existente.

---

# CRITÉRIOS DE ACEITE

A v0.8.3 só está concluída quando:

1. Learning Engine existir;
2. atividades concluídas gerarem Knowledge Events;
3. eventos forem idempotentes;
4. reabertura gerar reversão auditável;
5. recomplete funcionar;
6. baseline continuar preservado;
7. cobertura for calculada automaticamente;
8. profundidade for calculada automaticamente;
9. integração for calculada automaticamente;
10. Activity Types possuírem perfis distintos;
11. reading não gerar profundidade excessiva;
12. experiment contribuir fortemente para profundidade;
13. project contribuir para profundidade e integração;
14. diminishing returns funcionar;
15. diversity factor funcionar;
16. saturation funcionar;
17. hierarquia propagar impacto parcialmente;
18. multiple knowledge nodes funcionar;
19. primary/secondary weighting funcionar;
20. métricas permanecerem entre 0 e 100;
21. roadmap progress continuar separado de knowledge level;
22. Topic Mastery for derivado de evidências;
23. NOT_STARTED funcionar;
24. EXPOSED funcionar;
25. UNDERSTOOD funcionar;
26. PRACTICED funcionar;
27. APPLIED funcionar;
28. MASTERED exigir critérios fortes;
29. Knowledge Events armazenarem metadata explicável;
30. formulaVersion existir;
31. recalculation puder reconstruir métricas;
32. rebuild usar baseline + eventos;
33. rebuild não criar eventos novos;
34. Knowledge History continuar consistente;
35. Evolução mostrar eventos reais;
36. Knowledge UI mostrar baseline/current/change;
37. Mapa Stark atualizar automaticamente;
38. Gap Diagnostics usar valores recalculados;
39. pesquisa não gerar conhecimento automaticamente sem evidência explícita;
40. projeto não gerar conhecimento apenas por existir;
41. tarefa diária comum não alterar conhecimento;
42. AI Core conseguir explicar evolução;
43. AI Core não puder inventar Knowledge Events;
44. AI Core não puder aumentar conhecimento diretamente;
45. migrations preservarem dados;
46. recálculo for transacional/seguro;
47. TypeScript não apresentar erros;
48. testes passarem;
49. build funcionar;
50. Tauri iniciar normalmente;
51. módulos anteriores não sofrerem regressão;
52. `docs/versions/v0.8.3.md` existir.

---

# TESTE FINAL OBRIGATÓRIO

Criar ou utilizar:

```text
Roadmap:
Controle e Automação
```

Etapa:

```text
Eletrônica
```

Tópico:

```text
MOSFET
```

Relacionar ao Knowledge Node correspondente.

Criar atividades:

```text
READING
Ler fundamentos de MOSFET

LESSON
Assistir aula de MOSFET

QUIZ
Responder questionário

EXERCISE
Resolver exercícios

SIMULATION
Simular chaveamento

EXPERIMENT
Montar circuito com MOSFET

PROJECT
Aplicar MOSFET no ArcCore
```

---

## Estado inicial

Registrar:

```text
Cobertura atual
Profundidade atual
Integração atual
Topic Mastery
```

---

## Reading

Concluir:

```text
Ler fundamentos de MOSFET
```

Confirmar:

* KnowledgeEvent criado;
* cobertura aumenta;
* profundidade aumenta muito pouco;
* integração não aumenta significativamente;
* tópico torna-se pelo menos EXPOSED.

---

## Exercise

Concluir:

```text
Resolver exercícios
```

Confirmar:

* novo evento;
* profundidade recebe impacto maior;
* evolução aparece na timeline.

---

## Simulation / Experiment

Concluir:

```text
Simular chaveamento
Montar circuito com MOSFET
```

Confirmar:

* profundidade cresce;
* tópico evolui para PRACTICED quando requisitos forem satisfeitos.

---

## Project

Relacionar:

```text
ArcCore
```

e conhecimentos secundários apropriados.

Concluir:

```text
Aplicar MOSFET no ArcCore
```

Confirmar:

* evento PROJECT;
* profundidade aumenta;
* integração aumenta;
* múltiplos conhecimentos recebem impacto ponderado;
* tópico pode chegar a APPLIED quando critérios forem satisfeitos.

---

## Repetição

Criar múltiplas atividades READING adicionais no mesmo tópico.

Concluir.

Confirmar diminishing returns.

---

## Reabrir

Reabrir uma das atividades concluídas.

Confirmar:

* evento original preservado;
* reversal criado;
* métricas recalculadas;
* timeline auditável.

---

## Reinício

Fechar completamente o Azriel.

Abrir novamente.

Confirmar:

* atividades;
* eventos;
* métricas;
* mastery;
* timeline;

permanecem consistentes.

---

## Rebuild

Executar:

```text
RECALCULAR CONHECIMENTO
```

Confirmar que os valores resultantes permanecem iguais ao estado anterior.

Isso prova:

```text
BASELINE
+
EVENTS
=
CURRENT KNOWLEDGE STATE
```

---

## AI Core

Perguntar:

```text
Azriel, por que minha profundidade em MOSFET está nesse nível?
```

A resposta deve citar evidências reais.

Perguntar:

```text
O que mais contribuiu para meu conhecimento em MOSFET?
```

Perguntar:

```text
Qual é meu nível de domínio nesse tópico?
```

Perguntar:

```text
O que evoluiu recentemente em Eletrônica?
```

Não aceitar respostas inventadas.

---

# DOCUMENTAÇÃO

Criar:

```text
docs/versions/v0.8.3.md
```

Documentar:

* objetivo;
* filosofia baseada em evidências;
* Activity Types;
* Evidence Profiles;
* diminishing returns;
* diversity;
* saturation;
* hierarchy propagation;
* multiple knowledge nodes;
* primary/secondary weighting;
* Knowledge Events;
* reversals;
* Topic Mastery;
* recalculation;
* formula versioning;
* AI integration;
* limitações.

Atualizar:

```text
docs/roadmap.md
```

Marcar:

```text
v0.8.2 — concluída
v0.8.3 — Learning Engine — em desenvolvimento
```

Após validação:

```text
v0.8.3 — concluída
```

---

# RESULTADO ESPERADO

Ao concluir a v0.8.3, o Mapa Stark deve deixar de depender de alterações manuais para representar evolução cotidiana.

O fluxo passa a ser:

```text
ESTUDAR
↓
PRATICAR
↓
APLICAR
↓
REGISTRAR EVIDÊNCIA
↓
KNOWLEDGE EVENT
↓
LEARNING ENGINE
↓
MAPA STARK
```

O operador não recebe pontos porque marcou checkboxes.

O sistema registra evidências e utiliza essas evidências para estimar evolução.

Uma leitura deve demonstrar exposição.

Um exercício deve demonstrar prática cognitiva.

Um experimento deve demonstrar aplicação prática.

Um projeto interdisciplinar deve demonstrar profundidade e integração.

E qualquer número exibido pelo Mapa Stark deve possuir uma resposta para:

# "POR QUE EU TENHO ESTE NÍVEL?"

A filosofia da versão é:

# CONHECIMENTO NÃO É DECLARADO. É SUSTENTADO POR EVIDÊNCIAS.

---

# Estado da implementação — 03/09/2026

Status: **implementação concluída; validação operacional no aplicativo Tauri pendente**.

## Entregue

* migration `0012_learning_engine.sql` com ledger expandido, relações primária/secundárias e estado versionado do motor;
* fórmula `LEARNING_ENGINE_V1` centralizada, com perfis por atividade, diminishing returns, diversidade, saturação e propagação hierárquica limitada;
* conclusão de atividade gerando evidência auditável e idempotente;
* reabertura gerando evento de reversão, sem apagar histórico;
* recálculo transacional a partir de baseline + ledger, sem criar eventos novos;
* Topic Mastery derivado das evidências;
* proteção contra remoção ou alteração silenciosa de evidência concluída;
* editor de roadmap com conhecimento primário, múltiplos secundários, projeto e pesquisa opcionais;
* seis tools somente leitura no AI Core para progresso, domínio, evidências, eventos recentes, explicação e contribuição por roadmap;
* painel administrativo do Learning Engine nas configurações.

## Limite de validação

Lint, testes automatizados e builds validam contratos, cálculo e regressões. O comportamento visual final, a ergonomia do editor e a atualização dentro do WebView nativo ainda devem ser aceitos pelo operador no aplicativo Tauri antes de marcar a versão como concluída no roadmap.
