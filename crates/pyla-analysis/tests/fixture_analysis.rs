use pyla_analysis::analyze_path;
use pyla_frontend::Registry;
use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

#[test]
fn analyzes_mixed_language_fixtures_deterministically() {
    let report = analyze_path(&fixtures(), &Registry::default()).unwrap();

    assert_eq!(report.files_analyzed, 3);
    assert_eq!(report.languages.get("Rust"), Some(&1));
    assert_eq!(report.languages.get("Python"), Some(&1));
    assert_eq!(report.languages.get("C++"), Some(&1));
    assert_eq!(report.errors(), 0);
    assert_eq!(report.suggestions(), 3);

    let locations: Vec<_> = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.location.file.clone())
        .collect();
    let mut sorted = locations.clone();
    sorted.sort();
    assert_eq!(locations, sorted);
}
