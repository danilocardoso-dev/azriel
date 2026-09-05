# AZRIEL v0.8.4 — Interactive Roadmap Experience

As versões abaixo foram concluídas e validadas:

```text
v0.8.2 — Stark Knowledge System
v0.8.3 — Learning Engine
```

O Mapa Stark já centraliza:

* Visão Geral;
* Conhecimento;
* Roadmaps;
* Pesquisa;
* Evolução;
* Lacunas.

O Learning Engine já possui arquitetura baseada em:

```text
Activity
↓
Evidence
↓
Knowledge Event
↓
Learning Engine
↓
Coverage / Depth / Integration
↓
Mapa Stark
```

Porém, a experiência atual de Roadmaps ainda é essencialmente uma lista extensa.

Agora implemente:

# v0.8.4 — Interactive Roadmap Experience

---

# OBJETIVO

Transformar Roadmaps em uma experiência de estudo:

* navegável;
* hierárquica;
* interativa;
* progressiva;
* prática;
* visualmente compreensível.

O usuário deve conseguir entrar em um roadmap, navegar por suas etapas, abrir tópicos, visualizar atividades e registrar seu avanço sem enfrentar uma lista gigante.

A interface deve responder claramente:

```text
Onde estou?
↓
O que já concluí?
↓
O que estou estudando?
↓
O que vem depois?
↓
O que preciso fazer?
↓
Como isso está afetando meu conhecimento?
```

---

# PRINCÍPIO CENTRAL

O Roadmap não é uma checklist gigante.

Ele representa:

# UM CAMINHO DE APRENDIZADO.

A hierarquia deve ser visualmente evidente:

```text
ROADMAP
   ↓
ETAPA
   ↓
TÓPICO
   ↓
ATIVIDADES
   ↓
EVIDÊNCIAS
   ↓
CONHECIMENTO
```

---

# ANTES DE MODIFICAR CÓDIGO

1. leia completamente o README;
2. leia `docs/versions/v0.8.2.md`;
3. leia `docs/versions/v0.8.3.md`;
4. examine a implementação atual de Roadmaps;
5. examine Roadmap Stages;
6. examine Roadmap Topics;
7. examine Roadmap Activities;
8. examine Knowledge Events;
9. examine Learning Engine;
10. examine Knowledge Nodes;
11. examine o Mapa Stark;
12. preserve todos os dados existentes;
13. execute lint, testes e build antes de começar.

Não recriar o domínio de Roadmaps.

Esta versão deve evoluir a experiência utilizando a arquitetura existente.

---

# REFERÊNCIA VISUAL

A interface aprovada possui três regiões principais:

```text
┌─────────────────┬──────────────────────────────┬──────────────────┐
│                 │                              │                  │
│   ROADMAPS      │      ROADMAP ATUAL           │      TÓPICO      │
│                 │                              │                  │
│ lista           │ etapas / tópicos             │ detalhes         │
│ busca           │ progresso                    │ domínio          │
│ filtros         │ estrutura                    │ atividades       │
│                 │                              │ conhecimento     │
│                 │                              │ pré-requisitos   │
│                 │                              │ próximos         │
│                 │                              │                  │
└─────────────────┴──────────────────────────────┴──────────────────┘
```

Não copiar pixels literalmente.

Preservar a identidade HUD existente do Azriel.

---

# LAYOUT PRINCIPAL

Dentro:

```text
Mapa Stark → Roadmaps
```

criar layout desktop de três colunas.

## COLUNA 1

Roadmaps disponíveis.

## COLUNA 2

Estrutura do roadmap selecionado.

## COLUNA 3

Contexto/detalhes do tópico selecionado.

Em resoluções menores:

permitir adaptação responsiva sem destruir usabilidade.

---

# 1 — ROADMAP NAVIGATOR

Criar painel:

```text
SEUS ROADMAPS
```

Mostrar roadmaps existentes.

Exemplo:

```text
Engenharia de Controle e Automação
EM ANDAMENTO
23%

Biologia Molecular
EM ANDAMENTO
42%

Bioinformática
EM ANDAMENTO
35%

Eletrônica
PAUSADO
18%

IoT
EM ANDAMENTO
27%
```

---

# ROADMAP CARD

Cada item deve mostrar:

```text
nome
status
progresso
```

Opcionalmente:

```text
atividade atual
```

Não criar cards gigantes.

Manter alta densidade.

---

# ROADMAP SELECTION

Ao clicar em um roadmap:

carregar sua estrutura na coluna central.

Não navegar para uma página completamente diferente.

Manter contexto.

---

# BUSCA

Adicionar:

```text
BUSCAR ROADMAP
```

Pesquisar por nome.

---

# FILTROS

Adicionar:

```text
TODOS
ATIVOS
PAUSADOS
CONCLUÍDOS
```

Usar estados persistidos reais.

---

# NOVO ROADMAP

Preservar:

```text
+ NOVO ROADMAP
```

Não alterar CRUD existente sem necessidade.

---

# 2 — ROADMAP HEADER

Na coluna central mostrar:

```text
ENGENHARIA DE CONTROLE E AUTOMAÇÃO

EM ANDAMENTO

23%

29 / 128 atividades
```

Também:

```text
etapas
tópicos
atividades
duração estimada
```

quando esses dados existirem.

Não inventar duração.

---

# PROGRESSO

Roadmap Progress continua sendo:

```text
atividades concluídas
/
atividades totais
```

Não confundir com Knowledge Level.

Mostrar explicitamente:

```text
PROGRESSO DO ROADMAP
```

---

# ROADMAP INTERNAL NAVIGATION

Adicionar subnavegação quando útil:

```text
ESTRUTURA
VISÃO GERAL
PROJETOS
RECURSOS
NOTAS
```

Não criar abas vazias.

Se alguma seção não possuir implementação real:

não mostrar ainda.

A aba obrigatória desta versão é:

```text
ESTRUTURA
```

---

# 3 — STAGES

Roadmap deve ser dividido visualmente em etapas recolhíveis.

Exemplo:

```text
01 — FUNDAMENTAÇÃO MATEMÁTICA      72%
02 — ELETRÔNICA                    31%
03 — SISTEMAS EMBARCADOS           12%
04 — CONTROLE                       0%
05 — ROBÓTICA                       0%
06 — INTEGRAÇÃO E PROJETOS          0%
```

---

# COLLAPSIBLE STAGES

Cada etapa deve possuir:

```text
collapsed
expanded
```

Por padrão:

* etapa atual pode abrir automaticamente;
* demais podem permanecer recolhidas.

Adicionar:

```text
EXPANDIR TODAS
RECOLHER TODAS
```

se útil.

---

# STAGE PROGRESS

Mostrar:

```text
18 / 25 atividades
72%
```

calculado com dados reais.

---

# ACTIVE STAGE

Determinar visualmente a etapa atual.

Não precisa inventar inteligência complexa.

Pode ser:

primeira etapa incompleta relevante.

---

# 4 — TOPICS

Quando uma Stage estiver expandida:

mostrar seus tópicos.

Exemplo:

```text
2.1 Circuitos Básicos                100%
2.2 Componentes Eletrônicos           40%
2.3 Circuitos Analógicos               0%
2.4 Eletrônica de Potência             0%
```

---

# TOPIC STATUS

Mostrar:

```text
NOT_STARTED
EXPOSED
UNDERSTOOD
PRACTICED
APPLIED
MASTERED
```

utilizando estado calculado pelo Learning Engine.

Não recalcular mastery na UI.

---

# TOPIC PROGRESS

Mostrar:

```text
4 / 10 atividades
40%
```

Isso é progresso estrutural.

Separado de mastery.

---

# TOPIC SELECTION

Ao clicar em tópico:

selecionar.

Abrir detalhes na coluna direita.

Não expandir todas as atividades diretamente na lista central.

Essa decisão é importante para evitar novamente uma lista gigantesca.

---

# 5 — TOPIC INSPECTOR

Coluna direita:

```text
TÓPICO

Componentes Eletrônicos Fundamentais
```

Mostrar:

```text
Roadmap
Stage
Knowledge Node
Status
Progress
Mastery
```

---

# TOPIC INSPECTOR TABS

Criar:

```text
VISÃO GERAL
ATIVIDADES
RECURSOS
```

RECURSOS só deve aparecer se existir suporte real.

Obrigatórias:

```text
VISÃO GERAL
ATIVIDADES
```

---

# VISÃO GERAL DO TÓPICO

Mostrar:

## Estado de domínio

Visualizar:

```text
NÃO INICIADO
   ↓
EXPOSTO
   ↓
COMPREENDIDO
   ↓
PRATICADO
   ↓
APLICADO
   ↓
DOMINADO
```

Destacar estado atual.

Não permitir edição manual se v0.8.3 já calcula automaticamente.

---

# PROGRESSO

Mostrar:

```text
PROGRESSO

40%

4 / 10 atividades
```

---

# DESCRIÇÃO

Mostrar descrição do tópico quando existir.

---

# CONHECIMENTOS RELACIONADOS

Mostrar Knowledge Nodes vinculados.

Exemplo:

```text
Eletrônica
Semicondutores
Circuitos
Instrumentação
```

Clicar pode abrir Conhecimento dentro do Mapa Stark quando apropriado.

---

# PRÉ-REQUISITOS

Preparar suporte a pré-requisitos entre tópicos.

Estrutura conceitual:

```text
RoadmapTopicPrerequisite
```

ou reutilizar relacionamento existente.

Exemplo:

```text
Componentes Eletrônicos

Pré-requisito:
✓ Circuitos Básicos
```

---

# PREREQUISITE STATUS

Mostrar:

```text
✓ concluído
○ pendente
```

Não bloquear estudo obrigatoriamente nesta versão.

Pode apenas informar.

---

# PRÉ-REQUISITOS AUSENTES

Se tópico não possuir:

não mostrar painel vazio desnecessário.

---

# PRÓXIMOS TÓPICOS

Mostrar tópicos subsequentes relevantes.

Exemplo:

```text
Circuitos Analógicos
Eletrônica de Potência
```

Baseado na estrutura do roadmap.

Não utilizar IA para inventar sequência.

---

# 6 — ACTIVITIES

Na aba:

```text
ATIVIDADES
```

mostrar atividades apenas do tópico selecionado.

Exemplo:

```text
□ Ler fundamentos
□ Assistir aula
□ Resolver exercícios
□ Simular circuito
□ Montar circuito
□ Aplicar em projeto
```

---

# ACTIVITY ITEM

Mostrar:

```text
status
título
tipo
evidence category
```

Exemplo:

```text
□ Montar circuito com MOSFET

EXPERIMENT
Profundidade
```

Não mostrar fórmula matemática inteira.

---

# ACTIVITY INTERACTION

Permitir:

```text
PENDING
→
IN_PROGRESS
→
COMPLETED
```

A interação precisa funcionar diretamente no Roadmap.

---

# QUICK COMPLETE

Adicionar controle claro para:

```text
MARCAR COMO CONCLUÍDA
```

---

# IMPORTANTE

Concluir atividade deve utilizar o fluxo da v0.8.3:

```text
Activity
↓
Knowledge Event
↓
Learning Engine
↓
Recalculation
↓
Mapa Stark
```

A UI não altera métricas diretamente.

---

# FEEDBACK DE CONCLUSÃO

Ao concluir:

mostrar feedback discreto.

Exemplo:

```text
ATIVIDADE CONCLUÍDA

Knowledge Event registrado.

MOSFET
Profundidade ↑
```

Não mostrar XP.

---

# REOPEN

Permitir reabrir atividade concluída.

Usar mecanismo auditável da v0.8.3.

Não deletar Knowledge Event.

---

# ACTIVITY DETAILS

Ao clicar numa atividade:

mostrar detalhes.

Pode utilizar painel interno/modal coerente.

Mostrar:

```text
Título
Descrição
Tipo
Status
Knowledge relacionado
Projeto relacionado
Data de conclusão
Evidence Profile resumido
```

---

# ACTIVITY FILTER

Dentro de tópico com muitas atividades:

permitir:

```text
TODAS
PENDENTES
EM ANDAMENTO
CONCLUÍDAS
```

---

# CURRENT ACTIVITY

Identificar visualmente próxima atividade pendente.

Exemplo:

```text
PRÓXIMA ATIVIDADE
Resolver exercícios de MOSFET
```

---

# 7 — INTERAÇÃO COM LEARNING ENGINE

Após conclusão de atividade:

atualizar sem reload:

```text
activity status
topic progress
stage progress
roadmap progress
topic mastery
knowledge metrics
Mapa Stark
```

---

# EVENT FEEDBACK

Não bloquear interface esperando animações.

Atualização deve ser rápida.

---

# KNOWLEDGE IMPACT

Roadmap Header pode mostrar:

```text
IMPACTO RECENTE

Eletrônica ↑
IoT ↑
Sistemas Embarcados ↑
```

Somente quando houver eventos reais.

---

# 8 — ROADMAP OVERVIEW

Criar uma visão resumida do roadmap quando útil.

Mostrar:

```text
PROGRESSO
ETAPAS
TÓPICOS
ATIVIDADES

CURRENT STAGE
CURRENT TOPIC
NEXT ACTIVITY
```

---

# 9 — CONTINUE STUDY

Adicionar ação importante:

# CONTINUAR ESTUDO

Ao clicar:

abrir automaticamente:

```text
roadmap ativo
↓
primeira etapa relevante
↓
tópico atual
↓
próxima atividade pendente
```

Isso deve ser determinístico.

Não usar LLM para decidir nesta versão.

---

# Exemplo

```text
CONTINUAR ESTUDO
↓
Eletrônica
↓
Componentes Eletrônicos
↓
Resolver exercícios
```

---

# 10 — BREADCRUMB

Mostrar contexto:

```text
Mapa Stark
/
Roadmaps
/
Controle e Automação
/
Eletrônica
/
Componentes Eletrônicos
```

Pode ser compacto.

---

# 11 — URL/STATE NAVIGATION

Se arquitetura permitir:

preservar seleção ao trocar temporariamente de aba.

Exemplo:

usuário está em:

```text
Controle e Automação
→ Eletrônica
→ MOSFET
```

vai para Conhecimento e volta.

Idealmente retornar ao mesmo contexto.

---

# 12 — ROADMAP PROGRESS VISUAL

Não utilizar apenas uma barra global.

Mostrar progressão estrutural.

Exemplo:

```text
01 MATEMÁTICA       ███████░░ 72%
02 ELETRÔNICA       ███░░░░░░ 31%
03 EMBARCADOS       █░░░░░░░░ 12%
04 CONTROLE         ░░░░░░░░░  0%
```

---

# 13 — STATUS VISUAL

Criar linguagem consistente:

```text
NOT_STARTED
IN_PROGRESS
COMPLETED
PAUSED
```

para roadmaps/atividades quando aplicável.

Mastery continua separado.

---

# 14 — NÃO CONFUNDIR STATUS

Existem três conceitos diferentes:

```text
ROADMAP PROGRESS
atividade concluída

ACTIVITY STATUS
pending / in_progress / completed

KNOWLEDGE MASTERY
exposed / understood / practiced / applied / mastered
```

Nunca misturar.

---

# 15 — ROADMAP EDITING

Preservar CRUD existente.

Se já houver edição:

torná-la acessível através de:

```text
...
```

Não poluir interface principal com botões de administração.

---

# 16 — CRIAÇÃO DE STAGES/TOPICS/ACTIVITIES

Se CRUD já existir:

preservar.

Se faltar alguma operação essencial:

adicionar de forma coerente.

O usuário deve conseguir administrar manualmente seu roadmap.

---

# 17 — EMPTY STATES

Criar estados adequados:

```text
Nenhum roadmap cadastrado.
```

```text
Nenhuma etapa neste roadmap.
```

```text
Nenhum tópico nesta etapa.
```

```text
Nenhuma atividade neste tópico.
```

Não mostrar grandes áreas quebradas.

---

# 18 — PERFORMANCE

Um roadmap pode possuir:

```text
100+
atividades
```

Não renderizar todas detalhadamente simultaneamente.

Esse é um dos principais motivos da v0.8.4.

Usar:

```text
collapsed stages
+
topic inspector
+
activity view
```

para reduzir densidade desnecessária.

---

# 19 — ACESSIBILIDADE DE INTERAÇÃO

Todos os controles importantes devem funcionar por:

* mouse;
* teclado quando apropriado.

Hand tracking não entra no Roadmap.

---

# 20 — AI CORE

Atualizar ferramentas READ existentes quando necessário.

Azriel deve conseguir responder:

```text
Qual roadmap estou estudando?
```

```text
Onde parei?
```

```text
Qual é minha próxima atividade?
```

```text
Como está meu progresso em Eletrônica?
```

```text
Quais tópicos já concluí?
```

```text
Quais tópicos estão praticados?
```

---

# AI CORE — WRITE

Não permitir que o AI Core marque atividades como concluídas nesta versão.

Conclusão representa evidência de aprendizado.

Ela deve ser iniciada explicitamente pelo operador através da interface.

Isso evita:

```text
LLM decide que usuário concluiu algo
→ Knowledge Event
→ conhecimento aumenta
```

---

# 21 — COMMAND CENTER

Adicionar, se houver espaço apropriado:

```text
ROADMAP ATIVO

Controle e Automação
23%

ATUAL
Componentes Eletrônicos

PRÓXIMO
Resolver exercícios
```

Ao clicar:

abrir Roadmap no ponto atual.

Não obrigatório se comprometer layout.

---

# 22 — OPERAÇÕES DIÁRIAS

Preparar integração visual futura.

Uma Roadmap Activity pode futuramente gerar uma tarefa diária.

Não implementar automação bidirecional nesta versão.

Se relação já existir, apenas preservar.

---

# 23 — DESIGN

Preservar o estilo aprovado:

* fundo quase preto;
* azul petróleo;
* cyan técnico;
* linhas finas;
* alta densidade;
* tipografia técnica;
* poucos arredondamentos;
* pouco glow.

Evitar:

* LMS;
* Udemy;
* Notion;
* Trello;
* cards gigantes;
* gamificação infantil.

---

# DESIGN PRINCIPLE

A sensação deve ser:

# SISTEMA DE NAVEGAÇÃO DE CONHECIMENTO

e não:

# LISTA DE AULAS.

---

# 24 — RESPONSIVIDADE

Prioridade:

```text
1920x1080
2560x1440
```

Também validar funcionamento aceitável em:

```text
1366x768
```

No desktop amplo:

```text
Roadmap Navigator
+
Roadmap Structure
+
Topic Inspector
```

devem coexistir.

Em largura reduzida:

* Topic Inspector pode virar drawer/painel;
* Roadmap Navigator pode ser recolhível;
* conteúdo central deve manter prioridade.

Não criar overflow horizontal quebrado.

---

# 25 — PERSISTÊNCIA DE ESTADO DA INTERFACE

Persistir quando apropriado:

* roadmap selecionado;
* stage expandida;
* tópico selecionado;
* aba do Topic Inspector;
* filtros.

Não é obrigatório persistir cada detalhe visual.

O objetivo é evitar que o usuário perca constantemente o ponto onde estava.

---

# 26 — ESTADO ATUAL DO ESTUDO

Criar conceito derivado:

```text
CURRENT STUDY POSITION
```

Estrutura conceitual:

```ts
interface CurrentStudyPosition {
  roadmapId: string;
  stageId?: string;
  topicId?: string;
  activityId?: string;
}
```

Pode ser derivado deterministicamente ou persistido conforme a arquitetura atual.

---

# REGRA

`CurrentStudyPosition` não representa conhecimento.

Representa apenas:

> onde o operador está naquele caminho de estudo.

---

# 27 — ROADMAP COMPLETION

Quando todas as atividades estiverem concluídas:

mostrar:

```text
ROADMAP
COMPLETED
```

Não significa automaticamente:

```text
CONHECIMENTO
MASTERED
```

Essa distinção é obrigatória.

---

# 28 — TOPIC COMPLETION

Um tópico pode possuir:

```text
10 / 10 atividades
```

e ainda estar:

```text
PRACTICED
```

em vez de `MASTERED`.

Isso é válido.

O Learning Engine determina domínio.

---

# 29 — STAGE COMPLETION

Stage é considerada estruturalmente concluída quando todas as suas atividades elegíveis estiverem concluídas.

Mostrar:

```text
✓ ETAPA CONCLUÍDA
```

Não alterar Knowledge Metrics diretamente.

---

# 30 — VISUAL DE CONCLUSÃO

Evitar confete, XP ou gamificação.

Utilizar feedback coerente com o Azriel:

```text
STAGE 02
ELETRÔNICA

STATUS
COMPLETED
```

com alteração discreta no HUD.

---

# 31 — PRÓXIMA ATIVIDADE

Criar algoritmo determinístico:

1. roadmap ativo;
2. primeira stage incompleta;
3. primeiro topic incompleto;
4. primeira activity pendente/in_progress.

Respeitar `order`.

---

# PRÉ-REQUISITOS

Se atividade/tópico atual possuir pré-requisito não concluído:

mostrar aviso.

Não selecionar automaticamente outra atividade sem explicar.

---

# 32 — CONTINUAR ESTUDO

O botão:

```text
CONTINUAR ESTUDO
```

deve utilizar o algoritmo anterior.

Ao clicar:

* selecionar roadmap;
* expandir stage;
* selecionar topic;
* abrir aba Activities;
* destacar próxima activity.

---

# 33 — COMPLETION TRANSACTION

Marcar uma atividade como concluída envolve:

```text
Activity Status
+
Knowledge Event
+
Learning Engine
+
Roadmap Progress
```

Essa operação deve ser consistente.

Se Knowledge Event / Learning Engine falhar:

não deixar interface em estado incoerente.

Utilizar transação/rollback quando a arquitetura permitir.

---

# 34 — OPTIMISTIC UI

Não marcar definitivamente como concluída antes de confirmar persistência.

Pode existir feedback imediato visual, mas falha deve restaurar estado anterior.

---

# 35 — DUPLO CLIQUE / DUPLICAÇÃO

Proteger contra múltiplas submissões.

Uma atividade não pode gerar múltiplos Knowledge Events porque o usuário clicou duas vezes.

A idempotência da v0.8.3 continua obrigatória.

---

# 36 — ACTIVITY COMPLETION DATE

Ao concluir:

registrar:

```text
completedAt
```

Mostrar nos detalhes.

Ao reabrir:

preservar histórico através dos eventos existentes.

---

# 37 — EVIDENCE INDICATOR

Cada Activity pode mostrar discretamente sua principal natureza:

```text
READING       COBERTURA
EXERCISE      PROFUNDIDADE
EXPERIMENT    PROFUNDIDADE
PROJECT       INTEGRAÇÃO
```

Isso é orientação.

Não mostrar pontos.

---

# 38 — LEARNING ENGINE FEEDBACK

Após conclusão, permitir abrir:

```text
VER IMPACTO
```

que mostra o Knowledge Event recém-criado.

Exemplo:

```text
EVIDÊNCIA REGISTRADA

EXPERIMENT
Montar circuito com MOSFET

Conhecimentos afetados:
MOSFET
Semicondutores
Eletrônica

Impacto principal:
Profundidade

Formula:
Learning Engine V1
```

Valores detalhados podem ficar em área avançada.

---

# 39 — ROADMAP KNOWLEDGE IMPACT

Criar visão agregada baseada em Knowledge Events reais daquele roadmap.

Exemplo:

```text
IMPACTO DO ROADMAP

Eletrônica
Cobertura       ↑
Profundidade    ↑↑

Sistemas Embarcados
Cobertura       ↑
Profundidade    ↑

IoT
Integração      ↑
```

Não recalcular outra fórmula.

Consultar Knowledge Events existentes.

---

# 40 — ROADMAP STATISTICS

Mostrar quando útil:

```text
ATIVIDADES
128

CONCLUÍDAS
29

EM ANDAMENTO
2

TÓPICOS
42

ETAPAS
8
```

Dados reais.

---

# 41 — ACTIVITY COUNTERS

Stage:

```text
18 / 25
```

Topic:

```text
4 / 10
```

Roadmap:

```text
29 / 128
```

Todos devem derivar da mesma fonte de dados para evitar divergências.

---

# 42 — ROADMAP EMPTY STATE

A imagem atual possui um grande espaço vazio quando não há roadmap.

Melhorar para algo compacto:

```text
NENHUM ROADMAP

Crie um caminho de estudo para conectar:
tópicos → atividades → evidências → conhecimento.

[ + NOVO ROADMAP ]
```

Não ocupar metade da tela com empty state.

---

# 43 — CONFIRMAÇÃO DE EXCLUSÃO

Excluir:

* roadmap;
* stage;
* topic;
* activity;

deve respeitar relacionamentos existentes.

Se houver Knowledge Events relacionados:

não destruir histórico silenciosamente.

Seguir regras de integridade da v0.8.3.

---

# 44 — EVENTOS HISTÓRICOS

Uma atividade com evidências históricas não deve simplesmente desaparecer sem tratamento.

Se exclusão for necessária:

preservar Knowledge Events ou usar estratégia de archival/referência histórica.

Não quebrar capacidade de rebuild.

---

# 45 — ARCHIVE

Se for compatível com arquitetura atual, preferir:

```text
ARCHIVED
```

a exclusão destrutiva de Roadmaps já utilizados.

Não é obrigatório implementar sistema completo de archive se ainda não existir.

---

# 46 — TESTES DE UI / DOMÍNIO

Adicionar testes para:

1. roadmap selection;
2. roadmap search;
3. roadmap filters;
4. stage collapse;
5. stage expand;
6. expand all;
7. topic selection;
8. Topic Inspector;
9. Activities tab;
10. activity filters;
11. pending → in_progress;
12. in_progress → completed;
13. completion gera Knowledge Event;
14. double completion não duplica evento;
15. completed → reopened;
16. topic progress;
17. stage progress;
18. roadmap progress;
19. progress ≠ mastery;
20. Continue Study;
21. current study position;
22. prerequisite display;
23. next topic;
24. Learning Engine failure;
25. empty states;
26. large roadmap;
27. state persistence;
28. AI read queries;
29. responsive layout;
30. backward compatibility.

---

# 47 — TESTE DE ROADMAP GRANDE

Criar fixture com aproximadamente:

```text
8 stages
40 topics
120+ activities
```

Confirmar que:

* interface permanece navegável;
* todas as atividades não aparecem simultaneamente;
* stages recolhíveis funcionam;
* seleção de tópico é rápida;
* Topic Inspector não causa renderização excessiva;
* filtros funcionam.

---

# 48 — TESTE DE INTEGRAÇÃO COM LEARNING ENGINE

Criar:

```text
Roadmap
Controle e Automação

Stage
Eletrônica

Topic
MOSFET

Activities
Reading
Exercise
Experiment
Project
```

Concluir `Reading`.

Confirmar:

```text
Activity → COMPLETED
KnowledgeEvent → CREATED
Topic Progress → UPDATED
Stage Progress → UPDATED
Roadmap Progress → UPDATED
Knowledge → RECALCULATED
```

Sem reload.

---

# 49 — TESTE DE REABERTURA

Reabrir `Reading`.

Confirmar:

```text
Activity → PENDING
Reversal Event → CREATED
Progress → UPDATED
Knowledge → RECALCULATED
```

Sem apagar histórico.

---

# 50 — TESTE CONTINUAR ESTUDO

Roadmap contendo:

```text
Stage 01 — completed

Stage 02
  Topic A — completed
  Topic B
    Activity 1 — completed
    Activity 2 — pending
    Activity 3 — pending
```

Clicar:

```text
CONTINUAR ESTUDO
```

Resultado:

```text
Stage 02 expanded
Topic B selected
Activities opened
Activity 2 highlighted
```

---

# 51 — TESTE DE MASTERY

Criar tópico com:

```text
100% das atividades concluídas
```

mas Learning Engine retornando:

```text
PRACTICED
```

UI deve mostrar:

```text
PROGRESSO
100%

DOMÍNIO
PRATICADO
```

Não transformar automaticamente em `MASTERED`.

---

# 52 — TESTE DE PERSISTÊNCIA

Selecionar:

```text
Roadmap A
Stage 03
Topic B
Activities
```

Fechar/reabrir Azriel quando a persistência prevista se aplicar.

Confirmar restauração razoável do contexto.

---

# 53 — MIGRATIONS

Criar migration apenas se necessário para:

* prerequisites;
* current study position;
* preferências;
* novos relacionamentos.

Não alterar migrations antigas.

Não modificar schema apenas por conveniência visual.

---

# 54 — DOCUMENTAÇÃO

Criar:

```text
docs/versions/v0.8.4.md
```

Documentar:

* objetivo;
* problema da UI anterior;
* arquitetura de navegação;
* Roadmap Navigator;
* stages;
* topics;
* Topic Inspector;
* activities;
* Continue Study;
* current study position;
* progress vs mastery;
* Learning Engine integration;
* prerequisites;
* persistência;
* performance;
* limitações.

Atualizar:

```text
docs/roadmap.md
```

Registrar:

```text
v0.8.2 — Stark Knowledge System — concluída
v0.8.3 — Learning Engine — concluída
v0.8.4 — Interactive Roadmap Experience
```

---

# 55 — NÃO IMPLEMENTAR NESTA VERSÃO

Fora do escopo:

* geração automática de roadmap por IA;
* recomendação automática de estudo;
* scheduler;
* calendário de estudos;
* streak;
* XP;
* badges;
* ranking;
* competição;
* IA marcando atividade como concluída;
* alteração automática de evidência;
* voz;
* IoT;
* integração com Engineering Core.

---

# CRITÉRIOS DE ACEITE

A v0.8.4 só está concluída quando:

1. Roadmaps deixarem de aparecer como lista gigante;
2. Roadmap Navigator existir;
3. busca funcionar;
4. filtros funcionarem;
5. roadmap puder ser selecionado;
6. header mostrar progresso real;
7. stages forem recolhíveis;
8. stage progress funcionar;
9. topics aparecerem dentro das stages;
10. topic progress funcionar;
11. topic mastery vier do Learning Engine;
12. Topic Inspector existir;
13. Visão Geral do tópico existir;
14. Activities do tópico existirem em área própria;
15. atividades puderem ser iniciadas;
16. atividades puderem ser concluídas;
17. atividades puderem ser reabertas;
18. conclusão gerar Knowledge Event;
19. reabertura gerar reversão;
20. métricas atualizarem automaticamente;
21. roadmap progress atualizar;
22. stage progress atualizar;
23. topic progress atualizar;
24. progress e mastery permanecerem separados;
25. Knowledge Nodes relacionados aparecerem;
26. prerequisites puderem ser exibidos;
27. próximos tópicos aparecerem;
28. Continue Study funcionar;
29. current study position funcionar;
30. próxima atividade ser determinística;
31. activity filters funcionarem;
32. activity details funcionarem;
33. feedback de evidência funcionar;
34. Roadmap Knowledge Impact usar eventos reais;
35. roadmap grande continuar utilizável;
36. nenhuma lista de 100+ atividades ser renderizada aberta por padrão;
37. estado da interface for preservado quando apropriado;
38. AI Core conseguir consultar posição/progresso;
39. AI Core não puder concluir atividade;
40. Knowledge Events não puderem ser duplicados por UI;
41. histórico permanecer auditável;
42. migrations preservarem dados;
43. TypeScript não apresentar erros;
44. testes relevantes passarem;
45. build funcionar;
46. Tauri iniciar normalmente;
47. módulos anteriores não sofrerem regressão;
48. `docs/versions/v0.8.4.md` existir.

---

# TESTE FINAL OBRIGATÓRIO

Utilizar o roadmap real:

```text
Eletrônica e Sistemas Embarcados com ESP32
```

que já existe no banco.

Abrir:

```text
Mapa Stark
→ Roadmaps
```

Confirmar que ele aparece no Roadmap Navigator.

Selecioná-lo.

A coluna central deve mostrar suas etapas recolhidas.

Expandir:

```text
01 — Fundamentos de Eletricidade e Circuitos
```

Selecionar:

```text
Grandezas elétricas fundamentais
```

Confirmar que o Topic Inspector abre sem expandir todas as atividades na estrutura central.

Abrir:

```text
ATIVIDADES
```

Selecionar a primeira atividade.

Marcar:

```text
IN_PROGRESS
```

Confirmar persistência.

Depois:

```text
COMPLETED
```

Confirmar:

```text
Activity completed
↓
Knowledge Event created
↓
Learning Engine recalculated
↓
Topic progress updated
↓
Stage progress updated
↓
Roadmap progress updated
↓
Mapa Stark updated
```

Selecionar outro tópico.

Voltar.

Confirmar preservação razoável de contexto.

Clicar:

```text
CONTINUAR ESTUDO
```

Confirmar que o sistema encontra a próxima atividade pendente e abre exatamente seu contexto.

Reabrir uma atividade concluída.

Confirmar reversal e recálculo.

Finalmente testar o roadmap completo com suas:

```text
35 atividades atuais
```

e confirmar que a experiência agora parece um caminho navegável, não uma lista extensa.

---

# RESULTADO ESPERADO

Antes:

```text
ROADMAP
│
├── atividade
├── atividade
├── atividade
├── atividade
├── atividade
├── atividade
├── atividade
├── ...
└── atividade
```

Depois:

```text
ROADMAP
│
├── ETAPA 01 ───────── 72%
│   ├── Tópico
│   ├── Tópico
│   └── Tópico
│
├── ETAPA 02 ───────── 31%
│   ├── ✓ Circuitos Básicos
│   ├── ◉ Componentes Eletrônicos ← VOCÊ ESTÁ AQUI
│   ├── ○ Circuitos Analógicos
│   └── ○ Eletrônica de Potência
│
├── ETAPA 03 ───────── 12%
├── ETAPA 04 ─────────  0%
└── ...
```

Ao selecionar um tópico:

```text
TÓPICO
↓
VISÃO GERAL
↓
ATIVIDADES
↓
EVIDÊNCIAS
↓
LEARNING ENGINE
↓
CONHECIMENTO
```

O operador deve conseguir abrir o Azriel e, em poucos segundos, responder:

```text
Onde estou?
O que estou estudando?
O que já fiz?
Qual é a próxima atividade?
Quanto falta?
Que conhecimento isso está desenvolvendo?
```

O Roadmap deixa de ser armazenamento de tarefas.

Ele passa a ser:

# UMA INTERFACE PARA NAVEGAR PELO PRÓPRIO PROCESSO DE APRENDIZADO.

A filosofia da versão é:

# NÃO MOSTRE TODO O CAMINHO DE UMA VEZ. MOSTRE ONDE ESTOU, O QUE VEM DEPOIS E COMO ESTOU EVOLUINDO.

---

# Estado da implementação — 04/09/2026

Status: **implementação concluída; validação operacional no aplicativo Tauri pendente**.

## Arquitetura entregue

```text
Roadmap Navigator
↓
Roadmap Structure
↓
Topic Inspector / Activities
↓
transição atômica no backend
↓
Learning Engine
↓
Knowledge Events + recálculo
```

O Roadmap deixou de renderizar todas as atividades em cartões extensos. A interface usa três áreas especializadas, etapas recolhíveis e detalhes de atividade sob demanda. Busca, filtros, roadmap selecionado, tópico selecionado e etapas abertas preservam contexto local quando apropriado.

## Persistência e pré-requisitos

A migration `0013_interactive_roadmaps.sql` adiciona relações estruturadas entre tópicos e seus pré-requisitos. O editor aceita múltiplos pré-requisitos já existentes e o backend rejeita referências inválidas ou autorreferências. Registros legados que possuíam um ID válido no campo textual `Pré-requisitos:` são promovidos de forma conservadora.

## Atividades e Learning Engine

As transições `pending`, `in_progress` e `completed` possuem comando dedicado. A conclusão, o Knowledge Event, o recálculo e a persistência acontecem na mesma transação. Reabrir preserva o evento original e cria sua reversão. Repetir a mesma transição é idempotente.

A atualização retorna os roadmaps e impactos recalculados para a interface. Conhecimentos, eventos, progresso do tópico, etapa e roadmap são atualizados sem recarregar a página inteira.

## Posição atual

`CONTINUAR ESTUDO` procura primeiro uma atividade em andamento e, depois, a primeira pendente na ordem persistida. A interface expande a etapa, seleciona o tópico, abre Atividades e destaca o item encontrado. O AI Core utiliza a mesma regra por uma ferramenta somente leitura e não possui permissão para concluir atividades.

## Validação automatizada

Foram adicionados testes para progresso estrutural, filtros, posição determinística, perfil de evidência, transição direta, idempotência, reversão, migration e consulta do AI Core. O aceite visual e operacional no WebView Tauri permanece necessário antes de marcar a versão como concluída.
