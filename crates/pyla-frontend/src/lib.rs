use pyla_ir::{Function, NodeKind, Parameter, Program, SourceLocation, TypeKind};
use serde::{Deserialize, Serialize};
use std::{fmt, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub location: SourceLocation,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Suggestion,
}

#[derive(Debug)]
pub enum FrontendError {
    UnsupportedFile(String),
    InvalidSource(String),
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFile(path) => write!(f, "unsupported source file: {path}"),
            Self::InvalidSource(message) => write!(f, "invalid source: {message}"),
        }
    }
}

impl std::error::Error for FrontendError {}

pub trait LanguageAdapter: Send + Sync {
    fn language_id(&self) -> &'static str;
    fn file_extensions(&self) -> &'static [&'static str];
    fn parse(&self, source: &str, path: &Path) -> Result<Program, FrontendError>;
    fn analyze(&self, source: &str, program: &Program) -> Vec<Diagnostic>;
}

pub struct Registry {
    adapters: Vec<Box<dyn LanguageAdapter>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }
    pub fn register(&mut self, adapter: impl LanguageAdapter + 'static) {
        self.adapters.push(Box::new(adapter));
    }
    pub fn adapter_for(&self, path: &Path) -> Option<&dyn LanguageAdapter> {
        let extension = path.extension()?.to_str()?;
        self.adapters
            .iter()
            .find(|a| a.file_extensions().contains(&extension))
            .map(|a| a.as_ref())
    }
    pub fn languages(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.adapters.iter().map(|adapter| adapter.language_id())
    }
}

impl Default for Registry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(RustAdapter);
        registry.register(CppAdapter);
        registry.register(PythonAdapter);
        registry
    }
}

pub struct RustAdapter;
pub struct CppAdapter;
pub struct PythonAdapter;

fn parse_functions(source: &str, path: &Path, language: &str, marker: &str) -> Program {
    let mut program = Program::new(language, path.to_path_buf(), source.len());
    for (line_no, line) in source.lines().enumerate() {
        let Some(start) = line.find(marker) else {
            continue;
        };
        let rest = &line[start + marker.len()..];
        let name: String = rest
            .trim_start()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let location = SourceLocation {
            file: path.to_path_buf(),
            start: line_no + 1,
            end: line_no + 1,
        };
        program.nodes.push(NodeKind::Function(Function {
            name,
            parameters: Vec::<Parameter>::new(),
            return_type: TypeKind::Unknown,
            location,
        }));
    }
    program
}

fn parse_cpp_functions(source: &str, path: &Path) -> Program {
    let mut program = Program::new("C++", path.to_path_buf(), source.len());
    for (line_no, line) in source.lines().enumerate() {
        let Some(open) = line.find('(') else { continue };
        let prefix = line[..open].trim_end();
        let name: String = prefix
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        if name.is_empty() || name == "if" || name == "for" || name == "while" {
            continue;
        }
        program.nodes.push(NodeKind::Function(Function {
            name,
            parameters: Vec::new(),
            return_type: TypeKind::Unknown,
            location: SourceLocation {
                file: path.to_path_buf(),
                start: line_no + 1,
                end: line_no + 1,
            },
        }));
    }
    program
}

fn structural_diagnostics(source: &str, program: &Program, language: &str) -> Vec<Diagnostic> {
    let mut result = Vec::new();
    let braces = source.chars().fold(0i32, |count, c| {
        count
            + match c {
                '{' => 1,
                '}' => -1,
                _ => 0,
            }
    });
    if braces != 0 {
        result.push(Diagnostic {
            rule_id: format!("PYLA-{language}-001"),
            severity: Severity::Error,
            message: "unbalanced braces detected".into(),
            location: program.source.clone(),
            suggestion: Some("Add or remove a closing brace so blocks are balanced.".into()),
        });
    }
    for (line_no, line) in source.lines().enumerate() {
        let is_todo_comment = line
            .find("//")
            .or_else(|| line.find("/*"))
            .is_some_and(|comment_start| line[comment_start..].contains("TODO"));
        if is_todo_comment {
            result.push(Diagnostic {
                rule_id: format!("PYLA-{language}-002"),
                severity: Severity::Suggestion,
                message: "unfinished TODO found".into(),
                location: SourceLocation {
                    file: program.source.file.clone(),
                    start: line_no + 1,
                    end: line_no + 1,
                },
                suggestion: Some("Replace the TODO with an implementation or remove it.".into()),
            });
        }
    }
    result
}

impl LanguageAdapter for RustAdapter {
    fn language_id(&self) -> &'static str {
        "Rust"
    }
    fn file_extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }
    fn parse(&self, source: &str, path: &Path) -> Result<Program, FrontendError> {
        Ok(parse_functions(source, path, "Rust", "fn "))
    }
    fn analyze(&self, source: &str, program: &Program) -> Vec<Diagnostic> {
        structural_diagnostics(source, program, "RUST")
    }
}

impl LanguageAdapter for CppAdapter {
    fn language_id(&self) -> &'static str {
        "C++"
    }
    fn file_extensions(&self) -> &'static [&'static str] {
        &["c", "cc", "cpp", "cxx", "h", "hpp"]
    }
    fn parse(&self, source: &str, path: &Path) -> Result<Program, FrontendError> {
        Ok(parse_cpp_functions(source, path))
    }
    fn analyze(&self, source: &str, program: &Program) -> Vec<Diagnostic> {
        structural_diagnostics(source, program, "CPP")
    }
}

impl LanguageAdapter for PythonAdapter {
    fn language_id(&self) -> &'static str {
        "Python"
    }
    fn file_extensions(&self) -> &'static [&'static str] {
        &["py", "pyw"]
    }
    fn parse(&self, source: &str, path: &Path) -> Result<Program, FrontendError> {
        let mut program = Program::new("Python", path.to_path_buf(), source.len());
        for (line_no, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed.strip_prefix("def ") else {
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                continue;
            }
            program.nodes.push(NodeKind::Function(Function {
                name,
                parameters: Vec::new(),
                return_type: TypeKind::Unknown,
                location: SourceLocation {
                    file: path.to_path_buf(),
                    start: line_no + 1,
                    end: line_no + 1,
                },
            }));
        }
        Ok(program)
    }
    fn analyze(&self, source: &str, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (line_no, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("TODO") || line.contains("# TODO") {
                diagnostics.push(Diagnostic {
                    rule_id: "PYLA-PYTHON-001".into(),
                    severity: Severity::Suggestion,
                    message: "unfinished TODO found".into(),
                    location: SourceLocation {
                        file: program.source.file.clone(),
                        start: line_no + 1,
                        end: line_no + 1,
                    },
                    suggestion: Some(
                        "Replace the TODO with an implementation or remove it.".into(),
                    ),
                });
            }
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_selects_rust_and_reports_todo() {
        let registry = Registry::default();
        let path = Path::new("main.rs");
        let adapter = registry.adapter_for(path).unwrap();
        let program = adapter.parse("fn main() { /* TODO */ }", path).unwrap();
        assert_eq!(program.functions().count(), 1);
        assert_eq!(
            adapter.analyze("fn main() { /* TODO */ }", &program).len(),
            1
        );
    }

    #[test]
    fn cpp_adapter_extracts_function_name() {
        let adapter = CppAdapter;
        let program = adapter
            .parse(
                "int add(int a, int b) { return a + b; }",
                Path::new("main.cpp"),
            )
            .unwrap();
        assert_eq!(program.functions().next().unwrap().name, "add");
    }

    #[test]
    fn python_adapter_extracts_functions() {
        let adapter = PythonAdapter;
        let program = adapter
            .parse("def greet(name):\n    return name", Path::new("main.py"))
            .unwrap();
        assert_eq!(program.functions().next().unwrap().name, "greet");
    }
}
