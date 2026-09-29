# Study Lab — Catálogo de conhecimentos

Data: 29/09/2026

## Objetivo

Substituir o catálogo histórico de conhecimentos por um conjunto menor, coerente com o uso atual do Azriel, e devolver ao operador uma interface de gerenciamento dentro de **Estudos**.

## Catálogo canônico

| ID | Nome |
| --- | --- |
| `software-engineering` | Engenharia de Software |
| `systems-infrastructure` | Sistemas e Infraestrutura |
| `cybersecurity` | Cybersecurity |
| `dfir` | Forense Digital & Incident Response |
| `reliability-observability` | Reliability & Observabilidade |
| `ai-engineering` | Engenharia de IA |
| `automation` | Automação |
| `english` | Inglês |
| `technology-business` | Empreendedorismo Tecnológico |
| `engineering-leadership` | Liderança e Decisão Técnica |

Todos começam como nós raiz do tipo `area`, prioridade `high`, cobertura `0` e profundidade `0`.

## Migração

A migration `0041_knowledge_catalog_reset.sql`:

- remove relações, eventos, históricos e baselines do catálogo anterior;
- mantém projetos, tarefas, notas, roadmaps, atividades e pesquisas;
- limpa nesses registros apenas as referências aos conhecimentos removidos;
- cadastra os dez conhecimentos canônicos;
- reinicia as métricas do Learning Engine;
- preserva o projeto `azriel`;
- relaciona o projeto Azriel a `software-engineering`, `ai-engineering` e `automation`.

O histórico anterior deixa de existir no SQLite após a migration. Sua definição permanece recuperável pelo histórico Git anterior a esta alteração.

## Interface

A seção **CONHECIMENTOS** do Study Lab permite:

- pesquisar por ID, nome, categoria ou descrição;
- criar conhecimentos com ID explícito;
- editar metadados mantendo o ID imutável;
- configurar tipo, prioridade e conhecimento pai;
- visualizar vínculos com projetos, tópicos, atividades e filhos;
- excluir com confirmação;
- copiar ou exportar o catálogo em JSON para criação de roadmaps externos.

Conhecimentos com atividades ou eventos de aprendizagem vinculados não podem ser excluídos até que essas referências sejam migradas ou removidas.

## Formato de referência em roadmaps

`knowledgeNodeId`, `primaryKnowledgeNodeId` e `secondaryKnowledgeNodeIds` devem utilizar exclusivamente IDs existentes no catálogo exportado pelo Azriel.
