// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![cfg(feature = "full")]

//! Tests for public re-exports from the crate root and module roots.

use qubit_function as qf;

fn assert_type_is_exported<T>(expected_name: &str) {
    let type_name = std::any::type_name::<T>();
    assert!(
        type_name.contains(expected_name),
        "expected `{type_name}` to contain `{expected_name}`"
    );
}

#[test]
fn test_root_exports_conditional_consumer_types() {
    assert_type_is_exported::<qf::BoxConditionalConsumer<i32>>("BoxConditionalConsumer");
    assert_type_is_exported::<qf::RcConditionalConsumer<i32>>("RcConditionalConsumer");
    assert_type_is_exported::<qf::ArcConditionalConsumer<i32>>("ArcConditionalConsumer");
    assert_type_is_exported::<qf::BoxConditionalConsumerOnce<i32>>("BoxConditionalConsumerOnce");
    assert_type_is_exported::<qf::BoxConditionalBiConsumer<i32, i64>>("BoxConditionalBiConsumer");
    assert_type_is_exported::<qf::RcConditionalBiConsumer<i32, i64>>("RcConditionalBiConsumer");
    assert_type_is_exported::<qf::ArcConditionalBiConsumer<i32, i64>>("ArcConditionalBiConsumer");
    assert_type_is_exported::<qf::BoxConditionalBiConsumerOnce<i32, i64>>("BoxConditionalBiConsumerOnce");
    assert_type_is_exported::<qf::BoxConditionalStatefulConsumer<i32>>("BoxConditionalStatefulConsumer");
    assert_type_is_exported::<qf::RcConditionalStatefulConsumer<i32>>("RcConditionalStatefulConsumer");
    assert_type_is_exported::<qf::ArcConditionalStatefulConsumer<i32>>("ArcConditionalStatefulConsumer");
    assert_type_is_exported::<qf::BoxConditionalStatefulBiConsumer<i32, i64>>("BoxConditionalStatefulBiConsumer");
    assert_type_is_exported::<qf::RcConditionalStatefulBiConsumer<i32, i64>>("RcConditionalStatefulBiConsumer");
    assert_type_is_exported::<qf::ArcConditionalStatefulBiConsumer<i32, i64>>("ArcConditionalStatefulBiConsumer");
}

#[test]
fn test_root_exports_conditional_function_types() {
    assert_type_is_exported::<qf::BoxConditionalFunctionOnce<i32, i64>>("BoxConditionalFunctionOnce");
    assert_type_is_exported::<qf::BoxConditionalBiFunctionOnce<i32, i64, String>>("BoxConditionalBiFunctionOnce");
    assert_type_is_exported::<qf::BoxConditionalMutatingFunction<i32, i64>>("BoxConditionalMutatingFunction");
    assert_type_is_exported::<qf::RcConditionalMutatingFunction<i32, i64>>("RcConditionalMutatingFunction");
    assert_type_is_exported::<qf::ArcConditionalMutatingFunction<i32, i64>>("ArcConditionalMutatingFunction");
    assert_type_is_exported::<qf::BoxConditionalMutatingFunctionOnce<i32, i64>>("BoxConditionalMutatingFunctionOnce");
    assert_type_is_exported::<qf::BoxConditionalStatefulMutatingFunction<i32, i64>>(
        "BoxConditionalStatefulMutatingFunction",
    );
    assert_type_is_exported::<qf::RcConditionalStatefulMutatingFunction<i32, i64>>(
        "RcConditionalStatefulMutatingFunction",
    );
    assert_type_is_exported::<qf::ArcConditionalStatefulMutatingFunction<i32, i64>>(
        "ArcConditionalStatefulMutatingFunction",
    );
}

#[test]
fn test_root_exports_conditional_transformer_types() {
    assert_type_is_exported::<qf::BoxConditionalBiTransformer<i32, i64, String>>("BoxConditionalBiTransformer");
    assert_type_is_exported::<qf::RcConditionalBiTransformer<i32, i64, String>>("RcConditionalBiTransformer");
    assert_type_is_exported::<qf::ArcConditionalBiTransformer<i32, i64, String>>("ArcConditionalBiTransformer");
    assert_type_is_exported::<qf::BoxConditionalBiTransformerOnce<i32, i64, String>>("BoxConditionalBiTransformerOnce");
}

#[test]
fn test_module_roots_export_conditional_types() {
    assert_type_is_exported::<qf::consumers::BoxConditionalConsumer<i32>>("BoxConditionalConsumer");
    assert_type_is_exported::<qf::consumers::ArcConditionalStatefulBiConsumer<i32, i64>>(
        "ArcConditionalStatefulBiConsumer",
    );
    assert_type_is_exported::<qf::functions::BoxConditionalFunctionOnce<i32, i64>>("BoxConditionalFunctionOnce");
    assert_type_is_exported::<qf::functions::ArcConditionalStatefulMutatingFunction<i32, i64>>(
        "ArcConditionalStatefulMutatingFunction",
    );
    assert_type_is_exported::<qf::transformers::BoxConditionalBiTransformerOnce<i32, i64, String>>(
        "BoxConditionalBiTransformerOnce",
    );
    assert_type_is_exported::<qf::transformers::RcConditionalBiTransformer<i32, i64, String>>(
        "RcConditionalBiTransformer",
    );
}

#[test]
fn test_root_exports_stateful_bi_predicate_types() {
    assert_type_is_exported::<qf::BoxStatefulBiPredicate<i32, i64>>("BoxStatefulBiPredicate");
    assert_type_is_exported::<qf::RcStatefulBiPredicate<i32, i64>>("RcStatefulBiPredicate");
    assert_type_is_exported::<qf::ArcStatefulBiPredicate<i32, i64>>("ArcStatefulBiPredicate");
}

#[test]
fn test_root_exports_tester_types() {
    fn assert_tester<T: qf::Tester>() {}

    assert_tester::<qf::BoxTester>();
    assert_tester::<qf::RcTester>();
    assert_tester::<qf::ArcTester>();
    assert_type_is_exported::<qf::BoxTester>("BoxTester");
    assert_type_is_exported::<qf::RcTester>("RcTester");
    assert_type_is_exported::<qf::ArcTester>("ArcTester");
}

#[test]
fn test_root_exports_stateful_tester_types() {
    fn assert_stateful_tester<T: qf::StatefulTester>() {}

    assert_stateful_tester::<qf::BoxStatefulTester>();
    assert_stateful_tester::<qf::RcStatefulTester>();
    assert_stateful_tester::<qf::ArcStatefulTester>();
    assert_type_is_exported::<qf::BoxStatefulTester>("BoxStatefulTester");
    assert_type_is_exported::<qf::RcStatefulTester>("RcStatefulTester");
    assert_type_is_exported::<qf::ArcStatefulTester>("ArcStatefulTester");
}
