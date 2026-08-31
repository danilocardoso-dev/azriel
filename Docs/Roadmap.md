# AZRIEL — Roadmap

Este documento define a evolução planejada do Azriel até sua primeira versão estável.

O roadmap representa direção, não uma obrigação rígida.
Versões podem ser ajustadas conforme novas necessidades técnicas surgirem.

---

## Estado atual

Última versão concluída: **v0.4 — HUD Vivo**

Próxima versão planejada: **v0.5 — Knowledge Core**

Próximo objetivo:

Dar memória estrutural ao Azriel e substituir os dados mockados por
informações persistidas em SQLite.

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

Status: ⏳ Planejado

Objetivo:
Dar memória estrutural ao Azriel.

### Entregas

- SQLite
- banco de conhecimento
- banco de projetos
- estudos
- formação
- métricas
- histórico de evolução
- Mapa Stark baseado em dados reais

Resultado esperado:

O dashboard deixa de utilizar dados mockados.

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
