// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BiFunction;
use qubit_function::BiFunctionOnce;
use qubit_function::BiMutatingFunction;
use qubit_function::BiMutatingFunctionOnce;
use qubit_function::BoxBiFunction;
use qubit_function::BoxBiFunctionOnce;
use qubit_function::BoxBiMutatingFunction;
use qubit_function::BoxBiMutatingFunctionOnce;
use qubit_function::BoxFunction;
use qubit_function::BoxFunctionOnce;
use qubit_function::BoxMutatingFunction;
use qubit_function::BoxMutatingFunctionOnce;
use qubit_function::BoxStatefulFunction;
use qubit_function::BoxStatefulMutatingFunction;
use qubit_function::Function;
use qubit_function::FunctionOnce;
use qubit_function::MutatingFunction;
use qubit_function::MutatingFunctionOnce;
use qubit_function::StatefulFunction;
use qubit_function::StatefulMutatingFunction;

fn main() {
    println!("=== Function Family Demo ===\n");

    demo_borrowed_functions();
    demo_mutating_functions();
    demo_bi_functions();
    demo_bi_mutating_functions();
}

fn demo_borrowed_functions() {
    println!("--- Borrowed-input functions ---");

    let len = BoxFunction::new(|value: &String| value.len());
    let value = String::from("qubit");
    println!("Function length: {}", len.apply(&value));
    println!("Original value is still available: {value}");

    let greeting = BoxFunctionOnce::new(|name: &String| format!("hello, {name}"));
    println!("FunctionOnce greeting: {}", greeting.apply(&value));

    let mut call_count = 0;
    let mut stateful = BoxStatefulFunction::new(move |input: &i32| {
        call_count += 1;
        input + call_count
    });
    println!("StatefulFunction first call: {}", stateful.apply(&40));
    println!("StatefulFunction second call: {}", stateful.apply(&40));
    println!();
}

fn demo_mutating_functions() {
    println!("--- Mutating functions ---");

    let push_answer = BoxMutatingFunction::new(|items: &mut Vec<i32>| {
        items.push(42);
        items.len()
    });
    let mut items = vec![1, 2, 3];
    println!("MutatingFunction new length: {}", push_answer.apply(&mut items));
    println!("Items after mutation: {items:?}");

    let append_once = BoxMutatingFunctionOnce::new(|text: &mut String| {
        text.push_str(" once");
        text.len()
    });
    let mut text = String::from("called");
    println!("MutatingFunctionOnce length: {}", append_once.apply(&mut text));
    println!("Text after one-time mutation: {text}");

    let mut step = 0;
    let mut stateful = BoxStatefulMutatingFunction::new(move |value: &mut i32| {
        step += 1;
        *value += step;
        *value
    });
    let mut value = 40;
    println!("StatefulMutatingFunction first call: {}", stateful.apply(&mut value));
    println!("StatefulMutatingFunction second call: {}", stateful.apply(&mut value));
    println!();
}

fn demo_bi_functions() {
    println!("--- Bi-functions ---");

    let describe = BoxBiFunction::new(|name: &String, score: &i32| format!("{name} scored {score}"));
    let name = String::from("Alice");
    let score = 98;
    println!("BiFunction: {}", describe.apply(&name, &score));

    let once = BoxBiFunctionOnce::new(|left: &String, right: &String| format!("{left}-{right}"));
    let left = String::from("task");
    let right = String::from("done");
    println!("BiFunctionOnce: {}", once.apply(&left, &right));
    println!();
}

fn demo_bi_mutating_functions() {
    println!("--- Bi-mutating functions ---");

    let move_one = BoxBiMutatingFunction::new(|source: &mut Vec<i32>, target: &mut Vec<i32>| {
        if let Some(value) = source.pop() {
            target.push(value);
        }
        target.len()
    });
    let mut source = vec![1, 2, 3];
    let mut target = vec![10];
    println!(
        "BiMutatingFunction target length: {}",
        move_one.apply(&mut source, &mut target)
    );
    println!("Source: {source:?}, target: {target:?}");

    let merge_once = BoxBiMutatingFunctionOnce::new(|left: &mut String, right: &mut String| {
        left.push_str(right);
        right.clear();
        left.len()
    });
    let mut left = String::from("hello");
    let mut right = String::from(" world");
    println!(
        "BiMutatingFunctionOnce merged length: {}",
        merge_once.apply(&mut left, &mut right)
    );
    println!("Left: {left}, right is empty: {}", right.is_empty());
}
