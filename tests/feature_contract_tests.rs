// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
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
        "[package]\n\
         name = \"feature-contract-consumer\"\n\
         version = \"0.0.0\"\n\
         edition = \"2024\"\n\n\
         [dependencies]\n\
         qubit-function = {{ path = \"{}\", default-features = false{} }}\n\n\
         [workspace]\n",
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
        "status: {}\nstdout:\n{}\nstderr:\n{}",
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
        "expected diagnostic containing {expected_diagnostic:?}\n{diagnostics}",
    );
}

#[test]
fn test_baseline_task_wrappers_implement_debug_and_display() {
    assert_debug_and_display::<BoxCallable<(), ()>>();
    assert_debug_and_display::<BoxCallableWith<(), (), ()>>();
    assert_debug_and_display::<BoxRunnableWith<(), ()>>();
}

#[cfg(feature = "rc")]
#[test]
fn test_rc_task_wrappers_implement_debug_and_display() {
    assert_debug_and_display::<RcCallable<(), ()>>();
    assert_debug_and_display::<RcCallableWith<(), (), ()>>();
    assert_debug_and_display::<RcRunnableWith<(), ()>>();
}

#[cfg(feature = "stateful")]
#[test]
fn test_arc_task_wrappers_implement_debug_and_display() {
    assert_debug_and_display::<ArcCallable<(), ()>>();
    assert_debug_and_display::<ArcCallableWith<(), (), ()>>();
    assert_debug_and_display::<ArcRunnableWith<(), ()>>();
}

#[test]
fn test_baseline_accepts_box_consumer_when() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::BoxConsumer;

fn main() {
    let consumer = BoxConsumer::new(|_: &i32| {});
    let _conditional = consumer.when(|value: &i32| *value > 0);
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}

/// Verifies that semantic closure implementations do not depend on wrappers.
#[test]
fn test_baseline_semantic_trait_accepts_closure() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::Function;

fn invoke<F: Function<i32, i32>>(function: F) -> i32 {
    function.apply(&1)
}

fn main() {
    assert_eq!(invoke(|value: &i32| *value + 1), 2);
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}

#[test]
fn test_baseline_rejects_deep_arc_tester_path() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::testers::tester::arc_tester::ArcTester;

fn main() {
    let _tester = ArcTester::new(|| true);
}
"#,
    );

    assert_compile_failure(&output, "module `arc_tester` is private");
}

#[test]
fn test_stateful_rejects_deep_arc_stateful_tester_path() {
    let output = compile_consumer(
        &["stateful"],
        r#"
use qubit_function::testers::stateful_tester::arc_stateful_tester::ArcStatefulTester;

fn main() {
    let _tester = ArcStatefulTester::new(|| true);
}
"#,
    );

    assert_compile_failure(&output, "module `arc_stateful_tester` is private");
}

#[test]
fn test_baseline_rejects_deep_tester_ops_path() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::testers::tester::fn_tester_ops::FnTesterOps;

fn main() {
    let _tester = (|| true).and(|| true);
}
"#,
    );

    assert_compile_failure(&output, "fn_tester_ops");
}

#[test]
fn test_baseline_rejects_root_tester_ops_import() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::FnTesterOps;

fn main() {}
"#,
    );

    assert_compile_failure(&output, "FnTesterOps");
}

#[test]
fn test_baseline_rejects_raw_closure_when_without_ambiguity() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::*;

fn main() {
    let consumer = |_: &i32| {};
    let _conditional = consumer.when(|value: &i32| *value > 0);
}
"#,
    );

    let diagnostics = cargo_diagnostics(&output);
    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("no method named `when`"),
        "{diagnostics}",
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("multiple applicable items"),
        "{diagnostics}",
    );
}

#[test]
fn test_baseline_accepts_box_consumer_when_with_glob_import() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::*;

fn main() {
    let consumer = BoxConsumer::new(|_: &i32| {});
    let _conditional = consumer.when(|value: &i32| *value > 0);
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}

#[test]
fn test_baseline_accepts_box_runnable_then_callable() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::BoxRunnable;

fn main() {
    let runnable = BoxRunnable::new(|| Ok::<(), ()>(()));
    let _callable = runnable.then_callable(|| Ok::<i32, ()>(42));
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}

#[test]
fn test_baseline_accepts_box_runnable_with_then_callable() {
    let output = compile_consumer(
        &[],
        r#"
use qubit_function::BoxRunnableWith;

fn main() {
    let runnable = BoxRunnableWith::new(|_: &mut i32| Ok::<(), ()>(()));
    let _callable =
        runnable.then_callable_with(|value: &mut i32| Ok::<i32, ()>(*value));
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}

#[test]
fn test_once_accepts_box_runnable_once_then_callable() {
    let output = compile_consumer(
        &["once"],
        r#"
use qubit_function::BoxRunnableOnce;

fn main() {
    let runnable = BoxRunnableOnce::new(|| Ok::<(), ()>(()));
    let _callable = runnable.then_callable(|| Ok::<i32, ()>(42));
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}

#[test]
fn test_once_accepts_local_box_runnable_once_then_callable() {
    let output = compile_consumer(
        &["once"],
        r#"
use qubit_function::LocalBoxRunnableOnce;

fn main() {
    let runnable = LocalBoxRunnableOnce::new(|| Ok::<(), ()>(()));
    let _callable = runnable.then_callable(|| Ok::<i32, ()>(42));
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}

#[test]
fn test_task_box_rejects_non_send_constructor_capture() {
    let output = compile_consumer(
        &[],
        r#"
use std::rc::Rc;

use qubit_function::BoxCallable;

fn main() {
    let value = Rc::new(42);
    let _task = BoxCallable::<i32, ()>::new(move || Ok(*value));
}
"#,
    );

    assert_compile_failure(&output, "cannot be sent between threads safely");
}

#[test]
fn test_task_box_rejects_non_send_composition_capture() {
    let output = compile_consumer(
        &[],
        r#"
use std::rc::Rc;

use qubit_function::BoxCallable;

fn main() {
    let suffix = Rc::new(String::from("!"));
    let _task = BoxCallable::<String, ()>::new(|| Ok(String::from("task")))
        .map(move |value| format!("{value}{suffix}"));
}
"#,
    );

    assert_compile_failure(&output, "cannot be sent between threads safely");
}

#[test]
fn test_local_task_box_accepts_non_send_composition_capture() {
    let output = compile_consumer(
        &[],
        r#"
use std::rc::Rc;

use qubit_function::Callable;
use qubit_function::LocalBoxCallable;

fn main() {
    let suffix = Rc::new(String::from("!"));
    let mut task = LocalBoxCallable::<String, ()>::new(|| Ok(String::from("task")))
        .map(move |value| format!("{value}{suffix}"));
    assert_eq!(task.call(), Ok(String::from("task!")));
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}

#[test]
fn test_stateless_arc_rejects_send_non_sync_capture() {
    let output = compile_consumer(
        &[],
        r#"
use std::cell::Cell;

use qubit_function::ArcFunction;

fn main() {
    let value = Cell::new(1);
    let _function = ArcFunction::<i32, i32>::new(move |input: &i32| {
        value.set(*input);
        value.get()
    });
}
"#,
    );

    assert_compile_failure(&output, "cannot be shared between threads safely");
}

#[test]
fn test_stateful_arc_accepts_send_non_sync_capture() {
    let output = compile_consumer(
        &["stateful"],
        r#"
use std::cell::Cell;

use qubit_function::ArcStatefulSupplier;
use qubit_function::StatefulSupplier;

fn main() {
    let value = Cell::new(1);
    let mut supplier = ArcStatefulSupplier::new(move || {
        value.set(value.get() + 1);
        value.get()
    });
    assert_eq!(supplier.get(), 2);
}
"#,
    );

    assert!(output.status.success(), "{}", cargo_diagnostics(&output));
}
