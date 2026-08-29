// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![allow(
    dead_code,
    unused_imports,
    reason = "split contract fixtures share support definitions"
)]
// qubit-style: allow explicit-imports -- fixtures verify wildcard-import
// behavior.

use std::fmt::Debug;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

#[cfg(feature = "stateful")]
use qubit_function::ArcCallable;
#[cfg(feature = "stateful")]
use qubit_function::ArcCallableWith;
#[cfg(feature = "stateful")]
use qubit_function::ArcRunnableWith;
use qubit_function::BoxCallable;
use qubit_function::BoxCallableWith;
use qubit_function::BoxRunnableWith;
#[cfg(feature = "rc")]
use qubit_function::RcCallable;
#[cfg(feature = "rc")]
use qubit_function::RcCallableWith;
#[cfg(feature = "rc")]
use qubit_function::RcRunnableWith;

/// Asserts at compile time that T supports both formatting traits.
fn assert_debug_and_display<T: Debug + Display>() {}

/// Compiles a temporary consumer crate against this checkout.
///
/// The features argument selects path-dependency features, and source becomes
/// the fixture crate's main.rs. The returned output contains Cargo's status and
/// diagnostics.
fn compile_consumer(features: &[&str], source: &str) -> Output {
    let project_dir = tempfile::Builder::new()
        .prefix("qubit-function-feature-contract-")
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
    let feature_clause = if feature_list.is_empty() {
        String::new()
    } else {
        format!(", features = [{feature_list}]")
    };
    let dependency_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = format!(
        "[package]
\
         name = \"feature-contract-consumer\"
\
         version = \"0.0.0\"
\
         edition = \"2024\"

\
         [dependencies]
\
         qubit-function = {{ path = \"{}\", default-features = false{} }}

\
         [workspace]
",
        dependency_path.display(),
        feature_clause,
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

/// Formats Cargo diagnostics for an assertion failure.
fn cargo_diagnostics(output: &Output) -> String {
    format!(
        "status: {}
stdout:
{}
stderr:
{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

/// Asserts that compilation fails for the intended API-contract reason.
fn assert_compile_failure(output: &Output, expected_diagnostic: &str) {
    let diagnostics = cargo_diagnostics(output);
    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_diagnostic),
        "expected diagnostic containing {expected_diagnostic:?}
{diagnostics}",
    );
}

#[test]
fn test_discarded_callback_wrappers_trigger_unused_must_use() {
    let output = compile_consumer(
        &["full"],
        r#"
#![deny(unused_must_use)]

use qubit_function::ArcFunction;
use qubit_function::BoxCallable;
use qubit_function::LocalBoxCallable;
use qubit_function::RcFunction;

fn main() {
    BoxCallable::<i32, ()>::new(|| Ok(1));
    LocalBoxCallable::<i32, ()>::new(|| Ok(1));
    ArcFunction::<i32, i32>::new(|value: &i32| *value + 1);
    RcFunction::<i32, i32>::new(|value: &i32| *value + 1);
}
"#,
    );

    assert_compile_failure(&output, "callback wrappers do nothing unless stored or invoked");
}

#[test]
fn test_stored_callback_wrappers_satisfy_unused_must_use() {
    let output = compile_consumer(
        &["full"],
        r#"
#![deny(unused_must_use)]

use qubit_function::ArcFunction;
use qubit_function::BoxCallable;
use qubit_function::LocalBoxCallable;
use qubit_function::RcFunction;

fn main() {
    let _boxed = BoxCallable::<i32, ()>::new(|| Ok(1));
    let _local = LocalBoxCallable::<i32, ()>::new(|| Ok(1));
    let _shared = ArcFunction::<i32, i32>::new(|value: &i32| *value + 1);
    let _local_shared = RcFunction::<i32, i32>::new(|value: &i32| *value + 1);
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}
