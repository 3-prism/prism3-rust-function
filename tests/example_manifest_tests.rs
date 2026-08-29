// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ExampleEntry {
    name: String,
    path: String,
}

#[test]
fn all_manifest_examples_point_to_existing_files() {
    let manifest_dir = manifest_dir();

    for example in manifest_examples() {
        let path = manifest_dir.join(&example.path);

        assert!(
            path.is_file(),
            "example '{}' points to missing file '{}'",
            example.name,
            example.path
        );

        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .expect("example path should have a UTF-8 file stem");

        assert_eq!(
            example.name, stem,
            "example '{}' should use its file stem as manifest name",
            example.path
        );
    }
}

#[test]
fn all_example_files_are_listed_in_manifest() {
    let manifest_dir = manifest_dir();
    let mut actual = collect_example_files(&manifest_dir);
    let mut expected = manifest_examples()
        .into_iter()
        .map(|example| example.path)
        .collect::<Vec<_>>();

    actual.sort();
    expected.sort();

    assert_eq!(
        expected, actual,
        "Cargo.toml [[example]] entries should match examples/**/*.rs"
    );
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn manifest_examples() -> Vec<ExampleEntry> {
    let manifest_path = manifest_dir().join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("Cargo.toml should be readable");
    let mut examples = Vec::new();
    let mut name = None;
    let mut path = None;
    let mut in_example = false;

    for line in manifest.lines().map(str::trim) {
        match line {
            "[[example]]" => {
                push_example(&mut examples, name.take(), path.take());
                in_example = true;
            }
            value if value.starts_with('[') => {
                push_example(&mut examples, name.take(), path.take());
                in_example = false;
            }
            value if in_example => {
                if let Some(parsed_name) = parse_quoted_value(value, "name") {
                    name = Some(parsed_name);
                } else if let Some(parsed_path) = parse_quoted_value(value, "path") {
                    path = Some(parsed_path);
                }
            }
            _ => {}
        }
    }

    push_example(&mut examples, name, path);
    examples.sort();
    examples
}

fn push_example(examples: &mut Vec<ExampleEntry>, name: Option<String>, path: Option<String>) {
    match (name, path) {
        (Some(name), Some(path)) => examples.push(ExampleEntry { name, path }),
        (None, None) => {}
        (name, path) => {
            panic!("incomplete [[example]] entry: name={name:?}, path={path:?}");
        }
    }
}

fn parse_quoted_value(line: &str, key: &str) -> Option<String> {
    let value = line.strip_prefix(key)?.trim();
    let value = value.strip_prefix('=')?.trim();
    let value = value.strip_prefix('"')?.strip_suffix('"')?;

    Some(value.to_owned())
}

fn collect_example_files(manifest_dir: &Path) -> Vec<String> {
    let examples_dir = manifest_dir.join("examples");
    let mut files = Vec::new();

    collect_rs_files(manifest_dir, &examples_dir, &mut files);
    files.sort();
    files
}

fn collect_rs_files(manifest_dir: &Path, directory: &Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(directory).expect("examples directory should be readable") {
        let entry = entry.expect("example directory entry should be readable");
        let path = entry.path();

        if path.is_dir() {
            collect_rs_files(manifest_dir, &path, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            let relative_path = path
                .strip_prefix(manifest_dir)
                .expect("example path should be under manifest directory")
                .to_string_lossy()
                .replace('\\', "/");
            files.push(relative_path);
        }
    }
}
