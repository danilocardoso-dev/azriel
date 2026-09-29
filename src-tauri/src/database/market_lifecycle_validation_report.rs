use super::market_lifecycle_validation::{
    DatasetQualityReport, MultiPeriodLifecycleValidationReport,
};
use rusqlite::{params, Connection};
use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

pub const REPORT_STEM: &str = "AZRIEL_Market_Lab_v0.5.4_MultiPeriod_Validation_Report";

#[derive(Debug, Clone)]
pub struct ValidationReportArtifacts {
    pub markdown: PathBuf,
    pub html: PathBuf,
    pub json: PathBuf,
    pub period_csv: PathBuf,
    pub lifecycle_csv: PathBuf,
    pub late_reduction_csv: PathBuf,
    pub component_csv: PathBuf,
    pub evidence_csv: PathBuf,
    pub warnings_json: PathBuf,
}

fn decimal(value: f64) -> String {
    format!("{value:.4}")
}

fn optional(value: Option<f64>) -> String {
    value.map(decimal).unwrap_or_else(|| "N/A".into())
}

fn escape_csv(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn quality_for<'a>(
    report: &'a MultiPeriodLifecycleValidationReport,
    dataset_id: &str,
) -> Option<&'a DatasetQualityReport> {
    report
        .quality
        .iter()
        .find(|quality| quality.dataset_id == dataset_id)
}

pub fn markdown(report: &MultiPeriodLifecycleValidationReport) -> String {
    let mut output = String::new();
    let batch = &report.batch;
    let sample = &report.sample;
    writeln!(
        output,
        "# AZRIEL Market Lab v0.5.4 - Multi-Period Validation Report\n"
    )
    .unwrap();
    writeln!(output, "## 1. Cover / Metadata\n").unwrap();
    writeln!(output, "- Batch: `{}`", batch.name).unwrap();
    writeln!(output, "- Batch ID: `{}`", batch.batch_id).unwrap();
    writeln!(output, "- Status: **{}**", batch.status).unwrap();
    writeln!(
        output,
        "- Asset / timeframe: {} / {}",
        batch.asset, batch.timeframe
    )
    .unwrap();
    writeln!(
        output,
        "- Agent: {} ({})",
        batch.agent_id, batch.agent_version
    )
    .unwrap();
    writeln!(
        output,
        "- Coverage: {} to {}",
        sample.date_coverage_start, sample.date_coverage_end
    )
    .unwrap();
    writeln!(
        output,
        "- Generated at: {}\n",
        batch.completed_at.as_deref().unwrap_or("N/A")
    )
    .unwrap();

    writeln!(output, "## 2. Executive Technical Summary\n").unwrap();
    writeln!(output, "This report consolidates {} independent periods and {} lifecycle-level observations. The analysis is descriptive and observational; it does not alter the frozen decision path or create operational rules.", sample.periods, sample.lifecycles).unwrap();
    writeln!(output, "\n- Sample status: **{}**", sample.sample_status).unwrap();
    writeln!(output, "- Score stability: **{}**", sample.score_stability).unwrap();
    writeln!(
        output,
        "- Lifecycle late reduction rate: {:.2}%",
        sample.lifecycle_late_reduction_rate_pct
    )
    .unwrap();
    writeln!(
        output,
        "- Event late reduction rate: {:.2}%\n",
        sample.event_late_reduction_rate_pct
    )
    .unwrap();

    writeln!(output, "## 3. Dataset Quality\n").unwrap();
    writeln!(output, "| Dataset | Status | Candles | Sessions | Duplicates | Invalid OHLC | Unexpected gaps | Source file |\n|---|---:|---:|---:|---:|---:|---:|---|").unwrap();
    for item in &report.quality {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            item.dataset_name,
            item.status,
            item.candle_count,
            item.session_count,
            item.duplicate_timestamp_count,
            item.invalid_high_count + item.invalid_low_count,
            item.unexpected_gap_count,
            if item.source_file_available {
                "AVAILABLE"
            } else {
                "UNAVAILABLE - persisted candles used"
            }
        )
        .unwrap();
    }

    writeln!(output, "\n## 4. Temporal Coverage\n").unwrap();
    writeln!(
        output,
        "| Role | Dataset | Start | End | Overlap |\n|---|---|---|---|---|"
    )
    .unwrap();
    for period in &report.periods {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} |",
            period.source_role,
            period.dataset_name,
            period.start_at,
            period.end_at,
            period.overlap_status
        )
        .unwrap();
    }

    writeln!(output, "\n## 5. Execution / Resource Audit\n").unwrap();
    writeln!(
        output,
        "- Reused experiments: {}",
        report.audit.reused_experiments
    )
    .unwrap();
    writeln!(
        output,
        "- Rebuilt local artifacts: {}",
        report.audit.rebuilt_artifacts
    )
    .unwrap();
    writeln!(output, "- New LLM runs: {}", report.audit.new_llm_runs).unwrap();
    writeln!(output, "- Duration: {} ms", report.audit.duration_ms).unwrap();
    writeln!(output, "- Model: {}", report.audit.model).unwrap();
    writeln!(
        output,
        "- Prompt / context: {} / {}\n",
        report.audit.prompt_version, report.audit.context_version
    )
    .unwrap();

    writeln!(output, "## 6. Sample Overview\n").unwrap();
    writeln!(output, "Datasets: {} | Periods: {} | Experiments: {} | Lifecycles: {} | Closed: {} | Open: {} | Censored: {}\n", sample.datasets, sample.periods, sample.experiments, sample.lifecycles, sample.closed, sample.open, sample.censored).unwrap();

    writeln!(output, "## 7. Period Results\n").unwrap();
    writeln!(output, "| Role | Dataset | Lifecycles | Closed/Open | Median PnL | Median MFE | Median MAE | Median Giveback |\n|---|---|---:|---:|---:|---:|---:|---:|").unwrap();
    for period in &report.periods {
        writeln!(
            output,
            "| {} | {} | {} | {}/{} | {} | {} | {} | {} |",
            period.source_role,
            period.dataset_name,
            period.lifecycle_count,
            period.closed_count,
            period.open_count,
            decimal(period.pnl.median),
            decimal(period.mfe.median),
            decimal(period.mae.median),
            decimal(period.giveback.median)
        )
        .unwrap();
    }

    writeln!(output, "\n## 8. Lifecycle Results\n").unwrap();
    writeln!(output, "| Lifecycle | Status | Duration min | PnL % | MFE % | MAE % | Giveback % | Worst health | Max deterioration |\n|---|---|---:|---:|---:|---:|---:|---|---|").unwrap();
    for item in &report.lifecycles {
        writeln!(
            output,
            "| {} | {}{} | {} | {} | {} | {} | {} | {} | {} ({:.3}) |",
            item.lifecycle_id,
            item.status,
            if item.censored_at_dataset_end {
                " / CENSORED"
            } else {
                ""
            },
            item.duration_minutes,
            optional(item.pnl_pct),
            decimal(item.mfe_pct),
            decimal(item.mae_pct),
            decimal(item.giveback_pct),
            item.worst_health,
            item.max_deterioration_level,
            item.max_deterioration_score
        )
        .unwrap();
    }

    writeln!(output, "\n## 9. Late Reduction - Event vs Lifecycle\n").unwrap();
    writeln!(output, "| Dataset | Events | Eligible lifecycles | Affected lifecycles | Event rate % | Lifecycle rate % | FWD5 | MAE5 |\n|---|---:|---:|---:|---:|---:|---:|---:|").unwrap();
    for period in &report.periods {
        writeln!(
            output,
            "| {} | {} | {} | {} | {:.2} | {:.2} | {} | {} |",
            period.dataset_name,
            period.late_reduction_events,
            period.eligible_lifecycles,
            period.late_reduction_lifecycles,
            period.event_late_reduction_rate_pct,
            period.lifecycle_late_reduction_rate_pct,
            optional(period.average_forward_5),
            optional(period.average_mae_5)
        )
        .unwrap();
    }

    writeln!(output, "\n## 10. Response Delay\n").unwrap();
    writeln!(output, "| Lifecycle | First deterioration | First HIGH | First CRITICAL | Status |\n|---|---:|---:|---:|---|").unwrap();
    for item in &report.lifecycles {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} |",
            item.lifecycle_id,
            item.response_from_first_deterioration_minutes
                .map(|v| v.to_string())
                .unwrap_or_else(|| "UNRESOLVED".into()),
            item.response_from_high_minutes
                .map(|v| v.to_string())
                .unwrap_or_else(|| "N/A".into()),
            item.response_from_critical_minutes
                .map(|v| v.to_string())
                .unwrap_or_else(|| "N/A".into()),
            item.response_status
        )
        .unwrap();
    }

    writeln!(output, "\n## 11. Position Health\n").unwrap();
    writeln!(output, "| Worst health | Lifecycle N | Avg PnL % | Avg MFE % | Avg MAE % | Avg giveback % | Late reduction % |\n|---|---:|---:|---:|---:|---:|---:|").unwrap();
    for item in &report.health_outcomes {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} | {} | {:.2} |",
            item.label,
            item.lifecycle_count,
            optional(item.average_pnl_pct),
            decimal(item.average_mfe_pct),
            decimal(item.average_mae_pct),
            decimal(item.average_giveback_pct),
            item.late_reduction_rate_pct
        )
        .unwrap();
    }

    writeln!(output, "\n## 12. Deterioration\n").unwrap();
    writeln!(output, "| Max level | Lifecycle N | Avg PnL % | Median PnL % | Avg giveback % | Late reduction % |\n|---|---:|---:|---:|---:|---:|").unwrap();
    for item in &report.deterioration_outcomes {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} | {:.2} |",
            item.label,
            item.lifecycle_count,
            optional(item.average_pnl_pct),
            optional(item.median_pnl_pct),
            decimal(item.average_giveback_pct),
            item.late_reduction_rate_pct
        )
        .unwrap();
    }

    writeln!(output, "\n## 13. Component Analysis\n").unwrap();
    writeln!(output, "| Scope | Component | Lifecycle N | Average | Maximum | Avg PnL % | Avg MAE % | Late % |\n|---|---|---:|---:|---:|---:|---:|---:|").unwrap();
    for item in &report.components {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} | {} | {} | {:.2} |",
            item.period_id.as_deref().unwrap_or("ALL PERIODS"),
            item.component,
            item.lifecycle_count,
            decimal(item.average_value),
            decimal(item.maximum_value),
            optional(item.average_pnl_pct),
            decimal(item.average_mae_pct),
            item.late_reduction_rate_pct
        )
        .unwrap();
    }

    writeln!(output, "\n## 14. Giveback Analysis\n").unwrap();
    writeln!(
        output,
        "| Dataset | Average | Median | P25 | P75 | P90 |\n|---|---:|---:|---:|---:|---:|"
    )
    .unwrap();
    for period in &report.periods {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} | {} |",
            period.dataset_name,
            decimal(period.giveback.average),
            decimal(period.giveback.median),
            optional(period.giveback.p25),
            optional(period.giveback.p75),
            optional(period.giveback.p90)
        )
        .unwrap();
    }

    writeln!(output, "\n## 15. Cross-Period Stability\n").unwrap();
    writeln!(
        output,
        "| Metric | Median | Min | Max | Dispersion |\n|---|---:|---:|---:|---:|"
    )
    .unwrap();
    for item in &report.consistency {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} |",
            item.label,
            decimal(item.median),
            decimal(item.minimum),
            decimal(item.maximum),
            decimal(item.dispersion)
        )
        .unwrap();
    }

    writeln!(output, "\n## 16. Score Stability\n\n**{}**. Criteria: fewer than four lifecycles or fewer than two periods yields `INSUFFICIENT_SAMPLE`; otherwise score-median spread <=0.10 is `STABLE`, <=0.25 is `SHIFTED`, and larger spread is `HIGHLY_SHIFTED`.\n", sample.score_stability).unwrap();

    writeln!(output, "## 17. Outlier / Concentration Analysis\n").unwrap();
    writeln!(output, "| Metric | Largest lifecycle | Full average | Without largest outlier | Delta |\n|---|---|---:|---:|---:|").unwrap();
    for item in &report.outlier_sensitivity {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} |",
            item.metric,
            item.lifecycle_id.as_deref().unwrap_or("N/A"),
            decimal(item.full_average),
            decimal(item.without_largest_outlier_average),
            decimal(item.delta)
        )
        .unwrap();
    }

    writeln!(output, "\n## 18. Evidence Matrix\n").unwrap();
    writeln!(output, "| Dimension | Classification |\n|---|---|").unwrap();
    for (label, value) in [
        ("Sample Size", &report.evidence.sample_size),
        ("Period Coverage", &report.evidence.period_coverage),
        ("Late Reduction", &report.evidence.late_reduction),
        (
            "Deterioration Stability",
            &report.evidence.deterioration_stability,
        ),
        (
            "Health Outcome Relation",
            &report.evidence.health_outcome_relation,
        ),
        (
            "Component Consistency",
            &report.evidence.component_consistency,
        ),
        (
            "Giveback Consistency",
            &report.evidence.giveback_consistency,
        ),
    ] {
        writeln!(output, "| {label} | {value} |").unwrap();
    }

    writeln!(output, "\n## 19. Limitations\n").unwrap();
    writeln!(
        output,
        "- The sample is descriptive and does not establish statistical proof."
    )
    .unwrap();
    writeln!(output, "- Open positions are marked `OPEN_AT_END` and `CENSORED_AT_DATASET_END`; realized PnL uses CLOSED lifecycles only.").unwrap();
    writeln!(output, "- Candle-level health traces are secondary diagnostics; lifecycle/period is the primary unit.").unwrap();
    for warning in &report.warnings {
        writeln!(output, "- {warning}").unwrap();
    }

    writeln!(output, "\n## 20. Audit Appendix\n").unwrap();
    writeln!(
        output,
        "- Validation engine: {}",
        batch.validation_engine_version
    )
    .unwrap();
    writeln!(
        output,
        "- Lifecycle config: {}",
        batch.lifecycle_config_version
    )
    .unwrap();
    writeln!(
        output,
        "- Deterioration config: {}",
        batch.deterioration_config_version
    )
    .unwrap();
    writeln!(
        output,
        "- Hold Diagnostics: {}",
        batch.hold_diagnostics_version
    )
    .unwrap();
    writeln!(output, "- Trigger: {}", report.audit.trigger_version).unwrap();
    writeln!(
        output,
        "- Position sizing: {}",
        report.audit.position_sizing_version
    )
    .unwrap();
    writeln!(
        output,
        "- Risk policy: {}",
        report.audit.risk_policy_version
    )
    .unwrap();
    writeln!(
        output,
        "- Execution model: {}",
        report.audit.execution_model_version
    )
    .unwrap();
    writeln!(
        output,
        "- Fees / slippage: {:.4}% / {:.4}%",
        report.audit.fee_pct, report.audit.slippage_pct
    )
    .unwrap();
    writeln!(
        output,
        "- Temporal semantics: AVAILABLE AT T is isolated from POST-DECISION outcomes."
    )
    .unwrap();
    for period in &report.periods {
        let quality = quality_for(report, &period.dataset_id)
            .map(|item| item.status.as_str())
            .unwrap_or("UNKNOWN");
        writeln!(
            output,
            "- {} -> experiment `{}`; dataset `{}`; lifecycle run `{}`; quality {}",
            period.source_role,
            period.experiment_id,
            period.dataset_id,
            period.lifecycle_run_id,
            quality
        )
        .unwrap();
    }
    output
}

fn html_table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let head = headers
        .iter()
        .map(|value| format!("<th>{}</th>", escape_html(value)))
        .collect::<String>();
    let body = rows
        .into_iter()
        .map(|row| {
            let cells = row
                .into_iter()
                .map(|value| format!("<td>{}</td>", escape_html(&value)))
                .collect::<String>();
            format!("<tr>{cells}</tr>")
        })
        .collect::<String>();
    format!("<table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table>")
}

pub fn html(report: &MultiPeriodLifecycleValidationReport) -> String {
    let sample = &report.sample;
    let quality = html_table(
        &[
            "Dataset",
            "Status",
            "Candles",
            "Sessions",
            "Duplicates",
            "Invalid OHLC",
            "Gaps",
            "Source",
        ],
        report
            .quality
            .iter()
            .map(|item| {
                vec![
                    item.dataset_name.clone(),
                    item.status.clone(),
                    item.candle_count.to_string(),
                    item.session_count.to_string(),
                    item.duplicate_timestamp_count.to_string(),
                    (item.invalid_high_count + item.invalid_low_count).to_string(),
                    item.unexpected_gap_count.to_string(),
                    if item.source_file_available {
                        "AVAILABLE".into()
                    } else {
                        "UNAVAILABLE - persisted DB used".into()
                    },
                ]
            })
            .collect(),
    );
    let periods = html_table(
        &[
            "Role",
            "Dataset",
            "Range",
            "N",
            "Closed/Open",
            "PnL P50",
            "MFE P50",
            "MAE P50",
            "Giveback P50",
            "Late lifecycle",
        ],
        report
            .periods
            .iter()
            .map(|item| {
                vec![
                    item.source_role.clone(),
                    item.dataset_name.clone(),
                    format!("{} to {}", item.start_at, item.end_at),
                    item.lifecycle_count.to_string(),
                    format!("{}/{}", item.closed_count, item.open_count),
                    decimal(item.pnl.median),
                    decimal(item.mfe.median),
                    decimal(item.mae.median),
                    decimal(item.giveback.median),
                    format!("{:.2}%", item.lifecycle_late_reduction_rate_pct),
                ]
            })
            .collect(),
    );
    let lifecycles = html_table(
        &[
            "Lifecycle",
            "Status",
            "Duration",
            "PnL",
            "MFE",
            "MAE",
            "Giveback",
            "Worst health",
            "Max deterioration",
            "Response",
        ],
        report
            .lifecycles
            .iter()
            .map(|item| {
                vec![
                    item.lifecycle_id.clone(),
                    format!(
                        "{}{}",
                        item.status,
                        if item.censored_at_dataset_end {
                            " / CENSORED"
                        } else {
                            ""
                        }
                    ),
                    format!("{} min", item.duration_minutes),
                    optional(item.pnl_pct),
                    decimal(item.mfe_pct),
                    decimal(item.mae_pct),
                    decimal(item.giveback_pct),
                    item.worst_health.clone(),
                    format!(
                        "{} / {:.3}",
                        item.max_deterioration_level, item.max_deterioration_score
                    ),
                    item.response_status.clone(),
                ]
            })
            .collect(),
    );
    let consistency = html_table(
        &["Metric", "Median", "Min", "Max", "Dispersion"],
        report
            .consistency
            .iter()
            .map(|item| {
                vec![
                    item.label.clone(),
                    decimal(item.median),
                    decimal(item.minimum),
                    decimal(item.maximum),
                    decimal(item.dispersion),
                ]
            })
            .collect(),
    );
    let evidence = html_table(
        &["Dimension", "Classification"],
        vec![
            vec!["Sample Size".into(), report.evidence.sample_size.clone()],
            vec![
                "Period Coverage".into(),
                report.evidence.period_coverage.clone(),
            ],
            vec![
                "Late Reduction".into(),
                report.evidence.late_reduction.clone(),
            ],
            vec![
                "Deterioration Stability".into(),
                report.evidence.deterioration_stability.clone(),
            ],
            vec![
                "Health Outcome Relation".into(),
                report.evidence.health_outcome_relation.clone(),
            ],
            vec![
                "Component Consistency".into(),
                report.evidence.component_consistency.clone(),
            ],
            vec![
                "Giveback Consistency".into(),
                report.evidence.giveback_consistency.clone(),
            ],
        ],
    );
    let warnings = report
        .warnings
        .iter()
        .map(|item| format!("<li>{}</li>", escape_html(item)))
        .collect::<String>();
    let appendix = report.periods.iter().map(|period|format!("<li>{}: experiment <code>{}</code>; dataset <code>{}</code>; lifecycle run <code>{}</code></li>",escape_html(&period.source_role),escape_html(&period.experiment_id),escape_html(&period.dataset_id),escape_html(&period.lifecycle_run_id))).collect::<String>();
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>{title}</title><style>
@page{{size:A4 landscape;margin:13mm 12mm}}*{{box-sizing:border-box}}body{{font-family:Arial,sans-serif;color:#10222b;font-size:10pt;line-height:1.4;margin:0}}h1{{font-size:25pt;color:#003b4d;margin:0 0 8px}}h2{{font-size:15pt;color:#006b83;border-bottom:2px solid #24cce5;padding-bottom:4px;margin:18px 0 8px}}h3{{font-size:11pt}}.cover{{min-height:165mm;display:flex;flex-direction:column;justify-content:center;background:#001c25;color:#e8fbff;padding:22mm;page-break-after:always}}.cover h1{{color:#4ce6ff}}.cover strong{{color:#64ffbd}}.cards{{display:grid;grid-template-columns:repeat(4,1fr);gap:8px}}.card{{border:1px solid #8db6c0;background:#eefafd;padding:9px}}.card b{{display:block;font-size:16pt;color:#006b83}}table{{width:100%;border-collapse:collapse;margin:7px 0 13px;font-size:8.5pt;page-break-inside:auto}}tr{{page-break-inside:avoid}}th{{background:#00485b;color:white;text-align:left}}th,td{{border:1px solid #aac7ce;padding:5px;vertical-align:top}}tbody tr:nth-child(even){{background:#f1f8fa}}section{{page-break-inside:auto}}code{{font-family:Consolas,monospace;font-size:8pt}}.warning{{border-left:4px solid #d89b00;background:#fff8dd;padding:8px}}footer{{margin-top:18px;color:#53747c;font-size:8pt}}
</style></head><body>
<section class="cover"><div>AZRIEL // MARKET LAB v0.5.4</div><h1>Multi-Period Lifecycle Validation Report</h1><p><strong>{batch_name}</strong></p><p>{asset} / {timeframe}<br>{coverage_start} to {coverage_end}<br>Status: {status}</p><p>Lifecycle and period are the primary units. No operational rule or agent optimization is introduced.</p></section>
<section><h2>2. Executive Technical Summary</h2><div class="cards"><div class="card">Periods<b>{period_n}</b></div><div class="card">Lifecycles<b>{lifecycle_n}</b></div><div class="card">Closed / Open / Censored<b>{closed} / {open} / {censored}</b></div><div class="card">Sample status<b>{sample_status}</b></div></div><p>Lifecycle late reduction: <b>{late_lifecycle:.2}%</b>. Event late reduction: <b>{late_event:.2}%</b>. Score stability: <b>{score}</b>.</p></section>
<section><h2>3. Dataset Quality</h2>{quality}</section><section><h2>4. Temporal Coverage</h2>{periods}</section>
<section><h2>5. Execution / Resource Audit</h2><div class="cards"><div class="card">Reused experiments<b>{reused}</b></div><div class="card">Rebuilt artifacts<b>{rebuilt}</b></div><div class="card">New LLM runs<b>{llm}</b></div><div class="card">Duration<b>{duration} ms</b></div></div><p>Model: <code>{model}</code> | Prompt: <code>{prompt}</code> | Context: <code>{context}</code></p></section>
<section><h2>6. Sample Overview</h2><p>Datasets {datasets}; periods {period_n}; experiments {experiments}; lifecycles {lifecycle_n}; CLOSED {closed}; OPEN {open}; CENSORED {censored}.</p></section>
<section><h2>7. Period Results</h2>{periods}</section><section><h2>8. Lifecycle Results</h2>{lifecycles}</section>
<section><h2>9. Late Reduction - Event vs Lifecycle</h2><p>Event rate and lifecycle rate remain separate. The consolidated event rate is {late_event:.2}% and the lifecycle rate is {late_lifecycle:.2}%.</p>{periods}</section>
<section><h2>10. Response Delay</h2><p>Missing responses remain UNRESOLVED and are never replaced by lifecycle duration.</p>{lifecycles}</section>
<section><h2>11. Position Health</h2><p>Worst and dominant health are aggregated once per lifecycle. Candle-level traces remain secondary diagnostics.</p></section>
<section><h2>12. Deterioration</h2><p>Maximum deterioration and first HIGH/CRITICAL timestamps are descriptive and do not trigger actions.</p></section>
<section><h2>13. Component Analysis</h2><p>Only persisted V1 components are used. Weights remain frozen.</p></section>
<section><h2>14. Giveback Analysis</h2><p>Giveback is observed independently from the aggregate score and is not converted into an operational rule.</p></section>
<section><h2>15. Cross-Period Stability</h2>{consistency}</section><section><h2>16. Score Stability</h2><p><b>{score}</b>. With fewer than four lifecycle observations or two periods, the result is INSUFFICIENT_SAMPLE.</p></section>
<section><h2>17. Outlier / Concentration Analysis</h2><p>No lifecycle is removed automatically. Sensitivity results remain in the structured JSON and Markdown report.</p></section>
<section><h2>18. Evidence Matrix</h2>{evidence}</section><section><h2>19. Limitations</h2><div class="warning"><ul>{warnings}</ul></div></section>
<section><h2>20. Audit Appendix</h2><p>Batch <code>{batch_id}</code><br>Validation <code>{validation}</code><br>Lifecycle <code>{lifecycle_config}</code><br>Deterioration <code>{deterioration_config}</code><br>Hold diagnostics <code>{hold}</code><br>Trigger <code>{trigger}</code><br>Risk <code>{risk}</code><br>Execution <code>{execution}</code></p><ul>{appendix}</ul><p>AVAILABLE AT T and POST-DECISION data remain explicitly separated.</p></section><footer>AZRIEL Market Lab - generated from persisted local artifacts.</footer></body></html>"#,
        title = REPORT_STEM,
        batch_name = escape_html(&report.batch.name),
        asset = escape_html(&report.batch.asset),
        timeframe = escape_html(&report.batch.timeframe),
        coverage_start = escape_html(&sample.date_coverage_start),
        coverage_end = escape_html(&sample.date_coverage_end),
        status = escape_html(&report.batch.status),
        period_n = sample.periods,
        lifecycle_n = sample.lifecycles,
        closed = sample.closed,
        open = sample.open,
        censored = sample.censored,
        sample_status = escape_html(&sample.sample_status),
        late_lifecycle = sample.lifecycle_late_reduction_rate_pct,
        late_event = sample.event_late_reduction_rate_pct,
        score = escape_html(&sample.score_stability),
        quality = quality,
        periods = periods,
        reused = report.audit.reused_experiments,
        rebuilt = report.audit.rebuilt_artifacts,
        llm = report.audit.new_llm_runs,
        duration = report.audit.duration_ms,
        model = escape_html(&report.audit.model),
        prompt = escape_html(&report.audit.prompt_version),
        context = escape_html(&report.audit.context_version),
        datasets = sample.datasets,
        experiments = sample.experiments,
        lifecycles = lifecycles,
        consistency = consistency,
        evidence = evidence,
        warnings = warnings,
        batch_id = escape_html(&report.batch.batch_id),
        validation = escape_html(&report.batch.validation_engine_version),
        lifecycle_config = escape_html(&report.batch.lifecycle_config_version),
        deterioration_config = escape_html(&report.batch.deterioration_config_version),
        hold = escape_html(&report.batch.hold_diagnostics_version),
        trigger = escape_html(&report.audit.trigger_version),
        risk = escape_html(&report.audit.risk_policy_version),
        execution = escape_html(&report.audit.execution_model_version),
        appendix = appendix
    )
}

fn period_csv(report: &MultiPeriodLifecycleValidationReport) -> String {
    let mut output = "period_id,source_role,dataset_id,dataset_name,start_at,end_at,candles,sessions,lifecycles,closed,open,late_event_rate_pct,late_lifecycle_rate_pct,median_response_delay,quality_status\n".to_string();
    for item in &report.periods {
        let quality = quality_for(report, &item.dataset_id)
            .map(|value| value.status.as_str())
            .unwrap_or("UNKNOWN");
        writeln!(
            output,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            escape_csv(&item.period_id),
            escape_csv(&item.source_role),
            escape_csv(&item.dataset_id),
            escape_csv(&item.dataset_name),
            escape_csv(&item.start_at),
            escape_csv(&item.end_at),
            item.candle_count,
            item.session_count,
            item.lifecycle_count,
            item.closed_count,
            item.open_count,
            item.event_late_reduction_rate_pct,
            item.lifecycle_late_reduction_rate_pct,
            item.response_delay.median,
            escape_csv(quality)
        )
        .unwrap();
    }
    output
}

fn lifecycle_csv(report: &MultiPeriodLifecycleValidationReport) -> String {
    let mut output = "period_id,lifecycle_id,status,censored,entry_at,exit_at,duration_minutes,pnl_pct,mfe_pct,mae_pct,giveback_pct,worst_health,dominant_health,max_deterioration_level,max_deterioration_score,late_reduction_events,response_status,response_delay_minutes\n".to_string();
    for item in &report.lifecycles {
        writeln!(
            output,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            escape_csv(&item.period_id),
            escape_csv(&item.lifecycle_id),
            escape_csv(&item.status),
            item.censored_at_dataset_end,
            escape_csv(&item.entry_at),
            escape_csv(item.exit_at.as_deref().unwrap_or("")),
            item.duration_minutes,
            item.pnl_pct.map(|v| v.to_string()).unwrap_or_default(),
            item.mfe_pct,
            item.mae_pct,
            item.giveback_pct,
            escape_csv(&item.worst_health),
            escape_csv(&item.dominant_health),
            escape_csv(&item.max_deterioration_level),
            item.max_deterioration_score,
            item.late_reduction_events,
            escape_csv(&item.response_status),
            item.response_delay_minutes
                .map(|v| v.to_string())
                .unwrap_or_default()
        )
        .unwrap();
    }
    output
}

fn late_reduction_csv(report: &MultiPeriodLifecycleValidationReport) -> String {
    let mut output = "period_id,lifecycle_id,event_count,has_late_reduction,response_status,response_delay_minutes\n".to_string();
    for item in report
        .lifecycles
        .iter()
        .filter(|item| item.has_late_reduction)
    {
        writeln!(
            output,
            "{},{},{},{},{},{}",
            escape_csv(&item.period_id),
            escape_csv(&item.lifecycle_id),
            item.late_reduction_events,
            item.has_late_reduction,
            escape_csv(&item.response_status),
            item.response_delay_minutes
                .map(|v| v.to_string())
                .unwrap_or_default()
        )
        .unwrap();
    }
    output
}

fn component_csv(report: &MultiPeriodLifecycleValidationReport) -> String {
    let mut output = "period_id,component,lifecycle_count,average_value,maximum_value,average_pnl_pct,average_mae_pct,average_giveback_pct,late_reduction_rate_pct\n".to_string();
    for item in &report.components {
        writeln!(
            output,
            "{},{},{},{},{},{},{},{},{}",
            escape_csv(item.period_id.as_deref().unwrap_or("ALL")),
            escape_csv(&item.component),
            item.lifecycle_count,
            item.average_value,
            item.maximum_value,
            item.average_pnl_pct
                .map(|v| v.to_string())
                .unwrap_or_default(),
            item.average_mae_pct,
            item.average_giveback_pct,
            item.late_reduction_rate_pct
        )
        .unwrap();
    }
    output
}

fn evidence_csv(report: &MultiPeriodLifecycleValidationReport) -> String {
    let rows = [
        ("sample_size", &report.evidence.sample_size),
        ("period_coverage", &report.evidence.period_coverage),
        ("late_reduction", &report.evidence.late_reduction),
        (
            "deterioration_stability",
            &report.evidence.deterioration_stability,
        ),
        (
            "health_outcome_relation",
            &report.evidence.health_outcome_relation,
        ),
        (
            "component_consistency",
            &report.evidence.component_consistency,
        ),
        (
            "giveback_consistency",
            &report.evidence.giveback_consistency,
        ),
    ];
    let mut output = "dimension,classification\n".to_string();
    for (dimension, value) in rows {
        writeln!(output, "{},{}", escape_csv(dimension), escape_csv(value)).unwrap();
    }
    output
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content)
        .map_err(|error| format!("falha ao escrever {}: {error}", path.display()))
}

pub fn write_structured_bundle(
    report: &MultiPeriodLifecycleValidationReport,
    output_dir: &Path,
) -> Result<ValidationReportArtifacts, String> {
    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    let artifacts = ValidationReportArtifacts {
        markdown: output_dir.join(format!("{REPORT_STEM}.md")),
        html: output_dir.join(format!("{REPORT_STEM}.html")),
        json: output_dir.join(format!("{REPORT_STEM}.json")),
        period_csv: output_dir.join("period_summary.csv"),
        lifecycle_csv: output_dir.join("lifecycle_summary.csv"),
        late_reduction_csv: output_dir.join("late_reduction_events.csv"),
        component_csv: output_dir.join("component_analysis.csv"),
        evidence_csv: output_dir.join("evidence_matrix.csv"),
        warnings_json: output_dir.join("warnings.json"),
    };
    write_file(&artifacts.markdown, &markdown(report))?;
    write_file(&artifacts.html, &html(report))?;
    write_file(
        &artifacts.json,
        &serde_json::to_string_pretty(report).map_err(|error| error.to_string())?,
    )?;
    write_file(&artifacts.period_csv, &period_csv(report))?;
    write_file(&artifacts.lifecycle_csv, &lifecycle_csv(report))?;
    write_file(&artifacts.late_reduction_csv, &late_reduction_csv(report))?;
    write_file(&artifacts.component_csv, &component_csv(report))?;
    write_file(&artifacts.evidence_csv, &evidence_csv(report))?;
    write_file(
        &artifacts.warnings_json,
        &serde_json::to_string_pretty(&report.warnings).map_err(|error| error.to_string())?,
    )?;
    validate_structured_bundle(report, &artifacts)?;
    Ok(artifacts)
}

pub fn validate_structured_bundle(
    report: &MultiPeriodLifecycleValidationReport,
    artifacts: &ValidationReportArtifacts,
) -> Result<(), String> {
    for path in [
        &artifacts.markdown,
        &artifacts.html,
        &artifacts.json,
        &artifacts.period_csv,
        &artifacts.lifecycle_csv,
        &artifacts.late_reduction_csv,
        &artifacts.component_csv,
        &artifacts.evidence_csv,
        &artifacts.warnings_json,
    ] {
        if fs::metadata(path).map_err(|error| error.to_string())?.len() == 0 {
            return Err(format!("artefato vazio: {}", path.display()));
        }
    }
    let markdown = fs::read_to_string(&artifacts.markdown).map_err(|error| error.to_string())?;
    for required in [
        "## 18. Evidence Matrix",
        "## 19. Limitations",
        "## 20. Audit Appendix",
    ] {
        if !markdown.contains(required) {
            return Err(format!("seção obrigatória ausente: {required}"));
        }
    }
    let loaded: MultiPeriodLifecycleValidationReport = serde_json::from_str(
        &fs::read_to_string(&artifacts.json).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if loaded.periods.len() != report.periods.len()
        || loaded.lifecycles.len() != report.lifecycles.len()
    {
        return Err("contagens do relatório divergentes do batch".into());
    }
    Ok(())
}

pub fn register_artifact(
    connection: &Connection,
    batch_id: &str,
    artifact_type: &str,
    path: &Path,
) -> Result<(), String> {
    let byte_size = fs::metadata(path).map_err(|error| error.to_string())?.len();
    connection.execute(
        "INSERT INTO market_lifecycle_validation_artifacts(batch_id,artifact_type,path,byte_size,checksum) VALUES(?1,?2,?3,?4,NULL)
         ON CONFLICT(batch_id,artifact_type) DO UPDATE SET path=excluded.path,byte_size=excluded.byte_size,created_at=CURRENT_TIMESTAMP",
        params![batch_id,artifact_type,path.to_string_lossy(),byte_size],
    ).map_err(|error|error.to_string())?;
    Ok(())
}

pub fn register_structured_bundle(
    connection: &Connection,
    batch_id: &str,
    artifacts: &ValidationReportArtifacts,
) -> Result<(), String> {
    for (kind, path) in [
        ("REPORT_MD", &artifacts.markdown),
        ("REPORT_HTML", &artifacts.html),
        ("REPORT_JSON", &artifacts.json),
        ("PERIOD_SUMMARY_CSV", &artifacts.period_csv),
        ("LIFECYCLE_SUMMARY_CSV", &artifacts.lifecycle_csv),
        ("LATE_REDUCTION_EVENTS_CSV", &artifacts.late_reduction_csv),
        ("COMPONENT_ANALYSIS_CSV", &artifacts.component_csv),
        ("EVIDENCE_MATRIX_CSV", &artifacts.evidence_csv),
        ("WARNINGS_JSON", &artifacts.warnings_json),
    ] {
        register_artifact(connection, batch_id, kind, path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::market_lifecycle_validation::{
        EvidenceMatrix, LifecycleSampleSummary, LifecycleValidationBatch, ValidationAuditSummary,
    };

    #[test]
    fn markdown_contains_required_runbook_sections() {
        let report = MultiPeriodLifecycleValidationReport {
            batch: LifecycleValidationBatch {
                batch_id: "b".into(),
                name: "Batch".into(),
                status: "PARTIAL".into(),
                asset: "AAPL".into(),
                timeframe: "15M".into(),
                agent_id: "ai-intraday-v1".into(),
                agent_version: "AI_INTRADAY_V1".into(),
                lifecycle_config_version: "POSITION_LIFECYCLE_CONFIG_V1".into(),
                deterioration_config_version: "POSITION_DETERIORATION_CONFIG_V1".into(),
                hold_diagnostics_version: "HOLD_DIAGNOSTICS_V1".into(),
                execution_model_version: "EXECUTION_MODEL_V1".into(),
                validation_engine_version: "MULTI_PERIOD_LIFECYCLE_VALIDATION_V1".into(),
                dataset_count: 0,
                experiment_count: 0,
                lifecycle_count: 0,
                created_at: "2026-01-01".into(),
                started_at: None,
                completed_at: None,
            },
            sample: LifecycleSampleSummary::default(),
            periods: vec![],
            lifecycles: vec![],
            deterioration_by_period: vec![],
            deterioration_outcomes: vec![],
            health_outcomes: vec![],
            components: vec![],
            consistency: vec![],
            evidence: EvidenceMatrix::default(),
            outlier_sensitivity: vec![],
            quality: vec![],
            audit: ValidationAuditSummary::default(),
            warnings: vec!["INSUFFICIENT_SAMPLE".into()],
            post_decision_only: true,
        };
        let output = markdown(&report);
        for section in 1..=20 {
            assert!(
                output.contains(&format!("## {section}.")),
                "missing section {section}"
            );
        }
        assert!(output.contains("New LLM runs: 0"));
    }
}
