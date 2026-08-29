// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![cfg(feature = "full")]
use qubit_function::ArcBiFunction;
use qubit_function::ArcBiTransformer;
use qubit_function::ArcFunction;
use qubit_function::ArcStatefulBiTransformer;
use qubit_function::ArcStatefulTransformer;
use qubit_function::ArcTransformer;
use qubit_function::BiFunction;
use qubit_function::BiFunctionOnce;
use qubit_function::BiMutatingFunction;
use qubit_function::BiMutatingFunctionOnce;
use qubit_function::BiTransformer;
use qubit_function::BiTransformerOnce;
use qubit_function::BoxBiFunction;
use qubit_function::BoxBiFunctionOnce;
use qubit_function::BoxBiMutatingFunction;
use qubit_function::BoxBiMutatingFunctionOnce;
use qubit_function::BoxBiTransformer;
use qubit_function::BoxBiTransformerOnce;
use qubit_function::BoxFunction;
use qubit_function::BoxFunctionOnce;
use qubit_function::BoxMutatingFunction;
use qubit_function::BoxMutatingFunctionOnce;
use qubit_function::BoxStatefulBiTransformer;
use qubit_function::BoxStatefulTransformer;
use qubit_function::BoxTransformer;
use qubit_function::BoxTransformerOnce;
use qubit_function::Function;
use qubit_function::FunctionOnce;
use qubit_function::MutatingFunction;
use qubit_function::MutatingFunctionOnce;
use qubit_function::StatefulBiTransformer;
use qubit_function::StatefulTransformer;
use qubit_function::Transformer;
use qubit_function::TransformerOnce;

#[test]
fn function_constant_and_identity_allow_non_static_input_types() {
    let owned = String::from("input");
    let input = owned.as_str();

    assert_eq!(box_function_constant_non_static(input), 7);
    assert_eq!(arc_function_constant_non_static(input), 8);

    assert_eq!(box_function_identity_non_static(input), input);
    assert_eq!(box_function_once_identity_non_static(input), input);
    assert_eq!(box_mutating_identity_non_static(input), input);
    assert_eq!(box_mutating_once_identity_non_static(input), input);

    assert_eq!(box_bi_function_constant_non_static(input), 9);
    assert_eq!(arc_bi_function_constant_non_static(input), 10);
    assert_eq!(box_bi_function_once_constant_non_static(input), 11);
    assert_eq!(box_bi_mutating_function_constant_non_static(input), 12);
    assert_eq!(box_bi_mutating_function_once_constant_non_static(input), 13);
}

#[test]
fn transformer_constant_allow_non_static_input_types() {
    let owned = String::from("transform");
    let input = owned.as_str();

    assert_eq!(box_transformer_constant_non_static(input), 21);
    assert_eq!(arc_transformer_constant_non_static(input), 22);
    assert_eq!(box_transformer_once_constant_non_static(input), 23);

    assert_eq!(box_bi_transformer_constant_non_static(input), 24);
    assert_eq!(arc_bi_transformer_constant_non_static(input), 25);
    assert_eq!(box_bi_transformer_once_constant_non_static(input), 26);

    assert_eq!(box_stateful_transformer_constant_non_static(input), 27);
    assert_eq!(arc_stateful_transformer_constant_non_static(input), 28);

    assert_eq!(box_stateful_bi_transformer_constant_non_static(input), 29);
    assert_eq!(arc_stateful_bi_transformer_constant_non_static(input), 30);
}

fn box_function_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxFunction::<&'a str, i32>::constant(7);
    constant.apply(&input)
}

fn arc_function_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = ArcFunction::<&'a str, i32>::constant(8);
    constant.apply(&input)
}

fn box_function_identity_non_static<'a>(input: &'a str) -> &'a str {
    let identity = BoxFunction::<&'a str, &'a str>::identity();
    identity.apply(&input)
}

fn box_function_once_identity_non_static<'a>(input: &'a str) -> &'a str {
    let identity = BoxFunctionOnce::<&'a str, &'a str>::identity();
    identity.apply(&input)
}

fn box_mutating_identity_non_static<'a>(input: &'a str) -> &'a str {
    let identity = BoxMutatingFunction::<&'a str, &'a str>::identity();
    let mut value = input;
    identity.apply(&mut value)
}

fn box_mutating_once_identity_non_static<'a>(input: &'a str) -> &'a str {
    let identity = BoxMutatingFunctionOnce::<&'a str, &'a str>::identity();
    let mut value = input;
    identity.apply(&mut value)
}

fn box_bi_function_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxBiFunction::<&'a str, &'a str, i32>::constant(9);
    constant.apply(&input, &input)
}

fn arc_bi_function_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = ArcBiFunction::<&'a str, &'a str, i32>::constant(10);
    constant.apply(&input, &input)
}

fn box_bi_function_once_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxBiFunctionOnce::<&'a str, &'a str, i32>::constant(11);
    constant.apply(&input, &input)
}

fn box_bi_mutating_function_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxBiMutatingFunction::<&'a str, &'a str, i32>::constant(12);
    let mut left = input;
    let mut right = input;
    constant.apply(&mut left, &mut right)
}

fn box_bi_mutating_function_once_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxBiMutatingFunctionOnce::<&'a str, &'a str, i32>::constant(13);
    let mut left = input;
    let mut right = input;
    constant.apply(&mut left, &mut right)
}

fn box_transformer_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxTransformer::<&'a str, i32>::constant(21);
    constant.apply(input)
}

fn arc_transformer_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = ArcTransformer::<&'a str, i32>::constant(22);
    constant.apply(input)
}

fn box_transformer_once_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxTransformerOnce::<&'a str, i32>::constant(23);
    constant.apply(input)
}

fn box_bi_transformer_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxBiTransformer::<&'a str, &'a str, i32>::constant(24);
    constant.apply(input, input)
}

fn arc_bi_transformer_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = ArcBiTransformer::<&'a str, &'a str, i32>::constant(25);
    constant.apply(input, input)
}

fn box_bi_transformer_once_constant_non_static<'a>(input: &'a str) -> i32 {
    let constant = BoxBiTransformerOnce::<&'a str, &'a str, i32>::constant(26);
    constant.apply(input, input)
}

fn box_stateful_transformer_constant_non_static<'a>(input: &'a str) -> i32 {
    let mut constant = BoxStatefulTransformer::<&'a str, i32>::constant(27);
    constant.apply(input)
}

fn arc_stateful_transformer_constant_non_static<'a>(input: &'a str) -> i32 {
    let mut constant = ArcStatefulTransformer::<&'a str, i32>::constant(28);
    constant.apply(input)
}

fn box_stateful_bi_transformer_constant_non_static<'a>(input: &'a str) -> i32 {
    let mut constant = BoxStatefulBiTransformer::<&'a str, &'a str, i32>::constant(29);
    constant.apply(input, input)
}

fn arc_stateful_bi_transformer_constant_non_static<'a>(input: &'a str) -> i32 {
    let mut constant = ArcStatefulBiTransformer::<&'a str, &'a str, i32>::constant(30);
    constant.apply(input, input)
}
