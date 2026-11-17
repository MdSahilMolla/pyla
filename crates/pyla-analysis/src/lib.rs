use pyla_frontend::{Diagnostic, Registry, Severity};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub files_analyzed: usize,
    pub languages: BTreeMap<String, usize>,
    pub diagnostics: Vec<Diagnostic>,
}

impl AnalysisReport {
    pub fn errors(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }
    pub fn warnings(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }
    pub fn suggestions(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Suggestion)
            .count()
    }
}

pub fn analyze_path(root: &Path, registry: &Registry) -> io::Result<AnalysisReport> {
    if !root.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("path does not exist: {}", root.display()),
        ));
    }
    let mut report = AnalysisReport::default();
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    for path in files {
        let Some(adapter) = registry.adapter_for(&path) else {
            continue;
        };
        let source = fs::read_to_string(&path)?;
        let program = adapter
            .parse(&source, &path)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        *report
            .languages
            .entry(adapter.language_id().to_string())
            .or_default() += 1;
        report.files_analyzed += 1;
        report
            .diagnostics
            .extend(adapter.analyze(&source, &program));
    }
    Ok(report)
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if child.is_dir() {
            collect_files(&child, files)?;
        } else {
            files.push(child);
        }
    }
    files.sort();
    Ok(())
}
