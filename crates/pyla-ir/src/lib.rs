use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    Integer,
    Float,
    Boolean,
    String,
    Void,
    Unknown,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub ty: TypeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeKind,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Function(Function),
    Variable { name: String, ty: TypeKind },
    Expression,
    Block,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub language: String,
    pub source: SourceLocation,
    pub nodes: Vec<NodeKind>,
}

impl Program {
    pub fn new(language: impl Into<String>, file: PathBuf, source_len: usize) -> Self {
        Self {
            language: language.into(),
            source: SourceLocation {
                file,
                start: 0,
                end: source_len,
            },
            nodes: Vec::new(),
        }
    }

    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.nodes.iter().filter_map(|node| match node {
            NodeKind::Function(function) => Some(function),
            _ => None,
        })
    }
}
