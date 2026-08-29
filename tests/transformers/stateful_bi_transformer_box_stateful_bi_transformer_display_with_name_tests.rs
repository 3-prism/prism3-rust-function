// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::ArcStatefulBiTransformer;
use qubit_function::ArcStatefulBinaryOperator;
use qubit_function::ArcStatefulTransformer;
use qubit_function::BoxBiPredicate;
use qubit_function::BoxStatefulBiTransformer;
use qubit_function::BoxStatefulBinaryOperator;
use qubit_function::BoxStatefulTransformer;
use qubit_function::RcStatefulBiTransformer;
use qubit_function::RcStatefulBinaryOperator;
use qubit_function::RcStatefulTransformer;
use qubit_function::StatefulBiTransformer;
use qubit_function::StatefulBinaryOperator;

#[test]
fn test_box_stateful_bi_transformer_display_with_name() {
    let mut counter = 0;
    let transformer = BoxStatefulBiTransformer::new_with_name("add_counter", move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "BoxStatefulBiTransformer(add_counter)");
}

#[test]
fn test_box_stateful_bi_transformer_display_without_name() {
    let mut counter = 0;
    let transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "BoxStatefulBiTransformer");
}

#[test]
fn test_rc_stateful_bi_transformer_display_with_name() {
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new_with_name("add_counter", move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "RcStatefulBiTransformer(add_counter)");
}

#[test]
fn test_rc_stateful_bi_transformer_display_without_name() {
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "RcStatefulBiTransformer");
}

#[test]
fn test_arc_stateful_bi_transformer_display_with_name() {
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new_with_name("add_counter", move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "ArcStatefulBiTransformer(add_counter)");
}

#[test]
fn test_arc_stateful_bi_transformer_display_without_name() {
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "ArcStatefulBiTransformer");
}
