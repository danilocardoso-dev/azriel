# AZRIEL Market Lab v0.5.4 - Multi-Period Validation Report

## 1. Cover / Metadata

- Batch: `AAPL_15M_MULTIPERIOD_V054_VALIDATION_01`
- Batch ID: `lifecycle-validation-1790281887271788400`
- Status: **COMPLETED**
- Asset / timeframe: AAPL / 15M
- Agent: ai-intraday-v1 (AI_INTRADAY_V1)
- Coverage: 2026-03-12T13:30:00+00:00 to 2026-05-07T19:45:00+00:00
- Generated at: 2026-09-24 20:31:27

## 2. Executive Technical Summary

This report consolidates 2 independent periods and 4 lifecycle-level observations. The analysis is descriptive and observational; it does not alter the frozen decision path or create operational rules.

- Sample status: **INSUFFICIENT**
- Score stability: **SHIFTED**
- Lifecycle late reduction rate: 100.00%
- Event late reduction rate: 72.39%

## 3. Dataset Quality

| Dataset | Status | Candles | Sessions | Duplicates | Invalid OHLC | Unexpected gaps | Source file |
|---|---:|---:|---:|---:|---:|---:|---|
| AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated | PASS_WITH_WARNINGS | 520 | 20 | 0 | 0 | 0 | UNAVAILABLE - persisted candles used |
| AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated | PASS_WITH_WARNINGS | 520 | 20 | 0 | 0 | 0 | UNAVAILABLE - persisted candles used |

## 4. Temporal Coverage

| Role | Dataset | Start | End | Overlap |
|---|---|---|---|---|
| DEVELOPMENT | AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated | 2026-03-12T13:30:00+00:00 | 2026-04-09T19:45:00+00:00 | NON_OVERLAPPING |
| OOS | AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated | 2026-04-10T13:30:00+00:00 | 2026-05-07T19:45:00+00:00 | NON_OVERLAPPING |

## 5. Execution / Resource Audit

- Reused experiments: 2
- Rebuilt local artifacts: 0
- New LLM runs: 0
- Duration: 235 ms
- Model: qwen2.5:3b
- Prompt / context: MARKET_AI_INTRADAY_V1 / MARKET_AI_INTRADAY_CONTEXT_V1

## 6. Sample Overview

Datasets: 2 | Periods: 2 | Experiments: 2 | Lifecycles: 4 | Closed: 2 | Open: 2 | Censored: 2

## 7. Period Results

| Role | Dataset | Lifecycles | Closed/Open | Median PnL | Median MFE | Median MAE | Median Giveback |
|---|---|---:|---:|---:|---:|---:|---:|
| DEVELOPMENT | AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated | 2 | 1/1 | -2.4897 | 2.7681 | -2.6723 | 1.7626 |
| OOS | AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated | 2 | 1/1 | 4.7790 | 7.1201 | -1.0780 | 1.5678 |

## 8. Lifecycle Results

| Lifecycle | Status | Duration min | PnL % | MFE % | MAE % | Giveback % | Worst health | Max deterioration |
|---|---|---:|---:|---:|---:|---:|---|---|
| experiment-1790256677468863400:ai-intraday-v1:1 | CLOSED | 4350 | -2.4897 | 0.3350 | -3.9580 | 2.8247 | CRITICAL | CRITICAL (1.000) |
| experiment-1790256677468863400:ai-intraday-v1:2 | OPEN / CENSORED | 2700 | N/A | 5.2013 | -1.3865 | 0.7004 | CRITICAL | CRITICAL (0.856) |
| experiment-1790255634428168200:ai-intraday-v1:1 | CLOSED | 2955 | 4.7790 | 6.2221 | -0.9257 | 1.4431 | DETERIORATING | HIGH (0.748) |
| experiment-1790255634428168200:ai-intraday-v1:2 | OPEN / CENSORED | 2910 | N/A | 8.0180 | -1.2303 | 1.6926 | CRITICAL | CRITICAL (0.899) |

## 9. Late Reduction - Event vs Lifecycle

| Dataset | Events | Eligible lifecycles | Affected lifecycles | Event rate % | Lifecycle rate % | FWD5 | MAE5 |
|---|---:|---:|---:|---:|---:|---:|---:|
| AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated | 200 | 2 | 2 | 111.73 | 100.00 | -0.2259 | -0.5954 |
| AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated | 57 | 2 | 2 | 32.39 | 100.00 | -0.2678 | -0.5346 |

## 10. Response Delay

| Lifecycle | First deterioration | First HIGH | First CRITICAL | Status |
|---|---:|---:|---:|---|
| experiment-1790256677468863400:ai-intraday-v1:1 | 4005 | 3975 | 3930 | RESOLVED |
| experiment-1790256677468863400:ai-intraday-v1:2 | UNRESOLVED | N/A | N/A | UNRESOLVED |
| experiment-1790255634428168200:ai-intraday-v1:1 | 1620 | 780 | N/A | RESOLVED |
| experiment-1790255634428168200:ai-intraday-v1:2 | 420 | 240 | 195 | RESOLVED |

## 11. Position Health

| Worst health | Lifecycle N | Avg PnL % | Avg MFE % | Avg MAE % | Avg giveback % | Late reduction % |
|---|---:|---:|---:|---:|---:|---:|
| DETERIORATING | 1 | 4.7790 | 6.2221 | -0.9257 | 1.4431 | 100.00 |
| CRITICAL | 3 | -2.4897 | 4.5181 | -2.1916 | 1.7392 | 100.00 |

## 12. Deterioration

| Max level | Lifecycle N | Avg PnL % | Median PnL % | Avg giveback % | Late reduction % |
|---|---:|---:|---:|---:|---:|
| HIGH | 1 | 4.7790 | 4.7790 | 1.4431 | 100.00 |
| CRITICAL | 3 | -2.4897 | -2.4897 | 1.7392 | 100.00 |

## 13. Component Analysis

| Scope | Component | Lifecycle N | Average | Maximum | Avg PnL % | Avg MAE % | Late % |
|---|---|---:|---:|---:|---:|---:|---:|
| ALL PERIODS | TREND | 4 | 0.2115 | 1.0000 | 1.1446 | -1.8751 | 100.00 |
| ALL PERIODS | MOMENTUM | 4 | 0.4811 | 1.0000 | 1.1446 | -1.8751 | 100.00 |
| ALL PERIODS | GIVEBACK | 4 | 0.4751 | 1.0000 | 1.1446 | -1.8751 | 100.00 |
| ALL PERIODS | VWAP | 4 | 0.2272 | 1.0000 | 1.1446 | -1.8751 | 100.00 |
| ALL PERIODS | VOLATILITY | 4 | 0.7984 | 1.0000 | 1.1446 | -1.8751 | 100.00 |
| ALL PERIODS | TIME | 4 | 0.8804 | 1.0000 | 1.1446 | -1.8751 | 100.00 |
| lifecycle-validation-1790281887271788400-period-01 | TREND | 2 | 0.2029 | 1.0000 | -2.4897 | -2.6723 | 100.00 |
| lifecycle-validation-1790281887271788400-period-01 | MOMENTUM | 2 | 0.4174 | 1.0000 | -2.4897 | -2.6723 | 100.00 |
| lifecycle-validation-1790281887271788400-period-01 | GIVEBACK | 2 | 0.6187 | 1.0000 | -2.4897 | -2.6723 | 100.00 |
| lifecycle-validation-1790281887271788400-period-01 | VWAP | 2 | 0.3935 | 1.0000 | -2.4897 | -2.6723 | 100.00 |
| lifecycle-validation-1790281887271788400-period-01 | VOLATILITY | 2 | 0.8305 | 1.0000 | -2.4897 | -2.6723 | 100.00 |
| lifecycle-validation-1790281887271788400-period-01 | TIME | 2 | 0.8770 | 1.0000 | -2.4897 | -2.6723 | 100.00 |
| lifecycle-validation-1790281887271788400-period-02 | TREND | 2 | 0.2202 | 1.0000 | 4.7790 | -1.0780 | 100.00 |
| lifecycle-validation-1790281887271788400-period-02 | MOMENTUM | 2 | 0.5447 | 1.0000 | 4.7790 | -1.0780 | 100.00 |
| lifecycle-validation-1790281887271788400-period-02 | GIVEBACK | 2 | 0.3315 | 1.0000 | 4.7790 | -1.0780 | 100.00 |
| lifecycle-validation-1790281887271788400-period-02 | VWAP | 2 | 0.0609 | 1.0000 | 4.7790 | -1.0780 | 100.00 |
| lifecycle-validation-1790281887271788400-period-02 | VOLATILITY | 2 | 0.7663 | 1.0000 | 4.7790 | -1.0780 | 100.00 |
| lifecycle-validation-1790281887271788400-period-02 | TIME | 2 | 0.8839 | 1.0000 | 4.7790 | -1.0780 | 100.00 |

## 14. Giveback Analysis

| Dataset | Average | Median | P25 | P75 | P90 |
|---|---:|---:|---:|---:|---:|
| AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated | 1.7626 | 1.7626 | N/A | N/A | N/A |
| AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated | 1.5678 | 1.5678 | N/A | N/A | N/A |

## 15. Cross-Period Stability

| Metric | Median | Min | Max | Dispersion |
|---|---:|---:|---:|---:|
| LIFECYCLE LATE REDUCTION RATE | 100.0000 | 100.0000 | 100.0000 | 0.0000 |
| EVENT LATE REDUCTION RATE | 72.0591 | 32.3864 | 111.7318 | 79.3455 |
| MEDIAN RESPONSE DELAY | 2512.5000 | 1020.0000 | 4005.0000 | 2985.0000 |
| MFE | 4.9441 | 2.7681 | 7.1201 | 4.3519 |
| MAE | -1.8751 | -2.6723 | -1.0780 | 1.5943 |
| GIVEBACK | 1.6652 | 1.5678 | 1.7626 | 0.1947 |
| MAX DETERIORATION SCORE | 0.8758 | 0.8237 | 0.9279 | 0.1043 |
| HIGH LIFECYCLE REACH RATE | 100.0000 | 100.0000 | 100.0000 | 0.0000 |
| CRITICAL LIFECYCLE REACH RATE | 75.0000 | 50.0000 | 100.0000 | 50.0000 |

## 16. Score Stability

**SHIFTED**. Criteria: fewer than four lifecycles or fewer than two periods yields `INSUFFICIENT_SAMPLE`; otherwise score-median spread <=0.10 is `STABLE`, <=0.25 is `SHIFTED`, and larger spread is `HIGHLY_SHIFTED`.

## 17. Outlier / Concentration Analysis

| Metric | Largest lifecycle | Full average | Without largest outlier | Delta |
|---|---|---:|---:|---:|
| DURATION | experiment-1790256677468863400:ai-intraday-v1:1 | 3228.7500 | 2855.0000 | 373.7500 |
| PNL | experiment-1790255634428168200:ai-intraday-v1:1 | 1.1446 | -2.4897 | 3.6344 |
| MFE | experiment-1790255634428168200:ai-intraday-v1:2 | 4.9441 | 3.9195 | 1.0246 |
| MAE | experiment-1790256677468863400:ai-intraday-v1:1 | -1.8751 | -1.1808 | -0.6943 |
| GIVEBACK | experiment-1790256677468863400:ai-intraday-v1:1 | 1.6652 | 1.2787 | 0.3865 |

## 18. Evidence Matrix

| Dimension | Classification |
|---|---|
| Sample Size | INSUFFICIENT |
| Period Coverage | PARTIAL |
| Late Reduction | CONSISTENT |
| Deterioration Stability | SHIFTED |
| Health Outcome Relation | NO_SIGNAL |
| Component Consistency | MIXED |
| Giveback Consistency | CONSISTENT |

## 19. Limitations

- The sample is descriptive and does not establish statistical proof.
- Open positions are marked `OPEN_AT_END` and `CENSORED_AT_DATASET_END`; realized PnL uses CLOSED lifecycles only.
- Candle-level health traces are secondary diagnostics; lifecycle/period is the primary unit.
- AAPL_real_15m_20sessions_2026-03-12_to_2026-04-09_treated: SOURCE_FILE_UNAVAILABLE: quality gate executado sobre candles canônicos persistidos
- AAPL_real_15m_OOS_20sessions_2026-04-10_to_2026-05-07_treated: SOURCE_FILE_UNAVAILABLE: quality gate executado sobre candles canônicos persistidos
- OPEN_AT_END / CENSORED_AT_DATASET_END presente; PnL realizado usa somente CLOSED
- INSUFFICIENT SAMPLE: menos de 20 lifecycles independentes
- INSUFFICIENT_PERIOD_COVERAGE: 2 de 6 períodos desejados

## 20. Audit Appendix

- Validation engine: MULTI_PERIOD_LIFECYCLE_VALIDATION_V1
- Lifecycle config: POSITION_LIFECYCLE_CONFIG_V1
- Deterioration config: POSITION_DETERIORATION_CONFIG_V1
- Hold Diagnostics: HOLD_DIAGNOSTICS_V1
- Trigger: MARKET_DECISION_TRIGGER_V2
- Position sizing: POSITION_SIZING_V1
- Risk policy: RISK_POLICY_V2
- Execution model: EXECUTION_MODEL_V1
- Fees / slippage: 0.1000% / 0.0500%
- Temporal semantics: AVAILABLE AT T is isolated from POST-DECISION outcomes.
- DEVELOPMENT -> experiment `experiment-1790256677468863400`; dataset `dataset-1789263885968772900`; lifecycle run `lifecycle-intelligence-1790257474827524800`; quality PASS_WITH_WARNINGS
- OOS -> experiment `experiment-1790255634428168200`; dataset `dataset-1789397830614545700`; lifecycle run `lifecycle-intelligence-1790256451779518900`; quality PASS_WITH_WARNINGS
