use serde::Serialize;
use std::{path::Path, process::Command};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub date: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub available: bool,
    pub repository: bool,
    pub branch: Option<String>,
    pub clean: bool,
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub untracked: Vec<String>,
    pub last_commit: Option<GitCommit>,
    pub recent_commits: Vec<GitCommit>,
    pub error: Option<String>,
}

pub fn inspect(path: &Path) -> GitStatus {
    let mut status = empty_status();
    let output = match run_git(
        path,
        &[
            "status",
            "--porcelain=v1",
            "--branch",
            "--untracked-files=normal",
        ],
    ) {
        Ok(output) => output,
        Err(error) => {
            status.error = Some(error);
            return status;
        }
    };
    status.available = true;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        status.error = Some(if stderr.contains("not a git repository") {
            "A pasta não é um repositório Git".into()
        } else {
            "Não foi possível consultar o repositório Git".into()
        });
        return status;
    }
    status.repository = true;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(branch) = line.strip_prefix("## ") {
            status.branch = Some(
                branch
                    .split("...")
                    .next()
                    .unwrap_or(branch)
                    .trim()
                    .to_string(),
            );
            continue;
        }
        if line.len() < 3 {
            continue;
        }
        let code = &line[..2];
        let file = line[3..].trim().to_string();
        if code == "??" {
            status.untracked.push(file);
        } else if code.contains('D') {
            status.removed.push(file);
        } else if code.contains('A') {
            status.added.push(file);
        } else {
            status.modified.push(file);
        }
    }
    status.clean = status.modified.is_empty()
        && status.added.is_empty()
        && status.removed.is_empty()
        && status.untracked.is_empty();
    if let Ok(log) = run_git(
        path,
        &["log", "-5", "--format=%H%x1f%h%x1f%s%x1f%aI%x1f%an"],
    ) {
        if log.status.success() {
            status.recent_commits = parse_commits(&String::from_utf8_lossy(&log.stdout));
            status.last_commit = status.recent_commits.first().cloned();
        }
    }
    status
}

fn run_git(path: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .map_err(|_| "Git não está instalado ou não foi encontrado no PATH".into())
}

fn parse_commits(value: &str) -> Vec<GitCommit> {
    value
        .lines()
        .filter_map(|line| {
            let parts = line.split('\u{1f}').collect::<Vec<_>>();
            (parts.len() == 5).then(|| GitCommit {
                hash: parts[0].into(),
                short_hash: parts[1].into(),
                subject: parts[2].into(),
                date: parts[3].into(),
                author: parts[4].into(),
            })
        })
        .collect()
}

fn empty_status() -> GitStatus {
    GitStatus {
        available: false,
        repository: false,
        branch: None,
        clean: true,
        modified: vec![],
        added: vec![],
        removed: vec![],
        untracked: vec![],
        last_commit: None,
        recent_commits: vec![],
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_parser_ignores_invalid_lines() {
        let commits = parse_commits(
            "abcdef\u{1f}abc\u{1f}Release\u{1f}2026-08-31T10:00:00Z\u{1f}Azriel\ninvalid",
        );
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].short_hash, "abc");
    }
}
