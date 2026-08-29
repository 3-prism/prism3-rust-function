// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cell::Cell;
use std::io;
use std::rc::Rc;

use qubit_function::Callable;
use qubit_function::LocalBoxRunnable;
use qubit_function::Runnable;
#[cfg(feature = "once")]
use qubit_function::SupplierOnce;

#[test]
fn test_local_box_runnable_composition_accepts_rc_capture() {
    let count = Rc::new(Cell::new(0));
    let first_count = Rc::clone(&count);
    let second_count = Rc::clone(&count);
    let mut runnable = LocalBoxRunnable::new(move || {
        first_count.set(first_count.get() + 1);
        Ok::<(), io::Error>(())
    })
    .and_then(move || {
        second_count.set(second_count.get() + 1);
        Ok::<(), io::Error>(())
    });

    runnable.run().expect("local runnable should succeed");
    assert_eq!(count.get(), 2);
}

#[test]
fn test_local_box_runnable_constructors_and_name_management() {
    let mut named = LocalBoxRunnable::new_with_name("cleanup", || Ok::<(), io::Error>(()));
    assert_eq!(named.name(), Some("cleanup"));
    assert_eq!(named.to_string(), "LocalBoxRunnable(cleanup)");
    assert!(format!("{named:?}").contains("cleanup"));
    named.run().expect("named runnable should execute");

    named.set_name("renamed");
    assert_eq!(named.name(), Some("renamed"));
    named.clear_name();
    assert_eq!(named.name(), None);

    let mut optional =
        LocalBoxRunnable::new_with_optional_name(|| Ok::<(), io::Error>(()), Some(String::from("optional")))
            .with_name("final");
    assert_eq!(optional.name(), Some("final"));
    optional.run().expect("optionally named runnable should execute");
}

#[test]
fn test_local_box_runnable_from_supplier_is_reusable() {
    let count = Rc::new(Cell::new(0));
    let captured = Rc::clone(&count);
    let mut runnable = LocalBoxRunnable::from_supplier(move || {
        captured.set(captured.get() + 1);
        Ok::<(), io::Error>(())
    });

    runnable.run().expect("first run should succeed");
    runnable.run().expect("second run should succeed");
    assert_eq!(count.get(), 2);
}

#[test]
fn test_local_box_runnable_and_then_covers_both_paths() {
    let source_runs = Rc::new(Cell::new(0));
    let source_runs_capture = Rc::clone(&source_runs);
    let next_runs = Rc::new(Cell::new(0));
    let captured = Rc::clone(&next_runs);
    let mut runnable = LocalBoxRunnable::new(move || {
        source_runs_capture.set(source_runs_capture.get() + 1);
        if source_runs_capture.get() == 1 {
            Ok(())
        } else {
            Err(io::Error::other("source failed"))
        }
    })
    .and_then(move || {
        captured.set(captured.get() + 1);
        Ok::<(), io::Error>(())
    });

    runnable.run().expect("first chain run should succeed");
    assert_eq!(next_runs.get(), 1);

    let error = runnable.run().expect_err("source error should be preserved");
    assert_eq!(error.to_string(), "source failed");
    assert_eq!(next_runs.get(), 1);
}

#[test]
fn test_local_box_runnable_then_callable_covers_both_paths() {
    let source_runs = Rc::new(Cell::new(0));
    let source_runs_capture = Rc::clone(&source_runs);
    let callable_runs = Rc::new(Cell::new(0));
    let captured = Rc::clone(&callable_runs);
    let callable = move || {
        captured.set(captured.get() + 1);
        Ok::<i32, io::Error>(42)
    };
    let mut callable = LocalBoxRunnable::new(move || {
        source_runs_capture.set(source_runs_capture.get() + 1);
        if source_runs_capture.get() == 1 {
            Ok(())
        } else {
            Err(io::Error::other("prepare failed"))
        }
    })
    .then_callable(callable);
    assert_eq!(callable.call().expect("callable should run"), 42);
    assert_eq!(callable_runs.get(), 1);

    let error = callable
        .call()
        .expect_err("runnable error should short-circuit the callable");
    assert_eq!(error.to_string(), "prepare failed");
    assert_eq!(callable_runs.get(), 1);
}

#[cfg(feature = "once")]
#[test]
fn test_local_box_runnable_implements_supplier_once() {
    let runnable = LocalBoxRunnable::new(|| Ok::<(), io::Error>(()));

    SupplierOnce::get(runnable).expect("supplier adapter should run");
}
