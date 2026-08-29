// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io;
use std::rc::Rc;

use qubit_function::BoxCallable;
#[cfg(feature = "once")]
use qubit_function::BoxCallableOnce;
use qubit_function::BoxCallableWith;
use qubit_function::BoxRunnable;
#[cfg(feature = "once")]
use qubit_function::BoxRunnableOnce;
use qubit_function::BoxRunnableWith;
use qubit_function::Callable;
#[cfg(feature = "once")]
use qubit_function::CallableOnce;
use qubit_function::CallableWith;
use qubit_function::LocalBoxCallable;
#[cfg(feature = "once")]
use qubit_function::LocalBoxCallableOnce;
use qubit_function::LocalBoxCallableWith;
use qubit_function::LocalBoxRunnable;
#[cfg(feature = "once")]
use qubit_function::LocalBoxRunnableOnce;
use qubit_function::LocalBoxRunnableWith;
use qubit_function::Runnable;
#[cfg(feature = "once")]
use qubit_function::RunnableOnce;
use qubit_function::RunnableWith;

/// Returns `value` after proving at compile time that it is transferable.
fn require_send<T: Send>(value: T) -> T {
    value
}

#[test]
fn test_task_box_compositions_remain_send() {
    let mut callable = require_send(
        BoxCallable::new(|| Ok::<i32, io::Error>(20))
            .map(|value| value + 1)
            .and_then(|value| Ok(value * 2)),
    );
    assert_eq!(callable.call().expect("callable should succeed"), 42);

    let mut runnable = require_send(BoxRunnable::new(|| Ok::<(), io::Error>(())).and_then(|| Ok::<(), io::Error>(())));
    runnable.run().expect("runnable should succeed");
}

#[test]
fn test_task_box_with_compositions_remain_send() {
    let mut callable = require_send(
        BoxCallableWith::new(|value: &mut i32| {
            *value += 1;
            Ok::<i32, io::Error>(*value)
        })
        .map(|value| value * 2)
        .and_then(|value, input| {
            *input += 1;
            Ok(value + *input)
        }),
    );
    let mut input = 20;
    assert_eq!(
        callable.call_with(&mut input).expect("callable-with should succeed"),
        64
    );

    let mut runnable = require_send(
        BoxRunnableWith::new(|value: &mut i32| {
            *value += 1;
            Ok::<(), io::Error>(())
        })
        .and_then(|value: &mut i32| {
            *value *= 2;
            Ok::<(), io::Error>(())
        }),
    );
    let mut input = 20;
    runnable.run_with(&mut input).expect("runnable-with should succeed");
    assert_eq!(input, 42);
}

#[cfg(feature = "once")]
#[test]
fn test_task_box_once_compositions_remain_send() {
    let callable = require_send(
        BoxCallableOnce::new(|| Ok::<i32, io::Error>(20))
            .map(|value| value + 1)
            .and_then(|value| Ok(value * 2)),
    );
    assert_eq!(callable.call_once().expect("callable-once should succeed"), 42,);

    let runnable = require_send(BoxRunnableOnce::new(|| Ok::<(), io::Error>(())).and_then(|| Ok::<(), io::Error>(())));
    runnable.run_once().expect("runnable-once should succeed");
}

#[test]
fn test_local_task_box_compositions_accept_rc_captures() {
    let suffix = Rc::new(String::from("-local"));
    let callable_suffix = Rc::clone(&suffix);
    let mut callable = LocalBoxCallable::new(|| Ok::<String, io::Error>(String::from("callable")))
        .map(move |value| format!("{value}{callable_suffix}"));
    assert_eq!(
        callable.call().expect("local callable should succeed"),
        "callable-local"
    );

    let runnable_suffix = Rc::clone(&suffix);
    let mut runnable = LocalBoxRunnable::new(move || {
        assert_eq!(runnable_suffix.as_str(), "-local");
        Ok::<(), io::Error>(())
    })
    .and_then(|| Ok::<(), io::Error>(()));
    runnable.run().expect("local runnable should succeed");
}

#[test]
fn test_local_task_box_with_compositions_accept_rc_captures() {
    let offset = Rc::new(1);
    let callable_offset = Rc::clone(&offset);
    let mut callable = LocalBoxCallableWith::new(move |value: &mut i32| {
        *value += *callable_offset;
        Ok::<i32, io::Error>(*value)
    })
    .map(|value| value * 2);
    let mut input = 20;
    assert_eq!(
        callable
            .call_with(&mut input)
            .expect("local callable-with should succeed"),
        42
    );

    let runnable_offset = Rc::clone(&offset);
    let mut runnable = LocalBoxRunnableWith::new(move |value: &mut i32| {
        *value += *runnable_offset;
        Ok::<(), io::Error>(())
    })
    .and_then(|value: &mut i32| {
        *value *= 2;
        Ok::<(), io::Error>(())
    });
    let mut input = 20;
    runnable
        .run_with(&mut input)
        .expect("local runnable-with should succeed");
    assert_eq!(input, 42);
}

#[cfg(feature = "once")]
#[test]
fn test_local_task_box_once_compositions_accept_rc_captures() {
    let suffix = Rc::new(String::from("-local"));
    let callable_suffix = Rc::clone(&suffix);
    let callable = LocalBoxCallableOnce::new(|| Ok::<String, io::Error>(String::from("callable")))
        .map(move |value| format!("{value}{callable_suffix}"));
    assert_eq!(
        callable.call_once().expect("local callable-once should succeed"),
        "callable-local"
    );

    let runnable_suffix = Rc::clone(&suffix);
    let runnable = LocalBoxRunnableOnce::new(move || {
        assert_eq!(runnable_suffix.as_str(), "-local");
        Ok::<(), io::Error>(())
    })
    .and_then(|| Ok::<(), io::Error>(()));
    runnable.run_once().expect("local runnable-once should succeed");
}
