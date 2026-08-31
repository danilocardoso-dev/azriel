# AZRIEL v0.7 — System Core

A versão **v0.6 — AI Core** foi concluída e validada.

Agora implemente:

# v0.7 — System Core

Antes de modificar qualquer arquivo:

1. leia completamente o `README.md`;
2. leia `docs/roadmap.md`;
3. leia a documentação das versões anteriores, principalmente `v0.6.md`;
4. examine toda a arquitetura atual;
5. entenda a integração React ↔ Tauri ↔ Rust;
6. entenda o AI Core, Tool Registry e Tool Router;
7. execute lint, testes e build existentes;
8. preserve funcionalidades já validadas.

Não refatore componentes estáveis sem necessidade.

---

# Objetivo

A v0.7 deve permitir que o Azriel **observe o computador onde está executando**.

O System Core será responsável por fornecer informações reais e estruturadas sobre:

* sistema operacional;
* hardware;
* CPU;
* memória RAM;
* armazenamento;
* rede;
* processos;
* Ollama;
* workspaces autorizados;
* repositórios Git;
* estado geral do ambiente.

Nesta versão:

# SYSTEM CORE É SOMENTE LEITURA.

Azriel poderá observar e analisar.

Azriel NÃO poderá executar ações no computador.

---

# Princípio de segurança

A separação entre versões deve permanecer explícita:

```text
v0.7 → OBSERVAR
v0.8 → AGIR
```

Na v0.7 é proibido ao AI Core:

* executar comandos arbitrários;
* abrir aplicações;
* fechar aplicações;
* matar processos;
* editar arquivos;
* excluir arquivos;
* mover arquivos;
* criar arquivos arbitrariamente;
* executar scripts;
* executar shell;
* fazer Git commit;
* fazer Git push;
* fazer Git pull;
* alterar configurações do sistema;
* instalar software.

Não criar uma ferramenta genérica como:

```text
execute_command(command)
```

ou:

```text
run_shell(...)
```

nem mesmo escondida atrás de outro service.

---

# Arquitetura

Utilizar a arquitetura Tauri já existente.

Fluxo conceitual:

```text
React / HUD
     │
     ▼
System Service
     │
     ▼
Tauri Commands
     │
     ▼
Rust
     │
     ▼
Sistema Operacional
```

A camada React não deve implementar acesso nativo diretamente.

Informações do sistema devem ser obtidas pela camada nativa apropriada.

---

# System Core

Criar um módulo central responsável por representar o estado do computador.

Estrutura conceitual:

```text
System Core
│
├── System Information
├── CPU Monitor
├── Memory Monitor
├── Storage Monitor
├── Network Monitor
├── Process Monitor
├── Workspace Registry
├── Git Monitor
└── Service Monitor
```

Adaptar à arquitetura existente quando necessário.

---

# System Information

Coletar informações como:

* sistema operacional;
* versão;
* arquitetura;
* hostname;
* quantidade total de memória;
* quantidade de CPUs/cores disponível quando a API utilizada fornecer isso de forma confiável;
* uptime.

Não coletar identificadores sensíveis desnecessários.

Não coletar:

* serial de hardware;
* product keys;
* credenciais;
* identificadores de conta;
* informações pessoais desnecessárias.

---

# CPU

Implementar monitoramento de CPU.

Mostrar pelo menos:

```text
CPU

USO TOTAL       24%
CORES           8
```

Se for simples com a biblioteca escolhida, mostrar uso por core.

Não transformar isso em requisito obrigatório se gerar complexidade desnecessária.

Atualização deve ser periódica e eficiente.

Evitar polling agressivo.

---

# Memória

Mostrar:

```text
MEMÓRIA

USO             5.8 GB
TOTAL           8.0 GB
DISPONÍVEL      2.2 GB
UTILIZAÇÃO      72%
```

Valores devem ser reais.

Utilizar formatação adequada:

* MB;
* GB;
* porcentagem.

---

# Armazenamento

Mostrar unidades/discos relevantes.

Exemplo:

```text
ARMAZENAMENTO

C:
USADO           341 GB
LIVRE           135 GB
TOTAL           476 GB
```

Se houver múltiplas unidades, permitir visualizá-las.

Não acessar conteúdo dos arquivos apenas para calcular armazenamento.

---

# Rede

Implementar estado básico de rede.

Mostrar informações úteis como:

* interfaces;
* estado;
* tráfego recebido;
* tráfego enviado.

Evitar expor informações desnecessariamente sensíveis na interface.

Não implementar:

* packet capture;
* sniffing;
* inspeção de conteúdo;
* análise de pacotes.

O objetivo é telemetria, não análise forense de rede.

---

# Processos

Criar Process Monitor somente leitura.

Mostrar pelo menos:

```text
PROCESSO
PID
CPU
MEMÓRIA
```

Permitir:

* ordenar por CPU;
* ordenar por RAM;
* pesquisar por nome.

Não permitir:

```text
kill
terminate
suspend
restart
```

nesta versão.

---

# Ollama

Como o AI Core depende de Ollama, integrar seu estado ao System Core.

Mostrar:

```text
OLLAMA

STATUS          ONLINE
ENDPOINT        localhost:11434
MODELO          qwen2.5:0.5b
```

Reutilizar a configuração do AI Core.

Não criar configuração duplicada.

Se o modelo selecionado for alterado em Configurações, o System Core deve refletir essa mudança.

---

# Workspace Registry

Criar um conceito novo:

# Workspaces Autorizados

Azriel NÃO deve vasculhar automaticamente todo o computador.

O operador deve cadastrar explicitamente os diretórios que o Azriel pode observar como workspaces.

Exemplo:

```text
C:\Projetos\Azriel
C:\Projetos\GeneScope
C:\Projetos\Mendel-Lab
C:\Projetos\ArcCore
```

Persistir esses workspaces no SQLite.

---

# Workspace

Estrutura conceitual:

```ts
interface Workspace {
  id: string;
  name: string;
  path: string;
  projectId?: string;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}
```

Um workspace pode opcionalmente estar relacionado a um projeto existente do Azriel.

---

# Segurança dos workspaces

O System Core só deve realizar inspeções de projeto/arquivo dentro de workspaces explicitamente cadastrados.

Não permitir ao AI Core escolher caminhos arbitrários.

Ferramentas de workspace devem receber:

```text
workspaceId
```

e não um caminho arbitrário fornecido pelo LLM.

Exemplo correto:

```text
get_workspace_status(workspaceId)
```

Evitar:

```text
inspect_path("C:\\qualquer\\lugar")
```

---

# Inspeção de workspace

Nesta versão, a inspeção deve ser limitada.

Pode verificar:

* existência;
* disponibilidade;
* quantidade aproximada de arquivos quando necessário;
* presença de `.git`;
* estado do Git;
* informações básicas do diretório.

Não ler automaticamente o conteúdo de todos os arquivos.

Não indexar o disco.

Não criar ainda sistema de busca textual completo.

---

# Git Monitor

Para workspaces que sejam repositórios Git, mostrar:

```text
REPOSITÓRIO
GeneScope

BRANCH
main

STATUS
MODIFIED

ARQUIVOS ALTERADOS
4

ÚLTIMO COMMIT
há 3 dias
```

Obter pelo menos:

* branch atual;
* working tree limpa/suja;
* arquivos modificados;
* arquivos adicionados;
* arquivos removidos;
* arquivos não rastreados;
* último commit;
* data do último commit.

Se for simples e seguro, mostrar commits recentes.

---

# Git somente leitura

Permitido:

```text
status
log
branch atual
diff summary
```

Proibido:

```text
commit
push
pull
checkout
switch
merge
rebase
reset
clean
stash
add
restore
```

A v0.7 observa Git.

Não modifica Git.

---

# Evitar shell genérico

Se Git precisar ser consultado por processo externo, encapsular estritamente os comandos permitidos.

Preferir biblioteca apropriada quando isso for mais seguro e simples.

Se utilizar o executável `git`, construir internamente comandos fixos.

O LLM nunca deve fornecer argumentos arbitrários para shell.

---

# Persistência

Criar nova migration para workspaces.

Não modificar migrations já aplicadas.

Sugestão conceitual:

```text
workspaces
```

Campos:

```text
id
name
path
project_id
enabled
created_at
updated_at
```

Adicionar índices/constraints quando apropriado.

---

# Interface — System Core

Criar uma tela própria:

# Sistema

Preservar a identidade HUD.

Estrutura conceitual:

```text
┌────────────────────────────────────────────────────────────┐
│ SYSTEM CORE                                   STATUS ONLINE │
├──────────────────┬──────────────────┬──────────────────────┤
│ CPU              │ MEMORY           │ STORAGE              │
│                  │                  │                      │
│ 24%              │ 72%              │ C: 71%               │
├──────────────────┴──────────────────┴──────────────────────┤
│ NETWORK                                                   │
├────────────────────────────────────────────────────────────┤
│ PROCESSES                                                 │
├────────────────────────────────────────────────────────────┤
│ WORKSPACES                                                │
│                                                          │
│ Azriel        main       CLEAN                            │
│ GeneScope     main       MODIFIED                         │
│ Mendel Lab    main       CLEAN                            │
└────────────────────────────────────────────────────────────┘
```

Não transformar a tela em um clone do Task Manager.

Ela deve continuar parecendo parte do Azriel.

---

# Telemetria no Command Center

Integrar informações compactas ao Command Center.

Exemplo:

```text
SYSTEM CORE

CPU             24%
RAM             72%
DISCO           71%
REDE            ONLINE
OLLAMA          ONLINE
```

Esses valores devem ser reais.

Não duplicar lógica de coleta no frontend.

---

# Atualização em tempo real

CPU, memória e rede podem atualizar periodicamente.

Utilizar intervalo razoável.

Sugestão:

```text
2–5 segundos
```

Não é necessário atualizar dezenas de vezes por segundo.

Git e workspaces não precisam do mesmo intervalo.

Sugestão:

* telemetria: poucos segundos;
* workspaces/Git: sob demanda ou intervalo maior.

Evitar consumo desnecessário.

---

# AI Core

Expandir as tools somente leitura da v0.6.

Adicionar:

```text
get_system_status
get_cpu_usage
get_memory_usage
get_storage_status
get_network_status
list_processes
list_workspaces
get_workspace_status
get_git_status
get_git_recent_commits
get_ollama_status
```

Adaptar nomes conforme a arquitetura existente.

---

# Segurança das AI Tools

Todas continuam:

# READ ONLY

O LLM não recebe acesso a:

* shell;
* filesystem arbitrário;
* comandos;
* APIs de modificação.

Tools de workspace devem trabalhar com IDs previamente cadastrados.

---

# Consultas naturais

Azriel deve conseguir responder:

```text
Como está meu computador?
```

```text
Quanto de RAM estou usando?
```

```text
Como está minha CPU?
```

```text
Quanto espaço livre tenho?
```

```text
Quais processos estão consumindo mais memória?
```

```text
O Ollama está online?
```

```text
Quais workspaces estão cadastrados?
```

```text
Quais projetos Git possuem alterações?
```

```text
O GeneScope está limpo?
```

```text
Qual foi o último commit do Azriel?
```

As respostas devem utilizar dados reais.

---

# Evolução do comando "situação"

Atualizar:

```text
Azriel, situação.
```

O resumo agora pode combinar:

```text
Operações Diárias
+
Projetos
+
Knowledge Core
+
Formação
+
System Core
```

Não produzir relatório gigantesco.

Priorizar informações relevantes.

Exemplo conceitual:

```text
Você possui 3 tarefas para hoje e uma atividade atrasada.

Eletrônica permanece entre as principais lacunas.

CPU está em 21% e memória em 68%.

Ollama está online.

Há dois workspaces Git com alterações não commitadas.
```

Não usar exatamente esse texto hardcoded.

---

# Relação Workspace ↔ Project

Quando um workspace possuir `projectId`, permitir que o Azriel conecte informações.

Exemplo:

```text
Project
GeneScope

        ↕

Workspace
C:\Projetos\GeneScope

        ↕

Git
main / modified
```

Isso permitirá perguntas como:

```text
Como está o projeto GeneScope?
```

Azriel poderá combinar:

* Project Core;
* Daily Operations;
* Workspace;
* Git.

---

# Workspace Details

Ao abrir um workspace mostrar:

* nome;
* caminho;
* projeto relacionado;
* disponibilidade;
* Git;
* branch;
* alterações;
* último commit.

Não mostrar conteúdo completo dos arquivos nesta versão.

---

# Cadastro de workspace

A UI pode permitir adicionar e remover workspaces.

Essa operação é iniciada diretamente pelo operador através da interface.

Isso NÃO significa que o AI Core ganhou permissão de escrita.

O usuário pode administrar a configuração.

A IA apenas consulta workspaces já autorizados.

---

# Processos e privacidade

Não persistir lista completa de processos no SQLite.

Processos são telemetria transitória.

Não enviar automaticamente toda a lista de processos ao LLM.

Se o usuário perguntar:

```text
Quais processos estão usando mais RAM?
```

filtrar primeiro no System Core e enviar apenas os resultados relevantes.

---

# Dados sensíveis

Não incluir automaticamente em contexto de IA:

* nomes de usuário do sistema;
* caminhos pessoais desnecessários;
* variáveis de ambiente;
* tokens;
* chaves;
* credenciais;
* conteúdo de arquivos;
* argumentos potencialmente sensíveis de processos.

Minimizar dados enviados ao LLM, mesmo sendo local.

---

# Rust

Utilizar Rust/Tauri para capacidades nativas.

Manter comandos pequenos e específicos.

Exemplo conceitual:

```text
get_system_info
get_cpu_stats
get_memory_stats
get_storage_stats
get_process_summary
```

Evitar:

```text
execute_anything
```

---

# Dependências

Antes de adicionar bibliotecas:

1. verificar se já existe solução no projeto;
2. avaliar manutenção;
3. avaliar compatibilidade com Tauri;
4. evitar bibliotecas gigantes para tarefas simples.

Documentar dependências nativas adicionadas.

---

# Erros

Tratar:

* workspace inexistente;
* workspace removido externamente;
* Git não instalado;
* diretório não Git;
* Ollama offline;
* falha ao coletar telemetria;
* acesso negado;
* unidade indisponível.

Falha de um módulo não deve derrubar o System Core inteiro.

---

# Estados

Utilizar:

* loading;
* online;
* offline;
* unavailable;
* error.

Evitar apresentar `0` quando o dado na realidade não pôde ser obtido.

Diferenciar:

```text
CPU: 0%
```

de:

```text
CPU: INDISPONÍVEL
```

---

# Testes

Adicionar testes para lógica que puder ser isolada.

Prioridades:

1. formatação de memória;
2. formatação de armazenamento;
3. agregação de CPU;
4. ordenação de processos;
5. filtros de processos;
6. Workspace Registry;
7. workspace desabilitado;
8. workspace inexistente;
9. Git status parser;
10. Git clean;
11. Git modified;
12. relação Workspace ↔ Project;
13. AI tools somente leitura;
14. bloqueio de caminhos arbitrários;
15. resumo do sistema;
16. falha parcial de telemetria.

Não tornar testes dependentes de uma configuração específica do computador.

Utilizar mocks/fakes quando apropriado.

---

# Performance

Azriel deve poder permanecer aberto durante longos períodos.

Evitar:

* polling excessivo;
* vazamentos de memória;
* processos filhos abandonados;
* leitura repetitiva de disco;
* consultas Git excessivas;
* re-renderizações desnecessárias.

Monitorar apenas o necessário.

---

# Compatibilidade

O ambiente principal atual é Windows.

Projetar a arquitetura para permitir suporte futuro a Linux quando razoável.

Não prejudicar a implementação atual apenas para criar abstrações multiplataforma excessivas.

Se alguma funcionalidade for específica do Windows:

documentar.

---

# Não implementar nesta versão

Fora do escopo:

* execução arbitrária de comandos;
* shell para IA;
* abrir aplicações;
* fechar aplicações;
* matar processos;
* editar arquivos;
* excluir arquivos;
* criar arquivos por IA;
* Git commit;
* Git push;
* Git pull;
* Git checkout;
* automação;
* voz;
* MQTT;
* ESP32;
* IoT.

Tudo relacionado a ação fica para:

# v0.8 — Automation Core

---

# Documentação

Criar:

```text
docs/versions/v0.7.md
```

Documentar:

* objetivo;
* arquitetura;
* System Core;
* comandos Tauri;
* Workspace Registry;
* Git Monitor;
* telemetria;
* AI tools;
* segurança;
* privacidade;
* polling;
* dependências;
* limitações.

Atualizar:

```text
docs/roadmap.md
```

Marcar:

```text
v0.6 — concluída
v0.7 — System Core — em desenvolvimento
```

Depois da validação:

```text
v0.7 — concluída
```

---

# Critérios de aceite

A v0.7 só está concluída quando:

1. System Core existir;
2. informações reais do SO forem obtidas;
3. CPU real for exibida;
4. memória real for exibida;
5. armazenamento real for exibido;
6. estado básico de rede funcionar;
7. Process Monitor funcionar;
8. processos puderem ser ordenados;
9. processos puderem ser pesquisados;
10. nenhuma ação sobre processos existir;
11. status do Ollama for integrado;
12. Workspace Registry existir;
13. workspaces forem persistidos;
14. usuário puder cadastrar workspace pela UI;
15. usuário puder remover/desabilitar workspace;
16. workspace puder ser relacionado a projeto;
17. Git Monitor funcionar;
18. branch atual for exibida;
19. working tree clean/modified for detectada;
20. arquivos alterados forem identificados;
21. último commit puder ser consultado;
22. nenhuma operação Git de escrita existir;
23. Command Center mostrar telemetria real;
24. AI Core possuir tools do System Core;
25. AI tools permanecerem somente leitura;
26. caminhos arbitrários não puderem ser fornecidos pelo LLM;
27. "Como está meu computador?" funcionar;
28. perguntas sobre RAM/CPU/disco funcionarem;
29. perguntas sobre workspaces funcionarem;
30. perguntas sobre Git funcionarem;
31. "Azriel, situação" incorporar System Core;
32. falha parcial de telemetria for tratada;
33. Ollama offline não quebrar o sistema;
34. TypeScript não apresentar erros;
35. Rust compilar sem erros;
36. testes relevantes passarem;
37. build funcionar;
38. Tauri iniciar normalmente;
39. funcionalidades anteriores permanecerem funcionando;
40. `docs/versions/v0.7.md` existir.

---

# Teste final obrigatório

Com Azriel em execução:

Perguntar:

```text id="bbzq6k"
Azriel, como está meu computador?
```

Depois:

```text id="7xjkjp"
Quanto de RAM estou usando?
```

```text id="qk12fu"
Quais processos estão consumindo mais memória?
```

```text id="xgokmr"
O Ollama está online?
```

Cadastrar pelo menos um workspace Git através da interface.

Depois perguntar:

```text id="1m7tve"
Quais workspaces estão cadastrados?
```

```text id="d7i9pm"
Esse projeto possui alterações não commitadas?
```

```text id="7djbrt"
Qual foi o último commit?
```

Finalmente:

```text id="ft2h1j"
Azriel, situação.
```

Confirmar que a resposta utiliza informações reais de:

* Operações Diárias;
* Knowledge Core;
* Projetos;
* Formação;
* System Core.

Também testar:

* workspace inexistente;
* diretório sem Git;
* Ollama desligado;
* falha parcial de telemetria.

Nenhuma dessas situações deve derrubar o aplicativo.

---

# Resultado esperado

Ao concluir a v0.7, Azriel deve possuir consciência operacional básica do computador onde está executando.

Ele poderá responder:

* como está o sistema;
* quanto recurso está sendo utilizado;
* quais processos estão ativos;
* se Ollama está disponível;
* quais workspaces estão registrados;
* como estão os repositórios Git;
* quais projetos possuem alterações locais.

Mas continuará incapaz de modificar o computador por decisão própria.

A filosofia desta versão é:

# VISIBILIDADE ANTES DE AUTORIDADE.

Azriel primeiro aprende a observar corretamente.

Na próxima versão:

# v0.8 — Automation Core

começaremos a estudar como permitir que ele aja de maneira controlada, explícita e segura.

---

# Registro de implementação — 31 de agosto de 2026

Status: **concluída e validada em 31 de agosto de 2026**.

Entregue:

* telemetria nativa de sistema, CPU, memória, armazenamento, rede e uptime;
* monitor transitório de processos com pesquisa e ordenação;
* migration `0006_system_core.sql` e registro persistente de workspaces;
* seleção nativa de diretório, canonicalização e validação no backend Rust;
* ativação, desativação, edição, remoção e relação opcional com projetos;
* inspeção limitada da raiz do workspace;
* Git Monitor somente leitura com branch, estado, alterações e commits recentes;
* tratamento independente de pasta ausente, diretório sem Git e telemetria parcial;
* contexto compartilhado entre a página System e o Command Center;
* tools de sistema, processos, workspaces, Git e Ollama no AI Core;
* atualização da consulta `Azriel, situação` com o estado operacional;
* escala tipográfica global ampliada para melhorar a legibilidade;
* cabeçalho, menu lateral e barra de status fixos, com rolagem restrita à área de conteúdo;
* versão técnica atualizada para `0.7.0`.

Barreiras mantidas:

* nenhum encerramento ou controle de processos;
* nenhum comando de shell fornecido pela IA;
* nenhum caminho arbitrário aceito pelas tools;
* nenhuma escrita em arquivos ou repositórios Git;
* nenhum histórico de processos persistido;
* nenhuma varredura global do computador.

Validação automatizada:

* 41 testes TypeScript aprovados;
* lint aprovado sem erros;
* build frontend aprovado;
* 20 testes Rust aprovados, 1 smoke test do Ollama ignorado por depender de execução explícita;
* compilação Rust aprovada com `cargo check`;
* dependências Tauri alinhadas e auditadas sem vulnerabilidades conhecidas pelo `npm audit`;
* build Tauri de release aprovado;
* instaladores MSI e NSIS gerados para Windows x64.

Validação manual concluída pelo operador:

* apresentação visual e tipografia da página System aprovadas;
* navegação fixa durante a rolagem aprovada;
* cadastro de workspace pela interface confirmado;
* leitura do branch, das alterações e do último commit confirmada;
* telemetria e monitor de processos confirmados no aplicativo;
* integração do System Core com o Command Center e o AI Core aceita.

Encerramento:

* v0.7 — System Core: **concluída**;
* próxima etapa aprovada: **v0.7.1 — Autonomia dos Núcleos**;
* próxima versão principal planejada: **v0.8 — Automation Core**.

Artefatos da release:

* `src-tauri/target/release/bundle/msi/Azriel_0.7.0_x64_en-US.msi`;
* `src-tauri/target/release/bundle/nsis/Azriel_0.7.0_x64-setup.exe`.
