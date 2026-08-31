# AZRIEL v0.6 — AI Core

**Status:** ✅ Concluída em 31 de agosto de 2026

**Início:** 31 de agosto de 2026

As versões anteriores foram concluídas e validadas.

Agora implemente:

# v0.6 — AI Core

Antes de modificar qualquer arquivo:

1. leia completamente o `README`;
2. leia `Docs/Roadmap.md`;
3. leia `Docs/Version/Version_0.5.md`;
4. leia `Docs/Version/Version_0.5.1.md`;
5. examine toda a arquitetura atual;
6. entenda como SQLite, services, repositories, Tauri e React foram organizados;
7. identifique código antigo relacionado a IA/Ollama, caso ainda exista;
8. preserve tudo que já estiver funcionando;
9. execute o projeto, lint, testes e build antes de iniciar.

Não refatore partes estáveis apenas por preferência arquitetural.

---

# Objetivo

A v0.6 deve adicionar ao Azriel seu primeiro núcleo funcional de inteligência artificial.

O objetivo desta versão é permitir que o operador converse com o Azriel e faça perguntas sobre os dados reais já armazenados no sistema.

Ao final da versão, Azriel deve conseguir responder perguntas baseadas em:

* Operações Diárias;
* Projetos;
* Knowledge Core;
* Mapa Stark;
* Formação;
* histórico de conhecimento;
* estado interno do próprio Azriel.

A IA NÃO deve acessar diretamente o SQLite.

A IA deve utilizar ferramentas controladas pelo sistema.

---

# Tecnologia

Utilizar:

* Ollama local;
* modelo inicialmente configurado: `qwen2.5:0.5b`;
* arquitetura de provider desacoplada;
* React + TypeScript;
* Tauri;
* SQLite já existente.

Endpoint padrão do Ollama:

```text
http://localhost:11434
```

Não utilizar OpenAI API nesta versão.

Não adicionar dependência de serviços externos.

---

# Princípio central

O modelo de linguagem NÃO é o Azriel.

O modelo é apenas um componente utilizado pelo AI Core.

Arquitetura conceitual:

```text
AZRIEL
   │
   ▼
AI Core
   │
   ├── Conversation Manager
   ├── Context Builder
   ├── Tool Router
   ├── Tool Registry
   └── AI Provider
            │
            ▼
      OllamaProvider
            │
            ▼
       Ollama Local
            │
            ▼
      qwen2.5:0.5b
```

A arquitetura deve permitir trocar o modelo futuramente sem modificar o restante do Azriel.

---

# AIProvider

Criar ou preservar uma abstração equivalente a:

```ts
interface AIProvider {
  chat(request: AIRequest): Promise<AIResponse>;
  isAvailable(): Promise<boolean>;
  listModels?(): Promise<string[]>;
}
```

Adaptar ao padrão atual do projeto.

Implementação inicial:

```text
OllamaProvider
```

O restante do sistema NÃO deve depender diretamente do Ollama.

---

# OllamaProvider

Implementar comunicação com Ollama local.

O provider deve:

* verificar se o Ollama está disponível;
* enviar mensagens;
* receber respostas;
* tratar erros;
* possuir timeout apropriado;
* utilizar o modelo configurado;
* permitir mudança de modelo.

Não deixar:

```text
qwen2.5:0.5b
```

hardcoded em múltiplos arquivos.

Centralizar configuração.

---

# Descoberta de modelos

Se a API local do Ollama permitir de forma simples e confiável, implementar listagem dos modelos instalados.

A tela de Configurações deve permitir visualizar algo como:

```text
AI CORE

Provedor:
Ollama Local

Endpoint:
http://localhost:11434

Modelo:
qwen2.5:0.5b

Status:
ONLINE
```

Se existirem outros modelos instalados, permitir selecioná-los.

Persistir a seleção localmente.

Se a descoberta automática complicar excessivamente a versão, manter configuração manual, mas não acoplar o sistema a um único modelo.

---

# Limitação importante do modelo

O modelo inicial possui apenas 0.5B parâmetros.

Portanto:

NÃO delegar cálculos objetivos, contagens, filtros ou regras críticas ao LLM.

Exemplo incorreto:

```text
Enviar todas as tarefas ao modelo e perguntar:
"Quantas tarefas existem hoje?"
```

Exemplo correto:

```text
DailyOperationsService
      ↓
calcula quantidade real
      ↓
{
  today: 4,
  overdue: 1
}
      ↓
LLM apenas formula a resposta
```

Dados determinísticos devem ser obtidos por services/tools.

O modelo deve ser usado principalmente para:

* interpretar intenção;
* selecionar informações relevantes;
* produzir linguagem natural;
* resumir resultados estruturados;
* explicar informações;
* combinar resultados obtidos por tools.

---

# Tools

Criar um sistema de ferramentas internas.

A IA NÃO pode executar SQL.

Fluxo:

```text
Usuário
   ↓
AI Core
   ↓
Tool Router
   ↓
Tool
   ↓
Service existente
   ↓
Repository
   ↓
SQLite
```

As tools devem retornar dados estruturados.

---

# Tool Registry

Criar um registro central de ferramentas disponíveis.

Cada ferramenta deve possuir algo semelhante a:

```ts
interface AzrielTool {
  name: string;
  description: string;
  execute(input: unknown): Promise<unknown>;
}
```

Melhorar a tipagem conforme necessário.

Evitar `any`.

---

# Ferramentas iniciais

Implementar um conjunto pequeno e confiável.

## Operações Diárias

```text
get_today_tasks
get_overdue_tasks
get_upcoming_tasks
get_recent_notes
get_daily_operations_summary
```

---

## Projetos

```text
list_projects
get_project
get_project_tasks
get_project_knowledge
```

---

## Knowledge Core

```text
list_knowledge_areas
get_knowledge_area
get_knowledge_gaps
get_stark_map
get_knowledge_history
```

---

## Formação

```text
get_education
get_current_education
get_planned_education
```

---

## Azriel

```text
get_azriel_status
get_azriel_version
```

Não adicionar tools de escrita nesta versão.

---

# Tools somente leitura

Na v0.6, todas as tools acessíveis ao LLM devem ser:

# READ ONLY

Permitido:

* consultar;
* listar;
* resumir;
* comparar;
* recuperar.

Bloqueado:

* criar;
* editar;
* excluir;
* concluir tarefa;
* alterar projeto;
* executar comando;
* abrir programa;
* modificar arquivo;
* alterar banco.

A UI normal pode continuar realizando operações de escrita.

A restrição é especificamente para o AI Core.

---

# Tool Router

Criar uma camada responsável por decidir qual ferramenta utilizar.

Como o modelo inicial é pequeno, não depender exclusivamente de tool calling sofisticado do LLM.

Priorizar uma estratégia robusta.

Pode utilizar combinação de:

* classificação simples de intenção;
* palavras-chave;
* regras;
* contexto;
* LLM quando necessário.

Exemplo:

```text
"O que tenho para hoje?"
```

deve mapear de forma confiável para:

```text
get_today_tasks
```

Outro exemplo:

```text
"Qual minha maior lacuna?"
```

→

```text
get_knowledge_gaps
```

Outro:

```text
"Quais projetos trabalham genética?"
```

→ cruzar:

```text
list_projects
+
project_knowledge
```

Não criar um sistema excessivamente complexo.

O objetivo é confiabilidade.

---

# Context Builder

Criar componente/serviço:

```text
ContextBuilder
```

Responsável por montar apenas o contexto necessário para cada solicitação.

Não enviar todo o banco para o modelo.

Exemplo:

```text
Usuário:
"Como está minha formação?"

ContextBuilder:
→ chama get_education
→ monta contexto curto
→ envia ao provider
```

---

# Prompt de sistema

Criar um prompt de sistema externo ao código.

Sugestão:

```text
src/prompts/system.txt
```

ou adaptar à estrutura atual.

O prompt deve definir Azriel como:

* assistente pessoal local;
* objetivo;
* comportamento;
* restrições;
* uso das ferramentas;
* obrigação de não inventar dados;
* necessidade de admitir quando informação não estiver disponível.

Evitar prompt gigantesco.

O README não deve ser colocado inteiro no contexto.

---

# Regra contra alucinação

Azriel nunca deve afirmar que um dado existe se ele não foi encontrado.

Se uma ferramenta retornar vazio:

responder algo equivalente a:

```text
Não encontrei essa informação registrada no Azriel.
```

Não inventar:

* tarefas;
* projetos;
* métricas;
* datas;
* conhecimentos;
* status;
* formação.

---

# Conversation Manager

Criar gerenciamento de conversas.

Persistir conversas no SQLite.

Criar novas migrations.

Não modificar migrations antigas.

Sugestão de tabelas:

```text
conversations
messages
```

---

# conversations

Campos conceituais:

```text
id
title
created_at
updated_at
```

---

# messages

Campos:

```text
id
conversation_id
role
content
created_at
```

Roles iniciais:

```text
user
assistant
system
```

Se mensagens de tool forem persistidas, criar representação apropriada.

---

# Histórico

Ao fechar e reabrir o Azriel:

* conversas anteriores devem continuar existindo;
* mensagens devem continuar disponíveis;
* usuário deve conseguir abrir uma conversa anterior.

Não criar ainda memória vetorial ou embeddings.

Histórico de conversa e memória de longo prazo são conceitos diferentes.

---

# Contexto de conversa

Não enviar histórico infinito para o modelo.

Criar limite configurável.

Exemplo:

* últimas N mensagens;
* ou limite aproximado por tamanho.

Como o modelo é pequeno, manter contexto enxuto.

---

# Interface

Adicionar uma interface própria para o AI Core.

O núcleo central `AzrielCore` deve ser integrado ao chat.

Ao clicar no núcleo:

abrir o módulo de interação com Azriel.

Estrutura conceitual:

```text
┌─────────────────────────────────────────────────────┐
│ AZRIEL // AI CORE                        ONLINE     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ AZ > Azriel, situação.                              │
│                                                     │
│ AZRIEL > Consultando Operações Diárias...           │
│                                                     │
│          Você possui 4 atividades hoje.             │
│          Uma está atrasada.                         │
│                                                     │
├─────────────────────────────────────────────────────┤
│ > Digite um comando...                              │
└─────────────────────────────────────────────────────┘
```

Preservar a identidade HUD.

Não criar interface semelhante a ChatGPT.

Ela deve parecer integrada ao sistema Azriel.

---

# Estado visual do AzrielCore

Integrar os estados já existentes.

## idle

Estado normal.

## processing

Quando o modelo estiver processando.

## tool

Se necessário, criar um estado visual interno para consulta às ferramentas.

## alert

Falha ou indisponibilidade.

## offline

Ollama indisponível.

As animações devem continuar discretas.

---

# Feedback de tools

Durante uma consulta, a interface pode mostrar mensagens técnicas discretas como:

```text
CONSULTANDO OPERAÇÕES DIÁRIAS
```

```text
ANALISANDO KNOWLEDGE CORE
```

```text
RECUPERANDO PROJETO
```

Não mostrar detalhes internos desnecessários nem SQL.

---

# Comando especial: "Azriel, situação"

Implementar suporte confiável para:

```text
Azriel, situação.
```

ou simplesmente:

```text
situação
```

Esse comando deve produzir um resumo baseado em dados reais.

Consultar pelo menos:

* tarefas de hoje;
* atrasadas;
* projetos ativos;
* principais lacunas;
* formação atual.

O resultado deve ser curto e útil.

Não precisa listar tudo.

---

# Consultas obrigatórias

A v0.6 deve responder corretamente pelo menos:

```text
O que tenho para hoje?
```

```text
Tenho alguma atividade atrasada?
```

```text
Quais são meus projetos?
```

```text
Quais projetos estão relacionados a genética?
```

```text
Qual minha maior lacuna?
```

```text
Como está meu Mapa Stark?
```

```text
Como está minha formação?
```

```text
O que estou fazendo relacionado a bioinformática?
```

```text
Azriel, situação.
```

As respostas devem utilizar dados reais.

---

# Consultas cruzadas

Implementar pelo menos algumas consultas que combinem múltiplos domínios.

Exemplo:

```text
Quais projetos ajudam a reduzir minhas maiores lacunas?
```

Outro:

```text
O que estou fazendo atualmente relacionado a genética?
```

Pode exigir combinação de:

```text
Projects
+
Knowledge
+
Daily Operations
```

Não depender apenas do modelo para descobrir relacionamentos já existentes no banco.

Utilizar relações persistidas.

---

# Falha do Ollama

Se Ollama não estiver rodando:

a aplicação NÃO deve quebrar.

Mostrar:

```text
AI CORE OFFLINE
```

e instrução curta:

```text
Ollama não encontrado em localhost:11434.
```

Os demais módulos do Azriel devem continuar funcionando normalmente.

---

# Falha do modelo

Se o modelo configurado não existir:

mostrar erro claro.

Se possível, listar os modelos disponíveis.

Não substituir silenciosamente o modelo por outro.

---

# Timeout

Requisições ao Ollama devem possuir timeout e tratamento de cancelamento quando possível.

A interface não deve ficar indefinidamente em estado de processamento.

---

# Configurações

Adicionar seção de AI Core em Configurações.

Mostrar:

* provider;
* endpoint;
* modelo;
* status;
* teste de conexão.

Exemplo:

```text
PROVEDOR
Ollama Local

ENDPOINT
http://localhost:11434

MODELO
qwen2.5:0.5b

STATUS
ONLINE
```

Adicionar botão:

```text
TESTAR CONEXÃO
```

---

# Segurança

O AI Core deve continuar sem permissão de escrita.

Nenhuma tool pode:

* executar shell;
* acessar arbitrary filesystem;
* abrir aplicações;
* editar arquivos;
* modificar SQLite;
* controlar sistema operacional.

Essas capacidades pertencem a versões futuras.

---

# Logs

Adicionar logging técnico simples para:

* provider indisponível;
* modelo inexistente;
* erro de tool;
* timeout;
* falha de parsing.

Não registrar conteúdo sensível desnecessariamente.

---

# Testes

Adicionar testes principalmente para lógica determinística.

Prioridades:

1. roteamento de intenção;
2. execução correta de tools;
3. tools somente leitura;
4. `get_today_tasks`;
5. `get_knowledge_gaps`;
6. `get_project_knowledge`;
7. resumo de situação;
8. persistência das conversas;
9. falha do Ollama;
10. modelo inexistente;
11. ContextBuilder;
12. ausência de dados;
13. provider desacoplado.

Não depender de Ollama real em todos os testes.

Criar mock/fake provider para testes automatizados.

---

# FakeAIProvider

Criar um provider de teste simples.

Objetivo:

testar AI Core sem depender do Ollama em CI ou testes locais.

Exemplo conceitual:

```text
FakeAIProvider
```

Nunca utilizar como provider padrão em produção.

---

# Performance

O computador alvo possui recursos limitados.

Priorizar:

* baixo consumo;
* contexto pequeno;
* poucas chamadas;
* dados filtrados antes de chegar ao modelo;
* evitar processamento duplicado.

O modelo inicial é:

```text
qwen2.5:0.5b
```

Projetar considerando essa limitação.

---

# Não implementar nesta versão

Fora do escopo:

* OpenAI API;
* embeddings;
* vector database;
* RAG complexo;
* reconhecimento de voz;
* text-to-speech;
* automação do Windows;
* execução de shell;
* abertura de programas;
* manipulação de arquivos;
* MQTT;
* ESP32;
* IoT;
* controle de dispositivos;
* escrita de tarefas via IA;
* alteração de projetos via IA;
* alteração de Knowledge Core via IA.

---

# Documentação

Manter atualizada:

```text
Docs/Version/Version_0.6.md
```

Documentar:

* objetivo;
* arquitetura do AI Core;
* AIProvider;
* OllamaProvider;
* tools;
* Tool Router;
* Context Builder;
* conversas;
* migrations;
* configurações;
* limitações do qwen2.5:0.5b;
* segurança;
* como executar.

Atualizar:

```text
Docs/Roadmap.md
```

Marcar:

```text
v0.5.1 — concluída
v0.6 — AI Core — em desenvolvimento
```

Ao finalizar e validar:

```text
v0.6 — concluída
```

---

# Implementação da v0.6

## Arquitetura efetiva

O AI Core preserva a separação das versões anteriores:

```text
Chat HUD
  -> AICoreService
  -> ToolRouter + ContextBuilder + ToolRegistry
  -> services somente leitura
  -> repositories -> comandos Tauri -> SQLite

Contexto estruturado
  -> AIProvider -> OllamaProvider
  -> comando Tauri -> cliente HTTP Rust
  -> Ollama local
```

O modelo não recebe SQL, conexão com o banco, acesso ao sistema operacional ou
tools de escrita. O transporte Rust aceita somente endpoints HTTP de loopback
(`localhost`, `127.0.0.1` ou `::1`). A conexão SQLite não permanece bloqueada
durante a inferência.

## Persistência

A migration `src-tauri/migrations/0005_ai_core.sql` adiciona:

- `ai_settings`, com endpoint, modelo, limite de contexto e timeout;
- `conversations`, com título e timestamps;
- `messages`, com papéis `user`, `assistant` e `system`;
- índices para ordenação do histórico.

As migrations anteriores não foram modificadas. Existe teste específico que
aplica a migration 5 sobre um schema v0.5.1 e confirma a preservação de tarefas
e notas.

## Providers e Ollama

`AIProvider` define o contrato desacoplado. `OllamaProvider` é a implementação
de produção e `FakeAIProvider` permite testes sem servidor local. O cliente
Rust possui:

- descoberta dos modelos instalados por `/api/tags`;
- conversa não streaming por `/api/chat`;
- timeout configurável;
- erro específico para modelo ausente;
- estado offline sem interromper os outros módulos;
- logs técnicos sem registrar o conteúdo das conversas.

Em 31 de agosto de 2026, a API local foi confirmada em
`http://localhost:11434` e o modelo `qwen2.5:0.5b` foi encontrado e utilizado em
um smoke test real pelo cliente Rust.

## Tools e contexto

O registro possui 19 tools de leitura para Operações Diárias, Projetos,
Knowledge Core, Formação e estado do Azriel. Contagens, filtros, lacunas,
médias e relações são calculados antes da chamada ao modelo.

O `ToolRouter` usa regras normalizadas em português para as consultas
obrigatórias e combina tools em perguntas cruzadas. O `ContextBuilder` executa
somente as tools selecionadas e limita o contexto estruturado. Quando todas as
fontes retornam vazias, o AI Core responde deterministicamente que não encontrou
a informação, sem consultar o modelo.

## Interface

- módulo próprio `AI Core`, sem copiar o layout de um chat genérico;
- histórico lateral, nova conversa e exclusão confirmada de conversas com todas as mensagens associadas;
- mensagens do operador e do Azriel;
- feedback discreto do domínio consultado;
- estados reais `idle`, `tool`, `processing`, `alert` e `offline`;
- clique no núcleo central abre o AI Core;
- painel compacto no Command Center;
- configuração persistente de endpoint, modelo, contexto e timeout;
- descoberta e seleção dos modelos instalados;

## Qualidade das respostas e escopo

O roteador diferencia consultas aos dados internos do Azriel de perguntas de
conhecimento geral. Apenas consultas internas executam tools e recebem o
contexto estruturado do sistema; perguntas gerais usam um prompt próprio e não
recebem dados irrelevantes do banco local.

O provider usa penalidade de repetição e limites de geração adequados para
modelos locais pequenos. Respostas com repetição patológica são descartadas e
regeneradas uma vez com um perfil mais restritivo. Se a segunda tentativa ainda
for degenerada, o conteúdo não é persistido como resposta válida. Respostas
degeneradas antigas também são removidas do histórico enviado ao modelo.

O modelo `qwen2.5:0.5b` continua compatível, mas possui limitações importantes
de conhecimento e precisão. Para consultas gerais, o modelo recomendado para o
equipamento de desenvolvimento é `qwen2.5:3b`; a seleção permanece explícita em
Configurações, sem substituição silenciosa.

Em 31 de agosto de 2026, `qwen2.5:3b` foi instalado e validado localmente. Ele
eliminou o padrão de repetição observado no 0.5B e respondeu corretamente ao
cálculo de regressão (`5 × 542 = 2710`). Como qualquer modelo local pequeno,
continua sujeito a imprecisões factuais em temas gerais e não deve ser tratado
como fonte autoritativa sem verificação externa.
- teste de conexão na tela de Configurações.

O layout foi verificado em desktop, 1100 px e 740 px. Essa validação visual
identificou e corrigiu um retorno inválido no efeito de rolagem do chat que não
era detectado pelo build estático.

## Como executar

Com o Ollama e o modelo instalados:

```powershell
ollama pull qwen2.5:3b
npm.cmd run tauri dev
```

O `qwen2.5:3b` é o modelo recomendado para uso geral neste equipamento. O
`qwen2.5:0.5b` permanece compatível para ambientes com menos recursos.

O executável do Ollama não precisa estar no `PATH` se o processo local e a API
em `localhost:11434` estiverem ativos. O botão `TESTAR CONEXÃO` confirma o
status e lista os modelos descobertos.

Artefatos locais de validação:

```text
src-tauri/target/release/azriel.exe
src-tauri/target/release/bundle/nsis/Azriel_0.6.0_x64-setup.exe
```

## Validação final

- 33 testes TypeScript aprovados, cobrindo roteamento, tools, contexto, ausência
  de dados, provider desacoplado, separação de perguntas gerais, repetição e
  respostas truncadas;
- 13 testes Rust aprovados, cobrindo migration, persistência, segurança do
  endpoint, modo offline e modelo ausente;
- smoke tests reais aprovados com Ollama, `qwen2.5:0.5b` e `qwen2.5:3b`;
- lint e build TypeScript aprovados durante a implementação;
- inicialização nativa aprovada sobre o banco local existente;
- build Tauri release e instalador NSIS aprovados na versão 0.6.0.

A validação funcional do AI Core, do histórico, da exclusão de conversas e das
respostas com o modelo local foi aprovada pelo operador no encerramento da
v0.6. O modelo local continua sujeito a imprecisões factuais e não deve ser
tratado como fonte autoritativa sem verificação externa.

---

# Critérios de aceite

A v0.6 só está concluída quando:

1. OllamaProvider existir;
2. AIProvider estiver desacoplado;
3. qwen2.5:0.5b puder ser utilizado;
4. endpoint Ollama for configurável;
5. modelo for configurável;
6. status do Ollama puder ser consultado;
7. aplicação funcionar mesmo com Ollama offline;
8. Tool Registry existir;
9. Tools não acessarem SQLite diretamente fora da arquitetura existente;
10. AI Core possuir apenas tools de leitura;
11. Context Builder existir;
12. Tool Router funcionar;
13. Operações Diárias puderem ser consultadas;
14. Projetos puderem ser consultados;
15. Knowledge Core puder ser consultado;
16. Mapa Stark puder ser consultado;
17. Formação puder ser consultada;
18. histórico de conhecimento puder ser consultado;
19. "Azriel, situação" funcionar;
20. consultas cruzadas funcionarem;
21. conversas forem persistidas no SQLite;
22. mensagens forem persistidas;
23. conversas sobreviverem ao reinício;
24. interface de chat existir;
25. AzrielCore refletir estado de processamento;
26. erros de modelo forem tratados;
27. timeouts forem tratados;
28. FakeAIProvider existir para testes;
29. testes relevantes passarem;
30. TypeScript não apresentar erros;
31. build funcionar;
32. Tauri iniciar normalmente;
33. nenhuma funcionalidade das versões anteriores sofrer regressão relevante;
34. `Docs/Version/Version_0.6.md` existir;
35. a IA não possuir tools de escrita;
36. a IA não executar comandos no computador.

---

# Teste final obrigatório

Executar o Azriel com Ollama ativo e modelo:

```text
qwen2.5:0.5b
```

Testar manualmente:

```text
Azriel, situação.
```

```text
O que tenho para hoje?
```

```text
Tenho algo atrasado?
```

```text
Qual minha maior lacuna?
```

```text
Quais projetos trabalham genética?
```

```text
Como está minha formação?
```

```text
O que estou fazendo relacionado a bioinformática?
```

Fechar completamente o aplicativo.

Abrir novamente.

Confirmar que a conversa anterior continua disponível.

Também testar com Ollama desligado.

Azriel deve informar que o AI Core está offline sem comprometer o restante da aplicação.

---

# Resultado esperado

Ao concluir a v0.6, Azriel deve possuir sua primeira camada real de inteligência.

Ele ainda não pode controlar o computador.

Ele ainda não pode alterar dados por decisão própria.

Mas passa a conseguir:

* interpretar perguntas;
* consultar seus próprios módulos;
* recuperar informações reais;
* combinar dados;
* manter conversas;
* responder em linguagem natural.

A v0.6 deve provar que o Azriel consegue compreender o estado do próprio sistema antes de receber qualquer capacidade de agir sobre ele.

A partir daqui:

```text
v0.7 → System Core
v0.8 → Automation Core
v0.9 → IoT Core
v1.0 → Personal Intelligence System
```
