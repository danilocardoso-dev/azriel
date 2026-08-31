# AZRIEL v0.8.0 — Automation Core / Safe Actions

A versão **v0.7 — System Core** foi concluída e validada.

Agora implemente:

# v0.8.0 — Automation Core / Safe Actions

Antes de modificar qualquer arquivo:

1. leia completamente o `README.md`;
2. leia `docs/roadmap.md`;
3. leia `docs/versions/v0.7.md`;
4. examine a arquitetura atual;
5. entenda o AI Core, Tool Registry, Tool Router e System Core;
6. entenda o Workspace Registry e as integrações Tauri/Rust;
7. execute lint, testes e build;
8. preserve tudo que já estiver validado.

Não refatore partes estáveis sem necessidade.

---

# Objetivo

A v0.8.0 deve permitir que o Azriel execute **ações locais seguras e previamente autorizadas**.

O foco desta versão não é dar acesso irrestrito ao computador.

O foco é criar uma arquitetura confiável de autoridade.

Ao final desta versão, Azriel deve ser capaz de executar ações como:

* abrir aplicativo autorizado;
* abrir workspace autorizado;
* abrir projeto registrado;
* revelar workspace no explorador de arquivos;
* abrir URL previamente cadastrada.

Tudo deve passar por:

```text
Action Request
↓
Policy Engine
↓
Action Registry
↓
Confirmation Gate
↓
Automation Core
↓
Tauri / Rust
↓
Windows
↓
Action History
```

---

# Princípio central

A v0.8.0 deve implementar:

# AUTORIDADE LIMITADA E EXPLÍCITA.

Não criar acesso genérico ao sistema.

Não permitir que o LLM invente comandos, caminhos, URLs ou executáveis.

---

# Classes de permissão

Criar uma classificação central de ações.

Sugestão:

```ts
type ActionPermission =
  | "read"
  | "safe_write"
  | "confirm_write"
  | "blocked";
```

## READ

Consultas sem alteração.

Exemplos:

* sistema;
* projetos;
* Git;
* tarefas;
* conhecimento.

Já existem nas versões anteriores.

---

## SAFE_WRITE

Ações de baixo risco executadas somente quando o usuário solicitar claramente.

Exemplos iniciais:

* abrir aplicativo autorizado;
* abrir projeto registrado;
* abrir workspace autorizado;
* revelar workspace;
* abrir URL cadastrada.

---

## CONFIRM_WRITE

Ações que requerem confirmação explícita.

Na v0.8.0, essa categoria deve existir arquiteturalmente, mas pode possuir poucas ou nenhuma ação de produção.

Preparar a arquitetura para v0.8.1.

---

## BLOCKED

Ações proibidas.

Incluem:

* shell arbitrário;
* PowerShell arbitrário;
* cmd arbitrário;
* executar scripts fornecidos pelo LLM;
* matar processos;
* modificar arquivos;
* excluir arquivos;
* mover arquivos;
* alterar sistema;
* instalar software;
* Git commit;
* Git push;
* Git pull;
* Git reset;
* Git clean;
* Git checkout;
* executar comandos construídos dinamicamente pelo LLM.

---

# Regra crítica

É proibido criar algo como:

```ts
executeCommand(command: string)
```

ou:

```ts
runShell(args: string[])
```

acessível ao AI Core.

Mesmo que seja usado apenas internamente.

A arquitetura deve trabalhar com ações específicas e registradas.

---

# Action Registry

Criar um registro central de ações.

Exemplo conceitual:

```ts
interface RegisteredAction {
  id: string;
  name: string;
  description: string;
  permission: ActionPermission;
  execute(input: ActionInput): Promise<ActionResult>;
}
```

Evitar `any`.

Cada ação deve possuir:

* ID estável;
* nome;
* descrição;
* permissão;
* validação de input;
* executor específico.

---

# Ações iniciais

Implementar apenas:

```text
open_application
open_workspace
open_project
reveal_workspace
open_registered_url
```

Essas ações formam o escopo funcional da v0.8.0.

---

# open_application

Entrada:

```text
appId
```

Nunca receber caminho arbitrário.

Exemplo:

```text
open_application("vscode")
```

O sistema deve resolver internamente o executável cadastrado.

---

# Application Registry

Criar um registro persistente de aplicativos autorizados.

Tabela conceitual:

```text
applications
```

Campos:

```text
id
name
path
enabled
created_at
updated_at
```

Pode adicionar outros campos úteis, sem exagero.

Exemplo:

```text
vscode
Visual Studio Code
C:\...\Code.exe
enabled = true
```

O AI Core nunca deve receber ou decidir o caminho do executável.

---

# Cadastro de aplicativos

A interface deve permitir ao operador:

* adicionar aplicativo;
* editar nome;
* selecionar caminho;
* ativar/desativar;
* remover.

Essa é uma ação manual da UI.

Não é uma permissão do LLM.

---

# open_workspace

Entrada:

```text
workspaceId
```

Utilizar o Workspace Registry existente.

Nunca aceitar:

```text
path
```

fornecido pelo LLM.

---

# open_project

Entrada:

```text
projectId
```

Fluxo esperado:

```text
Project
↓
Workspace relacionado
↓
Application/handler definido
↓
Automation Core
↓
Windows
```

Se o projeto não possuir workspace associado, retornar erro útil.

Não tentar adivinhar caminhos.

---

# reveal_workspace

Entrada:

```text
workspaceId
```

Abrir o diretório autorizado no explorador de arquivos do sistema.

Não permitir diretório arbitrário.

---

# open_registered_url

Criar um registro de URLs autorizadas.

Tabela conceitual:

```text
registered_urls
```

Campos:

```text
id
name
url
enabled
created_at
updated_at
```

Exemplo:

```text
github_azriel
GitHub Azriel
https://github.com/...
```

O LLM recebe apenas:

```text
urlId
```

Nunca uma URL arbitrária.

---

# Policy Engine

Criar:

```text
PolicyEngine
```

Responsável por decidir:

* ação permitida;
* ação bloqueada;
* confirmação necessária;
* alvo autorizado.

O Policy Engine deve funcionar independentemente do LLM.

---

# Exemplo

```text
AI Core
↓
open_application("vscode")
↓
PolicyEngine
↓
application exists?
↓
enabled?
↓
permission = SAFE_WRITE?
↓
execute
```

---

# Confirmation Gate

Criar arquitetura de confirmação.

Mesmo que as ações SAFE_WRITE possam executar diretamente quando o pedido do usuário for explícito, preparar:

```text
ConfirmationGate
```

para futuras ações de maior impacto.

Criar suporte a:

```ts
interface ConfirmationRequest {
  actionId: string;
  targetName: string;
  description: string;
  impact: string;
}
```

---

# Regra de intenção explícita

SAFE_WRITE não deve executar apenas porque o LLM inferiu vagamente uma intenção.

Exemplo permitido:

```text
"Azriel, abra o GeneScope."
```

Exemplo que NÃO deve executar automaticamente:

```text
"Talvez eu trabalhe no GeneScope hoje."
```

Nesse caso, Azriel pode sugerir:

```text
"Quer que eu abra o GeneScope?"
```

mas não executar.

---

# Action Request

Criar estrutura formal.

Exemplo:

```ts
interface ActionRequest {
  actionId: string;
  source: "user" | "ai" | "ui";
  targetId?: string;
  requestedAt: string;
}
```

Adaptar conforme necessário.

---

# Action Result

Criar resposta padronizada:

```ts
interface ActionResult {
  success: boolean;
  message: string;
  errorCode?: string;
}
```

---

# Action History

Criar nova migration.

Adicionar tabela:

```text
action_history
```

Campos conceituais:

```text
id
action_id
source
target_type
target_id
target_name
permission
confirmation_required
confirmed
success
error
created_at
completed_at
```

Não armazenar dados sensíveis desnecessários.

---

# Auditoria

Toda tentativa de ação deve gerar log.

Isso inclui:

* sucesso;
* falha;
* bloqueio;
* alvo inválido;
* aplicativo desabilitado;
* workspace inexistente.

---

# Interface — Automation Core

Criar nova área:

# Automação

A interface deve permitir visualizar:

* aplicativos autorizados;
* URLs cadastradas;
* histórico de ações;
* status do Automation Core.

Estrutura conceitual:

```text
AUTOMATION CORE

STATUS             ONLINE
AÇÕES DISPONÍVEIS  05
APLICATIVOS         04
WORKSPACES          07

[ APLICATIVOS ]
[ URLs ]
[ HISTÓRICO ]
```

Preservar a identidade HUD.

---

# Command Center

Adicionar indicador compacto:

```text
AUTOMATION CORE
ONLINE
SAFE MODE
```

Não precisa ocupar grande espaço.

---

# AI Core

Adicionar tools específicas:

```text
open_application
open_workspace
open_project
reveal_workspace
open_registered_url
```

Mas essas tools NÃO executam diretamente.

Fluxo obrigatório:

```text
AI Tool
↓
Action Request
↓
Policy Engine
↓
Automation Core
```

---

# Não permitir bypass

O AI Core não pode chamar Tauri diretamente para ações.

O frontend não deve possuir uma segunda implementação paralela.

A UI e o AI Core devem utilizar o mesmo Automation Core sempre que possível.

---

# Segurança

Toda entrada do LLM deve ser considerada não confiável.

Validar:

* IDs;
* existência;
* enabled;
* associação;
* tipo de ação;
* permissão.

Nunca concatenar entrada do LLM em comando de shell.

---

# Paths

Caminhos devem vir exclusivamente de registros persistidos e autorizados.

Nunca do texto do usuário processado pelo LLM.

---

# URLs

URLs também devem vir do registry.

Não executar:

```text
open_url("qualquer string vinda do modelo")
```

---

# Windows

O ambiente principal é Windows.

Usar Tauri/Rust para abrir recursos com APIs apropriadas.

Evitar shell genérico.

Se precisar invocar processo específico, o comando deve ser construído internamente com dados previamente autorizados.

---

# Cadastro de aplicações e URLs

O operador pode configurar esses registros pela UI.

Essas operações de configuração não devem ser expostas como tools de escrita ao LLM nesta versão.

---

# Comandos naturais obrigatórios

Azriel deve conseguir lidar com:

```text
Abra o GeneScope.
```

```text
Abra o Visual Studio Code.
```

```text
Abra o workspace do Azriel.
```

```text
Mostre a pasta do ArcCore.
```

```text
Abra o GitHub do Azriel.
```

Somente se os respectivos recursos estiverem cadastrados e autorizados.

---

# Recurso não cadastrado

Exemplo:

```text
Abra o Photoshop.
```

Se Photoshop não estiver cadastrado:

Azriel deve responder algo equivalente a:

```text
O Photoshop não está registrado como aplicativo autorizado.
```

Não procurar executável automaticamente.

Não tentar localizar no disco.

---

# Recurso desativado

Se aplicativo/workspace/URL estiver cadastrado como desativado:

não executar.

Informar que o recurso está desativado.

---

# Confirmação futura

Preparar a arquitetura para que, na v0.8.1, possamos criar ações como:

```text
start_routine
```

que podem exigir confirmação.

Não implementar rotinas complexas agora.

---

# Logs técnicos

Registrar:

* action requested;
* policy result;
* execution started;
* execution completed;
* execution failed.

Evitar logar informações sensíveis.

---

# Estados visuais

Automation Core pode possuir:

```text
offline
safe
executing
waiting_confirmation
blocked
error
```

Integrar ao HUD quando fizer sentido.

---

# AzrielCore

Durante execução:

* alterar discretamente estado visual;
* mostrar algo como `EXECUTANDO AÇÃO`;
* voltar a idle após conclusão.

Não exagerar na animação.

---

# Testes obrigatórios

Criar testes para:

1. Action Registry;
2. Policy Engine;
3. ação permitida;
4. ação bloqueada;
5. ID inexistente;
6. registro desabilitado;
7. open_application autorizado;
8. open_workspace autorizado;
9. open_project sem workspace;
10. reveal_workspace;
11. open_registered_url;
12. action_history;
13. tentativa bloqueada gera log;
14. AI Core não recebe path arbitrário;
15. AI Core não recebe URL arbitrária;
16. confirmação futura;
17. erro nativo;
18. Automation Core offline/erro.

Utilizar mocks/fakes para evitar abrir programas durante testes automatizados.

---

# Fake Action Executor

Criar uma implementação fake para testes.

Exemplo conceitual:

```text
FakeActionExecutor
```

Assim testes não abrem aplicações reais.

---

# Não implementar nesta versão

Fora do escopo:

* shell genérico;
* PowerShell;
* cmd arbitrário;
* scripts;
* edição de arquivos;
* exclusão de arquivos;
* movimentação de arquivos;
* criação arbitrária de arquivos;
* matar processos;
* iniciar/parar serviços;
* Git commit;
* Git push;
* Git pull;
* Git checkout;
* instalação de software;
* download automático;
* voz;
* MQTT;
* ESP32;
* IoT;
* rotinas complexas.

---

# Documentação

Criar:

```text
docs/versions/v0.8.0.md
```

Documentar:

* objetivo;
* arquitetura;
* matriz de permissões;
* Action Registry;
* Policy Engine;
* Confirmation Gate;
* Application Registry;
* URL Registry;
* Action History;
* AI integration;
* segurança;
* limitações.

Atualizar:

```text
docs/roadmap.md
```

Marcar:

```text
v0.7 — concluída
v0.8.0 — Automation Core / Safe Actions — em desenvolvimento
```

Após validação:

```text
v0.8.0 — concluída
```

---

# Critérios de aceite

A v0.8.0 só está concluída quando:

1. Automation Core existir;
2. Action Registry existir;
3. Policy Engine existir;
4. Confirmation Gate estiver preparado;
5. Application Registry existir;
6. URL Registry existir;
7. Action History existir;
8. aplicativos forem persistidos;
9. URLs forem persistidas;
10. UI permitir cadastrar aplicativo;
11. UI permitir cadastrar URL;
12. `open_application` funcionar;
13. `open_workspace` funcionar;
14. `open_project` funcionar;
15. `reveal_workspace` funcionar;
16. `open_registered_url` funcionar;
17. AI Core puder solicitar essas ações;
18. nenhuma tool receber caminho arbitrário;
19. nenhuma tool receber URL arbitrária;
20. nenhum shell genérico existir;
21. ações bloqueadas forem rejeitadas;
22. toda ação gerar histórico;
23. erros gerarem histórico;
24. recursos desativados não puderem executar;
25. recursos inexistentes não forem pesquisados automaticamente;
26. Command Center mostrar Automation Core;
27. AzrielCore refletir execução;
28. testes relevantes passarem;
29. TypeScript não apresentar erros;
30. Rust compilar;
31. build funcionar;
32. Tauri iniciar normalmente;
33. versões anteriores não sofrerem regressão;
34. documentação da v0.8.0 existir.

---

# Teste final obrigatório

Cadastrar pela UI:

* Visual Studio Code;
* um workspace;
* um projeto relacionado;
* uma URL registrada.

Testar:

```text
Azriel, abra o Visual Studio Code.
```

Confirmar que o aplicativo abre.

Depois:

```text
Azriel, abra o GeneScope.
```

Confirmar que o projeto/workspace autorizado é aberto corretamente.

Depois:

```text
Azriel, mostre a pasta do GeneScope.
```

Confirmar que o explorador abre no workspace correto.

Depois:

```text
Azriel, abra o GitHub do Azriel.
```

Confirmar que apenas a URL registrada é utilizada.

Testar recurso não cadastrado:

```text
Azriel, abra um aplicativo inexistente.
```

Azriel deve recusar.

Testar tentativa de ação arbitrária.

Ela deve ser bloqueada.

Verificar:

```text
Automation Core → Histórico
```

Todas as tentativas devem estar registradas.

---

# Resultado esperado

Ao concluir a v0.8.0, Azriel deve conseguir realizar suas primeiras ações reais no computador.

Mas toda autoridade deve permanecer limitada a recursos previamente registrados e autorizados.

A filosofia desta versão é:

# CAPACIDADE SEM ACESSO IRRESTRITO.

Azriel começa a agir.

Mas apenas dentro de fronteiras explicitamente definidas pelo operador.

---

# Estado da implementação — 31/08/2026

Status: **concluída e aprovada em 31 de agosto de 2026**.

Implementado:

* migration SQLite `0007_automation_core.sql`;
* Application Registry e URL Registry com CRUD manual;
* vínculo opcional de aplicativo autorizado no Workspace Registry;
* Action Registry com as cinco ações previstas;
* Policy Engine e Confirmation Gate no backend Rust;
* executor nativo Windows sem shell genérico;
* Action History para sucessos, recusas e falhas;
* área Automation Core com Aplicativos, URLs e Histórico;
* indicador compacto no Command Center;
* estado visual `EXECUTANDO AÇÃO` no AzrielCore;
* rotas do AI Core restritas a pedidos imperativos explícitos;
* contratos da IA baseados somente em IDs, sem path ou URL;
* testes com executor fake para impedir abertura real durante a suíte automatizada.

Validação automatizada atual:

* Rust: 28 testes aprovados e 1 smoke test do Ollama ignorado por ser manual;
* TypeScript/Vitest: 53 testes aprovados;
* TypeScript, lint e build Vite: aprovados;
* build Tauri release e empacotamento MSI/NSIS: aprovados.

Artefatos gerados:

* `src-tauri/target/release/bundle/msi/Azriel_0.8.0_x64_en-US.msi`;
* `src-tauri/target/release/bundle/nsis/Azriel_0.8.0_x64-setup.exe`.

Validação de encerramento:

* fluxo funcional aprovado pelo operador no aplicativo Tauri;
* cadastro manual de aplicativos e URLs confirmado;
* vínculo entre workspace, projeto e aplicativo de abertura confirmado;
* abertura segura restrita a registros autorizados;
* recusas e falhas disponíveis no Action History;
* correção do foco nos modais compartilhados validada;
* nenhuma interface de shell ou comando arbitrário foi adicionada.

Resultado final: **v0.8.0 — Automation Core / Safe Actions concluída**.

A abertura direta por botões no System Core não faz parte do encerramento desta
versão. A operação permanece disponível pelo Automation Core e por solicitações
explícitas no AI Core, preservando um único fluxo de política e auditoria.
