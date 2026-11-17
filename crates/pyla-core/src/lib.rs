use pyla_analysis::{analyze_path, AnalysisReport};
use pyla_frontend::Registry;
use pyla_transform::{
    compare_outputs, detect_python_optimizations, render_stub, DifferentialReport, MigrationPlan,
    OptimizationCandidate, Patch, PatchTransaction, Worktree,
};
use pyla_validate::{validate_project, Language, ValidationResult};
use std::{
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

#[derive(Default)]
pub struct Engine {
    registry: Registry,
}

#[derive(Debug)]
pub enum ExecutionError {
    Patch(pyla_transform::PatchError),
    ValidationUnavailable(String),
    ValidationFailed(String),
    Rollback(pyla_transform::PatchError),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Patch(error) => write!(f, "patch failed: {error}"),
            Self::ValidationUnavailable(detail) => write!(f, "validation unavailable: {detail}"),
            Self::ValidationFailed(detail) => write!(f, "validation failed: {detail}"),
            Self::Rollback(error) => write!(f, "rollback failed: {error}"),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl Engine {
    pub fn analyze(&self, path: &Path) -> io::Result<AnalysisReport> {
        analyze_path(path, &self.registry)
    }

    pub fn migration_plan(&self, path: &Path, target: &str) -> io::Result<Vec<MigrationPlan>> {
        if target.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "target language cannot be empty",
            ));
        }
        let mut plans = Vec::new();
        collect_source_files(path, &mut plans, target, &self.registry)?;
        Ok(plans)
    }

    pub fn validate(&self, path: &Path, language: Language) -> ValidationResult {
        validate_project(path, language)
    }

    pub fn apply_patch(&self, patch: &Patch) -> Result<(), pyla_transform::PatchError> {
        patch.apply()
    }

    pub fn optimization_candidates(
        &self,
        source: &str,
        module: &str,
    ) -> Vec<OptimizationCandidate> {
        detect_python_optimizations(source, module)
    }

    pub fn compare_outputs(&self, cases: Vec<(String, String, String)>) -> DifferentialReport {
        compare_outputs(cases)
    }

    /// Applies a candidate only while validation passes; otherwise restores the exact source.
    pub fn apply_patch_validated(
        &self,
        patch: &Patch,
        project: &Path,
        language: Language,
    ) -> Result<ValidationResult, ExecutionError> {
        let transaction = patch.begin_transaction().map_err(ExecutionError::Patch)?;
        let result = self.validate(project, language);
        match (result.available, result.passed) {
            (true, Some(true)) => {
                transaction.commit();
                Ok(result)
            }
            (false, _) => Self::rollback(
                transaction,
                ExecutionError::ValidationUnavailable(result.detail),
            ),
            (_, _) => Self::rollback(transaction, ExecutionError::ValidationFailed(result.detail)),
        }
    }

    pub fn create_worktree(
        &self,
        repository: &Path,
        path: &Path,
        branch: &str,
    ) -> Result<Worktree, pyla_transform::WorktreeError> {
        Worktree::create(repository, path, branch)
    }

    fn rollback(
        transaction: PatchTransaction,
        error: ExecutionError,
    ) -> Result<ValidationResult, ExecutionError> {
        transaction.rollback().map_err(ExecutionError::Rollback)?;
        Err(error)
    }

    pub fn generate_migration(
        &self,
        path: &Path,
        target: &str,
        output: &Path,
    ) -> io::Result<Vec<std::path::PathBuf>> {
        let extension = match target.to_ascii_lowercase().as_str() {
            "python" | "py" => "py",
            "rust" | "rs" => "rs",
            "cpp" | "c++" | "c" => "cpp",
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unsupported target language: {target}"),
                ))
            }
        };
        if resolve_path(output)?.starts_with(resolve_path(path)?) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "migration output must not be inside the source tree",
            ));
        }
        let plans = self.migration_plan(path, target)?;
        let destinations = plans
            .iter()
            .enumerate()
            .map(|(index, plan)| {
                let stem = plan
                    .source
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("migration");
                output.join(format!("{stem}.{index}.{extension}"))
            })
            .collect::<Vec<_>>();
        // Check every destination before writing so a conflict leaves no partial output.
        if let Some(existing) = destinations.iter().find(|destination| destination.exists()) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("refusing to overwrite {}", existing.display()),
            ));
        }
        std::fs::create_dir_all(output)?;
        for (plan, destination) in plans.iter().zip(&destinations) {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination)?
                .write_all(render_stub(plan).as_bytes())?;
        }
        Ok(destinations)
    }
}

/// Resolves symlinks, `.` and `..` so paths can be compared reliably, even when
/// the trailing part of the path does not exist yet.
fn resolve_path(path: &Path) -> io::Result<PathBuf> {
    let mut resolved = PathBuf::new();
    for component in std::path::absolute(path)?.components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            other => {
                resolved.push(other);
                if resolved.exists() {
                    resolved = resolved.canonicalize()?;
                }
            }
        }
    }
    Ok(resolved)
}

fn collect_source_files(
    path: &Path,
    plans: &mut Vec<MigrationPlan>,
    target: &str,
    registry: &Registry,
) -> io::Result<()> {
    if path.is_file() {
        if let Some(adapter) = registry.adapter_for(path) {
            let source = std::fs::read_to_string(path)?;
            let program = adapter
                .parse(&source, path)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            plans.push(MigrationPlan::from_program(&program, target));
        }
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let child = entry?.path();
        collect_source_files(&child, plans, target, registry)?;
    }
    plans.sort_by(|left, right| left.source.cmp(&right.source));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pyla-core-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("main.rs"), "fn main() {}\n").unwrap();
        dir
    }

    #[test]
    fn migration_output_inside_source_is_rejected_even_via_dot_dot() {
        let source = scratch_dir("inside");
        let output = source.join("sub").join("..").join("out");
        let error = Engine::default()
            .generate_migration(&source, "Python", &output)
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(!source.join("out").exists());
        std::fs::remove_dir_all(source).unwrap();
    }

    #[test]
    fn unsupported_target_creates_no_output_directory() {
        let source = scratch_dir("target");
        let output = source.with_extension("out");
        let error = Engine::default()
            .generate_migration(&source, "Cobol", &output)
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(!output.exists());
        std::fs::remove_dir_all(source).unwrap();
    }

    #[test]
    fn migration_writes_stubs_outside_source() {
        let source = scratch_dir("ok");
        let output = source.with_extension("out");
        let _ = std::fs::remove_dir_all(&output);
        let files = Engine::default()
            .generate_migration(&source, "Python", &output)
            .unwrap();
        assert_eq!(files, vec![output.join("main.0.py")]);
        assert!(std::fs::read_to_string(&files[0])
            .unwrap()
            .contains("def main("));
        std::fs::remove_dir_all(source).unwrap();
        std::fs::remove_dir_all(output).unwrap();
    }
}
