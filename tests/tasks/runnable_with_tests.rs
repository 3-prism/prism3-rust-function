// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for runnable-with task types.

use std::cell::Cell;
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_function::ArcRunnableWith;
use qubit_function::BoxRunnableWith;
use qubit_function::CallableWith;
use qubit_function::RcRunnableWith;
use qubit_function::RunnableWith;

#[derive(Clone)]
struct AddRunnableWith {
    amount: i32,
}

impl RunnableWith<i32, io::Error> for AddRunnableWith {
    fn run_with(&mut self, input: &mut i32) -> Result<(), io::Error> {
        *input += self.amount;
        Ok(())
    }
}

#[test]
fn test_runnable_with_closure_run_with_returns_success() {
    let mut input = 10;
    let mut task = |value: &mut i32| {
        *value += 5;
        Ok::<(), io::Error>(())
    };

    RunnableWith::run_with(&mut task, &mut input).expect("runnable-with closure should succeed");

    assert_eq!(input, 15);
}

#[test]
fn test_runnable_with_closure_run_with_returns_error() {
    let mut input = 10;
    let mut task = |_value: &mut i32| Err::<(), _>(io::Error::other("failed"));

    let error = RunnableWith::run_with(&mut task, &mut input).expect_err("runnable-with closure should fail");

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "failed");
    assert_eq!(input, 10);
}

#[test]
fn test_box_runnable_with_name_management() {
    let mut task = BoxRunnableWith::<i32, io::Error>::new_with_name("adjust", |input: &mut i32| {
        *input += 1;
        Ok(())
    });

    assert_eq!(task.name(), Some("adjust"));
    assert_eq!(task.to_string(), "BoxRunnableWith(adjust)");
    assert!(format!("{task:?}").contains("adjust"));

    task.set_name("renamed");
    assert_eq!(task.name(), Some("renamed"));

    task.clear_name();
    assert_eq!(task.name(), None);
    assert_eq!(task.to_string(), "BoxRunnableWith");
}

#[test]
fn test_box_runnable_with_and_then_runs_in_order() {
    let first = BoxRunnableWith::new(|input: &mut i32| {
        *input += 2;
        Ok::<(), io::Error>(())
    });
    let mut chained = first.and_then(|input: &mut i32| {
        *input *= 3;
        Ok::<(), io::Error>(())
    });
    let mut input = 4;

    chained
        .run_with(&mut input)
        .expect("chained runnable-with should succeed");

    assert_eq!(input, 18);
}

#[test]
fn test_box_runnable_with_then_callable_runs_after_success() {
    let first = BoxRunnableWith::new_with_name("prepare", |input: &mut i32| {
        *input += 2;
        Ok::<(), io::Error>(())
    });
    let mut callable = first.then_callable_with(|input: &mut i32| {
        *input *= 2;
        Ok::<i32, io::Error>(*input)
    });
    let mut input = 5;

    assert_eq!(callable.name(), None);
    assert_eq!(
        callable.call_with(&mut input).expect("callable-with should succeed"),
        14
    );
    assert_eq!(input, 14);
}

#[test]
fn test_rc_runnable_with_shares_state_between_clones() {
    let count = Rc::new(Cell::new(0));
    let captured = Rc::clone(&count);
    let mut shared = RcRunnableWith::new(move |input: &mut i32| {
        *input += 1;
        captured.set(captured.get() + 1);
        Ok::<(), io::Error>(())
    });
    let mut clone = shared.clone();
    let mut input = 0;

    shared.run_with(&mut input).expect("first call");
    clone.run_with(&mut input).expect("second call");

    assert_eq!(count.get(), 2);
    assert_eq!(input, 2);
}

#[test]
fn test_arc_runnable_with_shares_state_between_clones() {
    let count = Arc::new(AtomicUsize::new(0));
    let captured = Arc::clone(&count);
    let mut shared = ArcRunnableWith::new(move |input: &mut i32| {
        *input += 2;
        captured.fetch_add(1, Ordering::SeqCst);
        Ok::<(), io::Error>(())
    });
    let mut clone = shared.clone();
    let mut input = 0;

    shared.run_with(&mut input).expect("first call");
    clone.run_with(&mut input).expect("second call");

    assert_eq!(count.load(Ordering::SeqCst), 2);
    assert_eq!(input, 4);
}

#[test]
fn test_box_runnable_with_combinators_cover_error_branches() {
    let next_runs = Arc::new(AtomicUsize::new(0));
    let next_runs_capture = Arc::clone(&next_runs);
    let mut chained = BoxRunnableWith::<i32, io::Error>::new(|value: &mut i32| {
        if *value < 0 {
            Err(io::Error::other("first failed"))
        } else {
            *value += 1;
            Ok(())
        }
    })
    .and_then(move |value: &mut i32| {
        next_runs_capture.fetch_add(1, Ordering::SeqCst);
        *value *= 2;
        Ok::<(), io::Error>(())
    });

    let mut input = 1;
    chained.run_with(&mut input).expect("and_then should run after success");
    assert_eq!(input, 4);
    assert_eq!(next_runs.load(Ordering::SeqCst), 1);

    let mut input = -1;
    let error = chained.run_with(&mut input).expect_err("and_then should short-circuit");
    assert_eq!(error.to_string(), "first failed");
    assert_eq!(input, -1);
    assert_eq!(next_runs.load(Ordering::SeqCst), 1);

    let callable_runs = Arc::new(AtomicUsize::new(0));
    let callable_runs_capture = Arc::clone(&callable_runs);
    let mut callable = BoxRunnableWith::<i32, io::Error>::new(|value: &mut i32| {
        if *value < 0 {
            Err(io::Error::other("prepare failed"))
        } else {
            *value += 1;
            Ok(())
        }
    })
    .then_callable_with(move |value: &mut i32| {
        callable_runs_capture.fetch_add(1, Ordering::SeqCst);
        Ok::<i32, io::Error>(*value * 2)
    });

    let mut input = 2;
    assert_eq!(
        callable
            .call_with(&mut input)
            .expect("then_callable_with should run after success"),
        6,
    );
    assert_eq!(input, 3);
    assert_eq!(callable_runs.load(Ordering::SeqCst), 1);

    let mut input = -1;
    let error = callable
        .call_with(&mut input)
        .expect_err("then_callable_with should short-circuit");
    assert_eq!(error.to_string(), "prepare failed");
    assert_eq!(input, -1);
    assert_eq!(callable_runs.load(Ordering::SeqCst), 1);
}
