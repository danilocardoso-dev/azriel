# AZRIEL v0.7.1 — Autonomia dos Núcleos

Status: **concluída e validada em 31 de agosto de 2026**.

A versão **v0.7 — System Core** foi concluída e validada em 31 de agosto de
2026. Antes de iniciar a v0.8, esta versão intermediária amplia a autonomia do
operador sobre os dados que o Azriel já mantém no SQLite.

---

# Objetivo

Permitir que os principais núcleos deixem de ser apenas áreas de consulta e se
tornem módulos operacionais completos.

Ao concluir a v0.7.1, o operador deverá conseguir administrar projetos,
conhecimentos e formações diretamente pela interface, recuperar notas
arquivadas e iniciar operações frequentes a partir do Command Center.

Esta versão administra dados internos do Azriel. Ela não antecipa a Automation
Core e não concede à IA permissão para alterar esses dados.

---

# Diagnóstico confirmado

## Project Core

A página de Projetos atualmente permite pesquisar, listar e consultar detalhes.
O contexto, os services, os repositories e os comandos Tauri já oferecem
salvamento e exclusão. A lacuna está na interface.

## Knowledge Core

A página de Conhecimento lista áreas e permite atualizar cobertura e
profundidade, mas não oferece criação, edição dos dados gerais ou exclusão. A
camada de persistência já suporta essas operações.

## Formação

A página de Formação é somente leitura. O backend e o frontend já possuem as
operações de salvar e excluir registros.

## Operações Diárias

Tarefas e notas já possuem os fluxos essenciais. Notas arquivadas permanecem no
banco, porém não existe uma visão para consultá-las ou restaurá-las.

## Mapa Stark e Command Center

Essas telas representam ou agregam dados pertencentes a outros núcleos. Elas
não devem ganhar uma segunda implementação de CRUD. Devem reutilizar editores e
ações dos módulos proprietários.

## Pesquisa, AI Core e System Core

O Research Core ainda utiliza dados estáticos e exige uma futura camada de
persistência. O AI Core possui gerenciamento básico de conversas, mas continua
somente leitura sobre os demais núcleos. Workspaces já têm CRUD; processos, Git
e arquivos permanecem deliberadamente sem ações destrutivas.

---

# Escopo aprovado

## 1. Projetos

Adicionar à página de Projetos:

* criação;
* edição;
* exclusão;
* formulário com validação;
* estados de salvamento e erro;
* confirmação específica antes da exclusão;
* atualização imediata da lista e dos detalhes após salvar.

O formulário deve cobrir os campos persistidos pelo modelo atual, incluindo
nome, descrição, objetivo, status, tecnologias, repositório e conhecimentos
relacionados, quando disponíveis no contrato existente.

### Exclusão de projeto

A confirmação deve informar claramente o projeto afetado e as consequências:

* relações com conhecimentos são removidas;
* tarefas e notas são preservadas e perdem apenas o vínculo com o projeto;
* workspaces são preservados e perdem apenas o vínculo com o projeto;
* pastas, arquivos e repositórios físicos não são removidos.

## 2. Knowledge Core

Adicionar à página de Conhecimento:

* criação de área de conhecimento;
* edição dos dados gerais;
* manutenção das métricas existentes;
* exclusão com confirmação específica;
* validação e feedback de persistência.

O editor deve ser reutilizável pelo Mapa Stark para evitar regras e formulários
duplicados.

### Exclusão de conhecimento

A confirmação deve informar que:

* relações com projetos e histórico associado são removidos conforme o schema;
* tarefas e notas são preservadas e perdem apenas o vínculo com o conhecimento;
* nenhum projeto, tarefa ou nota é excluído junto com a área.

## 3. Formação

Adicionar à página de Formação:

* criação;
* edição;
* exclusão;
* suporte aos campos existentes de tipo, instituição, status, período, datas,
  descrição e domínios;
* validação de datas e campos obrigatórios;
* estados de loading, erro, vazio e salvamento.

## 4. Notas arquivadas

Ampliar Operações Diárias com:

* visão ou filtro de notas arquivadas;
* abertura dos detalhes de uma nota arquivada;
* restauração para o estado ativo;
* manutenção da exclusão definitiva com confirmação.

Arquivar continua sendo uma ação reversível. Excluir continua sendo definitivo.

## 5. Mapa Stark

Ao selecionar uma área, permitir abrir o mesmo editor usado pelo Knowledge
Core. Alterações devem atualizar o mapa e os demais painéis dependentes sem
recarregar o aplicativo.

## 6. Command Center

Adicionar atalhos compactos para operações frequentes:

* novo projeto;
* nova tarefa;
* nova nota.

Os atalhos devem levar ao módulo proprietário e iniciar o fluxo correto. O
Command Center não deve duplicar formulários completos.

## 7. Padrão de interação

Os novos fluxos devem preservar o HUD técnico e operacional existente:

* tipografia legível;
* alta densidade de informação;
* drawers ou modais consistentes;
* botões com ações explícitas;
* confirmação contextual para exclusão;
* bloqueio de envio duplicado enquanto uma operação estiver em andamento;
* mensagem útil quando uma persistência falhar.

---

# Arquitetura

Manter a separação existente:

```text
Page / Component
        ↓
Context
        ↓
Service
        ↓
Repository TypeScript
        ↓
Comando Tauri
        ↓
Repository Rust
        ↓
SQLite
```

Componentes React não devem acessar SQLite ou `invoke` diretamente quando já
existir repository para a entidade.

Projetos, conhecimentos e formação já possuem contratos de persistência. Não é
esperada uma nova migration para o escopo principal. Caso uma limitação real do
schema seja descoberta, ela deve ser documentada antes de qualquer migration e
uma migration nova deverá ser criada; migrations aplicadas não serão editadas.

---

# Integridade e segurança

* preservar todos os dados existentes das versões v0.5 a v0.7;
* respeitar as foreign keys e seus comportamentos atuais;
* não remover arquivos ou diretórios físicos ao excluir um projeto;
* não executar comandos Git de escrita;
* não encerrar processos;
* não aceitar comandos arbitrários de shell;
* não disponibilizar tools de escrita ao AI Core;
* nunca interpretar exclusão de registro como autorização para excluir recursos
  externos.

---

# Fora do escopo

Não implementar nesta versão:

* persistência ou CRUD do Research Core;
* escrita de projetos, conhecimentos, formação, tarefas ou notas pela IA;
* renomear, exportar ou excluir conversas em massa no AI Core;
* persistência das preferências visuais de Configurações;
* exclusão de workspaces físicos;
* operações destrutivas sobre processos;
* escrita em arquivos;
* commit, push, pull, checkout ou outras mutações Git;
* abertura de programas ou execução de automações;
* recursos da v0.8 — Automation Core.

---

# Riscos e cuidados

## Relações entre entidades

Edições e exclusões devem manter listas, contadores, detalhes e seleções
sincronizados. Nenhuma referência removida pode deixar a interface em estado
inválido.

## Formulários compartilhados

O editor reutilizado pelo Knowledge Core e pelo Mapa Stark deve possuir uma
única regra de validação e persistência.

## Exclusões

O modal deve exibir o nome do registro e diferenciar claramente uma ação
reversível, como arquivar, de uma exclusão definitiva.

## Navegação

Atalhos do Command Center devem abrir o fluxo solicitado sem interferir na
navegação fixa, na rolagem interna ou nos estados já carregados dos núcleos.

---

# Validação obrigatória

## Projetos

1. criar um projeto;
2. editar seus campos;
3. alterar conhecimentos relacionados;
4. excluir com confirmação;
5. confirmar que tarefas, notas e workspaces relacionados foram preservados;
6. confirmar que nenhum arquivo físico foi removido.

## Conhecimentos

1. criar uma área;
2. editar seus dados e métricas;
3. editar a mesma área pelo Mapa Stark;
4. excluir com confirmação;
5. confirmar que tarefas, notas e projetos continuam existentes.

## Formação

1. criar uma formação;
2. editar status, datas e informações gerais;
3. excluir com confirmação;
4. validar datas incompletas e inválidas.

## Notas

1. arquivar uma nota;
2. encontrá-la na visão de arquivadas;
3. restaurá-la;
4. confirmar que retorna à visão de notas ativas;
5. excluir definitivamente uma nota arquivada.

## Integração e regressão

1. usar os três atalhos do Command Center;
2. fechar e reabrir o aplicativo e confirmar a persistência;
3. validar estados de loading, erro e vazio;
4. executar testes TypeScript e Rust relevantes;
5. executar lint;
6. executar build frontend;
7. executar build Tauri;
8. confirmar manualmente os fluxos na aplicação desktop.

---

# Critérios de aceite

A v0.7.1 só poderá ser concluída quando:

1. projetos puderem ser criados, editados e excluídos pela interface;
2. conhecimentos puderem ser criados, editados e excluídos pela interface;
3. formações puderem ser criadas, editadas e excluídas pela interface;
4. o Mapa Stark reutilizar a edição do Knowledge Core;
5. notas arquivadas puderem ser consultadas e restauradas;
6. o Command Center oferecer os atalhos aprovados;
7. exclusões apresentarem confirmação contextual;
8. estados de loading, erro, vazio e salvamento estiverem tratados;
9. relações e dados dependentes forem preservados conforme o schema;
10. nenhum arquivo físico ou repositório Git for alterado por uma exclusão;
11. o AI Core continuar somente leitura sobre os núcleos;
12. dados existentes sobreviverem à atualização;
13. testes relevantes, lint e builds forem aprovados;
14. a validação funcional desktop for aprovada pelo operador;
15. `Docs/Version/Version_0.7.1.md` permanecer atualizado com o resultado real.

---

# Resultado esperado

Ao concluir a v0.7.1, o Azriel não apenas apresentará os dados do operador. Ele
permitirá mantê-los organizados e atuais diretamente em seus núcleos, com
operações previsíveis, reversibilidade quando aplicável e confirmações claras
para ações definitivas.

A versão seguinte continuará sendo:

# v0.8 — Automation Core

Nela será avaliada, separadamente, a execução controlada de ações no computador.

---

# Registro de implementação — 31 de agosto de 2026

Status: **concluída e validada pelo operador em 31 de agosto de 2026**.

Entregue:

* criação, edição e exclusão de projetos;
* seleção dos conhecimentos relacionados a cada projeto;
* criação, edição de dados gerais e exclusão de conhecimentos;
* preservação do fluxo histórico de cobertura e profundidade;
* criação, edição e exclusão de formações;
* validação de datas e campos obrigatórios nos novos formulários;
* editor de conhecimento compartilhado com o Mapa Stark;
* visão de notas arquivadas e restauração para o estado ativo;
* atalhos para novo projeto, nova tarefa e nova nota no Command Center;
* modais de exclusão específicos, com identificação do registro e consequências;
* estados vazios que continuam permitindo a criação do primeiro registro;
* sincronização de projetos, conhecimentos, tarefas, notas e workspaces depois
  de alterações em relações;
* versão técnica atualizada para `0.7.1` no frontend, Tauri, Cargo e AI Core.

Validação automatizada atual:

* 43 testes TypeScript aprovados;
* 20 testes Rust aprovados e 1 smoke test do Ollama ignorado por depender de
  execução explícita;
* lint aprovado sem erros;
* build frontend aprovado;
* `git diff --check` aprovado.

Validação visual e operacional:

* identificação `HUD V0.7.1` confirmada no shell web;
* o preview web bloqueou corretamente o acesso ao banco fora do Tauri;
* fluxos funcionais e visuais aprovados pelo operador no aplicativo desktop;
* persistência disponível pela ponte Tauri e pelo SQLite local.

Build de release aprovado:

* `src-tauri/target/release/azriel.exe`;
* `src-tauri/target/release/bundle/msi/Azriel_0.7.1_x64_en-US.msi`;
* `src-tauri/target/release/bundle/nsis/Azriel_0.7.1_x64-setup.exe`.

Encerramento:

* v0.7.1 — Autonomia dos Núcleos: **concluída**;
* próxima versão principal planejada: **v0.8 — Automation Core**.
