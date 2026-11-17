use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    Cpp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub language: Language,
    pub tool: String,
    pub available: bool,
    pub passed: Option<bool>,
    pub detail: String,
}

impl ValidationResult {
    pub fn unavailable(language: Language, tool: &str) -> Self {
        Self {
            language,
            tool: tool.into(),
            available: false,
            passed: None,
            detail: "tool not found".into(),
        }
    }
}

pub fn detect(language: Language) -> ValidationResult {
    let (tool, args) = match language {
        Language::Rust => ("cargo", vec!["--version"]),
        Language::Python => ("python3", vec!["--version"]),
        Language::Cpp => ("cmake", vec!["--version"]),
    };
    match Command::new(tool).args(args).output() {
        Ok(output) if output.status.success() => ValidationResult {
            language,
            tool: tool.into(),
            available: true,
            passed: None,
            detail: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        },
        _ => ValidationResult::unavailable(language, tool),
    }
}

pub fn validate_project(root: &Path, language: Language) -> ValidationResult {
    let mut result = detect(language);
    if !result.available {
        return result;
    }
    let (tool, args) = match language {
        Language::Rust => ("cargo", vec!["check", "--quiet"]),
        Language::Python => ("python3", vec!["-m", "compileall", "-q", "."]),
        Language::Cpp => ("cmake", vec!["--build", "."]),
    };
    result.tool = tool.into();
    match Command::new(tool).args(args).current_dir(root).output() {
        Ok(output) => {
            result.passed = Some(output.status.success());
            result.detail = if output.status.success() {
                "validation passed".into()
            } else {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            };
        }
        Err(error) => {
            result.passed = Some(false);
            result.detail = error.to_string();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_tool_names_are_stable() {
        assert_eq!(detect(Language::Rust).language, Language::Rust);
        assert_eq!(detect(Language::Python).language, Language::Python);
    }
}
