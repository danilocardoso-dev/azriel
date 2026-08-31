# AZRIEL v0.5.1 — Operações Diárias

**Status:** ✅ Concluída em 31 de agosto de 2026

A versão **v0.5 — Knowledge Core** foi concluída.

Agora implemente:

# v0.5.1 — Operações Diárias

Antes de modificar qualquer arquivo:

1. leia o `README`;
2. leia `Docs/Roadmap.md`;
3. leia `Docs/Version/Version_0.5.md`;
4. examine a arquitetura atual;
5. entenda como SQLite, migrations, services e repositories foram implementados;
6. preserve tudo que já estiver funcionando;
7. execute o projeto e os testes disponíveis antes de iniciar.

Não refatore partes estáveis sem necessidade.

---

# Objetivo

Adicionar ao Azriel um módulo chamado:

# Operações Diárias

O objetivo é substituir o uso de aplicativos externos de notas rápidas e tarefas por uma central integrada ao próprio Azriel.

O módulo deve permitir administrar:

* tarefas;
* notas rápidas;
* prioridades;
* prazos;
* atividades do dia;
* atividades futuras;
* atividades concluídas;
* relações com projetos;
* relações com áreas de conhecimento.

A v0.5.1 deve aproveitar o SQLite já implementado na v0.5.

---

# Princípio do módulo

Nem tudo é uma tarefa.

O sistema deve diferenciar claramente:

## Tarefa

Algo que precisa ser realizado.

Exemplo:

`Revisar lógica de comparação do GeneScope`

## Nota

Uma informação, ideia ou observação que deve ser registrada.

Exemplo:

`Ideia: utilizar MQTT entre Azriel e ESP32 futuramente.`

Uma nota poderá futuramente ser convertida em tarefa, mas isso não é obrigatório nesta primeira versão.

---

# Nome do módulo

Nome visual:

`Operações Diárias`

Nome técnico sugerido:

`DailyOperations`

Adicionar esse módulo à navegação principal do Azriel.

---

# Estrutura visual

Criar uma tela própria seguindo a identidade HUD atual.

Estrutura conceitual:

```text
┌──────────────────────────────────────────────────────────────┐
│ OPERAÇÕES DIÁRIAS                    06 PENDENTES / 02 ALTAS │
├──────────────────┬────────────────────────┬──────────────────┤
│                  │                        │                  │
│ CAIXA DE ENTRADA │         HOJE           │     DETALHES     │
│                  │                        │                  │
│ Entrada rápida   │ □ Atividade 1          │ prioridade       │
│                  │ □ Atividade 2          │ projeto          │
│ + tarefa         │ ✓ Atividade 3          │ prazo            │
│ + nota           │                        │ conhecimento     │
│                  │                        │                  │
├──────────────────┴────────────────────────┴──────────────────┤
│ NOTAS RÁPIDAS                                               │
│                                                            │
│ [ ideia ] [ pesquisa ] [ lembrete ]                         │
└──────────────────────────────────────────────────────────────┘
```

Não copiar literalmente essa disposição se a arquitetura existente tiver solução melhor.

Priorizar:

* leitura rápida;
* poucos cliques;
* alta densidade de informação;
* uso diário;
* consistência com o HUD atual.

A tela deve parecer parte do Azriel, não um aplicativo de produtividade externo embutido.

---

# Entrada Rápida

Criar uma área de captura rápida.

Exemplo:

```text
> Adicionar tarefa ou nota...
```

Deve permitir registrar algo com o mínimo possível de interação.

Fluxo desejado:

1. escrever;
2. escolher tarefa ou nota;
3. salvar.

Não exigir formulário complexo para a captura inicial.

Depois o usuário pode editar os detalhes.

---

# Tarefas

Criar uma entidade de tarefas persistente.

Estrutura conceitual:

```ts
interface Task {
  id: string;
  title: string;
  description?: string;

  status:
    | "inbox"
    | "pending"
    | "in_progress"
    | "completed"
    | "cancelled";

  priority:
    | "low"
    | "medium"
    | "high"
    | "critical";

  dueDate?: string;

  projectId?: string;
  knowledgeAreaId?: string;

  createdAt: string;
  updatedAt: string;
  completedAt?: string;
}
```

Adaptar nomes e tipos à arquitetura atual.

---

# Notas

Criar uma entidade separada.

Estrutura conceitual:

```ts
interface Note {
  id: string;
  title?: string;
  content: string;

  status:
    | "active"
    | "archived";

  projectId?: string;
  knowledgeAreaId?: string;

  createdAt: string;
  updatedAt: string;
}
```

Notas devem ser simples e rápidas de criar.

---

# Banco de dados

Criar uma NOVA migration.

Não editar migrations antigas já aplicadas.

Sugestão:

```text
002_daily_operations.sql
```

ou seguir a convenção existente no projeto.

Criar pelo menos:

```text
tasks
notes
```

As tabelas devem possuir foreign keys opcionais para:

```text
projects
knowledge_areas
```

quando fizer sentido.

---

# Relação com projetos

Uma tarefa ou nota pode opcionalmente estar vinculada a um projeto.

Exemplo:

```text
Tarefa:
Implementar cálculo de amplicon

Projeto:
PCR Simulator
```

Outro exemplo:

```text
Nota:
Investigar sensores de corrente

Projeto:
ArcCore
```

Ao visualizar um projeto futuramente, deve ser possível recuperar suas atividades relacionadas.

Nesta versão, basta garantir a estrutura e permitir associação no módulo de Operações Diárias.

---

# Relação com conhecimentos

Uma tarefa ou nota também pode estar ligada a uma área de conhecimento.

Exemplo:

```text
Tarefa:
Estudar transistor MOSFET

Conhecimento:
Eletrônica
```

Outro:

```text
Tarefa:
Revisar expressão gênica

Conhecimento:
Biologia Molecular
```

Essa relação será utilizada futuramente pelo Knowledge Core e pela AI Core.

---

# Visões do módulo

Criar pelo menos as seguintes visões ou filtros.

## Hoje

Mostrar:

* tarefas previstas para hoje;
* tarefas atrasadas ainda não concluídas;
* tarefas sem prazo que estejam marcadas como prioridade atual.

Não incluir tarefas concluídas por padrão.

---

## Caixa de Entrada

Itens criados rapidamente e ainda não organizados.

Principal objetivo:

capturar pensamentos sem interromper o fluxo de trabalho.

---

## Próximas

Mostrar tarefas futuras.

Organização sugerida:

* amanhã;
* esta semana;
* posteriormente.

Não implementar calendário complexo.

---

## Concluídas

Mostrar atividades finalizadas.

Ordenar pelas mais recentemente concluídas.

---

## Notas

Mostrar notas rápidas ativas.

Permitir:

* abrir;
* editar;
* arquivar;
* excluir.

---

# Detalhes da tarefa

Ao selecionar uma tarefa, abrir um painel de detalhes.

Permitir editar:

* título;
* descrição;
* status;
* prioridade;
* prazo;
* projeto relacionado;
* conhecimento relacionado.

Também mostrar:

* data de criação;
* última atualização;
* data de conclusão, se existir.

---

# Fluxos essenciais

Implementar pelo menos:

## Criar tarefa

```text
Entrada rápida
→ tarefa
→ salvar
```

## Criar nota

```text
Entrada rápida
→ nota
→ salvar
```

## Concluir tarefa

```text
Tarefa pendente
→ concluir
→ completedAt registrado
```

## Editar tarefa

```text
Selecionar
→ alterar campos
→ salvar
```

## Arquivar nota

```text
Nota
→ arquivar
```

## Excluir

Permitir excluir tarefas e notas.

Adicionar confirmação antes da remoção definitiva.

---

# Persistência

Todos os dados devem sobreviver ao fechamento do aplicativo.

Teste obrigatório:

1. criar uma tarefa;
2. criar uma nota;
3. fechar completamente o Azriel;
4. abrir novamente;
5. confirmar que ambas continuam disponíveis.

---

# Service / Repository

Seguir a arquitetura criada na v0.5.

Não acessar SQLite diretamente em componentes React.

Criar estruturas equivalentes a:

```text
taskService
taskRepository

noteService
noteRepository
```

ou adaptar ao padrão já existente.

---

# Operações necessárias

## Task Repository / Service

Implementar:

* listTasks;
* getTaskById;
* createTask;
* updateTask;
* deleteTask;
* completeTask;
* listTodayTasks;
* listUpcomingTasks;
* listInboxTasks;
* listCompletedTasks.

Evitar duplicar SQL desnecessariamente.

---

## Note Repository / Service

Implementar:

* listNotes;
* getNoteById;
* createNote;
* updateNote;
* archiveNote;
* deleteNote.

---

# Datas

Tratar datas corretamente.

Diferenciar:

* data de criação;
* data de conclusão;
* prazo.

Evitar bugs de timezone.

Se o sistema atual já possuir utilitários de data, reutilizar.

---

# Prioridades

Representar visualmente:

```text
BAIXA
MÉDIA
ALTA
CRÍTICA
```

Usar a identidade visual existente.

Não utilizar cores excessivamente chamativas.

Prioridade crítica deve chamar atenção, mas sem quebrar o estilo do HUD.

---

# Contadores

Adicionar indicadores no módulo.

Exemplo:

```text
PENDENTES      06
HOJE           03
ATRASADAS      01
PRIORIDADE     02
NOTAS          08
```

Os números devem vir do banco.

Não criar valores mockados.

---

# Integração com Command Center

Adicionar uma representação compacta das Operações Diárias ao Command Center.

Não criar um segundo dashboard inteiro.

Exemplo:

```text
OPERAÇÕES DIÁRIAS

03 tarefas hoje
01 atrasada
02 alta prioridade
```

Ao clicar:

abrir o módulo completo.

---

# Empty states

Criar estados vazios úteis.

Exemplo:

```text
Nenhuma tarefa para hoje.
```

ou:

```text
Caixa de entrada vazia.
```

Evitar telas quebradas ou painéis simplesmente vazios.

---

# Loading e erros

Seguir o padrão da v0.5.

Adicionar:

* loading;
* error;
* empty;
* success.

Falhas do banco devem ser tratadas.

---

# Atalhos

Se for simples e seguro na arquitetura atual, adicionar:

```text
Ctrl + N
```

para abrir Entrada Rápida quando o módulo estiver ativo.

Não criar atalhos globais do Windows nesta versão.

Não interceptar atalhos fora da aplicação.

Se isso complicar a implementação, pode ficar fora da v0.5.1.

---

# Não implementar agora

A v0.5.1 NÃO deve virar um sistema completo de produtividade.

Fora do escopo:

* calendário completo;
* Google Calendar;
* notificações do Windows;
* tarefas recorrentes;
* subtarefas;
* Kanban;
* Pomodoro;
* sincronização cloud;
* colaboração;
* anexos;
* reconhecimento de voz;
* IA;
* Ollama;
* automação;
* lembretes inteligentes.

Essas funcionalidades poderão ser avaliadas posteriormente.

---

# Interface

Preservar integralmente a identidade criada na v0.4 e mantida na v0.5.

O módulo precisa priorizar função.

Evitar:

* cards gigantes;
* excesso de arredondamento;
* aparência de Trello;
* aparência de Notion;
* aparência de Microsoft To Do;
* aparência de dashboard SaaS.

Queremos uma:

# Central operacional pessoal.

---

# Dados existentes

Não alterar nem destruir dados existentes de:

* projetos;
* conhecimentos;
* histórico;
* formação;
* Mapa Stark.

A migration deve preservar completamente o banco da v0.5.

---

# Testes

Adicionar testes para pelo menos:

1. criação de tarefa;
2. atualização de tarefa;
3. conclusão;
4. `completedAt`;
5. persistência;
6. criação de nota;
7. arquivamento de nota;
8. relação tarefa ↔ projeto;
9. relação tarefa ↔ conhecimento;
10. filtros Hoje / Próximas / Caixa de Entrada.

---

# Documentação

Manter atualizada:

```text
Docs/Version/Version_0.5.1.md
```

Documentar:

* objetivo;
* arquitetura;
* migration;
* tabelas;
* operações;
* integrações;
* limitações.

Atualizar:

```text
Docs/Roadmap.md
```

marcando:

```text
v0.5 — concluída
v0.5.1 — Operações Diárias
```

Não alterar a direção planejada da v0.6.

---

# Implementação da v0.5.1

## Arquitetura

O módulo segue a separação criada na v0.5 e mantém o SQLite fora dos
componentes React:

```text
DailyOperationsPage
        ↓
DailyOperationsContext
        ↓
taskService / noteService
        ↓
taskRepository / noteRepository
        ↓
Comandos Tauri
        ↓
daily_repository.rs
        ↓
SQLite
```

O contexto de operações diárias é independente do contexto de projetos,
conhecimentos e formação. Uma falha no novo módulo não invalida os demais
núcleos.

## Migration

A migration adicionada é:

```text
src-tauri/migrations/0004_daily_operations.sql
```

Ela cria `tasks` e `notes`, com índices para status, prazos, conclusão e
relacionamentos. As chaves estrangeiras opcionais para `projects` e
`knowledge_areas` usam `ON DELETE SET NULL`. Nenhuma migration anterior foi
editada.

Um teste específico parte do schema 3 da v0.5, aplica apenas a migration 4 e
confirma que projetos, conhecimentos, formação e histórico mantêm as mesmas
contagens.

## Operações

Tarefas possuem criação, busca, edição, conclusão, reabertura, exclusão e os
filtros Hoje, Caixa de Entrada, Próximas e Concluídas. A conclusão e o registro
de `completed_at` acontecem juntos no repository Rust; alterar o status para um
estado não concluído limpa esse timestamp.

Notas possuem criação, busca, edição, arquivamento, listagem e exclusão. A
captura rápida cria tarefas em `inbox` e notas em `active`.

Ambas as entidades podem ser vinculadas opcionalmente a um projeto e a uma área
de conhecimento.

## Datas e filtros

- `due_date` usa somente a data local no formato `YYYY-MM-DD`;
- timestamps de auditoria continuam sendo gerados pelo SQLite;
- o frontend envia a data local ao Rust, evitando deslocamento de dia por UTC;
- Hoje contém prazos de hoje, tarefas atrasadas e tarefas sem prazo em andamento
  com prioridade alta ou crítica;
- Próximas é agrupada em amanhã, esta semana e posteriormente;
- Concluídas é ordenada pela conclusão mais recente.

## Interface

O módulo mantém o HUD da v0.4/v0.5 e usa uma organização operacional de alta
densidade:

- captura rápida em dois passos;
- cinco visões;
- contadores vindos do banco;
- lista central;
- editor lateral de detalhes;
- modal de exclusão próprio do Azriel, com identificação do registro e aviso
  de irreversibilidade;
- estados de loading, erro e vazio;
- atalho `Ctrl + N` somente enquanto o módulo está aberto;
- resumo compacto clicável no Command Center.

O layout foi verificado nos breakpoints desktop, 1100 px e 740 px. O preview
web mostra uma orientação compreensível para executar o Tauri, sem recorrer a
mocks ou dados alternativos.

## Validação automática atual

- 10 testes TypeScript cobrindo captura, validação e agrupamento de datas;
- 8 testes Rust cobrindo migration, CRUD, conclusão, `completed_at`, filtros,
  vínculos, arquivamento e persistência após reabrir o arquivo;
- lint e build TypeScript aprovados durante o desenvolvimento;
- inicialização Tauri sobre o banco local existente sem erro de migration.
- build Tauri release aprovado, com executável e instalador NSIS na versão
  0.5.1.

Artefatos locais:

```text
src-tauri/target/release/azriel.exe
src-tauri/target/release/bundle/nsis/Azriel_0.5.1_x64-setup.exe
```

## Validação manual

Validação funcional aprovada pelo operador em 31 de agosto de 2026. Foram
confirmados o funcionamento do módulo, a persistência dos registros e o fluxo
de exclusão com o novo modal visual.

## Limitações

- não há recorrência, subtarefas, calendário completo ou notificações;
- notas arquivadas permanecem no banco, mas a visão padrão mostra apenas notas
  ativas.

---

# Critérios de aceite

A v0.5.1 só está concluída quando:

1. existir o módulo Operações Diárias;
2. tarefas forem persistidas no SQLite;
3. notas forem persistidas no SQLite;
4. for possível criar tarefas;
5. for possível editar tarefas;
6. for possível concluir tarefas;
7. `completedAt` funcionar;
8. for possível excluir tarefas;
9. for possível criar notas;
10. for possível editar notas;
11. for possível arquivar notas;
12. for possível excluir notas;
13. tarefas puderem ser relacionadas a projetos;
14. tarefas puderem ser relacionadas a conhecimentos;
15. notas puderem ser relacionadas a projetos;
16. notas puderem ser relacionadas a conhecimentos;
17. existir visão Hoje;
18. existir Caixa de Entrada;
19. existir visão Próximas;
20. existir área de Concluídas;
21. existir área de Notas;
22. os contadores forem calculados com dados reais;
23. existir integração compacta no Command Center;
24. migrations funcionarem sobre um banco v0.5 existente;
25. dados anteriores não forem perdidos;
26. TypeScript não apresentar erros;
27. testes relevantes passarem;
28. build funcionar;
29. Tauri iniciar corretamente;
30. a interface preservar a identidade do Azriel;
31. `Docs/Version/Version_0.5.1.md` existir.

---

# Resultado esperado

Ao concluir a v0.5.1, o Azriel deve se tornar útil durante o dia inteiro.

O operador deverá conseguir abrir o sistema e responder rapidamente:

* O que preciso fazer hoje?
* O que está atrasado?
* Quais são minhas prioridades?
* O que anotei recentemente?
* Qual tarefa pertence a qual projeto?
* Que atividade está relacionada a determinado conhecimento?

A versão deve substituir de forma prática o uso de notas adesivas para organização cotidiana.

Ainda não existe inteligência artificial nesta camada.

Mas quando a AI Core chegar na v0.6, ela encontrará uma estrutura persistente capaz de responder a uma pergunta fundamental:

> **O que o operador precisa fazer agora?**
