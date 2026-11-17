use clap::{Args, Parser, Subcommand, ValueEnum};
use pyla_core::Engine;
use pyla_validate::Language;
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, Parser)]
#[command(
    name = "pyla",
    version,
    about = "Deterministic code engineering platform"
)]
struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Analyze(PathCommand),
    Migrate {
        path: PathBuf,
        #[arg(long, default_value = "rust")]
        to: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Validate {
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = LanguageArg::Rust)]
        language: LanguageArg,
    },
    Optimize(PathCommand),
    Compare(PathCommand),
    Worktree {
        #[command(subcommand)]
        command: WorktreeCommand,
    },
}

#[derive(Debug, Args)]
struct PathCommand {
    path: PathBuf,
}

#[derive(Debug, Subcommand)]
enum WorktreeCommand {
    Create {
        repository: PathBuf,
        path: PathBuf,
        branch: String,
    },
    Status {
        repository: PathBuf,
        path: PathBuf,
    },
    Remove {
        repository: PathBuf,
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LanguageArg {
    Rust,
    Python,
    Cpp,
}

impl From<LanguageArg> for Language {
    fn from(value: LanguageArg) -> Self {
        match value {
            LanguageArg::Rust => Self::Rust,
            LanguageArg::Python => Self::Python,
            LanguageArg::Cpp => Self::Cpp,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;
    match cli.command {
        Command::Analyze(command) => run_analyze(&command.path, json),
        Command::Migrate { path, to, output } => run_migrate(&path, &to, output.as_deref(), json),
        Command::Validate { path, language } => run_validate(&path, language.into(), json),
        Command::Optimize(command) => run_optimize(&command.path, json),
        Command::Compare(command) => run_compare(&command.path, json),
        Command::Worktree { command } => run_worktree(command, json),
    }
}

fn run_worktree(command: WorktreeCommand, json: bool) -> ExitCode {
    let engine = Engine::default();
    let result = match command {
        WorktreeCommand::Create {
            repository,
            path,
            branch,
        } => engine
            .create_worktree(&repository, &path, &branch)
            .map(|worktree| serde_json::json!({"path": worktree.path, "branch": worktree.branch})),
        WorktreeCommand::Status { repository, path } => pyla_transform::Worktree {
            repository,
            path,
            branch: String::new(),
        }
        .status()
        .map(|status| serde_json::json!({"status": status})),
        WorktreeCommand::Remove { repository, path } => pyla_transform::Worktree {
            repository,
            path,
            branch: String::new(),
        }
        .remove()
        .map(|_| serde_json::json!({"removed": true})),
    };
    match result {
        Ok(value) if json => print_json(&value, false),
        Ok(value) => {
            println!("{}", value);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("pyla: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_optimize(path: &Path, json: bool) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("pyla: {error}");
            return ExitCode::from(1);
        }
    };
    let module = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("pyla_optimized");
    let candidates = Engine::default().optimization_candidates(&source, module);
    if json {
        return print_json(&candidates, false);
    }
    if candidates.is_empty() {
        println!("No deterministic optimization candidates found.");
    }
    for candidate in candidates {
        println!(
            "{}: {} ({})\n{}",
            candidate.rule_id, candidate.function, candidate.reason, candidate.generated_module
        );
    }
    ExitCode::SUCCESS
}

fn run_compare(path: &Path, json: bool) -> ExitCode {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("pyla: {error}");
            return ExitCode::from(1);
        }
    };
    let cases = match serde_json::from_str(&contents) {
        Ok(cases) => cases,
        Err(error) => {
            eprintln!("pyla: invalid comparison JSON: {error}");
            return ExitCode::from(1);
        }
    };
    let report = Engine::default().compare_outputs(cases);
    if json {
        return print_json(&report, !report.passed);
    }
    println!(
        "Differential verification: {}",
        if report.passed { "passed" } else { "failed" }
    );
    if report.passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_validate(path: &Path, language: Language, json: bool) -> ExitCode {
    let result = Engine::default().validate(path, language);
    if json {
        return print_json(&result, result.passed == Some(false));
    }
    println!(
        "Tool: {}\nAvailable: {}\nPassed: {}\n{}",
        result.tool,
        result.available,
        result
            .passed
            .map_or("not run", |passed| if passed { "yes" } else { "no" }),
        result.detail
    );
    if result.passed == Some(false) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_migrate(path: &Path, target: &str, output: Option<&Path>, json: bool) -> ExitCode {
    let engine = Engine::default();
    if let Some(output) = output {
        return match engine.generate_migration(path, target, output) {
            Ok(files) if json => print_json(&files, false),
            Ok(files) => {
                for file in files {
                    println!("Generated {}", file.display());
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("pyla: {error}");
                ExitCode::from(1)
            }
        };
    }
    match engine.migration_plan(path, target) {
        Ok(plans) if json => print_json(&plans, false),
        Ok(plans) => {
            for plan in plans {
                println!("{plan}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("pyla: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_analyze(path: &Path, json: bool) -> ExitCode {
    let report = match Engine::default().analyze(path) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("pyla: {error}");
            return ExitCode::from(1);
        }
    };
    if json {
        return print_json(&report, false);
    }
    println!(
        "PYLA Code Analysis\n------------------\n\nProject: {}\n\nLanguages detected:",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".")
    );
    for (language, count) in &report.languages {
        println!("  {language}: {count} files");
    }
    println!("\nFiles analyzed: {}\n\nDiagnostics:\n  Errors:       {}\n  Warnings:     {}\n  Suggestions:  {}\n\nAnalysis completed.", report.files_analyzed, report.errors(), report.warnings(), report.suggestions());
    for diagnostic in &report.diagnostics {
        println!(
            "\n{}:{}: {} [{}]",
            diagnostic.location.file.display(),
            diagnostic.location.start,
            diagnostic.message,
            diagnostic.rule_id
        );
        if let Some(suggestion) = &diagnostic.suggestion {
            println!("  suggestion: {suggestion}");
        }
    }
    ExitCode::SUCCESS
}

fn print_json<T: serde::Serialize>(value: &T, failed: bool) -> ExitCode {
    match serde_json::to_string_pretty(value) {
        Ok(serialized) => {
            println!("{serialized}");
            if failed {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("pyla: cannot serialize JSON output: {error}");
            ExitCode::from(1)
        }
    }
}
