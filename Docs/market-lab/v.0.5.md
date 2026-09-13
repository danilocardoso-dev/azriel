# AZRIEL MARKET LAB v0.5 — Intraday Trading Foundation

## CONTEXTO

O primeiro grande bloco do Market Lab foi concluído.

Estado atual:

```text
v0.1     Deterministic Backtest Core             COMPLETED
v0.2     Multi-Agent Behavioral Observatory      COMPLETED
v0.3     Scientific Validation & Market Regimes  COMPLETED

v0.4     AI Agent Integration                    COMPLETED
v0.4.1   Decision Diagnostics                    COMPLETED
v0.4.2   Signal Scoring                          COMPLETED
v0.4.3   Position Lifecycle                      COMPLETED
v0.4.4.1 Intent Contract                         COMPLETED
v0.4.4.2 Risk-Reducing Actions Policy            COMPLETED
```

A arquitetura atual já possui:

* Market Replay;
* Feature Engine;
* Signal Engine;
* agentes determinísticos;
* AI Technical V4.2;
* Position Manager;
* Position Sizing;
* Risk Engine;
* Execution Simulator;
* Portfolio Engine;
* Scientific Validation;
* Structured Output;
* Risk Decision Trace;
* métricas comportamentais;
* auditoria;
* comparação entre agentes.

Agora implemente:

# Market Lab v0.5 — Intraday Trading Foundation

---

# OBJETIVO

Adicionar suporte arquitetural real a trading intraday.

Primeiro timeframe validado:

```text
15 MINUTES
```

A v0.5 deve permitir executar experimentos históricos com candles de 15 minutos sem assumir que cada candle representa um dia.

O sistema deve permanecer preparado para suportar futuramente:

```text
1H
30M
15M
5M
1M
```

mas nesta versão validar oficialmente apenas:

```text
1D
15M
```

---

# PRINCÍPIO CENTRAL

Não criar um segundo Market Lab separado para intraday.

A mesma arquitetura deve funcionar para:

```text
1D
15M
```

O timeframe deve ser uma propriedade do dataset e do experimento.

---

# NÃO ALTERAR

Preservar:

```text
SIGNAL_ENGINE_V1
SIGNAL_CONFIG_V1

MARKET_AI_AGENT_V4_2
POSITION_SIZING_V1
RISK_POLICY_V2

Portfolio Engine
Execution Simulator
Scientific Validation
```

Não otimizar estratégia nesta versão.

O objetivo é infraestrutura intraday.

---

# PARTE 1 — TIMEFRAME MODEL

Criar representação explícita de timeframe.

Exemplo:

```ts
type MarketTimeframe =
  | "1D"
  | "1H"
  | "30M"
  | "15M"
  | "5M"
  | "1M";
```

Adaptar ao padrão existente do projeto.

---

# TIMEFRAME METADATA

Todo dataset deve persistir:

```text
asset
market
timeframe
timezone
session_type
```

Exemplo:

```json
{
  "asset": "AAPL",
  "market": "NASDAQ",
  "timeframe": "15M",
  "timezone": "America/New_York",
  "session_type": "REGULAR"
}
```

---

# TIMEZONE

Não tratar timestamps intraday como datas simples.

Preferir armazenamento interno em:

```text
UTC
```

com metadata da timezone original.

Nunca depender da timezone local da máquina.

---

# UI

Mostrar claramente:

```text
TIMEFRAME
15M

MARKET TIMEZONE
America/New_York
```

---

# PARTE 2 — INTRADAY DATASET VALIDATION

Expandir Dataset Validator.

Campos mínimos continuam:

```text
timestamp
open
high
low
close
volume
```

Para intraday validar também:

```text
timeframe continuity
duplicate timestamps
out-of-order timestamps
invalid OHLC
invalid volume
unexpected intraday gaps
session boundaries
```

---

# GAPS

Não assumir que qualquer gap é erro.

Exemplos normais:

```text
overnight
weekend
holiday
```

Criar classificação:

```text
EXPECTED_SESSION_GAP
UNEXPECTED_INTRADAY_GAP
```

---

# SESSION MODEL

Criar abstração:

```text
MarketSession
```

Para ações americanas inicialmente:

```text
REGULAR SESSION
09:30 → 16:00
America/New_York
```

Não hardcode isso profundamente.

Usar configuração.

---

# SESSION ID

Cada candle deve poder ser associado a:

```text
session_id
```

Exemplo:

```text
2026-09-12
```

---

# SESSION STATE

Suportar conceitualmente:

```text
SESSION_OPEN
SESSION_ACTIVE
SESSION_CLOSE
SESSION_CLOSED
```

---

# PARTE 3 — MARKET REPLAY INTRADAY

Market Replay deve continuar estritamente sequencial.

Exemplo:

```text
09:30
↓
09:45
↓
10:00
↓
10:15
↓
...
```

Nunca disponibilizar candle futuro.

---

# NO LOOK-AHEAD

Regra crítica:

features no candle T só podem usar dados até T.

Nenhum:

```text
T+1
T+5
T+10
```

pode entrar no input do agente.

---

# PARTE 4 — FEATURE ENGINE TIMEFRAME-AWARE

Auditar todas as features atuais.

Nenhuma deve assumir:

```text
1 candle = 1 day
```

Exemplo:

```text
return_5
```

deve significar:

```text
5 candles
```

Em 15m:

```text
5 candles = 75 minutos de mercado
```

---

# NAMING

Preferir nomenclatura explícita internamente:

```text
return_1_candle
return_5_candles
return_10_candles
```

Se necessário, manter aliases para compatibilidade.

---

# SMA / EMA

Mesma regra.

```text
SMA20
```

significa:

```text
20 candles
```

não 20 dias.

---

# MOMENTUM

Momentum também deve ser baseado em candles.

---

# VOLATILITY

Volatilidade rolling deve usar períodos do timeframe atual.

Não aplicar automaticamente:

```text
252
```

para annualization intraday.

---

# PARTE 5 — ANNUALIZATION CONTEXT

Criar:

```text
AnnualizationContext
```

Para `1D`:

```text
~252 períodos/ano
```

Para `15M` em ações americanas:

```text
26 candles/sessão
×
~252 sessões/ano
```

Não espalhar esse cálculo por componentes diferentes.

---

# SCIENTIFIC METRICS

Sharpe, Sortino e métricas annualizadas devem usar o fator correto.

Persistir no experimento:

```text
annualization_factor
```

---

# PARTE 6 — EXECUTION TIMING

Auditar o modelo atual de execução.

Para intraday precisamos de regra explícita.

Preferência:

```text
CANDLE T CLOSE
↓
FEATURES
↓
SIGNAL
↓
DECISION
↓
ORDER
↓
NEXT CANDLE OPEN
```

Ou seja:

```text
decisão usa close de T
execução ocorre no open de T+1
```

---

# IMPORTANTE

Não executar decisão baseada no close de T usando um preço que já pressupõe conhecimento futuro.

---

# EXECUTION MODEL VERSION

Criar:

```text
EXECUTION_MODEL_V1
```

Persistir em metadata.

---

# PARTE 7 — DECISION TRIGGER ENGINE

Criar:

# MarketDecisionTriggerEngine

Objetivo:

evitar chamar o LLM a cada candle.

Fluxo:

```text
CANDLE
↓
FEATURE ENGINE
↓
SIGNAL ENGINE
↓
DECISION TRIGGER ENGINE
      │
      ├── NO TRIGGER
      │      ↓
      │   NO_LLM_CALL
      │
      └── TRIGGER
             ↓
            LLM
```

---

# RESPONSABILIDADE

O Trigger Engine NÃO decide:

```text
BUY
SELL
HOLD
```

Ele decide apenas:

```text
SHOULD_AI_EVALUATE_NOW?
```

---

# OUTPUT

Criar estrutura equivalente:

```ts
interface DecisionTriggerResult {
  shouldEvaluate: boolean;
  reasons: string[];
  triggerStrength?: number;
}
```

---

# PARTE 8 — TRIGGERS V1

Implementar triggers simples e determinísticos.

## 1. BIAS CHANGE

Exemplos:

```text
NEUTRAL → BULLISH
BULLISH → BEARISH
BEARISH → BULLISH
```

---

## 2. SIGNAL STRENGTH CHANGE

Exemplo:

```text
WEAK → STRONG
```

---

## 3. REGIME CHANGE

Exemplo:

```text
SIDEWAYS → BULL
BULL → BEAR
```

---

## 4. CONFLICT RESOLUTION

Exemplo:

```text
HIGH CONFLICT
↓
LOW CONFLICT
```

com sinal direcional relevante.

---

## 5. POSITION EVENT

Se posição aberta e sinais mudarem de forma relevante:

```text
trigger
```

mesmo que bias geral ainda não tenha mudado.

---

## 6. PERIODIC EVALUATION

Mesmo sem evento:

permitir chamada periódica.

Default inicial:

```text
every 8 candles
```

Em 15m:

```text
8 candles = 2 horas
```

---

# PARTE 9 — TRIGGER COOLDOWN

Criar:

```text
minCandlesBetweenAICalls
```

Default:

```text
2
```

Evitar chamadas repetidas em candles consecutivos.

---

# EXCEÇÃO

Evento crítico de posição pode ignorar cooldown se arquitetura permitir.

Não criar complexidade excessiva nesta versão.

---

# TRIGGER REASON ENUM

Criar:

```text
BIAS_CHANGE
SIGNAL_STRENGTH_CHANGE
REGIME_CHANGE
CONFLICT_RESOLVED
POSITION_RISK_CHANGE
PERIODIC_EVALUATION
SESSION_OPEN_CHECK
SESSION_CLOSE_CHECK
```

Implementar apenas os necessários inicialmente.

---

# PARTE 10 — TRIGGER AUDIT

Para cada candle registrar:

```text
timestamp
triggered
trigger_reasons
signal_before
signal_after
regime_before
regime_after
AI_called
```

---

# NO LLM CALL REASONS

Separar:

```text
NO_LLM_CALL_NO_TRIGGER
NO_LLM_CALL_COOLDOWN
NO_LLM_CALL_COMPUTE_BUDGET
NO_LLM_CALL_CADENCE
```

quando aplicável.

---

# PARTE 11 — TRIGGER METRICS

Adicionar:

```text
TOTAL CANDLES
TRIGGER EVENTS
AI CALLS
AI CALL RATE

BIAS CHANGE TRIGGERS
REGIME CHANGE TRIGGERS
POSITION TRIGGERS
PERIODIC TRIGGERS

COOLDOWN SUPPRESSED
```

---

# EXEMPLO

```text
TOTAL CANDLES
520

TRIGGERS
84

AI CALLS
61

CALL RATE
11.7%
```

---

# PARTE 12 — AI COMPUTE BUDGET

Criar:

```text
AIComputeBudget
```

Exemplo:

```ts
interface AIComputeBudget {
  maxCallsPerExperiment?: number;
  maxCallsPerSession?: number;
}
```

---

# QUANDO BUDGET ACABA

Não criar ordem.

Registrar:

```text
NO_LLM_CALL_COMPUTE_BUDGET
```

Position Manager mantém posição atual.

Risk Engine continua operando em regras determinísticas.

---

# PARTE 13 — SESSION METRICS

Adicionar métricas por sessão:

```text
session_return
session_drawdown

entries
exits
executions

time_in_market

avg_exposure
max_exposure

AI_calls
trigger_events
```

---

# DAILY LOSS

`max daily loss` deve usar sessão de mercado.

Não agrupar por 24h arbitrárias.

---

# PARTE 14 — HOLDING TIME

Além de:

```text
holding_candles
```

calcular:

```text
holding_market_minutes
```

---

# EXEMPLO

```text
6 candles
15m

= 90 minutos
```

---

# OVERNIGHT

Se posição atravessar sessão:

```text
overnight = true
```

Não contar horas com mercado fechado como holding market minutes.

---

# PARTE 15 — TRADE LIFECYCLE INTRADAY

Trade Lifecycle deve registrar:

```text
entry_timestamp
exit_timestamp

entry_session
exit_session

holding_candles
holding_market_minutes

overnight
```

---

# PARTE 16 — SESSION OPEN / CLOSE

Adicionar eventos:

```text
SESSION_OPEN
SESSION_CLOSE
```

Inicialmente apenas informativos.

Não fechar posição automaticamente no fim do dia.

---

# IMPORTANTE

Não implementar ainda:

```text
force close at session end
```

Isso pode ser política futura de agentes day-trade-only.

---

# PARTE 17 — INTRADAY UI

Adicionar claramente:

```text
TIMEFRAME
15M
```

na tela do experimento.

---

# MARKET CLOCK

Mostrar durante replay:

```text
SESSION
2026-09-12

MARKET TIME
11:45 ET

SESSION STATE
ACTIVE
```

quando aplicável.

---

# INTRADAY OBSERVATORY

Adicionar:

```text
AI CALL RATE
TRIGGER EVENTS
AVG HOLDING MINUTES
OVERNIGHT POSITIONS
SESSION P&L
```

---

# TRIGGER INSPECTOR

Ao selecionar candle:

```text
SIGNAL BEFORE
NEUTRAL

SIGNAL AFTER
BULLISH

TRIGGER
YES

REASON
BIAS_CHANGE

AI CALLED
YES
```

---

# PARTE 18 — DATASET SIZE PROTECTION

Intraday datasets podem ser muito maiores.

Não renderizar tudo de uma vez.

---

# TABLES

Usar:

```text
pagination
```

ou:

```text
virtualization/windowing
```

para listas grandes.

---

# CHARTS

Não desenhar dezenas de milhares de pontos sem necessidade.

Usar:

```text
downsampling
```

apenas na visualização.

Nunca reduzir dados usados pelo backtest.

---

# PARTE 19 — MEMORY / PERFORMANCE

Evitar manter cópias redundantes do dataset em memória.

Preferir:

```text
single normalized dataset representation
```

---

# IMPORTANTE

Seu hardware atual deve continuar sendo considerado ambiente limitado.

Não carregar:

```text
500k candles
×
5 agents
×
full duplicated state
```

sem necessidade.

---

# PARTE 20 — FIRST INTRADAY DATASET

Usar inicialmente:

```text
AAPL
15M
5 TRADING SESSIONS
```

Aproximadamente:

```text
130 candles
```

---

# PRIMEIRO TESTE

Executar apenas:

```text
Cash Control
Buy & Hold
Simple Trend
```

Sem LLM.

Objetivo:

validar infraestrutura intraday.

---

# CRITÉRIOS DO PRIMEIRO TESTE

Confirmar:

```text
timestamp ordering
session boundaries
feature calculation
signal engine
execution timing
portfolio
fees
slippage
equity curve
```

---

# PARTE 21 — SECOND INTRADAY TEST

Depois:

```text
AAPL
15M
~20 trading sessions
```

Aproximadamente:

```text
520 candles
```

---

# AINDA SEM LLM

Executar agentes determinísticos primeiro.

---

# PARTE 22 — AI INTRADAY TEST

Somente depois de validar infraestrutura:

Adicionar:

```text
AI Technical V4.2
```

com:

```text
MarketDecisionTriggerEngine
```

---

# META

Evitar:

```text
520 candles
→ 520 LLM calls
```

Desejado:

```text
520 candles
→ talvez 40–100 calls
```

dependendo dos triggers.

Não usar número fixo como critério formal.

---

# PARTE 23 — AI RUNTIME INTRADAY

Adicionar:

```text
CANDLES
TRIGGERS
CALLS
CALL RATE

AVG LATENCY
P50
P95
MAX

SKIPPED NO TRIGGER
SKIPPED COOLDOWN
SKIPPED BUDGET
```

---

# PARTE 24 — SIGNAL ENGINE

Não alterar Signal Engine nesta versão.

Mas garantir compatibilidade com 15M.

---

# IMPORTANTE

Se Signal Engine produzir comportamento ruim em intraday:

registrar.

Não otimizar ainda.

Isso pertence a versão futura.

---

# PARTE 25 — SIMPLE TREND

Simple Trend deve funcionar no timeframe atual.

Não criar versão exclusiva de intraday.

---

# PARTE 26 — BENCHMARKS

Manter:

```text
Cash Control
Buy & Hold
```

Mesmo em intraday.

Buy & Hold compra e mantém durante o período do dataset.

---

# PARTE 27 — SCIENTIFIC VALIDATION

Scientific Validation deve continuar funcionando em 15M.

Auditar:

```text
temporal split
walk-forward
OOS
regime analysis
```

---

# IMPORTANTE

Splits devem respeitar sequência temporal.

Nunca shuffle.

---

# PARTE 28 — WALK-FORWARD

Janelas passam a ser expressas em:

```text
candles
```

ou duração de mercado bem definida.

---

# PARTE 29 — MARKET REGIME

Market Regime Engine deve aceitar 15M.

Não assumir horizonte diário.

---

# PARTE 30 — TESTES AUTOMATIZADOS

Adicionar testes para:

1. parse 1D;
2. parse 15M;
3. timeframe metadata;
4. timezone;
5. duplicate timestamp;
6. out-of-order timestamp;
7. invalid OHLC;
8. expected overnight gap;
9. unexpected intraday gap;
10. session open;
11. session close;
12. session id;
13. 15m continuity;
14. Market Replay ordering;
15. no future candle;
16. return_1 = 1 candle;
17. return_5 = 5 candles;
18. SMA candle semantics;
19. momentum candle semantics;
20. volatility timeframe-aware;
21. annualization 1D;
22. annualization 15M;
23. Signal Engine 15M;
24. Simple Trend 15M;
25. execution at next candle;
26. no look-ahead execution;
27. bias change trigger;
28. strength trigger;
29. regime trigger;
30. periodic trigger;
31. position trigger;
32. cooldown;
33. trigger audit;
34. no-trigger audit;
35. AI call rate;
36. compute budget;
37. max calls per session;
38. max calls per experiment;
39. session metrics;
40. session return;
41. session drawdown;
42. holding minutes;
43. overnight detection;
44. lifecycle intraday;
45. pagination/windowing;
46. chart downsampling does not alter data;
47. 1D regression;
48. Scientific Validation 15M;
49. Walk-Forward 15M;
50. Risk Engine unchanged.

---

# TESTE SESSION GAP

Último candle:

```text
2026-09-12 15:45 ET
```

Próximo:

```text
2026-09-15 09:30 ET
```

Esperado:

```text
EXPECTED_SESSION_GAP
```

---

# TESTE INTRADAY GAP

Exemplo:

```text
10:00
10:15
10:45
```

Esperado:

```text
missing 10:30
↓
UNEXPECTED_INTRADAY_GAP
```

---

# TESTE RETURN

Candles:

```text
100
101
102
103
104
105
```

`return_5` no último:

```text
105 / 100 - 1
```

Não usar dias.

---

# TESTE EXECUTION

Candle:

```text
10:00–10:15
close = 100
```

Agente decide no fechamento.

Próximo candle:

```text
10:15–10:30
open = 101
```

Esperado:

```text
execution base price = 101
```

antes de fee/slippage conforme engine.

---

# TESTE TRIGGER

Signal:

```text
NEUTRAL
↓
BULLISH
```

Esperado:

```text
BIAS_CHANGE
shouldEvaluate = true
```

---

# TESTE NO TRIGGER

Signal permanece:

```text
BULLISH
```

sem mudança relevante.

Esperado:

```text
shouldEvaluate = false
```

salvo periodic evaluation.

---

# TESTE COOLDOWN

Call no candle:

```text
100
```

novo trigger em:

```text
101
```

com cooldown = 2.

Esperado:

```text
NO_LLM_CALL_COOLDOWN
```

salvo exceção crítica explicitamente configurada.

---

# TESTE PERIODIC

Sem triggers durante 8 candles.

Esperado:

```text
PERIODIC_EVALUATION
```

---

# TESTE COMPUTE BUDGET

Config:

```text
maxCallsPerSession = 5
```

Após 5 chamadas:

```text
trigger ocorre
```

Esperado:

```text
NO_LLM_CALL_COMPUTE_BUDGET
```

---

# TESTE HOLDING MINUTES

Entrada:

```text
10:00
```

Saída:

```text
11:30
```

15M timeframe.

Esperado:

```text
90 market minutes
```

conforme semântica documentada.

---

# TESTE OVERNIGHT

Entry:

```text
15:30
```

Exit:

```text
next session 10:00
```

Esperado:

```text
overnight = true
```

Holding market minutes deve contar somente candles de mercado.

---

# PARTE 31 — REGRESSION TEST 1D

Rodar:

```text
AAPL 1D 2022
AAPL 1D 2023
```

Não exigir mesmos resultados byte-for-byte se houver migration interna irrelevante.

Mas comportamento geral e compatibilidade devem permanecer.

---

# PARTE 32 — DOCUMENTAÇÃO

Criar:

```text
docs/market-lab/v0.5.md
```

Documentar:

* objetivo;
* timeframe model;
* timezone;
* Market Session;
* intraday gaps;
* annualization;
* execution timing;
* Decision Trigger Engine;
* trigger reasons;
* cooldown;
* compute budget;
* session metrics;
* holding minutes;
* overnight positions;
* dataset sizing;
* performance considerations;
* limitações.

Atualizar roadmap:

```text
v0.4.x — AI Agent Foundation — COMPLETED
v0.5   — Intraday Trading Foundation
```

---

# PARTE 33 — NÃO IMPLEMENTAR

Fora do escopo:

```text
1M production
5M production
order book
tick data
websocket live data
exchange integration
broker integration
paper trading
real money
short selling
leverage
news
sentiment
multi-asset portfolio
multi-timeframe AI reasoning
reinforcement learning
```

---

# PARTE 34 — CRITÉRIOS DE ACEITE

A v0.5 só está concluída quando:

1. MarketTimeframe existir;
2. 1D continuar funcionando;
3. 15M funcionar;
4. timezone for persistida;
5. session model existir;
6. session boundaries funcionarem;
7. expected gaps forem reconhecidos;
8. unexpected intraday gaps forem reconhecidos;
9. Market Replay 15M funcionar;
10. no-look-ahead for preservado;
11. Feature Engine não assumir dias;
12. return_5 representar 5 candles;
13. moving averages representarem candles;
14. volatility for timeframe-aware;
15. annualization 15M funcionar;
16. Scientific Metrics usarem fator correto;
17. execution timing for explícito;
18. Decision Trigger Engine existir;
19. trigger output existir;
20. bias change trigger funcionar;
21. regime trigger funcionar;
22. periodic trigger funcionar;
23. position trigger funcionar;
24. cooldown funcionar;
25. Trigger Audit existir;
26. NO_LLM_CALL_NO_TRIGGER existir;
27. NO_LLM_CALL_COOLDOWN existir;
28. AI Call Rate existir;
29. Compute Budget existir;
30. session metrics existirem;
31. daily loss usar sessão;
32. holding minutes funcionar;
33. overnight detection funcionar;
34. lifecycle intraday funcionar;
35. Intraday Observatory existir;
36. Trigger Inspector existir;
37. UI suportar dataset maior;
38. chart não comprometer performance;
39. dataset AAPL 15M pequeno rodar;
40. determinísticos rodarem antes do AI;
41. AI V4.2 rodar com triggers;
42. Signal Engine V1 permanecer inalterado;
43. Risk Policy V2 permanecer inalterada;
44. regressão 1D passar;
45. Scientific Validation 15M funcionar;
46. testes passarem;
47. build funcionar;
48. Tauri iniciar normalmente;
49. documentação existir.

---

# PARTE 35 — TESTE FINAL OBRIGATÓRIO

## TEST A — INFRAESTRUTURA

Dataset:

```text
AAPL
15M
~5 sessões
~130 candles
```

Agentes:

```text
Cash Control
Buy & Hold
Simple Trend
```

Sem AI.

Confirmar:

```text
dataset
session
features
signals
execution
risk
portfolio
metrics
```

---

## TEST B — DATASET MAIOR

```text
AAPL
15M
~20 sessões
~520 candles
```

Mesmos agentes determinísticos.

Confirmar performance aceitável.

---

## TEST C — AI INTRADAY

Mesmo dataset de ~520 candles.

Adicionar:

```text
AI Technical V4.2
```

com:

```text
MarketDecisionTriggerEngine
```

Mostrar:

```text
TOTAL CANDLES
TRIGGERS
AI CALLS
CALL RATE

NO TRIGGER
COOLDOWN
COMPUTE BUDGET

RETURN
DRAWDOWN
EXPOSURE
EXECUTIONS
```

---

# RESULTADO ESPERADO

Antes:

```text
1D DATA
↓
FEATURES
↓
SIGNAL
↓
AI every N candles
↓
POSITION
↓
RISK
↓
EXECUTION
```

Depois:

```text
15M MARKET DATA
        ↓
SESSION-AWARE REPLAY
        ↓
TIMEFRAME-AWARE FEATURES
        ↓
SIGNAL ENGINE
        ↓
DECISION TRIGGER ENGINE
        │
        ├── NOTHING RELEVANT
        │        ↓
        │   NO_LLM_CALL
        │
        └── RELEVANT EVENT
                 ↓
                AI
                 ↓
          POSITION MANAGER
                 ↓
             RISK ENGINE
                 ↓
              EXECUTION
                 ↓
        INTRADAY LIFECYCLE
```

A v0.5 deve permitir responder:

```text
O Market Lab entende candles intraday?

Ele respeita sessões?

Ele evita look-ahead?

As métricas continuam corretas?

O Risk Engine continua funcionando?

O LLM é chamado apenas quando faz sentido?

Quanto custa computacionalmente operar em 15M?

Quanto tempo uma posição fica aberta em minutos?

O sistema consegue carregar posição overnight?

A arquitetura continua válida sem reescrever tudo?
```

# A v0.5 NÃO É PARA FAZER MAIS TRADES.

# É PARA FAZER O MARKET LAB ENTENDER TEMPO INTRADAY CORRETAMENTE.
