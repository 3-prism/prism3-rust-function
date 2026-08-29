// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for LocalBoxCallableOnce.

use std::io;
use std::rc::Rc;

use qubit_function::CallableOnce;
use qubit_function::LocalBoxCallableOnce;
use qubit_function::SupplierOnce;

#[test]
fn test_local_box_callable_once_new_allows_non_send_capture() {
    let text = Rc::new(String::from("local"));
    let captured = Rc::clone(&text);
    let task = LocalBoxCallableOnce::new(move || Ok::<String, io::Error>(captured.to_string()));

    assert_eq!(
        task.call_once()
            .expect("local callable-once should allow non-send capture"),
        "local"
    );
}

#[test]
fn test_local_box_callable_once_from_supplier() {
    let text = Rc::new(String::from("supplier"));
    let captured = Rc::clone(&text);
    let supplier = move || Ok::<String, io::Error>(captured.to_string());

    let task = LocalBoxCallableOnce::from_supplier(supplier);

    assert_eq!(
        SupplierOnce::get(task).expect("supplier-backed local callable should succeed"),
        "supplier"
    );
}

#[test]
fn test_local_box_callable_once_map_and_then_support_local_captures() {
    let suffix = Rc::new(String::from("-mapped"));
    let mapped_suffix = Rc::clone(&suffix);
    let task = LocalBoxCallableOnce::new(|| Ok::<String, io::Error>(String::from("local")))
        .map(move |value| format!("{value}{mapped_suffix}"));

    let next_suffix = Rc::clone(&suffix);
    let chained = task.and_then(move |value| Ok(format!("{value}{next_suffix}")));

    assert_eq!(
        chained.call_once().expect("chained local callable should succeed"),
        "local-mapped-mapped"
    );
}

#[test]
fn test_local_box_callable_once_map_err_transforms_local_error() {
    let prefix = Rc::new(String::from("local"));
    let captured = Rc::clone(&prefix);
    let task = LocalBoxCallableOnce::new(|| Err::<i32, _>(io::Error::other("raw")));

    let mapped = task.map_err(move |error| format!("{captured}: {error}"));

    assert_eq!(
        mapped.call_once().expect_err("local map_err should transform error"),
        "local: raw"
    );
}
