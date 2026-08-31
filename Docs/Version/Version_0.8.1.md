# AZRIEL v0.8.1 — Automation Core / Rotinas

A versão **v0.8.0 — Automation Core / Safe Actions** foi concluída e validada.

Agora implemente:

# v0.8.1 — Rotinas

Antes de modificar qualquer arquivo:

1. leia completamente o `README.md`;
2. leia `docs/roadmap.md`;
3. leia `docs/versions/v0.8.0.md`;
4. examine a arquitetura atual do Automation Core;
5. entenda `Action Registry`, `Policy Engine`, `Confirmation Gate`, `Application Registry`, `Workspace Registry` e `Action History`;
6. execute lint, testes e build;
7. preserve tudo que já estiver funcionando.

Não refatore a arquitetura da v0.8.0 sem necessidade.

---

# Objetivo

A v0.8.1 deve permitir que o Azriel execute **rotinas compostas por ações já autorizadas**.

Uma rotina representa uma sequência explícita de ações.

Exemplo:

```text
ROTINA: Ambiente de Desenvolvimento

1. abrir Visual Studio Code
2. abrir workspace Azriel
3. abrir DBeaver
```

Outro exemplo:

```text
ROTINA: Biomedicina

1. abrir Mendel Lab
2. abrir GeneScope
3. abrir URL de pesquisa cadastrada
```

A rotina NÃO deve possuir acesso a comandos livres.

Ela apenas combina ações já existentes no `Action Registry`.

---

# Princípio central

A v0.8.1 deve implementar:

# COMPOSIÇÃO DE AÇÕES AUTORIZADAS.

Uma rotina não aumenta permissões.

Se uma ação não é permitida individualmente, ela também não pode ser executada dentro de uma rotina.

---

# Arquitetura

Fluxo conceitual:

```text
Usuário / AI Core
        ↓
Routine Request
        ↓
Routine Registry
        ↓
Routine Validator
        ↓
Policy Engine
        ↓
Confirmation Gate
        ↓
Routine Executor
        ↓
Action Registry
        ↓
Automation Core
        ↓
Windows
        ↓
Action History
```

---

# Routine Registry

Criar persistência para rotinas.

Nova migration.

Tabela conceitual:

```text
routines
```

Campos sugeridos:

```text
id
name
description
enabled
confirmation_required
created_at
updated_at
```

Criar também estrutura para passos da rotina:

```text
routine_steps
```

Campos:

```text
id
routine_id
step_order
action_id
target_type
target_id
delay_ms
enabled
created_at
```

Adaptar à arquitetura existente.

---

# Estrutura de rotina

Criar tipos claros.

Exemplo:

```ts
interface Routine {
  id: string;
  name: string;
  description?: string;
  enabled: boolean;
  confirmationRequired: boolean;
  steps: RoutineStep[];
}
```

```ts
interface RoutineStep {
  id: string;
  order: number;
  actionId: string;
  targetType: string;
  targetId: string;
  delayMs?: number;
  enabled: boolean;
}
```

Evitar `any`.

---

# Ações permitidas em rotinas

Na v0.8.1, permitir inicialmente somente:

```text
open_application
open_workspace
open_project
reveal_workspace
open_registered_url
```

Não adicionar novas ações de maior autoridade apenas para completar rotinas.

---

# Validação

Antes de executar uma rotina, validar todos os passos.

Verificar:

* rotina existe;
* rotina está habilitada;
* ação existe;
* ação está habilitada;
* alvo existe;
* alvo está habilitado;
* permissão da ação;
* associação correta;
* ordem dos passos.

Se qualquer passo for inválido, a rotina não deve iniciar parcialmente por padrão.

Retornar erro claro.

---

# Execução sequencial

As ações devem ser executadas na ordem definida.

Exemplo:

```text
1 → abrir VS Code
2 → aguardar intervalo opcional
3 → abrir workspace
4 → abrir DBeaver
```

Não é necessário paralelismo nesta versão.

Priorizar previsibilidade.

---

# Delay entre passos

Permitir `delayMs` opcional.

Usar apenas para pequenos intervalos necessários entre aplicações.

Criar limites razoáveis.

Não permitir delays absurdamente longos.

Sugestão:

```text
0 até 10000 ms
```

Validar o valor.

---

# Confirmation Gate

Rotinas devem integrar com o sistema de confirmação já criado.

Por padrão:

rotinas com múltiplas ações devem poder exigir confirmação.

Exemplo:

```text
AZRIEL // CONFIRMAÇÃO DE ROTINA

Rotina:
Ambiente de Desenvolvimento

Ações:
• Abrir Visual Studio Code
• Abrir workspace Azriel
• Abrir DBeaver

[ EXECUTAR ]
[ CANCELAR ]
```

---

# Regra de confirmação

Uma rotina deve exigir confirmação quando:

* `confirmation_required = true`;
* possuir ação classificada como `confirm_write`;
* política futura determinar isso.

SAFE_WRITE individual não deve automaticamente remover a necessidade de confirmação da rotina.

---

# Interface — Rotinas

Adicionar seção no módulo Automação:

```text
AUTOMAÇÃO

[ APLICATIVOS ]
[ URLs ]
[ ROTINAS ]
[ HISTÓRICO ]
```

Criar tela de gerenciamento de rotinas.

Permitir:

* criar rotina;
* editar nome;
* editar descrição;
* ativar/desativar;
* adicionar passos;
* remover passos;
* reordenar passos;
* configurar delay;
* definir confirmação obrigatória;
* excluir rotina.

---

# Editor de rotina

A UI deve permitir montar uma rotina visualmente.

Exemplo:

```text
ROTINA
Ambiente de Desenvolvimento

PASSO 01
Ação: Abrir aplicativo
Alvo: Visual Studio Code

PASSO 02
Ação: Abrir workspace
Alvo: Azriel

PASSO 03
Ação: Abrir aplicativo
Alvo: DBeaver
```

Não permitir digitar caminhos arbitrários ou comandos.

Todos os alvos devem vir dos registros já autorizados.

---

# Routine Executor

Criar componente/serviço responsável por executar a sequência.

Responsabilidades:

* validar;
* solicitar confirmação;
* executar ações;
* respeitar ordem;
* respeitar delay;
* coletar resultados;
* interromper quando necessário;
* registrar histórico.

---

# Política de falha

Por padrão:

# STOP ON ERROR

Se um passo falhar:

* parar a rotina;
* não executar os próximos passos;
* registrar o passo que falhou;
* informar claramente o resultado.

Exemplo:

```text
PASSO 1 → OK
PASSO 2 → ERRO
ROTINA → INTERROMPIDA
```

Não continuar silenciosamente.

---

# Resultado da rotina

Criar estrutura equivalente a:

```ts
interface RoutineExecutionResult {
  success: boolean;
  routineId: string;
  completedSteps: number;
  failedStep?: number;
  error?: string;
}
```

---

# Histórico

Expandir auditoria.

Criar registro de execução de rotina.

Pode ser nova tabela:

```text
routine_history
```

ou integrar adequadamente ao `action_history`.

Registrar:

```text
routine_id
routine_name
source
started_at
completed_at
success
failed_step
error
confirmed
```

Cada ação individual da rotina também deve continuar aparecendo no histórico de ações.

---

# Origem

Rotinas podem ser iniciadas por:

```text
ui
user
ai
```

Manter rastreabilidade.

---

# AI Core

Adicionar tool:

```text
list_routines
```

e:

```text
run_routine
```

Entrada:

```text
routineId
```

Nunca receber descrição arbitrária de passos do LLM.

O LLM apenas seleciona uma rotina existente.

---

# Intenção explícita

Exemplo permitido:

```text
Azriel, execute a rotina Ambiente de Desenvolvimento.
```

Exemplo que NÃO deve executar automaticamente:

```text
Talvez eu programe um pouco hoje.
```

Azriel pode sugerir a rotina, mas não executá-la.

---

# Sugestão de rotina

O AI Core pode responder:

```text
Existe uma rotina chamada "Ambiente de Desenvolvimento".
Deseja executá-la?
```

Mas deve aguardar intenção explícita.

---

# Comandos obrigatórios

Azriel deve conseguir responder corretamente:

```text
Quais rotinas eu tenho?
```

```text
Execute a rotina Ambiente de Desenvolvimento.
```

```text
O que a rotina Biomedicina faz?
```

```text
Quais ações existem na rotina de desenvolvimento?
```

---

# Segurança

Rotinas não podem:

* criar ações novas;
* executar shell;
* receber caminhos arbitrários;
* receber URLs arbitrárias;
* modificar arquivos;
* matar processos;
* executar Git de escrita;
* instalar programas;
* contornar `Policy Engine`;
* contornar `Confirmation Gate`.

---

# Não permitir escalada

Uma rotina criada manualmente não pode referenciar action IDs inexistentes ou bloqueados.

O backend deve validar isso.

Não confiar apenas na UI.

---

# Cancelamento

Se for simples na arquitetura atual, permitir cancelar uma rotina antes do próximo passo.

Se houver suporte:

```text
CANCELAR ROTINA
```

A execução atual não precisa ser revertida.

Não implementar rollback nesta versão.

Se cancelamento gerar complexidade excessiva, documentar como limitação.

---

# Rollback

Não implementar rollback.

Exemplo:

se VS Code foi aberto e o passo seguinte falhou, não tentar fechar o VS Code automaticamente.

Ações executadas permanecem executadas.

---

# Rotinas sugeridas para teste

Não criar automaticamente sem necessidade, mas utilizar como referência.

## Ambiente de Desenvolvimento

```text
Visual Studio Code
+
workspace Azriel
+
DBeaver
```

## Pesquisa Biomédica

```text
Mendel Lab
+
GeneScope
+
URL de pesquisa cadastrada
```

## ArcCore

```text
Visual Studio Code
+
workspace ArcCore
```

---

# Command Center

Adicionar indicador compacto:

```text
ROTINAS

03 CADASTRADAS
01 EXECUTADA HOJE
```

Não criar um painel excessivamente grande.

---

# AzrielCore

Durante execução de rotina:

estado visual:

```text
EXECUTANDO ROTINA
```

Opcionalmente mostrar:

```text
PASSO 2 / 3
```

Manter animação discreta.

---

# Estados

Adicionar estados conceituais:

```text
idle
validating
waiting_confirmation
executing
completed
cancelled
failed
```

---

# Persistência

Rotinas devem sobreviver ao reinício.

Teste obrigatório:

1. criar rotina;
2. adicionar passos;
3. fechar Azriel;
4. abrir novamente;
5. confirmar que a rotina permanece intacta.

---

# Testes

Adicionar testes para:

1. criação de rotina;
2. persistência;
3. ordenação de passos;
4. validação de ação;
5. validação de alvo;
6. rotina desabilitada;
7. ação bloqueada;
8. confirmação obrigatória;
9. execução sequencial;
10. delay;
11. stop on error;
12. histórico;
13. AI `list_routines`;
14. AI `run_routine`;
15. rotina inexistente;
16. tentativa de bypass;
17. fake executor.

Não abrir aplicações reais em testes automatizados.

---

# FakeRoutineExecutor

Criar fake/mocks quando apropriado.

Testes devem conseguir simular:

```text
passo 1 → sucesso
passo 2 → sucesso
```

e:

```text
passo 1 → sucesso
passo 2 → falha
```

---

# Não implementar nesta versão

Fora do escopo:

* shell;
* PowerShell arbitrário;
* cmd arbitrário;
* scripts;
* edição de arquivo;
* exclusão de arquivo;
* manipulação Git;
* kill process;
* scheduler;
* execução em horário programado;
* recorrência automática;
* gatilhos por evento;
* voz;
* IoT;
* MQTT;
* ESP32;
* rollback.

Rotinas são iniciadas manualmente ou por solicitação explícita ao AI Core.

---

# Documentação

Criar:

```text
docs/versions/v0.8.1.md
```

Documentar:

* objetivo;
* Routine Registry;
* Routine Executor;
* Routine Steps;
* validação;
* Confirmation Gate;
* stop on error;
* histórico;
* AI integration;
* segurança;
* limitações.

Atualizar:

```text
docs/roadmap.md
```

Marcar:

```text
v0.8.0 — concluída
v0.8.1 — Rotinas — em desenvolvimento
```

Após validação:

```text
v0.8.1 — concluída
```

---

# Critérios de aceite

A v0.8.1 só está concluída quando:

1. Routine Registry existir;
2. rotinas forem persistidas;
3. passos forem persistidos;
4. UI permitir criar rotina;
5. UI permitir editar rotina;
6. UI permitir excluir rotina;
7. passos puderem ser adicionados;
8. passos puderem ser removidos;
9. passos puderem ser reordenados;
10. delay puder ser configurado;
11. apenas ações registradas puderem ser utilizadas;
12. apenas alvos autorizados puderem ser utilizados;
13. Routine Validator existir;
14. Routine Executor existir;
15. Policy Engine for respeitado;
16. Confirmation Gate for respeitado;
17. execução for sequencial;
18. stop on error funcionar;
19. falhas forem registradas;
20. sucessos forem registrados;
21. action_history continuar sendo preenchido;
22. histórico de rotinas existir;
23. AI Core puder listar rotinas;
24. AI Core puder solicitar execução;
25. AI Core não puder criar passos arbitrários;
26. intenção explícita for exigida;
27. AzrielCore refletir execução;
28. Command Center mostrar resumo;
29. TypeScript não apresentar erros;
30. Rust compilar;
31. testes passarem;
32. build funcionar;
33. Tauri iniciar normalmente;
34. funcionalidades anteriores não sofrerem regressão;
35. `docs/versions/v0.8.1.md` existir.

---

# Teste final obrigatório

Cadastrar:

```text
Rotina:
Ambiente de Desenvolvimento
```

Passos:

```text
1. abrir Visual Studio Code
2. abrir workspace Azriel
3. abrir DBeaver
```

Executar pela UI.

Confirmar:

* validação;
* confirmação quando configurada;
* execução na ordem correta;
* histórico.

Depois perguntar:

```text
Azriel, quais rotinas eu tenho?
```

Depois:

```text
Azriel, o que faz a rotina Ambiente de Desenvolvimento?
```

Depois:

```text
Azriel, execute a rotina Ambiente de Desenvolvimento.
```

Confirmar que a mesma arquitetura é usada.

Criar também uma rotina com um passo inválido/desativado.

A rotina deve falhar na validação antes de iniciar.

Simular falha no segundo passo.

Confirmar:

```text
passo 1 → executado
passo 2 → falhou
passo 3 → NÃO executado
```

Verificar histórico completo.

---

# Resultado esperado

Ao concluir a v0.8.1, Azriel deixa de apenas executar ações isoladas e passa a organizar essas capacidades em procedimentos reutilizáveis.

O operador poderá transformar sequências repetitivas em rotinas controladas.

Exemplo:

```text
"Azriel, iniciar ambiente de desenvolvimento."
```

não significa acesso livre ao computador.

Significa:

```text
Rotina registrada
↓
Passos conhecidos
↓
Permissões verificadas
↓
Confirmação quando necessária
↓
Execução auditada
```

A filosofia da versão é:

# AUTOMAÇÃO POR COMPOSIÇÃO, NÃO POR IMPROVISO.

---

# Estado da implementação — 31/08/2026

Status: **concluída e validada em 31/08/2026**.

## Arquitetura entregue

```text
UI / AI Core
↓
Routine Request por ID
↓
Routine Registry
↓
validação integral de todos os passos
↓
Confirmation Gate
↓
execução sequencial pelo Policy Engine existente
↓
Routine History + Action History
```

## Persistência

A migration `0008_routines.sql` adiciona:

* `routines` com nome, descrição, estado, confirmação e revisão;
* `routine_steps` com ordem, action ID, target ID, intervalo e estado;
* `routine_history` com origem, estado, progresso, falha e confirmação;
* correlação opcional de cada `action_history` com a rotina e o passo.

Salvar uma rotina substitui seus passos dentro de uma transação e incrementa sua revisão. Uma confirmação pendente deixa de ser válida se a rotina for alterada antes da execução.

## Validação e segurança

O backend valida:

* ID e estado da rotina;
* ordem contínua dos passos;
* limite de 20 passos;
* intervalo individual de 0 a 10000 ms e soma máxima de 60000 ms;
* action ID pertencente às cinco ações seguras da v0.8.0;
* correspondência entre ação e tipo de alvo;
* existência e estado do alvo pelo `PolicyEngine`;
* existência de ao menos um passo ativo.

A validação completa acontece antes de criar o histórico de execução ou chamar o executor. Cada passo volta a passar pelo mesmo Policy Engine durante a execução. Rotinas não recebem comandos, paths ou URLs livres.

## Execução e Confirmation Gate

Os passos ativos são executados sequencialmente. O intervalo configurado é aplicado depois de um passo bem-sucedido e antes do próximo. A execução usa uma conexão SQLite dedicada, evitando manter o mutex principal do banco bloqueado durante os intervalos.

Quando `Abrir aplicativo` é seguido por `Abrir workspace` vinculado ao mesmo aplicativo, o executor garante automaticamente ao menos 2000 ms para a inicialização do programa. As operações nativas, o acesso ao banco e os intervalos são executados fora da thread da interface, evitando congelar o modal e o restante do aplicativo.

A confirmação é exigida quando:

* a rotina possui `confirmation_required = true`;
* uma política de ação exigir `confirm_write`;
* o AI Core solicitar uma rotina com mais de um passo.

Confirmações são de uso único. É possível cancelar enquanto a execução aguarda confirmação. Depois que a sequência começa, não há cancelamento intermediário nesta versão, mas o modal pode ser ocultado sem interromper a rotina. O estado visual do modal é recriado para cada histórico, evitando que uma nova execução herde o estado `PROCESSANDO` da anterior. Confirmações pendentes abandonadas por um reinício são encerradas como canceladas na próxima abertura do Azriel.

## Stop on error

Ao primeiro erro:

* a rotina é marcada como `failed`;
* o passo que falhou é registrado;
* os passos posteriores não são executados;
* as ações já concluídas permanecem executadas;
* não existe rollback automático.

## Interface

O Automation Core agora possui as abas:

```text
APLICATIVOS | URLs | ROTINAS | HISTÓRICO
```

O editor visual permite criar, editar, ativar, excluir e reordenar rotinas, escolher ações e alvos já autorizados e definir intervalos. O modal global de confirmação funciona fora da página de Automação, permitindo confirmar uma solicitação iniciada no AI Core.

O Command Center mostra rotinas ativas e execuções do dia. O AzrielCore possui o estado `EXECUTANDO ROTINA`.

## AI Core

Foram adicionadas:

* `list_routines`, somente leitura;
* `run_routine`, que resolve uma rotina existente e envia apenas seu ID.

O Tool Router exige verbo imperativo explícito para executar. Consultas como “quais rotinas eu tenho?” apenas listam os registros. Frases vagas não iniciam execução.

## Validação automatizada

* frontend: 56 testes aprovados;
* Rust: 35 testes aprovados e 1 smoke test manual do Ollama ignorado;
* lint: aprovado;
* TypeScript e build Vite: aprovados;
* build Rust: aprovado;
* persistência após reabertura do SQLite: aprovada;
* fake executor: ordem, delay, confirmação, cancelamento e stop on error aprovados;
* inspeção visual no navegador: abas e editor de rotinas aprovados.

## Validação operacional concluída

A rotina foi validada pelo operador no aplicativo Tauri. Durante o aceite, foram corrigidos o reaproveitamento indevido do estado `PROCESSANDO` ao abrir novamente o modal e a abertura prematura do workspace logo após iniciar o aplicativo vinculado.

Com a confirmação operacional do fluxo e a geração dos instaladores MSI e NSIS, a v0.8.1 foi encerrada em 31/08/2026.
