use azriel_lib::database::{
    self,
    market_lifecycle_validation::{LifecycleValidationBatchInput, LifecycleValidationPeriodInput},
    market_lifecycle_validation_report::{
        register_artifact, register_structured_bundle, write_structured_bundle, REPORT_STEM,
    },
    market_lifecycle_validation_repository,
};
use rusqlite::OptionalExtension;
use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

#[derive(Debug)]
struct Arguments {
    database: PathBuf,
    output: PathBuf,
    batch_name: String,
    dev_experiment: String,
    oos_experiment: String,
}

fn value(arguments: &[String], flag: &str) -> Result<String, String> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
        .ok_or_else(|| format!("argumento obrigatório ausente: {flag}"))
}

fn arguments() -> Result<Arguments, String> {
    let raw = env::args().skip(1).collect::<Vec<_>>();
    Ok(Arguments {
        database: PathBuf::from(value(&raw, "--database")?),
        output: PathBuf::from(value(&raw, "--output")?),
        batch_name: value(&raw, "--batch-name")?,
        dev_experiment: value(&raw, "--dev")?,
        oos_experiment: value(&raw, "--oos")?,
    })
}

fn browser_path() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(root) = env::var_os("PROGRAMFILES") {
        candidates.push(PathBuf::from(root).join("Google/Chrome/Application/chrome.exe"));
    }
    if let Some(root) = env::var_os("PROGRAMFILES(X86)") {
        candidates.push(PathBuf::from(root).join("Google/Chrome/Application/chrome.exe"));
    }
    if let Some(root) = env::var_os("PROGRAMFILES(X86)") {
        candidates.push(PathBuf::from(root).join("Microsoft/Edge/Application/msedge.exe"));
    }
    if let Some(root) = env::var_os("PROGRAMFILES") {
        candidates.push(PathBuf::from(root).join("Microsoft/Edge/Application/msedge.exe"));
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn file_url(path: &Path) -> Result<String, String> {
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    let canonical_text = canonical.to_string_lossy();
    let windows_path = canonical_text
        .strip_prefix(r"\\?\")
        .unwrap_or(&canonical_text);
    Ok(format!(
        "file:///{}",
        windows_path
            .replace('\\', "/")
            .replace(' ', "%20")
            .replace('#', "%23")
    ))
}

fn create_pdf(html: &Path, pdf: &Path, output: &Path) -> Result<(), String> {
    let browser = browser_path()
        .ok_or_else(|| "Chrome ou Microsoft Edge não encontrado para gerar PDF".to_string())?;
    let profile = env::temp_dir().join(format!(
        "azriel-validation-report-profile-{}",
        std::process::id()
    ));
    let pending_pdf = output.join(format!(".{REPORT_STEM}-{}.partial.pdf", std::process::id()));
    let command_file = output.join(format!(".validation-report-{}.cmd", std::process::id()));
    let _ = fs::remove_file(&pending_pdf);
    fs::create_dir_all(&profile).map_err(|error| error.to_string())?;
    let command_line = format!(
        "\"{}\" --headless=new --no-first-run --disable-extensions --disable-gpu \
         --allow-file-access-from-files --run-all-compositor-stages-before-draw \
         --virtual-time-budget=3000 --no-pdf-header-footer \
         \"--user-data-dir={}\" \"--print-to-pdf={}\" \"{}\"",
        browser.display(),
        profile.display(),
        pending_pdf.display(),
        file_url(html)?.replace('%', "%%")
    );
    fs::write(
        &command_file,
        format!("@echo off\r\n{}\r\nexit /b %errorlevel%\r\n", command_line),
    )
    .map_err(|error| error.to_string())?;
    let status = Command::new("cmd.exe")
        .args(["/d", "/s", "/c", "call"])
        .arg(&command_file)
        .status()
        .map_err(|error| format!("falha ao iniciar o renderizador PDF: {error}"))?;
    if !status.success() {
        let _ = fs::remove_file(&pending_pdf);
        let _ = fs::remove_file(&command_file);
        let _ = fs::remove_dir_all(&profile);
        return Err(format!("renderizador PDF encerrou com {status}"));
    }

    // Chromium may finish the parent process before its renderer has flushed all pages.
    thread::sleep(Duration::from_millis(1_500));
    let bytes = fs::read(&pending_pdf).map_err(|error| error.to_string())?;
    let page_marker = b"/Type /Page";
    let page_count = bytes
        .windows(page_marker.len() + 1)
        .filter(|window| window.starts_with(page_marker) && window[page_marker.len()] != b's')
        .count();
    let valid = bytes.len() >= 10_000
        && bytes.starts_with(b"%PDF-")
        && bytes.ends_with(b"%%EOF\n")
        && page_count >= 2;
    if !valid {
        let _ = fs::remove_file(&pending_pdf);
        return Err(format!(
            "PDF gerado é inválido ou incompleto ({} bytes, {} página(s))",
            bytes.len(),
            page_count
        ));
    }

    if pdf.exists() {
        fs::remove_file(pdf).map_err(|error| error.to_string())?;
    }
    fs::rename(&pending_pdf, pdf).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&command_file);
    let _ = fs::remove_dir_all(&profile);
    Ok(())
}

fn run() -> Result<(), String> {
    let arguments = arguments()?;
    let mut connection = database::open(&arguments.database)?;
    let existing = connection
        .query_row(
            "SELECT id FROM market_lifecycle_validation_batches WHERE name=?1 ORDER BY created_at DESC LIMIT 1",
            [&arguments.batch_name],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let report = if let Some(batch_id) = existing {
        market_lifecycle_validation_repository::resume(&mut connection, &batch_id)?
    } else {
        market_lifecycle_validation_repository::create(
            &mut connection,
            &LifecycleValidationBatchInput {
                name: arguments.batch_name,
                periods: vec![
                    LifecycleValidationPeriodInput {
                        experiment_id: arguments.dev_experiment,
                        source_role: "DEVELOPMENT".into(),
                    },
                    LifecycleValidationPeriodInput {
                        experiment_id: arguments.oos_experiment,
                        source_role: "OOS".into(),
                    },
                ],
            },
        )?
    };
    let artifacts = write_structured_bundle(&report, &arguments.output)?;
    register_structured_bundle(&connection, &report.batch.batch_id, &artifacts)?;
    let pdf = arguments.output.join(format!("{REPORT_STEM}.pdf"));
    create_pdf(&artifacts.html, &pdf, &arguments.output)?;
    register_artifact(&connection, &report.batch.batch_id, "REPORT_PDF", &pdf)?;

    let structured = vec![
        artifacts.json,
        artifacts.period_csv,
        artifacts.lifecycle_csv,
        artifacts.late_reduction_csv,
        artifacts.component_csv,
        artifacts.evidence_csv,
        artifacts.warnings_json,
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "validationBatch": report.batch.name,
            "batchId": report.batch.batch_id,
            "status": report.batch.status,
            "periods": report.sample.periods,
            "lifecycles": report.sample.lifecycles,
            "closed": report.sample.closed,
            "open": report.sample.open,
            "censored": report.sample.censored,
            "reusedExperiments": report.audit.reused_experiments,
            "newLlmRuns": report.audit.new_llm_runs,
            "failedPeriods": report.audit.failed_periods,
            "reportPdf": pdf,
            "reportMd": artifacts.markdown,
            "structuredArtifacts": structured,
            "warnings": report.warnings,
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("VALIDATION FAILED: {error}");
        std::process::exit(1);
    }
}
