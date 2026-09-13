# AZRIEL — Roadmap

Este documento define a evolução planejada do Azriel até sua primeira versão estável.

O roadmap representa direção, não uma obrigação rígida.
Versões podem ser ajustadas conforme novas necessidades técnicas surgirem.

---

## Estado atual

Versão atual implementada: **v0.8.4 — Interactive Roadmap Experience**

Versão em desenvolvimento: **v0.8.4 — validação operacional pendente**

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

# v0.8.2 — Stark Knowledge System

Status: ✅ Concluída

Objetivo:
Consolidar conhecimento, roadmaps, pesquisa, evolução e lacunas dentro do Mapa Stark, preservando todos os dados existentes.

### Entregas

- navegação interna unificada com seis áreas
- baseline idempotente das métricas atuais
- hierarquia de conhecimento preparada
- roadmaps persistentes com etapas, tópicos e atividades
- progresso estrutural separado do nível de conhecimento
- pesquisa persistente e relacionada a conhecimento, roadmap e projeto
- Knowledge Events preparados sem automação
- Gap Diagnostics incorporado ao Mapa Stark
- consultas compatíveis no AI Core
- migration aditiva e teste sobre cópia do banco real

### Limites

- sem Learning Engine
- sem alteração automática das métricas
- sem geração automática de eventos ou roadmaps

---

# v0.8.3 — Learning Engine

Status: ✅ Concluída

Objetivo:
Transformar evidências reais de atividades em Knowledge Events auditáveis e, somente então, calcular evolução de conhecimento.

---

# v0.8.4 — Interactive Roadmap Experience

Status: 🧪 Implementada em 04 de setembro de 2026; aceite operacional pendente

Objetivo:
Transformar roadmaps extensos em uma experiência navegável, com posição atual, estrutura recolhível, inspetor de tópico e transições de atividade integradas ao Learning Engine.

### Entregas

- Roadmap Navigator com busca, filtros e seleção persistida localmente
- estrutura central recolhível por etapa, sem abrir centenas de atividades simultaneamente
- Topic Inspector com visão geral, domínio, progresso, relações, pré-requisitos e atividades
- início, conclusão e reabertura transacionais, com evidência e reversão auditáveis
- Continue Study determinístico, priorizando atividade em andamento e depois a primeira pendente
- feedback de impacto do Learning Engine sem recarregar a página
- consulta somente leitura da posição de estudo pelo AI Core
- layout responsivo para desktop amplo, intermediário e compacto

### Limites

- AI Core não conclui nem reabre atividades
- pré-requisitos informam pendências, mas não bloqueiam o estudo nesta versão
- atalho compacto no Command Center permanece opcional

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

---

# Trilha experimental — Market Lab

O Market Lab evolui fora do núcleo central e não altera a sequência oficial até a v1.0.

## Market Lab v0.1 — Deterministic Backtest Core

Status: ✅ Concluída e validada em 06/09/2026.

- importação e validação de candles históricos em CSV local;
- replay cronológico sem look-ahead;
- cinco estratégias determinísticas registradas dinamicamente;
- Risk Engine único, execução simulada e portfolios isolados;
- fees, slippage, auditoria, métricas, histórico e reexecução;
- nenhuma conexão com corretora, dinheiro real, AI Core ou internet.

## Market Lab v0.2 — Multi-Agent Experiment & Behavioral Observatory

Status: ✅ Concluída e validada em 08/09/2026.

- coortes determinísticas com 3 a 5 agentes sob o mesmo snapshot experimental;
- métricas comportamentais, episódios de posição e fórmulas versionadas;
- comparação sem pontuação única, com CASH e BUY & HOLD como benchmarks;
- correlação de retornos da equity e similaridade de decisões;
- observatório por agente com performance, comportamento, risco, decisões e posições;
- nenhuma conexão com corretora, dinheiro real, AI Core ou internet.

## Market Lab v0.3 — Scientific Validation & Market Regimes

Status: ✅ Concluída e validada em 08/09/2026.

- splits temporais explícitos e ordenados em In-Sample, Validation e Out-of-Sample;
- walk-forward sequencial sem otimização automática ou acesso a dados futuros;
- Sharpe, Sortino, Calmar e métricas rolling com fator de anualização auditável;
- regimes determinísticos de direção e volatilidade, com aviso de amostra insuficiente;
- comparação contra CASH e BUY & HOLD, dispersão entre janelas e alerta de possível overfitting;
- relatório conservador de robustez, sem promoção automática de estratégias;
- configurações, versões de fórmulas e identidade do dataset congeladas por Validation Run;
- nenhuma conexão com corretora, dinheiro real, AI Core ou internet.

## Market Lab v0.4 — AI Agent Integration

Status: 🧪 Implementada em 08/09/2026; validação operacional com Ollama pendente.

- primeiro agente LLM submetido ao mesmo replay, capital, risco e execução dos agentes determinísticos;
- provider abstrato com adaptador para Ollama local e fake provider para testes;
- snapshot compacto sem dados futuros e resposta JSON estritamente validada;
- timeout, retry limitado, fallback `HOLD` e distinção auditável de `NO_LLM_CALL`;
- configuração e prompt versionados e congelados por experimento e validação;
- métricas de chamadas, falhas, fallbacks e latência, com Decision Inspector;
- participação em Observatory, OOS, walk-forward e análise de regimes;
- limite de um AI Agent e cinco agentes totais, sem corretora ou dinheiro real.

## Market Lab v0.4.1 — AI Agent Calibration & Decision Diagnostics

Status: 🧪 Implementada em 08/09/2026; validação operacional com Ollama pendente.

- `AI Technical V2` preserva integralmente o agente V1 e recebe contexto técnico mais rico, sem dados futuros;
- snapshot auditável com retornos passados, tendência, momentum, volatilidade, regime, posição e decisão anterior;
- BUY, SELL, HOLD retornado pelo LLM e `NO_LLM_CALL` contabilizados separadamente;
- distribuição e faixas de confiança, inspeção do input exato e análise pós-decisão por horizontes 1, 5 e 10;
- comparação A/B entre experimentos V1 e V2, com aviso quando as condições congeladas diferirem;
- mesma proteção de schema, Risk Engine, execução simulada, portfólio, validação científica e auditoria da v0.4;
- suíte automatizada validada com provider falso; resposta real do modelo local permanece como aceite operacional.

## Market Lab v0.4.2 — Decision Architecture & Signal Scoring

Status: 🧪 Implementada em 08/09/2026; validação operacional com AAPL 2022/2023 e Ollama pendente.

- `MarketSignalEngine` determinístico e versionado separa evidência técnica da decisão do LLM;
- tendência e momentum produzem direção, enquanto volatilidade atua somente como contexto e redutor de força;
- `AI Technical V3` recebe scores pré-calculados, conflito e viés, mas continua livre para retornar BUY, SELL ou HOLD;
- respostas V3 usam `reason_code` controlado e exposição total desejada, mantendo schema estrito e fallback seguro;
- traces persistem features, scores, configuração, validação, ação e divergência entre sinal e decisão;
- diagnóstico inclui matrizes sinal × ação e conflito × ação, distribuição de motivos, response rate e alertas de colapso;
- comparação A/B/C preserva V1 e V2, exige condições experimentais equivalentes e não promove agente automaticamente;
- Risk Engine, execução simulada, limites de coorte, ausência de corretora e ausência de dinheiro real permanecem inalterados.

## Market Lab v0.4.3 — Position Management & Trade Lifecycle

Status: 🧪 Implementada em 08/09/2026; validação operacional com AAPL 2022/2023 e Ollama pendente.

- `PositionManager` e `PositionDecisionValidator` adicionam semântica explícita para FLAT, LONG_OPEN e LONG_REDUCED;
- `AI Technical V4` administra exposição total com ENTER, HOLD, INCREASE, REDUCE, EXIT e STAY_FLAT;
- ações incoerentes são convertidas em fallback seguro antes do Risk Engine, que permanece inalterado;
- ciclos completos, eventos, motivos, MFE, MAE, giveback, eficiência de saída, reentrada e churn são persistidos;
- execuções parciais permanecem separadas da contagem de lifecycles completos;
- o Observatory mostra status, tabela, inspetor e timeline de posição;
- V1, V2 e V3, Signal Engine V1, executor, portfólio e Scientific Validation permanecem preservados.

## Market Lab v0.4.4 — Structured Output Reliability & Lifecycle Audit

Status: 🧪 Implementada em 12/09/2026; validação operacional com AAPL 2022/2023 e Ollama pendente.

- `AI Technical V4.1` usa contrato JSON estrito e structured output por capability do provider;
- Normalizer, JSON Parser, Schema Validator, Semantic Validator e Position Validator geram estágios auditáveis;
- uma tentativa inicial e no máximo um retry corretivo preservam raw output e causa detalhada da falha;
- fallback é semântico e separado do LLM: STAY_FLAT quando FLAT e HOLD_POSITION na exposição atual quando LONG;
- métricas distinguem first-pass valid, retry recovered, final valid/invalid, fallback e latências P50/P95;
- Decision Inspector mostra tentativas e pipeline, sem solicitar ou armazenar chain-of-thought;
- Lifecycle Audit separa eventos, execuções e ciclos completos, com MFE, MAE, giveback e eficiência de saída;
- V1–V4, Signal Engine V1, Risk Engine, Portfolio Engine e lógica financeira permanecem preservados.

## Market Lab v0.4.4.1 — Contract Simplification & Position Context Fix

Status: ✅ Concluída e validada em 12/09/2026.

- `AI Technical V4.2` recebe posição explícita e decide apenas `ENTER`, `HOLD`, `REDUCE` ou `EXIT`;
- `confidence_pct` usa a faixa 0–100 e é normalizada internamente, eliminando a ambiguidade observada na V4.1;
- `POSITION_SIZING_V1` converte intent em target determinístico antes do Risk Engine;
- estado de posição inconsistente gera `POSITION_CONTEXT_ERROR` e não chama o LLM;
- Inspector e comparação A/B mostram contexto, intent, target gerado, confiabilidade, fallback e latência;
- V1–V4.1, Signal Engine V1, Risk Engine, execução, portfólio e validação científica permanecem preservados.
- aceite operacional AAPL 2022 e AAPL 2023: final valid rate de 100%, fallback de 0% e timeout de 0 em ambos.

## Market Lab v0.4.4.2 — Risk-Reducing Actions Policy

Status: 🧪 Implementada em 12/09/2026; aceite operacional AAPL 2022/2023 pendente.

- `RISK_POLICY_V2` classifica cada target por exposição atual versus exposição alvo, com tolerância numérica;
- `MAX_OPERATIONS`, perda diária e drawdown continuam bloqueando novo risco, mas não aprisionam posições ao rejeitar redução ou saída;
- posição máxima e exposição máxima limitam apenas ações que aumentam risco;
- Kill Switch preserva a política explícita `BLOCK_ALL`, inclusive para redução e saída;
- cada regra persiste `PASSED`, `REJECTED` ou `NOT_APPLICABLE`, com motivo e ordem auditáveis;
- Decision Inspector mostra classificação, exposições, delta, matriz aplicada, resultado final e motivo exato;
- comparação A/B inclui classes de risco, rejeições por operações, regras não aplicáveis, reduções e saídas executadas;
- V4.2, Position Sizing V1, Signal Engine V1, Execution Simulator, Portfolio Engine e Scientific Validation permanecem preservados.

## Market Lab v0.5 — Intraday Trading Foundation

Status: 🧪 Implementada em 12/09/2026; validação operacional AAPL 15M/Ollama pendente.

- o mesmo Market Lab aceita datasets `1D` e `15M` com metadados explícitos de mercado, timezone e sessão;
- timestamps intraday são normalizados para UTC e associados a sessões configuradas;
- gaps entre sessões são distinguidos de gaps intraday inesperados;
- annualization é derivada do timeframe: 252 em `1D` e 6552 em `15M`;
- `EXECUTION_MODEL_V1` preserva decisão no fechamento T e execução na abertura T+1;
- `MARKET_DECISION_TRIGGER_V1` adiciona gatilhos, cooldown, compute budget e auditoria por candle;
- perda diária usa sessão de mercado, mantendo as regras de `RISK_POLICY_V2`;
- lifecycles registram sessões, minutos de mercado e overnight;
- Overview exibe métricas por sessão e Trigger Inspector paginado;
- V1–V4.2, Signal Engine V1, Position Sizing V1, Risk Policy V2 e Scientific Validation permanecem preservados.
