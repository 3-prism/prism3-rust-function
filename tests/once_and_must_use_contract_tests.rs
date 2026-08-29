// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow explicit-imports -- fixtures verify wildcard-import
// behavior.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

fn compile_consumer(features: &[&str], source: &str) -> Output {
    let project_dir = tempfile::Builder::new()
        .prefix("qubit-function-once-contract-")
        .tempdir()
        .expect("temporary consumer directory should be created");
    let project_root = project_dir.path();
    let source_root = project_root.join("src");
    fs::create_dir_all(&source_root).expect("temporary consumer source directory should be created");
    let feature_list = features
        .iter()
        .map(|feature| format!("\"{feature}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let dependency_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = format!(
        "[package]\nname = \"once-contract-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nqubit-function = {{ path = \"{}\", default-features = false, features = [{feature_list}] }}\n\n[workspace]\n",
        dependency_path.display(),
    );
    fs::write(project_root.join("Cargo.toml"), manifest).expect("temporary consumer manifest should be written");
    fs::write(source_root.join("main.rs"), source).expect("temporary consumer source should be written");
    Command::new("cargo")
        .args(["+1.94.0", "check", "--offline", "--quiet", "--target-dir"])
        .arg(project_root.join("target"))
        .current_dir(project_root)
        .output()
        .expect("temporary consumer should invoke Cargo")
}

fn diagnostics(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

fn assert_compile_failure(output: &Output, expected: &str) {
    let diagnostics = diagnostics(output);
    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "expected diagnostic containing {expected:?}\n{diagnostics}",
    );
}

#[test]
fn test_once_accepts_explicit_once_method_names() {
    let output = compile_consumer(
        &["once"],
        r#"
use qubit_function::CallableOnce;
use qubit_function::RunnableOnce;

fn main() {
    assert_eq!(CallableOnce::call_once(|| Ok::<i32, ()>(42)), Ok(42));
    assert_eq!(RunnableOnce::run_once(|| Ok::<(), ()>(())), Ok(()));
}
"#,
    );
    assert!(output.status.success(), "{}", diagnostics(&output));
}

#[test]
fn test_once_rejects_reusable_method_names() {
    let callable = compile_consumer(
        &["once"],
        r#"
use qubit_function::CallableOnce;
struct OnceCallable;
impl CallableOnce<i32, ()> for OnceCallable {
    fn call_once(self) -> Result<i32, ()> { Ok(42) }
}
fn main() { let _ = OnceCallable.call(); }
"#,
    );
    assert_compile_failure(&callable, "no method named `call`");
    let runnable = compile_consumer(
        &["once"],
        r#"
use qubit_function::RunnableOnce;
struct OnceRunnable;
impl RunnableOnce<()> for OnceRunnable {
    fn run_once(self) -> Result<(), ()> { Ok(()) }
}
fn main() { let _ = OnceRunnable.run(); }
"#,
    );
    assert_compile_failure(&runnable, "no method named `run`");
}

#[test]
fn test_once_method_names_are_unambiguous_with_glob_import() {
    let output = compile_consumer(
        &["once"],
        r#"
use qubit_function::*;
fn main() {
    let mut reusable = || Ok::<i32, ()>(1);
    assert_eq!(reusable.call(), Ok(1));
    assert_eq!(reusable.call(), Ok(1));
    assert_eq!(BoxCallableOnce::new(|| Ok::<i32, ()>(2)).call_once(), Ok(2));
    assert_eq!(BoxRunnableOnce::new(|| Ok::<(), ()>(())).run_once(), Ok(()));
}
"#,
    );
    assert!(output.status.success(), "{}", diagnostics(&output));
}

#[test]
fn test_value_producing_methods_have_must_use_contracts() {
    let output = compile_consumer(
        &["full"],
        r#"
#![deny(unused_must_use)]
use qubit_function::BoxComparator;
use qubit_function::Comparator;
use qubit_function::Function;
use qubit_function::Predicate;
use qubit_function::Supplier;
fn main() {
    (|value: &i32| *value + 1).apply(&1);
    (|value: &i32| *value > 0).test(&1);
    (|| 42).get();
    (|left: &i32, right: &i32| left.cmp(right)).compare(&1, &2);
    BoxComparator::<i32>::new(|left: &i32, right: &i32| left.cmp(right)).into_fn();
}
"#,
    );
    assert_compile_failure(&output, "unused return value");
}
