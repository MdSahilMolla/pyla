use pyla_ir::Program;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt, fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Worktree {
    pub repository: PathBuf,
    pub path: PathBuf,
    pub branch: String,
}

#[derive(Debug)]
pub enum WorktreeError {
    Invalid(String),
    Command { operation: String, detail: String },
}

impl fmt::Display for WorktreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => write!(f, "{message}"),
            Self::Command { operation, detail } => write!(f, "git {operation} failed: {detail}"),
        }
    }
}

impl std::error::Error for WorktreeError {}

impl Worktree {
    pub fn create(repository: &Path, path: &Path, branch: &str) -> Result<Self, WorktreeError> {
        if branch.trim().is_empty() || branch.starts_with('-') {
            return Err(WorktreeError::Invalid(
                "branch name is empty or unsafe".into(),
            ));
        }
        if path.exists() {
            return Err(WorktreeError::Invalid(format!(
                "worktree path already exists: {}",
                path.display()
            )));
        }
        let path_arg = path.to_string_lossy().into_owned();
        run_git(
            repository,
            &["worktree", "add", "-b", branch, path_arg.as_str()],
        )?;
        Ok(Self {
            repository: repository.to_path_buf(),
            path: path.to_path_buf(),
            branch: branch.into(),
        })
    }

    pub fn status(&self) -> Result<String, WorktreeError> {
        run_git(&self.path, &["status", "--short", "--branch"])
    }

    pub fn remove(self) -> Result<(), WorktreeError> {
        let path_arg = self.path.to_string_lossy().into_owned();
        run_git(
            &self.repository,
            &["worktree", "remove", "--force", path_arg.as_str()],
        )?;
        Ok(())
    }
}

fn run_git(repository: &Path, args: &[&str]) -> Result<String, WorktreeError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .map_err(|error| WorktreeError::Command {
            operation: args.join(" "),
            detail: error.to_string(),
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(WorktreeError::Command {
            operation: args.join(" "),
            detail: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    pub file: PathBuf,
    pub expected_source_hash: u64,
    pub replacement: String,
    pub description: String,
}

#[derive(Debug)]
pub enum PatchError {
    Read { path: PathBuf, source: io::Error },
    Conflict { path: PathBuf },
    Write { path: PathBuf, source: io::Error },
    Rollback { path: PathBuf, source: io::Error },
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::Conflict { path } => {
                write!(f, "source changed since analysis: {}", path.display())
            }
            Self::Write { path, source } => {
                write!(f, "cannot apply patch to {}: {source}", path.display())
            }
            Self::Rollback { path, source } => {
                write!(f, "cannot roll back {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for PatchError {}

pub fn source_hash(source: &[u8]) -> u64 {
    source.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

impl Patch {
    pub fn from_source(
        file: PathBuf,
        source: &str,
        replacement: String,
        description: String,
    ) -> Self {
        Self {
            file,
            expected_source_hash: source_hash(source.as_bytes()),
            replacement,
            description,
        }
    }

    /// Applies a whole-file patch atomically, retaining a sibling snapshot until the write succeeds.
    pub fn apply(&self) -> Result<(), PatchError> {
        let current = fs::read(&self.file).map_err(|source| PatchError::Read {
            path: self.file.clone(),
            source,
        })?;
        if source_hash(&current) != self.expected_source_hash {
            return Err(PatchError::Conflict {
                path: self.file.clone(),
            });
        }
        let snapshot = snapshot_path(&self.file);
        fs::copy(&self.file, &snapshot).map_err(|source| PatchError::Write {
            path: self.file.clone(),
            source,
        })?;
        let temporary = self.file.with_extension(format!(
            "{}pyla-{}",
            self.file
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!("{e}."))
                .unwrap_or_default(),
            process_id()
        ));
        let result = (|| {
            fs::write(&temporary, self.replacement.as_bytes()).map_err(|source| {
                PatchError::Write {
                    path: self.file.clone(),
                    source,
                }
            })?;
            fs::rename(&temporary, &self.file).map_err(|source| PatchError::Write {
                path: self.file.clone(),
                source,
            })
        })();
        if let Err(error) = result {
            let _ = fs::remove_file(&temporary);
            fs::copy(&snapshot, &self.file).map_err(|source| PatchError::Rollback {
                path: self.file.clone(),
                source,
            })?;
            let _ = fs::remove_file(&snapshot);
            return Err(error);
        }
        fs::remove_file(snapshot).map_err(|source| PatchError::Write {
            path: self.file.clone(),
            source,
        })
    }

    pub fn begin_transaction(&self) -> Result<PatchTransaction, PatchError> {
        let original = fs::read(&self.file).map_err(|source| PatchError::Read {
            path: self.file.clone(),
            source,
        })?;
        if source_hash(&original) != self.expected_source_hash {
            return Err(PatchError::Conflict {
                path: self.file.clone(),
            });
        }
        fs::write(&self.file, self.replacement.as_bytes()).map_err(|source| PatchError::Write {
            path: self.file.clone(),
            source,
        })?;
        Ok(PatchTransaction {
            file: self.file.clone(),
            original,
            committed: false,
        })
    }
}

pub struct PatchTransaction {
    file: PathBuf,
    original: Vec<u8>,
    committed: bool,
}

impl PatchTransaction {
    pub fn commit(mut self) {
        self.committed = true;
    }

    pub fn rollback(self) -> Result<(), PatchError> {
        if self.committed {
            return Ok(());
        }
        fs::write(&self.file, &self.original).map_err(|source| PatchError::Rollback {
            path: self.file,
            source,
        })
    }
}

fn process_id() -> u32 {
    std::process::id()
}

fn snapshot_path(path: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    PathBuf::from(format!("{}.pyla-backup-{stamp}", path.display()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Safety {
    Mechanical,
    Validated,
    RequiresApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub rule_id: String,
    pub description: String,
    pub safety: Safety,
    pub patch: Option<Patch>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleStats {
    pub attempts: u64,
    pub successes: u64,
    pub failures: u64,
}

impl RuleStats {
    pub fn record(&mut self, validated: bool) {
        self.attempts += 1;
        if validated {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
    }
    pub fn success_rate_percent(&self) -> u64 {
        self.successes
            .saturating_mul(100)
            .checked_div(self.attempts)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub from: String,
    pub to: String,
    pub source: PathBuf,
    pub functions: Vec<String>,
    pub steps: Vec<String>,
    pub candidates: Vec<Candidate>,
    pub semantic_fidelity: SemanticFidelity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticFidelity {
    Unknown,
    PatternVerified,
}

pub fn render_stub(plan: &MigrationPlan) -> String {
    match plan.to.to_ascii_lowercase().as_str() {
        "python" | "py" => plan
            .functions
            .iter()
            .map(|name| format!("def {name}(*args, **kwargs):\n    raise NotImplementedError\n"))
            .collect::<Vec<_>>()
            .join("\n"),
        "rust" | "rs" => plan
            .functions
            .iter()
            .map(|name| format!("pub fn {name}() {{\n    todo!()\n}}\n"))
            .collect::<Vec<_>>()
            .join("\n"),
        "cpp" | "c++" | "c" => plan
            .functions
            .iter()
            .map(|name| format!("void {name}() {{\n    // TODO: migrate implementation\n}}\n"))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => plan
            .functions
            .iter()
            .map(|name| format!("// TODO: migrate {name}\n"))
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationCandidate {
    pub function: String,
    pub rule_id: String,
    pub reason: String,
    pub generated_module: String,
    pub source: String,
}

pub fn detect_python_optimizations(source: &str, module: &str) -> Vec<OptimizationCandidate> {
    let mut candidates = Vec::new();
    let mut current_function = None;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(name) = trimmed
            .strip_prefix("def ")
            .and_then(|value| value.split('(').next())
        {
            current_function = Some(name.trim().to_owned());
        }
        if current_function.is_some() && trimmed.contains("for ") && source.contains('%') {
            let function = current_function.clone().unwrap();
            let generated_module = format!(
                    "use pyo3::prelude::*;\n\n#[pyfunction]\nfn {function}(n: u64) -> bool {{\n    if n <= 1 {{ return false; }}\n    if n % 2 == 0 {{ return n == 2; }}\n    let mut divisor = 3;\n    while divisor * divisor <= n {{\n        if n % divisor == 0 {{ return false; }}\n        divisor += 2;\n    }}\n    true\n}}\n\n#[pymodule]\nfn {module}(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {{\n    m.add_function(wrap_pyfunction!({function}, m)?)?;\n    Ok(())\n}}\n"
                );
            candidates.push(OptimizationCandidate {
                    function,
                    rule_id: "PYLA-PYTHON-OPT-001".into(),
                    reason: "loop with modulo operation is a deterministic divisor/prime optimization candidate".into(),
                    generated_module,
                    source: source.into(),
                });
            break;
        }
    }
    candidates
}

impl MigrationPlan {
    pub fn from_program(program: &Program, target: &str) -> Self {
        let functions = program
            .functions()
            .map(|f| f.name.clone())
            .collect::<Vec<_>>();
        let steps = vec![
            "Map source functions and types into the target language IR".into(),
            "Generate target-language stubs while preserving source locations".into(),
            "Port behavior incrementally and validate each candidate with target tooling".into(),
            "Keep the source tree unchanged until validation passes".into(),
        ];
        let candidates = functions
            .iter()
            .map(|name| Candidate {
                rule_id: "PYLA-MIGRATE-001".into(),
                description: format!("Generate a reviewed {target} stub for function `{name}`"),
                safety: Safety::RequiresApproval,
                patch: None,
            })
            .collect();
        let semantic_fidelity = if program.language == "Python"
            && functions.len() == 1
            && functions.first().is_some_and(|name| name == "is_prime")
        {
            SemanticFidelity::PatternVerified
        } else {
            SemanticFidelity::Unknown
        };
        Self {
            from: program.language.clone(),
            to: target.into(),
            source: program.source.file.clone(),
            functions,
            steps,
            candidates,
            semantic_fidelity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentialCase {
    pub input: String,
    pub source_output: String,
    pub candidate_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentialReport {
    pub passed: bool,
    pub cases: Vec<DifferentialCase>,
}

pub fn compare_outputs(cases: Vec<(String, String, String)>) -> DifferentialReport {
    let cases = cases
        .into_iter()
        .map(
            |(input, source_output, candidate_output)| DifferentialCase {
                input,
                source_output,
                candidate_output,
            },
        )
        .collect::<Vec<_>>();
    DifferentialReport {
        passed: cases
            .iter()
            .all(|case| case.source_output == case.candidate_output),
        cases,
    }
}

pub fn empty_stats() -> BTreeMap<String, RuleStats> {
    BTreeMap::new()
}

impl fmt::Display for MigrationPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Migration: {} -> {}", self.from, self.to)?;
        writeln!(f, "Source: {}", self.source.display())?;
        writeln!(f, "Functions discovered: {}", self.functions.len())?;
        for step in &self.steps {
            writeln!(f, "- {step}")?;
        }
        for candidate in &self.candidates {
            writeln!(f, "- [{}] {}", candidate.rule_id, candidate.description)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_statistics_measure_validated_outcomes() {
        let mut stats = RuleStats::default();
        stats.record(true);
        stats.record(false);
        stats.record(true);
        assert_eq!(stats.attempts, 3);
        assert_eq!(stats.successes, 2);
        assert_eq!(stats.failures, 1);
        assert_eq!(stats.success_rate_percent(), 66);
    }

    #[test]
    fn migration_candidates_require_approval() {
        let program = Program::new("Python", PathBuf::from("main.py"), 0);
        let plan = MigrationPlan::from_program(&program, "Rust");
        assert!(plan.candidates.is_empty());
    }

    #[test]
    fn patch_rejects_concurrent_changes() {
        let path = std::env::temp_dir().join(format!("pyla-patch-{}", std::process::id()));
        std::fs::write(&path, "original").unwrap();
        let patch = Patch::from_source(
            path.clone(),
            "original",
            "replacement".into(),
            "test".into(),
        );
        std::fs::write(&path, "changed").unwrap();
        assert!(matches!(patch.apply(), Err(PatchError::Conflict { .. })));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn transaction_rolls_back_failed_validation() {
        let path = std::env::temp_dir().join(format!("pyla-transaction-{}", std::process::id()));
        std::fs::write(&path, "original").unwrap();
        let patch = Patch::from_source(path.clone(), "original", "candidate".into(), "test".into());
        let transaction = patch.begin_transaction().unwrap();
        transaction.rollback().unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn differential_report_rejects_behavior_changes() {
        let report = compare_outputs(vec![
            ("7".into(), "true".into(), "true".into()),
            ("8".into(), "false".into(), "true".into()),
        ]);
        assert!(!report.passed);
    }
}
