// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for LocalBoxRunnableOnce.

use std::cell::Cell;
use std::io;
use std::rc::Rc;

use qubit_function::CallableOnce;
use qubit_function::LocalBoxRunnableOnce;
use qubit_function::RunnableOnce;
use qubit_function::SupplierOnce;

#[test]
fn test_local_box_runnable_once_new_allows_non_send_capture() {
    let flag = Rc::new(Cell::new(false));
    let captured = Rc::clone(&flag);
    let task = LocalBoxRunnableOnce::new(move || {
        captured.set(true);
        Ok::<(), io::Error>(())
    });

    task.run_once()
        .expect("local runnable-once should allow non-send capture");
    assert!(flag.get());
}

#[test]
fn test_local_box_runnable_once_from_supplier() {
    let flag = Rc::new(Cell::new(false));
    let captured = Rc::clone(&flag);
    let supplier = move || {
        captured.set(true);
        Ok::<(), io::Error>(())
    };

    let task = LocalBoxRunnableOnce::from_supplier(supplier);

    SupplierOnce::get(task).expect("supplier-backed local runnable should succeed");
    assert!(flag.get());
}

#[test]
fn test_local_box_runnable_once_and_then_supports_local_next_task() {
    let events = Rc::new(Cell::new(0));
    let first_events = Rc::clone(&events);
    let second_events = Rc::clone(&events);
    let first = LocalBoxRunnableOnce::new(move || {
        first_events.set(1);
        Ok::<(), io::Error>(())
    });
    let second = move || {
        second_events.set(2);
        Ok::<(), io::Error>(())
    };

    let chained = first.and_then(second);

    chained.run_once().expect("chained local runnable should run");
    assert_eq!(events.get(), 2);
}

#[test]
fn test_local_box_runnable_once_then_callable_supports_local_callable() {
    let text = Rc::new(String::from("value"));
    let captured = Rc::clone(&text);
    let task = LocalBoxRunnableOnce::new(|| Ok::<(), io::Error>(()));
    let callable = move || Ok::<String, io::Error>(captured.to_string());

    let chained = task.then_callable(callable);

    assert_eq!(
        chained.call_once().expect("local callable should run after runnable"),
        "value"
    );
}

#[test]
fn test_local_box_runnable_once_then_callable_clears_name() {
    let task = LocalBoxRunnableOnce::new_with_name("prepare", || Ok::<(), io::Error>(()));
    let chained = task.then_callable(|| Ok::<i32, io::Error>(42));

    assert_eq!(chained.name(), None);
    assert_eq!(chained.call_once().expect("callable should succeed"), 42);
}
