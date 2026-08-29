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
use qubit_function::LocalBoxCallable;

#[test]
fn test_local_box_callable_composition_accepts_rc_capture() {
    let suffix = Rc::new(String::from("!"));
    let mut callable = LocalBoxCallable::new(|| Ok::<String, io::Error>(String::from("local")))
        .map(move |value| format!("{value}{suffix}"))
        .and_then(|value| Ok(format!("{value}?")));

    assert_eq!(callable.call().expect("local callable should succeed"), "local!?");
}

#[test]
fn test_local_box_callable_constructors_and_name_management() {
    let mut named = LocalBoxCallable::new_with_name("compute", || Ok::<i32, io::Error>(1));
    assert_eq!(named.name(), Some("compute"));
    assert_eq!(named.to_string(), "LocalBoxCallable(compute)");
    assert!(format!("{named:?}").contains("compute"));

    named.set_name("renamed");
    assert_eq!(named.name(), Some("renamed"));
    named.clear_name();
    assert_eq!(named.name(), None);

    let optional = LocalBoxCallable::new_with_optional_name(|| Ok::<i32, io::Error>(2), Some(String::from("optional")))
        .with_name("final");
    assert_eq!(optional.name(), Some("final"));
}

#[test]
fn test_local_box_callable_from_supplier_is_reusable() {
    let count = Rc::new(Cell::new(0));
    let captured = Rc::clone(&count);
    let mut callable = LocalBoxCallable::from_supplier(move || {
        captured.set(captured.get() + 1);
        Ok::<i32, io::Error>(captured.get())
    });

    assert_eq!(callable.call().expect("first call should succeed"), 1);
    assert_eq!(callable.call().expect("second call should succeed"), 2);
}

#[test]
fn test_local_box_callable_combinators_cover_success_and_error_paths() {
    let mut mapped = LocalBoxCallable::new(|| Ok::<i32, &'static str>(3)).map(|value| value + 1);
    assert_eq!(mapped.call().expect("map should transform success"), 4);

    let mut map_failure = LocalBoxCallable::new(|| Err::<i32, _>("map failed")).map(|value| value + 1);
    assert_eq!(
        map_failure.call().expect_err("map should preserve errors"),
        "map failed"
    );

    let mut mapped_error = LocalBoxCallable::new(|| Err::<i32, _>("raw")).map_err(|error| format!("mapped: {error}"));
    assert_eq!(
        mapped_error.call().expect_err("map_err should transform errors"),
        "mapped: raw"
    );

    let mut map_err_success =
        LocalBoxCallable::new(|| Ok::<i32, &'static str>(5)).map_err(|error| format!("mapped: {error}"));
    assert_eq!(map_err_success.call().expect("map_err should preserve success"), 5);

    let next_runs = Rc::new(Cell::new(0));
    let next_runs_capture = Rc::clone(&next_runs);
    let mut chained = LocalBoxCallable::new(|| Ok::<i32, &'static str>(6)).and_then(move |value| {
        next_runs_capture.set(next_runs_capture.get() + 1);
        Ok(value * 2)
    });
    assert_eq!(chained.call().expect("and_then should succeed"), 12);
    assert_eq!(next_runs.get(), 1);

    let next_runs_capture = Rc::clone(&next_runs);
    let mut short_circuited = LocalBoxCallable::new(|| Err::<i32, _>("source failed")).and_then(move |value| {
        next_runs_capture.set(next_runs_capture.get() + 1);
        Ok(value * 2)
    });
    assert_eq!(
        short_circuited
            .call()
            .expect_err("and_then should preserve source errors"),
        "source failed"
    );
    assert_eq!(next_runs.get(), 1);
}
