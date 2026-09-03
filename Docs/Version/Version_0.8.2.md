# AZRIEL v0.8.2 — Stark Knowledge System

Status: **implementada em 03/09/2026; validação operacional no Tauri pendente**.

## Objetivo

A v0.8.2 consolida Mapa Stark, Conhecimento, Roadmaps, Pesquisa, Evolução e Lacunas em um único domínio. A navegação principal deixa de expor Conhecimento e Pesquisa como módulos separados, sem apagar seus dados.

## Arquitetura

```text
Mapa Stark
├── Visão Geral
├── Conhecimento
├── Roadmaps
├── Pesquisa
├── Evolução
└── Lacunas
```

O backend mantém responsabilidades separadas entre o repositório legado de conhecimento e o novo domínio Stark para baselines, roadmaps, pesquisas e eventos. A UI usa serviços tipados e comandos Tauri; o AI Core consulta os mesmos dados persistentes.

## Migration 0011

A migration `0011_stark_knowledge_system.sql` é aditiva e preserva as tabelas anteriores. Ela:

- adapta `knowledge_areas` com `node_type` e `parent_id`;
- cria `knowledge_baselines`;
- cria `study_roadmaps`, `roadmap_stages`, `roadmap_topics` e `roadmap_activities`;
- cria `research_items` e seus vínculos opcionais;
- cria `knowledge_events`.

Os valores de cobertura e profundidade existentes são copiados para um baseline único por conhecimento. A rotina de inicialização também garante, de forma idempotente, baseline para bancos novos e para novos conhecimentos.

Os seis registros anteriormente simulados na tela Pesquisa são importados uma única vez para SQLite. Um marcador em `_azriel_seeds` impede que uma pesquisa excluída pelo operador reapareça na próxima inicialização.

## Knowledge hierarchy

`knowledge_areas` passa a representar nós dos tipos:

```text
area → discipline → topic → competency
```

Os IDs anteriores são mantidos e recebem o tipo `area`. Relações existentes com projetos, tarefas, notas e histórico continuam apontando para os mesmos registros.

## Roadmaps

Roadmaps possuem etapas, tópicos e atividades ordenadas. Tópicos podem apontar para um conhecimento e possuem estado manual entre `NOT_STARTED` e `MASTERED`.

O progresso é calculado exclusivamente por:

```text
atividades concluídas / atividades totais
```

Salvar ou concluir uma atividade não altera cobertura, profundidade ou integração e não cria Knowledge Events.

## Pesquisa

Pesquisas agora são persistentes e podem se relacionar opcionalmente a conhecimento, roadmap, tópico de roadmap e projeto. O backend valida que um tópico realmente pertence ao roadmap informado.

Concluir uma pesquisa não altera métricas de conhecimento nesta versão.

## Evolução e Knowledge Events

A aba Evolução reúne:

- baseline;
- `knowledge_history` existente;
- Knowledge Events reais, quando existirem.

`knowledge_events` é uma estrutura de histórico append-only. Não existem comandos de edição ou exclusão e nenhum evento é produzido automaticamente na v0.8.2.

## AI Core

As tools anteriores de conhecimento e Mapa Stark permanecem compatíveis. Foram adicionadas consultas para:

- listar roadmaps;
- consultar detalhes de um roadmap;
- listar pesquisas e relações;
- explicar a origem disponível de uma métrica por baseline, histórico e eventos.

Quando somente o baseline existe, esse limite é enviado explicitamente ao modelo. O Learning Engine permanece desativado.

## Segurança e integridade

- nenhuma migration anterior foi modificada;
- nenhuma tabela anterior é removida;
- relações usam foreign keys e políticas explícitas de cascade/set-null;
- atividades e pesquisas não atualizam conhecimento automaticamente;
- Knowledge Events não podem ser editados pela API desta versão.

## Validação automatizada

- frontend: 159 testes aprovados;
- Rust: 46 testes aprovados e 1 smoke test do Ollama ignorado;
- TypeScript e build Vite aprovados;
- migration aplicada sobre uma cópia do banco real schema 10;
- 27 conhecimentos e todos os contadores das tabelas anteriores preservados;
- 27 baselines criados;
- `PRAGMA foreign_key_check`: zero violações.

## Limitações

- não há cálculo automático de cobertura, profundidade ou integração;
- não há pesos, XP, gamificação ou recomendação automática;
- roadmaps não são gerados por IA;
- Knowledge Events automáticos entram apenas na v0.8.3 — Learning Engine;
- o aceite visual e operacional no aplicativo Tauri ainda deve ser realizado pelo operador.
