# AZRIEL MARKET LAB v0.5.4 — VALIDATION RUNBOOK
## Multi-Period Lifecycle Validation + Automated Report

> Para o Codex: a v0.5.4 já está implementada. Não criar nova versão. Este arquivo define a execução da validação, persistência dos resultados e geração automática do relatório final.

## 1. Objetivo

Executar uma bateria multi-período do `AI Intraday V1` congelado e gerar automaticamente um relatório consolidado, evitando análise manual de milhares de linhas.

O relatório deve responder:
- Late Reduction persiste em períodos independentes?
- Quantos lifecycles independentes apresentam o fenômeno?
- Response Delay é consistente?
- HIGH/CRITICAL generalizam?
- Quais componentes do DeteriorationScore carregam informação?
- Giveback é mais estável que o score agregado?
- Há períodos/lifecycles dominando os agregados?
- A amostra é suficiente para justificar nova etapa?

## 2. Configuração congelada

Não alterar:
- AI Intraday V1
- MARKET_AI_INTRADAY_V1
- MARKET_AI_INTRADAY_CONTEXT_V1
- qwen2.5:3b
- temperature=0.1
- seed=42
- MARKET_DECISION_TRIGGER_V2
- POSITION_SIZING_V1
- RISK_POLICY_V2
- EXECUTION_MODEL_V1
- HOLD_DIAGNOSTICS_V1
- POSITION_LIFECYCLE_CONFIG_V1
- POSITION_DETERIORATION_CONFIG_V1

Proibido: calibrar prompt/threshold/pesos, criar novo agente, HIGH=>REDUCE, CRITICAL=>EXIT, otimizar usando resultados ou integrar corretora.

## 3. Períodos existentes

Incluir, se persistidos e compatíveis:

DEV:
`AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated`

OOS:
`AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated`

Não rerodar Ollama quando histórico suficiente existir.

## 4. Descoberta dos datasets

Listar datasets AAPL/15M reais disponíveis localmente. Para cada um:
- identificar range;
- candles e sessões;
- metadata asset/timeframe;
- detectar duplicata/sobreposição;
- validar OHLCV;
- validar timestamps/timezone;
- validar US_EQUITIES regular session.

Priorizar períodos independentes, não sobrepostos, idealmente 20 sessões.

Não inventar OHLCV. Se não houver dados reais suficientes, executar com os disponíveis e registrar `INSUFFICIENT_PERIOD_COVERAGE`.

Meta inicial desejada: 6 períodos. Meta de expansão: >=20 lifecycles; mais ampla >=30; preferencial >=50. Essas metas não garantem validade estatística e não são gates automáticos.

## 5. Dataset Quality Gate

Validar:
- timestamp crescente;
- duplicatas;
- NaN;
- preços >0;
- volume >=0;
- high >= open/close/low;
- low <= open/close/high;
- sessões/candles;
- timezone;
- primeira/última data;
- gaps.

Status: PASS / PASS_WITH_WARNINGS / FAIL.

FAIL não entra silenciosamente no batch.

## 6. Reutilização e novos experimentos

Reutilizar candles, decisions, executions, positions, risk traces, Hold Diagnostics e Position Lifecycles.

`REBUILD MISSING ARTIFACTS` deve reconstruir apenas artefatos locais e não chamar Ollama.

Somente dataset real válido sem experimento compatível pode gerar novo experimento, usando exatamente a configuração congelada e apenas `AI Intraday V1`.

## 7. Execução resiliente

Criar/usar batch:
`AAPL_15M_MULTIPERIOD_V054_VALIDATION_01`

Pipeline:
SELECT → COMPATIBILITY → OVERLAP → QUALITY → REUSE/REBUILD → REQUIRED RUNS → LIFECYCLES → AGGREGATION → STABILITY → REPORT.

Persistir checkpoint após cada período. Permitir resume. Não rerodar período COMPLETED.

Status por período: PENDING / RUNNING / COMPLETED / FAILED / SKIPPED.
Batch: COMPLETED / PARTIAL / FAILED.

Não executar inferências Ollama em paralelo no hardware atual. Processamento local pode usar paralelismo moderado quando seguro.

## 8. Unidade de análise

Unidade primária = lifecycle/período, não candle.

Nunca tratar múltiplos candles HIGH/CRITICAL da mesma posição como posições independentes.

Mostrar:
- datasets N;
- periods N;
- experiments N;
- lifecycles N;
- closed/open/censored N.

## 9. Métricas por período

Calcular:
- lifecycle count;
- closed/open/censored;
- avg/median duration;
- avg/median PnL;
- avg/median MFE;
- avg/median MAE;
- avg/median giveback;
- event late reduction count/rate;
- lifecycle late reduction count/rate;
- avg/median response delay;
- unresolved response;
- HIGH lifecycle reach;
- CRITICAL lifecycle reach;
- max deterioration.

Usar P25/P50/P75/P90 quando N permitir.

## 10. Late Reduction

Separar obrigatoriamente:

`EVENT_LATE_REDUCTION_RATE`

e

`LIFECYCLE_LATE_REDUCTION_RATE =
lifecycles com >=1 POTENTIAL_LATE_REDUCTION / lifecycles elegíveis`

Mostrar também events/lifecycle, FWD5, MAE5 e response delay.

## 11. Response Delay

Calcular:
- first deterioration → REDUCE/EXIT;
- first HIGH → REDUCE/EXIT;
- first CRITICAL → REDUCE/EXIT.

Sem resposta = `UNRESOLVED`. Não substituir por duração arbitrária.

## 12. Position Health e Deterioration

Por lifecycle:
- dominant health;
- worst health;
- % em cada health;
- max deterioration score/level;
- first MODERATE/HIGH/CRITICAL;
- time to first HIGH/CRITICAL.

Gerar `WORST HEALTH × OUTCOME` e `MAX DETERIORATION × OUTCOME` usando lifecycle N, PnL, MFE, MAE, giveback e late-reduction lifecycle rate.

Manter análises candle-level apenas como diagnóstico secundário.

## 13. Component Analysis

Usar somente componentes realmente existentes no V1, como trend/momentum/giveback/vwap/volatility quando presentes.

Por lifecycle:
- avg;
- max;
- value at first HIGH;
- value at first CRITICAL;
- dominant component.

Gerar `COMPONENT × OUTCOME`. Não alterar pesos.

## 14. Giveback

Analisar isoladamente:
- avg/median por período;
- distribuição;
- giveback × late reduction;
- giveback × PnL;
- giveback × MAE;
- giveback at first HIGH/CRITICAL.

Objetivo: observar se é mais consistente que o score agregado. Não transformá-lo em regra operacional.

## 15. Cross-Period / Score Stability

Comparar por período:
- lifecycle/event late reduction;
- response delay;
- MFE/MAE/giveback;
- median/P75/P90 deterioration score;
- HIGH/CRITICAL event share;
- HIGH/CRITICAL lifecycle reach.

Mostrar mediana, min, max e dispersão.

Score Stability: STABLE / SHIFTED / HIGHLY_SHIFTED / INSUFFICIENT_SAMPLE, com critérios documentados.

## 16. Contexto do período

Quando possível calcular period return, realized volatility, avg ATR%, trend proxy e avg relative volume. Usar regime persistido se existir; não inventar regimes.

Somente contextualização.

## 17. Outliers e concentração

Identificar lifecycles extremos em PnL, duration, MFE, MAE, giveback e response delay.

Não remover automaticamente. Mostrar análise de sensibilidade com/sem maior outlier.

Detectar se um período domina a amostra e gerar CONCENTRATION WARNING.

## 18. Open/Censored

Dataset termina com posição aberta:
- OPEN_AT_END
- CENSORED_AT_DATASET_END

Realized PnL somente CLOSED. MFE/MAE de OPEN pode aparecer como parcial.

Não tratar fim do dataset como EXIT.

## 19. Evidence Matrix

Gerar sem score único:

- Sample Size: INSUFFICIENT / GROWING / BROADER
- Period Coverage: INSUFFICIENT / PARTIAL / BROADER
- Late Reduction: INCONSISTENT / MIXED / CONSISTENT
- Deterioration Stability: STABLE / SHIFTED / HIGHLY_SHIFTED
- Health Outcome Relation: NO_SIGNAL / MIXED / OBSERVABLE
- Component Consistency: INSUFFICIENT / MIXED / CONSISTENT
- Giveback Consistency: INSUFFICIENT / MIXED / CONSISTENT

Não usar “scientifically proven”.

## 20. Relatório automático

Ao final gerar:

`AZRIEL_Market_Lab_v0.5.4_MultiPeriod_Validation_Report.pdf`

e

`AZRIEL_Market_Lab_v0.5.4_MultiPeriod_Validation_Report.md`

Reutilizar mecanismo de exportação do projeto quando possível.

O PDF deve conter:

1. Cover/Metadata
2. Executive Technical Summary
3. Dataset Quality
4. Temporal Coverage
5. Execution/Resource Audit
6. Sample Overview
7. Period Results
8. Lifecycle Results
9. Late Reduction — event vs lifecycle
10. Response Delay
11. Position Health
12. Deterioration
13. Component Analysis
14. Giveback Analysis
15. Cross-Period Stability
16. Score Stability
17. Outlier/Concentration Analysis
18. Evidence Matrix
19. Limitations
20. Audit Appendix

Não despejar timelines completas no corpo principal. Agregar e mover detalhes para appendix quando necessário.

## 21. Dados estruturados

Além do PDF/MD, persistir em formato já suportado:
- period_summary;
- lifecycle_summary;
- late_reduction_events;
- component_analysis;
- evidence_matrix;
- warnings.

Preferir JSON/CSV conforme padrões existentes.

## 22. Auditabilidade

Registrar:
- batch ID;
- dataset IDs;
- experiment IDs;
- ranges;
- agent/model;
- prompt/context versions;
- config versions;
- fees/slippage;
- reused artifacts;
- rebuilt artifacts;
- new LLM runs;
- failures;
- timestamps;
- duração.

## 23. Failure Handling

Falha em período:
- preservar concluídos;
- registrar erro;
- continuar quando seguro;
- batch PARTIAL;
- gerar relatório parcial;
- listar FAILED/SKIPPED;
- refletir impacto na Evidence Matrix.

## 24. Validação do relatório

Antes de COMPLETED/PARTIAL verificar:
- PDF existe e não vazio;
- MD existe;
- períodos do relatório = batch;
- lifecycle count = batch;
- configs presentes;
- Evidence Matrix presente;
- Limitations presente;
- Audit Appendix presente.

## 25. Critérios de aceite

A execução é aceita quando:
1. datasets reais forem descobertos;
2. quality gate rodar;
3. overlap/compatibility rodar;
4. batch persistido existir;
5. históricos forem reutilizados;
6. novos runs necessários forem sequenciais;
7. checkpoint/resume funcionar;
8. lifecycles forem agregados;
9. event vs lifecycle late reduction estiver separado;
10. response delay estiver correto;
11. health/deterioration lifecycle-level existirem;
12. component analysis existir;
13. giveback analysis existir;
14. cross-period e score stability existirem;
15. outlier/concentration existirem;
16. censored handling existir;
17. Evidence Matrix existir;
18. MD/PDF forem gerados;
19. artefatos estruturados forem persistidos;
20. nenhuma configuração congelada tiver sido alterada;
21. nenhuma otimização/regra operacional nova tiver sido criada.

## 26. Saída final do Codex

Ao terminar, responder de forma curta:

```text
VALIDATION BATCH
AAPL_15M_MULTIPERIOD_V054_VALIDATION_01

STATUS
COMPLETED / PARTIAL / FAILED

PERIODS
N

LIFECYCLES
N

CLOSED / OPEN / CENSORED
N / N / N

REUSED EXPERIMENTS
N

NEW LLM RUNS
N

FAILED PERIODS
N

REPORT PDF
<path>

REPORT MD
<path>

STRUCTURED ARTIFACTS
<paths>

WARNINGS
<summary>
```

Não interpretar os resultados extensamente na resposta do Codex. O relatório deve conter os dados; a análise será feita depois.

---

## REGRA FINAL

Esta é uma execução de validação da v0.5.4, não desenvolvimento orientado a resultado.

Se a conclusão for `INSUFFICIENT_SAMPLE`, `HIGHLY_SHIFTED` ou `NO_SIGNAL`, preservar essa conclusão.

Não modificar o agente para fazer os dados parecerem melhores.

---

## Estado da execução — 24/09/2026

Batch executado:

`AAPL_15M_MULTIPERIOD_V054_VALIDATION_01`

Resultado operacional: `COMPLETED`.

- períodos reais reutilizados: 2 (`DEVELOPMENT` e `OOS`);
- experimentos reutilizados: 2;
- novas chamadas ao LLM: 0;
- lifecycles: 4;
- `CLOSED / OPEN / CENSORED`: `2 / 2 / 2`;
- períodos com falha: 0;
- quality gate: `PASS_WITH_WARNINGS` nos dois períodos, com 520 candles, 20 sessões, zero duplicatas, zero OHLC inválido e zero gaps inesperados;
- checkpoint e resume validados sem duplicação;
- relatório Markdown, HTML, PDF de seis páginas e artefatos estruturados gerados e registrados.

Conclusão preservada: `INSUFFICIENT SAMPLE` e `INSUFFICIENT_PERIOD_COVERAGE` (2 de 6 períodos desejados). Os CSVs de origem já não estavam disponíveis nos caminhos históricos; por isso, o quality gate foi executado sobre os candles canônicos persistidos, sem alterar candles, decisões ou resultados.

Artefatos:

`output/pdf/market-lab-v0.5.4/`
