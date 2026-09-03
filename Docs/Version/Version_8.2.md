# AZRIEL v0.8.2 — Stark Knowledge System

O Azriel já possui módulos separados relacionados à evolução intelectual do operador:

* Mapa Stark;
* Conhecimento;
* Pesquisa;
* Roadmaps de estudo.

Essa separação começou a gerar sobreposição conceitual e navegação desnecessária.

Agora implemente:

# v0.8.2 — Stark Knowledge System

Esta versão é uma **consolidação estrutural e visual**.

NÃO implementar ainda o cálculo automático de evolução baseado em atividades.

O Learning Engine será implementado na próxima versão.

---

# Antes de modificar qualquer arquivo

1. leia completamente o `README.md`;
2. leia `docs/roadmap.md`;
3. examine a implementação atual de:

   * Mapa Stark;
   * Conhecimento;
   * Pesquisa;
   * Roadmaps;
   * Knowledge Core;
   * SQLite;
   * migrations;
   * services;
   * repositories;
4. identifique todas as tabelas e relacionamentos utilizados;
5. identifique dados reais existentes;
6. execute lint, testes e build antes de alterar;
7. preserve todos os dados existentes.

Não apagar tabelas ou registros simplesmente porque a UI será reorganizada.

---

# Objetivo

Transformar:

```text
Pesquisa
Conhecimento
Mapa Stark
Roadmaps
```

em um único domínio funcional chamado:

# MAPA STARK

O Mapa Stark deixa de ser apenas uma visualização de:

* cobertura;
* profundidade;
* integração.

Ele passa a ser a central de:

# CONHECIMENTO + APRENDIZADO + PESQUISA + EVOLUÇÃO

---

# Navegação principal

Remover da navegação principal:

```text
Conhecimento
Pesquisa
```

Não remover os dados.

Essas funcionalidades passam a existir dentro do Mapa Stark.

Manter `Roadmaps` dentro do Mapa Stark.

A navegação principal deve ficar conceitualmente semelhante a:

```text
CENTRAL
PROJETOS
MAPA STARK
FORMAÇÃO
OPERAÇÕES DIÁRIAS
SISTEMA
AUTOMAÇÃO
ENGINEERING CORE
CONFIGURAÇÕES
```

Adaptar aos nomes reais existentes.

---

# Estrutura interna do Mapa Stark

Criar navegação interna:

```text
MAPA STARK

├── VISÃO GERAL
├── CONHECIMENTO
├── ROADMAPS
├── PESQUISA
├── EVOLUÇÃO
└── LACUNAS
```

Pode utilizar tabs, sidebar interna ou outra solução coerente com o HUD atual.

Não criar seis aplicações diferentes.

Tudo deve parecer parte de um único sistema.

---

# 1 — VISÃO GERAL

Esta será a página inicial do Mapa Stark.

Mostrar:

```text
COBERTURA
PROFUNDIDADE
INTEGRAÇÃO
```

Preservar os valores atuais.

Também mostrar:

* principais áreas;
* maiores lacunas;
* evolução recente quando houver dados;
* roadmaps ativos;
* pesquisas ativas;
* conhecimentos prioritários.

---

# Visualização Stark

Preservar o gráfico atual sempre que possível.

Não reconstruir apenas por preferência visual.

Ele deve continuar permitindo comparar:

```text
COBERTURA
vs
PROFUNDIDADE
```

por área.

---

# Baseline

Os valores atuais do Mapa Stark são importantes.

Eles representam conhecimento adquirido antes do novo sistema automático.

Criar conceito:

# KNOWLEDGE BASELINE

O baseline representa o estado conhecido antes da introdução do Learning Engine.

Estrutura conceitual:

```ts
interface KnowledgeBaseline {
  knowledgeAreaId: string;
  coverage: number;
  depth: number;
  recordedAt: string;
}
```

---

# Regra do baseline

Não zerar valores existentes.

Exemplo:

```text
ELETRÔNICA

Cobertura atual:
20

Profundidade atual:
10
```

deve gerar baseline equivalente.

Posteriormente:

```text
valor atual =
baseline
+
evolução calculada
```

Mas NÃO implementar essa fórmula ainda.

Apenas preparar os dados.

---

# Baseline idempotente

A migration/seed não pode criar novos baselines toda vez que o Azriel iniciar.

Cada conhecimento deve possuir apenas o baseline inicial apropriado.

---

# 2 — CONHECIMENTO

Mover a experiência atual do módulo Conhecimento para dentro:

```text
Mapa Stark → Conhecimento
```

Preservar:

* áreas;
* categorias;
* cobertura;
* profundidade;
* prioridade;
* projetos relacionados;
* histórico;
* lacunas.

---

# Hierarquia de conhecimento

Preparar uma estrutura mais rica:

```text
ÁREA
↓
DISCIPLINA
↓
TÓPICO
↓
COMPETÊNCIA
```

Exemplo:

```text
Engenharia
└── Eletrônica
    ├── Circuitos
    │   ├── Lei de Ohm
    │   └── Kirchhoff
    │
    ├── Semicondutores
    │   ├── Diodos
    │   ├── Transistores
    │   └── MOSFET
    │
    └── Instrumentação
```

Não é obrigatório migrar todo conhecimento existente para essa profundidade agora.

Mas a arquitetura deve suportar.

---

# Knowledge Node

Criar ou adaptar uma estrutura conceitual:

```ts
interface KnowledgeNode {
  id: string;
  name: string;

  type:
    | "area"
    | "discipline"
    | "topic"
    | "competency";

  parentId?: string;

  coverage: number;
  depth: number;

  priority?: string;

  createdAt: string;
  updatedAt: string;
}
```

Adaptar ao schema atual sem destruir compatibilidade.

---

# 3 — ROADMAPS

Mover/expandir o sistema atual para:

```text
Mapa Stark → Roadmaps
```

O sistema deve suportar múltiplos roadmaps.

Exemplo:

```text
ROADMAPS

Controle e Automação
Biologia Molecular
Bioinformática
IoT
Matemática para Engenharia
Inteligência Artificial
```

Não hardcodar esses roadmaps se já houver sistema persistente.

---

# Roadmap

Estrutura conceitual:

```text
ROADMAP
│
├── ETAPA
│   ├── TÓPICO
│   │   ├── ATIVIDADE
│   │   ├── ATIVIDADE
│   │   └── ATIVIDADE
│   │
│   └── TÓPICO
│
└── ETAPA
```

---

# Roadmap Entity

Criar/adaptar:

```ts
interface StudyRoadmap {
  id: string;
  name: string;
  description?: string;
  status: "planned" | "active" | "paused" | "completed";

  createdAt: string;
  updatedAt: string;
}
```

---

# Roadmap Stage

```ts
interface RoadmapStage {
  id: string;
  roadmapId: string;

  name: string;
  description?: string;

  order: number;
}
```

---

# Roadmap Topic

```ts
interface RoadmapTopic {
  id: string;
  stageId: string;

  name: string;
  description?: string;

  knowledgeNodeId?: string;

  order: number;
}
```

A relação:

```text
Roadmap Topic
↕
Knowledge Node
```

é extremamente importante.

---

# Roadmap Activity

Preparar:

```ts
interface RoadmapActivity {
  id: string;
  topicId: string;

  title: string;
  description?: string;

  type: ActivityType;

  status:
    | "pending"
    | "in_progress"
    | "completed";

  completedAt?: string;

  order: number;
}
```

---

# Activity Types

Preparar tipos:

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

Esses tipos serão utilizados pelo Learning Engine na próxima versão.

Nesta versão:

não calcular impacto automaticamente.

---

# Estado de domínio do tópico

Preparar níveis:

```text
NOT_STARTED
EXPOSED
UNDERSTOOD
PRACTICED
APPLIED
MASTERED
```

Não atualizar automaticamente ainda.

Pode ser manual na v0.8.2.

---

# Roadmap Progress

Calcular progresso estrutural simples:

```text
atividades concluídas
/
atividades totais
```

Isso NÃO é conhecimento.

Manter distinção:

```text
ROADMAP PROGRESS
≠
KNOWLEDGE LEVEL
```

Essa separação é obrigatória.

---

# Exemplo

```text
CONTROLE E AUTOMAÇÃO

PROGRESSO DO ROADMAP
24%

Fundamentos Matemáticos
████████░░ 72%

Eletrônica
███░░░░░░░ 31%

Sistemas Embarcados
█░░░░░░░░░ 12%

Controle
░░░░░░░░░░ 0%
```

---

# Tela do tópico

Ao abrir um tópico:

mostrar:

```text
MOSFET

ROADMAP
Controle e Automação

CONHECIMENTO
Eletrônica → Semicondutores → MOSFET

ESTADO
EXPOSED

ATIVIDADES

✓ Ler fundamentos
✓ Assistir aula
□ Resolver exercícios
□ Simular circuito
□ Montar circuito
□ Aplicar em projeto
```

---

# 4 — PESQUISA

Mover a experiência existente:

```text
Pesquisa
```

para:

```text
Mapa Stark → Pesquisa
```

Preservar todos os dados.

---

# Pesquisa contextual

Preparar relações opcionais entre pesquisa e:

```text
Knowledge Node
Roadmap
Roadmap Topic
Project
```

Exemplo:

```text
PESQUISA

Controle PID

Relacionada a:
Roadmap: Controle e Automação
Tópico: PID
Conhecimento: Controle
Projeto: ArcCore
```

---

# Não transformar pesquisa em conhecimento automaticamente

Concluir uma pesquisa NÃO deve alterar cobertura/profundidade nesta versão.

O Learning Engine cuidará disso.

---

# 5 — EVOLUÇÃO

Criar nova área:

```text
Mapa Stark → Evolução
```

Nesta versão ela será principalmente estrutural.

Mostrar:

* histórico já existente;
* baselines;
* alterações manuais existentes;
* espaço preparado para Knowledge Events.

---

# Knowledge Event

Criar a entidade agora.

Ela será fundamental na próxima versão.

Estrutura conceitual:

```ts
interface KnowledgeEvent {
  id: string;

  knowledgeNodeId: string;

  sourceType:
    | "baseline"
    | "roadmap"
    | "project"
    | "research"
    | "manual"
    | "education";

  sourceId?: string;

  eventType: string;

  coverageDelta: number;
  depthDelta: number;
  integrationDelta: number;

  description?: string;

  createdAt: string;
}
```

---

# IMPORTANTE

Nesta v0.8.2:

NÃO gerar automaticamente Knowledge Events quando atividades forem concluídas.

Apenas criar:

* schema;
* repository;
* service;
* tipos;
* visualização básica.

A automação entra na:

# v0.8.3 — Learning Engine

---

# Knowledge Event immutability

Knowledge Events devem ser tratados conceitualmente como histórico.

Evitar edição destrutiva silenciosa.

Se um evento precisar ser corrigido futuramente, preferir estratégia auditável.

Preparar arquitetura.

---

# Evolução visual

Se existirem eventos/baselines suficientes:

mostrar timeline.

Exemplo:

```text
ELETRÔNICA

BASELINE
20 / 10

EVENTOS
...

COBERTURA
...

PROFUNDIDADE
...
```

Não inventar histórico inexistente.

---

# Origem dos valores

Na UI deve ser possível futuramente responder:

```text
POR QUE ESTE NÍVEL É 47%?
```

Preparar interface:

```text
ORIGEM

Baseline          +20
Roadmaps          +...
Projetos          +...
Pesquisa          +...
Formação          +...
```

Nesta versão alguns valores podem permanecer vazios.

Não fabricar dados.

---

# 6 — LACUNAS

Mover Gap Diagnostics para:

```text
Mapa Stark → Lacunas
```

Preservar regras atuais.

Mostrar:

* conhecimento;
* cobertura;
* profundidade;
* prioridade;
* projetos relacionados;
* roadmaps relacionados.

---

# Relação lacuna → roadmap

Quando possível:

mostrar:

```text
LACUNA
Eletrônica

ROADMAPS RELACIONADOS
Controle e Automação
IoT
```

Sem recomendar automaticamente novos tópicos ainda.

---

# Banco

Reutilizar SQLite existente.

Criar migrations novas.

Não editar migrations antigas.

---

# Migração

Esta é uma versão sensível.

Antes de mudar schema:

identificar dados existentes.

Criar migrations que preservem:

* knowledge areas;
* history;
* research;
* roadmaps;
* relationships;
* projects;
* education.

---

# Nenhuma perda de dados

Esse é critério obrigatório.

Se alguma estrutura antiga precisar ser substituída:

migrar dados primeiro.

Não dropar tabela com dados sem migration explícita.

---

# Compatibilidade

Services antigos podem ser adaptados.

Evitar manter dois sistemas paralelos indefinidamente.

Ao final:

a UI principal deve utilizar o novo domínio consolidado.

---

# Estrutura sugerida

Adaptar à arquitetura existente.

Conceitualmente:

```text
stark/
├── overview/
├── knowledge/
├── roadmaps/
├── research/
├── evolution/
├── gaps/
└── services/
```

---

# Stark Knowledge Service

Criar uma camada de domínio que possa coordenar:

* knowledge;
* roadmap;
* research;
* evolution;
* gaps.

Não criar um "God Service" gigante.

Separar responsabilidades internamente.

---

# Interface

Preservar a identidade HUD atual.

O Mapa Stark deve parecer uma:

# CENTRAL DE EVOLUÇÃO INTELECTUAL

Não uma aplicação de cursos.

Evitar aparência de:

* Udemy;
* Trello;
* LMS corporativo;
* checklist genérico.

---

# Visão Geral visual

Pode mostrar algo conceitualmente semelhante:

```text
MAPA STARK

COBERTURA       35%
PROFUNDIDADE    28%
INTEGRAÇÃO      22%

ROADMAPS
03 ativos

PESQUISAS
05 abertas

LACUNAS
04 críticas

CONHECIMENTO
21 áreas
```

Usar dados reais.

---

# Command Center

Atualizar referências.

Onde existirem links separados para:

```text
Conhecimento
Pesquisa
Mapa Stark
```

consolidar para:

```text
Mapa Stark
```

---

# AI Core

Atualizar tools existentes sem quebrar compatibilidade.

O AI Core deve continuar conseguindo consultar:

* conhecimento;
* gaps;
* Stark Map;
* research;
* roadmaps.

Internamente podem apontar para novos services.

---

# Novas consultas

Preparar suporte para:

```text
Quais roadmaps estão ativos?
```

```text
Como está meu roadmap de Controle e Automação?
```

```text
Quais tópicos ainda não comecei?
```

```text
Quais conhecimentos estão ligados a este roadmap?
```

```text
Quais pesquisas estão relacionadas a Bioinformática?
```

```text
Por que minha cobertura em Eletrônica está nesse nível?
```

Para a última pergunta:

responder apenas com dados disponíveis.

Se só houver baseline:

informar isso.

---

# Não implementar nesta versão

Fora do escopo:

* cálculo automático de cobertura;
* cálculo automático de profundidade;
* cálculo automático de integração;
* pesos de atividades;
* progressão automática;
* gamificação;
* XP;
* níveis artificiais;
* recomendação automática;
* geração automática de roadmap por IA;
* alteração automática de conhecimento pelo LLM.

Tudo isso será avaliado na v0.8.3.

---

# Testes

Adicionar testes principalmente para:

1. migration sem perda;
2. baseline;
3. baseline idempotente;
4. knowledge hierarchy;
5. roadmap CRUD;
6. stages;
7. topics;
8. activities;
9. roadmap progress;
10. progress ≠ knowledge;
11. research relationships;
12. KnowledgeEvent schema;
13. gaps;
14. AI compatibility;
15. consolidated navigation.

---

# Teste de migração obrigatório

Usar um banco representando v0.8.1 com dados.

Executar migrations.

Confirmar que:

* projetos permanecem;
* conhecimentos permanecem;
* métricas permanecem;
* pesquisas permanecem;
* roadmaps permanecem;
* histórico permanece;
* operações diárias permanecem;
* configurações permanecem;
* Automation Core permanece intacto.

---

# Documentação

Criar:

```text
docs/versions/v0.8.2.md
```

Documentar:

* objetivo;
* motivação da consolidação;
* arquitetura;
* navegação;
* baseline;
* knowledge hierarchy;
* roadmaps;
* research;
* evolution;
* Knowledge Events;
* gaps;
* migrations;
* compatibilidade;
* limitações.

Atualizar:

```text
docs/roadmap.md
```

Registrar:

```text
v0.8.1 — concluída
v0.8.2 — Stark Knowledge System
v0.8.3 — Learning Engine — planejada
```

---

# Critérios de aceite

A v0.8.2 só está concluída quando:

1. Mapa Stark for o domínio principal;
2. Conhecimento sair da navegação principal;
3. Pesquisa sair da navegação principal;
4. nenhum dado for apagado;
5. Visão Geral existir;
6. Conhecimento existir dentro do Mapa Stark;
7. Roadmaps existir dentro do Mapa Stark;
8. Pesquisa existir dentro do Mapa Stark;
9. Evolução existir;
10. Lacunas existir;
11. valores atuais forem preservados;
12. baseline existir;
13. baseline for idempotente;
14. knowledge hierarchy estiver preparada;
15. múltiplos roadmaps forem suportados;
16. roadmap possuir stages;
17. stages possuírem topics;
18. topics puderem relacionar knowledge;
19. topics possuírem activities;
20. activities possuírem tipos;
    21.Continuando os critérios de aceite da **v0.8.2 — Stark Knowledge System**:

21) activities possuírem tipos;
22) progresso estrutural do roadmap funcionar;
23) `roadmap progress` permanecer separado de `knowledge level`;
24) tópicos puderem possuir estado manual;
25) pesquisas puderem ser relacionadas a conhecimento;
26) pesquisas puderem ser relacionadas a roadmaps;
27) pesquisas puderem ser relacionadas a projetos;
28) KnowledgeEvent existir;
29) KnowledgeEvent ainda não for criado automaticamente;
30) Gap Diagnostics funcionar dentro do Mapa Stark;
31) AI Core continuar consultando conhecimento;
32) AI Core continuar consultando lacunas;
33) AI Core conseguir consultar roadmaps;
34) AI Core conseguir consultar pesquisas;
35) Command Center não apontar mais para módulos removidos;
36) migrations preservarem banco existente;
37) SQLite permanecer íntegro;
38) TypeScript não apresentar erros;
39) testes relevantes passarem;
40) build funcionar;
41) Tauri iniciar normalmente;
42) módulos anteriores não sofrerem regressão;
43) `docs/versions/v0.8.2.md` existir.

---

# Teste final obrigatório

Utilizar um banco real de desenvolvimento contendo dados das versões anteriores.

Antes da migration:

registrar ou conferir pelo menos:

```text
1 projeto
1 knowledge area
1 pesquisa
1 roadmap
1 tarefa diária
1 workspace
1 aplicação autorizada
1 rotina
```

Executar a migration da v0.8.2.

Abrir Azriel.

Confirmar que todos continuam existentes.

---

## Navegação

Confirmar que a navegação principal não possui mais:

```text
Conhecimento
Pesquisa
```

Abrir:

```text
Mapa Stark
```

Confirmar as áreas:

```text
Visão Geral
Conhecimento
Roadmaps
Pesquisa
Evolução
Lacunas
```

---

## Baseline

Selecionar uma área existente.

Exemplo:

```text
Eletrônica
```

Confirmar que seus valores anteriores de:

```text
Cobertura
Profundidade
```

foram preservados.

Confirmar existência do baseline correspondente.

Fechar Azriel.

Abrir novamente.

Confirmir que outro baseline não foi criado.

---

## Roadmap

Criar ou utilizar:

```text
Controle e Automação
```

Criar etapa:

```text
Eletrônica
```

Criar tópico:

```text
MOSFET
```

Relacionar ao conhecimento correspondente.

Adicionar atividades:

```text
Ler fundamentos
Resolver exercícios
Simular circuito
Montar circuito
Aplicar em projeto
```

Marcar duas como concluídas.

Confirmar:

```text
PROGRESSO DO ROADMAP
2 / 5
```

ou percentual equivalente.

Confirmar que isso NÃO modificou automaticamente:

```text
Cobertura
Profundidade
Integração
```

---

## Pesquisa

Abrir:

```text
Mapa Stark → Pesquisa
```

Criar ou editar uma pesquisa.

Relacionar a:

```text
Roadmap
Knowledge
Project
```

Confirmar persistência.

---

## Evolução

Abrir:

```text
Mapa Stark → Evolução
```

Confirmar visualização do baseline e histórico real existente.

Confirmar que nenhum histórico falso foi criado.

---

## Lacunas

Abrir:

```text
Mapa Stark → Lacunas
```

Confirmar que o Gap Diagnostics continua utilizando dados reais existentes.

---

## AI Core

Perguntar:

```text
Quais roadmaps estão ativos?
```

Depois:

```text
Como está meu roadmap de Controle e Automação?
```

Depois:

```text
Quais pesquisas estão relacionadas a Eletrônica?
```

Depois:

```text
Por que minha cobertura em Eletrônica está nesse nível?
```

Se somente baseline existir, Azriel deve dizer isso claramente.

Não inventar eventos de evolução.

---

# Resultado esperado

Ao concluir a v0.8.2, o Azriel deve deixar de tratar:

```text
Conhecimento
Pesquisa
Roadmaps
Mapa Stark
```

como sistemas independentes.

Eles passam a formar:

# STARK KNOWLEDGE SYSTEM

A arquitetura deve representar:

```text
                    MAPA STARK
                        │
       ┌────────────────┼────────────────┐
       │                │                │
  CONHECIMENTO       ROADMAPS         PESQUISA
       │                │                │
       └────────────────┼────────────────┘
                        │
                    EVOLUÇÃO
                        │
                 KNOWLEDGE EVENTS
                        │
                        ▼
                     LACUNAS
```

Nesta versão:

```text
ROADMAP
→ organiza o caminho

CONHECIMENTO
→ representa o estado

PESQUISA
→ registra investigação

EVOLUÇÃO
→ mostra o histórico

MAPA STARK
→ integra tudo
```

Na próxima versão:

# v0.8.3 — Learning Engine

a conclusão de atividades deixará de ser apenas progresso de checklist.

Ela começará a produzir evidências rastreáveis capazes de atualizar o Knowledge Core automaticamente.

A filosofia desta versão é:

# PRIMEIRO UNIFICAR O SISTEMA. DEPOIS AUTOMATIZAR A EVOLUÇÃO.
