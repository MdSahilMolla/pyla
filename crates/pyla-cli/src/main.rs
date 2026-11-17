use pyla_core::Engine;
use pyla_validate::Language;
use std::{env, path::Path, process::ExitCode};

fn main() -> ExitCode {
    let mut raw_args = env::args().skip(1).collect::<Vec<_>>();
    let json = raw_args.iter().any(|arg| arg == "--json");
    raw_args.retain(|arg| arg != "--json");
    let mut args = raw_args.into_iter();
    match (args.next().as_deref(), args.next()) {
        (Some("analyze"), Some(path)) => run_analyze(Path::new(&path), json),
        (Some("migrate"), Some(path)) => {
            let target = args
                .next()
                .and_then(|flag| flag.strip_prefix("--to=").map(str::to_owned));
            let output = args
                .next()
                .and_then(|flag| flag.strip_prefix("--output=").map(std::path::PathBuf::from));
            run_migrate(
                Path::new(&path),
                target.as_deref().unwrap_or("Rust"),
                output.as_deref(),
                json,
            )
        }
        (Some("validate"), Some(path)) => {
            let language = args
                .next()
                .and_then(|flag| parse_language(flag.strip_prefix("--language=")?));
            run_validate(Path::new(&path), language.unwrap_or(Language::Rust), json)
        }
        (Some("worktree"), Some(action)) => run_worktree(&action, &mut args),
        (Some("optimize"), Some(path)) => run_optimize(Path::new(&path), json),
        (Some("compare"), Some(path)) => run_compare(Path::new(&path), json),
        _ => {
            eprintln!("Usage: pyla analyze <path> | pyla migrate <path> --to=<language> [--output=<dir>] | pyla validate <path> --language=<rust|python|cpp> | pyla optimize <python-file> [--json] | pyla compare <case-file.json> [--json] | pyla worktree <create|status|remove> ...");
            ExitCode::from(2)
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
    } else {
        for candidate in candidates {
            println!(
                "{}: {} ({})",
                candidate.rule_id, candidate.function, candidate.reason
            );
            println!("{}", candidate.generated_module);
        }
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
    let cases: Vec<(String, String, String)> = match serde_json::from_str(&contents) {
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
    for case in report.cases {
        println!(
            "{}: {} => {}",
            case.input, case.source_output, case.candidate_output
        );
    }
    if report.passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_worktree(action: &str, args: &mut impl Iterator<Item = String>) -> ExitCode {
    let engine = Engine::default();
    let result = match action {
        "create" => {
            let repository = args.next().map(std::path::PathBuf::from);
            let path = args.next().map(std::path::PathBuf::from);
            let branch = args.next();
            match (repository, path, branch) {
                (Some(repository), Some(path), Some(branch)) => engine
                    .create_worktree(&repository, &path, &branch)
                    .map(|worktree| {
                        println!(
                            "Created worktree {} on branch {}",
                            worktree.path.display(),
                            worktree.branch
                        )
                    }),
                _ => Err(pyla_transform::WorktreeError::Invalid(
                    "usage: pyla worktree create <repository> <path> <branch>".into(),
                )),
            }
        }
        "status" => {
            let repository = args.next().map(std::path::PathBuf::from);
            let path = args.next().map(std::path::PathBuf::from);
            match (repository, path) {
                (Some(repository), Some(path)) => pyla_transform::Worktree {
                    repository,
                    path,
                    branch: String::new(),
                }
                .status()
                .map(|status| println!("{status}")),
                _ => Err(pyla_transform::WorktreeError::Invalid(
                    "usage: pyla worktree status <repository> <path>".into(),
                )),
            }
        }
        "remove" => {
            let repository = args.next().map(std::path::PathBuf::from);
            let path = args.next().map(std::path::PathBuf::from);
            match (repository, path) {
                (Some(repository), Some(path)) => pyla_transform::Worktree {
                    repository,
                    path,
                    branch: String::new(),
                }
                .remove(),
                _ => Err(pyla_transform::WorktreeError::Invalid(
                    "usage: pyla worktree remove <repository> <path>".into(),
                )),
            }
        }
        _ => Err(pyla_transform::WorktreeError::Invalid(
            "unknown worktree action".into(),
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pyla: {error}");
            ExitCode::from(1)
        }
    }
}

fn parse_language(value: &str) -> Option<Language> {
    match value.to_ascii_lowercase().as_str() {
        "rust" => Some(Language::Rust),
        "python" | "py" => Some(Language::Python),
        "cpp" | "c++" | "c" => Some(Language::Cpp),
        _ => None,
    }
}

fn run_validate(path: &Path, language: Language, json: bool) -> ExitCode {
    let result = Engine::default().validate(path, language);
    if json {
        return print_json(&result, result.passed == Some(false));
    }
    let passed = result
        .passed
        .map_or("not run", |passed| if passed { "yes" } else { "no" });
    println!(
        "Tool: {}\nAvailable: {}\nPassed: {}\n{}",
        result.tool, result.available, passed, result.detail
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
            Ok(files) => {
                if json {
                    return print_json(&files, false);
                }
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
        Ok(plans) => {
            if json {
                return print_json(&plans, false);
            }
            if plans.is_empty() {
                println!("No supported source files found.");
            }
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
    let project = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(".");
    println!("PYLA Code Analysis\n------------------\n\nProject: {project}\n\nLanguages detected:");
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
