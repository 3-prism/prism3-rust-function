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

use qubit_function::CallableWith;
use qubit_function::LocalBoxRunnableWith;
use qubit_function::RunnableWith;

#[test]
fn test_local_box_runnable_with_composition_accepts_rc_capture() {
    let offset = Rc::new(1);
    let mut runnable = LocalBoxRunnableWith::new(move |input: &mut i32| {
        *input += *offset;
        Ok::<(), io::Error>(())
    })
    .and_then(|input: &mut i32| {
        *input *= 2;
        Ok::<(), io::Error>(())
    });
    let mut input = 20;

    runnable
        .run_with(&mut input)
        .expect("local runnable-with should succeed");
    assert_eq!(input, 42);
}

#[test]
fn test_local_box_runnable_with_constructors_and_name_management() {
    let mut named = LocalBoxRunnableWith::new_with_name("prepare", |_: &mut i32| Ok::<(), io::Error>(()));
    assert_eq!(named.name(), Some("prepare"));
    assert_eq!(named.to_string(), "LocalBoxRunnableWith(prepare)");
    assert!(format!("{named:?}").contains("prepare"));

    named.set_name("renamed");
    assert_eq!(named.name(), Some("renamed"));
    named.clear_name();
    assert_eq!(named.name(), None);

    let optional = LocalBoxRunnableWith::new_with_optional_name(
        |_: &mut i32| Ok::<(), io::Error>(()),
        Some(String::from("optional")),
    )
    .with_name("final");
    assert_eq!(optional.name(), Some("final"));
}

#[test]
fn test_local_box_runnable_with_and_then_covers_both_paths() {
    let next_runs = Rc::new(Cell::new(0));
    let captured = Rc::clone(&next_runs);
    let mut chained = LocalBoxRunnableWith::new(|value: &mut i32| {
        if *value < 0 {
            Err(io::Error::other("source failed"))
        } else {
            *value += 1;
            Ok(())
        }
    })
    .and_then(move |value: &mut i32| {
        captured.set(captured.get() + 1);
        *value *= 2;
        Ok::<(), io::Error>(())
    });

    let mut input = 2;
    chained.run_with(&mut input).expect("chain should succeed");
    assert_eq!(input, 6);
    assert_eq!(next_runs.get(), 1);

    let mut input = -1;
    let error = chained
        .run_with(&mut input)
        .expect_err("source error should short-circuit the chain");
    assert_eq!(error.to_string(), "source failed");
    assert_eq!(next_runs.get(), 1);
}

#[test]
fn test_local_box_runnable_with_then_callable_covers_both_paths() {
    let callable_runs = Rc::new(Cell::new(0));
    let captured = Rc::clone(&callable_runs);
    let mut callable = LocalBoxRunnableWith::new(|value: &mut i32| {
        if *value < 0 {
            Err(io::Error::other("prepare failed"))
        } else {
            *value += 1;
            Ok(())
        }
    })
    .then_callable_with(move |value: &mut i32| {
        captured.set(captured.get() + 1);
        Ok::<i32, io::Error>(*value * 2)
    });

    let mut input = 2;
    assert_eq!(
        callable
            .call_with(&mut input)
            .expect("callable should run after success"),
        6
    );
    assert_eq!(callable_runs.get(), 1);

    let mut input = -1;
    let error = callable
        .call_with(&mut input)
        .expect_err("runnable error should skip the callable");
    assert_eq!(error.to_string(), "prepare failed");
    assert_eq!(callable_runs.get(), 1);
}
