# AZRIEL — Roadmap

Este documento define a evolução planejada do Azriel até sua primeira versão estável.

O roadmap representa direção, não uma obrigação rígida.
Versões podem ser ajustadas conforme novas necessidades técnicas surgirem.

---

## Estado atual

Versão atual concluída: **v0.8.1 — Automation Core / Rotinas**

Próxima versão principal planejada: **v0.9 — IoT Core**

---

# v0.4 — HUD Vivo

Status: ✅ Concluída em 31 de agosto de 2026

Objetivo:
Criar a primeira interface funcional do Azriel.

### Entregas

- Tauri 2
- React
- TypeScript
- interface desktop
- Command Center
- Azriel Core
- Mapa Stark interativo
- módulo de projetos
- módulo de conhecimento
- módulo de formação
- diagnóstico de lacunas
- navegação entre módulos

### Ainda não entra

- SQLite
- IA
- Ollama
- automações
- IoT

---

# v0.5 — Knowledge Core

Status: ✅ Concluída em 31 de agosto de 2026

Objetivo:
Dar memória estrutural ao Azriel.

### Entregas

- SQLite local com migrations versionadas — implementado
- banco de conhecimento — implementado
- banco de projetos e relações N:N — implementado
- formação — implementado
- métricas e atualização transacional — implementado
- histórico de evolução — implementado
- Mapa Stark baseado em dados reais — implementado
- confirmação manual de persistência após fechar e reabrir — aprovada

Resultado esperado:

O dashboard deixa de utilizar dados mockados.

---

# v0.5.1 — Operações Diárias

Status: ✅ Concluída em 31 de agosto de 2026

Objetivo:
Transformar o Azriel em uma central operacional de uso cotidiano antes da
introdução da AI Core.

### Entregas

- tarefas persistentes
- notas rápidas persistentes
- prioridades e prazos
- visões Hoje, Caixa de Entrada, Próximas, Concluídas e Notas
- vínculos com projetos e conhecimentos
- integração compacta com o Command Center
- modal visual próprio para exclusões definitivas
- validação funcional e de persistência aprovada

Resultado esperado:

O operador consegue registrar e organizar o que precisa fazer e lembrar sem
depender de um aplicativo externo de notas rápidas.

---

# v0.6 — AI Core

Status: ✅ Concluída em 31 de agosto de 2026

Objetivo:
Permitir que Azriel compreenda e consulte seus próprios dados.

### Entregas

- Ollama local e providers desacoplados
- AI Core com perguntas gerais e consultas internas separadas
- tools somente leitura
- contexto controlado
- acesso às Operações Diárias, Knowledge Core e Project Core
- histórico persistente e exclusão de conversas
- proteção contra repetição e respostas truncadas
- modelos `qwen2.5:0.5b` e `qwen2.5:3b` validados

Exemplo:

"Azriel, qual minha maior lacuna?"

---

# v0.7 — System Core

Status: ✅ Concluída em 31 de agosto de 2026

Objetivo:
Conectar Azriel ao computador.

### Entregas

- CPU, memória, armazenamento, rede e uptime nativos
- monitor de processos somente leitura
- workspaces autorizados persistidos no SQLite
- relação opcional entre workspace e projeto
- Git Monitor somente leitura
- telemetria real no Command Center
- ferramentas do System Core no AI Core

Validação automatizada, visual e operacional concluída.

---

# v0.7.1 — Autonomia dos Núcleos

Status: ✅ Concluída em 31 de agosto de 2026

Objetivo:
Permitir que o operador crie, edite, organize e remova seus próprios dados nas
telas que já possuem persistência, sem antecipar as ações de sistema da v0.8.

### Entregas

- CRUD completo de projetos
- CRUD completo de áreas de conhecimento
- CRUD completo de formação
- consulta e restauração de notas arquivadas
- editor de conhecimento compartilhado com o Mapa Stark
- atalhos operacionais no Command Center
- confirmações de exclusão específicas e tratamento consistente de erros

Validação automatizada, visual e operacional concluída.

### Limites

- Research Core permanece sem CRUD até possuir persistência própria
- AI Core permanece somente leitura sobre os dados dos núcleos
- processos, arquivos e Git permanecem sem ações destrutivas
- Automation Core continua reservada para a v0.8

---

# v0.8.0 — Automation Core / Safe Actions

Status: ✅ Concluída em 31 de agosto de 2026

Objetivo:
Permitir que Azriel execute ações locais de baixo risco, somente sobre recursos previamente registrados e autorizados.

### Entregas

- Application Registry e URL Registry persistentes
- Action Registry com cinco ações específicas
- Policy Engine independente do LLM
- Confirmation Gate preparado para a v0.8.1
- abrir aplicativo, workspace e projeto autorizados
- revelar workspace no Explorador
- abrir URL registrada
- auditoria completa no Action History
- integração controlada com o AI Core

### Limites

- nenhum shell, PowerShell ou cmd genérico
- nenhuma edição, movimentação ou exclusão de arquivos
- nenhuma ação destrutiva em processos ou Git
- paths, URLs e executáveis nunca são escolhidos pelo LLM

Exemplo:

"Azriel, abra o GeneScope."

Validação automatizada, visual e operacional concluída.

---

# v0.8.1 — Automation Core / Rotinas

Status: ✅ Concluída em 31 de agosto de 2026

Objetivo:
Transformar ações autorizadas em sequências reutilizáveis, previsíveis e auditadas.

### Entregas

- Routine Registry e Routine Steps persistentes
- editor visual para criar, editar, reordenar, ativar e excluir rotinas
- validação integral antes do primeiro passo
- execução sequencial com intervalo limitado
- Confirmation Gate global para UI e AI Core
- confirmação assíncrona sem bloqueio da interface e recuperação de confirmações abandonadas
- proteção de inicialização entre aplicativo e workspace vinculado
- política stop on error sem rollback
- histórico de rotinas e vínculo com o Action History
- tools `list_routines` e `run_routine` baseadas somente em IDs
- resumo de rotinas no Command Center
- estado visual `EXECUTANDO ROTINA` no AzrielCore

### Limites

- nenhum comando, caminho ou URL arbitrária
- nenhuma elevação sobre as cinco ações seguras da v0.8.0
- cancelamento disponível enquanto aguarda confirmação; execução iniciada não possui cancelamento intermediário
- sem rollback, scheduler, recorrência ou gatilhos automáticos

Validação automatizada, empacotamento e aceite operacional concluídos.

---

# v0.9 — IoT Core

Status: ⏳ Planejado

Objetivo:
Conectar Azriel ao mundo físico.

### Entregas

- MQTT
- ESP32
- sensores
- dispositivos
- telemetria
- automação física

Arquitetura:

Azriel
  ↓
IoT Core
  ↓
MQTT
  ↓
ESP32
  ↓
Sensores / Atuadores

---

# v1.0 — Personal Intelligence System

Status: 🎯 Objetivo

Azriel deverá integrar:

- conhecimento
- projetos
- memória
- inteligência artificial
- computador
- automação
- IoT

Objetivo da v1.0:

Azriel conhece meus projetos, estudos e conhecimentos,
identifica lacunas, acompanha minha evolução,
conversa sobre essas informações e executa ações
autorizadas no computador e em dispositivos físicos.

---

# Depois da v1.0

As próximas versões serão definidas pelos problemas reais encontrados
durante o desenvolvimento.

Não adicionar funcionalidades apenas porque são tecnicamente possíveis.

Prioridade:

**utilidade → integração → confiabilidade → evolução.**
