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
use qubit_function::LocalBoxCallableWith;

#[test]
fn test_local_box_callable_with_composition_accepts_rc_capture() {
    let offset = Rc::new(1);
    let mut callable = LocalBoxCallableWith::new(move |input: &mut i32| {
        *input += *offset;
        Ok::<i32, io::Error>(*input)
    })
    .map(|value| value * 2)
    .and_then(|value, input| Ok(value + *input));
    let mut input = 20;

    assert_eq!(
        callable
            .call_with(&mut input)
            .expect("local callable-with should succeed"),
        63
    );
}

#[test]
fn test_local_box_callable_with_constructors_and_name_management() {
    let mut named = LocalBoxCallableWith::new_with_name("compute", |input: &mut i32| Ok::<i32, io::Error>(*input));
    assert_eq!(named.name(), Some("compute"));
    assert_eq!(named.to_string(), "LocalBoxCallableWith(compute)");
    assert!(format!("{named:?}").contains("compute"));

    named.set_name("renamed");
    assert_eq!(named.name(), Some("renamed"));
    named.clear_name();
    assert_eq!(named.name(), None);

    let optional = LocalBoxCallableWith::new_with_optional_name(
        |input: &mut i32| Ok::<i32, io::Error>(*input),
        Some(String::from("optional")),
    )
    .with_name("final");
    assert_eq!(optional.name(), Some("final"));
}

#[test]
fn test_local_box_callable_with_combinators_cover_all_result_paths() {
    let mut input = 2;
    let mut mapped = LocalBoxCallableWith::new(|value: &mut i32| {
        *value += 1;
        Ok::<i32, &'static str>(*value)
    })
    .map(|value| value * 2);
    assert_eq!(mapped.call_with(&mut input).expect("map should succeed"), 6);

    let mut mapped_failure =
        LocalBoxCallableWith::new(|_: &mut i32| Err::<i32, _>("map failed")).map(|value| value * 2);
    assert_eq!(
        mapped_failure
            .call_with(&mut input)
            .expect_err("map should preserve errors"),
        "map failed"
    );

    let mut mapped_error =
        LocalBoxCallableWith::new(|_: &mut i32| Err::<i32, _>("raw")).map_err(|error| format!("mapped: {error}"));
    assert_eq!(
        mapped_error
            .call_with(&mut input)
            .expect_err("map_err should transform errors"),
        "mapped: raw"
    );

    let mut map_err_success = LocalBoxCallableWith::new(|value: &mut i32| Ok::<i32, &'static str>(*value))
        .map_err(|error| format!("mapped: {error}"));
    assert_eq!(
        map_err_success
            .call_with(&mut input)
            .expect("map_err should preserve success"),
        input
    );

    let next_runs = Rc::new(Cell::new(0));
    let captured = Rc::clone(&next_runs);
    let mut chained =
        LocalBoxCallableWith::new(|value: &mut i32| Ok::<i32, &'static str>(*value)).and_then(move |value, input| {
            captured.set(captured.get() + 1);
            *input += value;
            Ok(*input)
        });
    assert_eq!(chained.call_with(&mut input).expect("and_then should succeed"), 6);
    assert_eq!(next_runs.get(), 1);

    let captured = Rc::clone(&next_runs);
    let mut short_circuited =
        LocalBoxCallableWith::new(|_: &mut i32| Err::<i32, _>("source failed")).and_then(move |value, input| {
            captured.set(captured.get() + 1);
            *input += value;
            Ok(*input)
        });
    assert_eq!(
        short_circuited
            .call_with(&mut input)
            .expect_err("and_then should preserve source errors"),
        "source failed"
    );
    assert_eq!(next_runs.get(), 1);
}
