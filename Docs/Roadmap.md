# AZRIEL — Roadmap

Este documento define a evolução planejada do Azriel até sua primeira versão estável.

O roadmap representa direção, não uma obrigação rígida.
Versões podem ser ajustadas conforme novas necessidades técnicas surgirem.

---

## Estado atual

Versão concluída mais recente: **v0.5.1 — Operações Diárias**

Próxima versão principal planejada: **v0.6 — AI Core**

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

Status: ⏳ Planejado

Objetivo:
Permitir que Azriel compreenda e consulte seus próprios dados.

### Entregas

- Ollama
- IA local
- Assistant
- memória
- contexto
- acesso ao Knowledge Core
- acesso ao Project Core

Exemplo:

"Azriel, qual minha maior lacuna?"

---

# v0.7 — System Core

Status: ⏳ Planejado

Objetivo:
Conectar Azriel ao computador.

### Entregas

- CPU
- RAM
- armazenamento
- processos
- arquivos
- projetos locais
- Git
- telemetria

---

# v0.8 — Automation Core

Status: ⏳ Planejado

Objetivo:
Permitir que Azriel execute ações.

### Entregas

- abrir programas
- abrir projetos
- executar comandos autorizados
- preparar ambientes
- executar rotinas
- automações pessoais

Exemplo:

"Azriel, abra o GeneScope."

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
