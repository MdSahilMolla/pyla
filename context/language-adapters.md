# Language Adapters

Each language implements:

```rust
pub trait LanguageAdapter: Send + Sync {
    fn language_id(&self) -> &'static str;
    fn file_extensions(&self) -> &'static [&'static str];
    fn parse(&self, source: &str, path: &Path) -> Result<Program, FrontendError>;
    fn analyze(&self, source: &str, program: &Program) -> Vec<Diagnostic>;
}
```

## Current adapters

- Rust: `.rs`
- C/C++: `.c`, `.cc`, `.cpp`, `.cxx`, `.h`, `.hpp`
- Python: `.py`, `.pyw`

The current parsers are intentionally lightweight structural frontends. They
extract functions and basic diagnostics; they are not substitutes for rustc,
Clang, or Python's full AST in complex transformations.

## Adding an adapter

1. Implement `LanguageAdapter`.
2. Register it in `Registry::default`.
3. Add deterministic parsing and diagnostics tests.
4. Add fixtures covering malformed and representative source.
5. Add target validation in `pyla-validate`.
6. Document supported extensions and known semantic limits.
