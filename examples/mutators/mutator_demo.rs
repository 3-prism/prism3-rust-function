// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Mutator Type Demo
//!
//! This example demonstrates the three implementations of Mutator (BoxMutator,
//! ArcMutator, RcMutator) and their various usage patterns.
//!
//! Mutator is used to modify values, unlike the non-mutating Consumer.

use std::thread;

use qubit_function::ArcMutator;
use qubit_function::BoxMutator;
use qubit_function::Mutator;
use qubit_function::RcMutator;

fn main() {
    println!("=== Mutator Demo ===\n");

    // ========================================================================
    // Example 1: BoxMutator Basic Usage
    // ========================================================================
    println!("Example 1: BoxMutator Basic Usage");
    println!("{}", "-".repeat(50));

    let mutator = BoxMutator::new(|x: &mut i32| {
        *x *= 2;
    });
    let mut value = 5;
    println!("Initial value: {}", value);
    mutator.apply(&mut value);
    println!("After BoxMutator execution: {}\n", value);

    // ========================================================================
    // Example 2: BoxMutator Method Chaining
    // ========================================================================
    println!("Example 2: BoxMutator Method Chaining");
    println!("{}", "-".repeat(50));

    let chained = BoxMutator::new(|x: &mut i32| {
        *x *= 2; // multiply by 2
    })
    .and_then(|x: &mut i32| {
        *x += 10; // add 10
    })
    .and_then(|x: &mut i32| {
        *x = *x * *x; // square
    });

    let mut value = 5;
    println!("Initial value: {}", value);
    chained.apply(&mut value);
    println!("Result: {} (5 * 2 + 10 = 20, 20 * 20 = 400)\n", value);

    // ========================================================================
    // Example 3: Explicitly wrapping a closure before composition
    // ========================================================================
    println!("Example 3: Explicit Closure Wrapping");
    println!("{}", "-".repeat(50));

    let closure_chain = BoxMutator::new(|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

    let mut value = 5;
    println!("Initial value: {}", value);
    closure_chain.apply(&mut value);
    println!("Result: {} (5 * 2 + 10 = 20)\n", value);

    // ========================================================================
    // Example 4: BoxMutator Factory Methods
    // ========================================================================
    println!("Example 4: BoxMutator Factory Methods");
    println!("{}", "-".repeat(50));

    // noop
    let noop = BoxMutator::<i32>::noop();
    let mut value = 42;
    println!("Before noop: {}", value);
    noop.apply(&mut value);
    println!("After noop: {} (unchanged)\n", value);

    // ========================================================================
    // Example 5: Conditional Mutator
    // ========================================================================
    println!("Example 5: Conditional Mutator");
    println!("{}", "-".repeat(50));

    // when (conditional execution)
    let increment_if_positive = BoxMutator::new(|x: &mut i32| *x += 1).when(|x: &i32| *x > 0);

    let mut positive = 5;
    let mut negative = -5;
    println!("Before when - positive: {}, negative: {}", positive, negative);
    increment_if_positive.apply(&mut positive);
    increment_if_positive.apply(&mut negative);
    println!("After when - positive: {}, negative: {}\n", positive, negative);

    // when().or_else() (conditional branching)
    let adjust = BoxMutator::new(|x: &mut i32| *x *= 2)
        .when(|x: &i32| *x > 0)
        .or_else(|x: &mut i32| *x = -*x);

    let mut positive = 10;
    let mut negative = -10;
    println!(
        "Before when().or_else() - positive: {}, negative: {}",
        positive, negative
    );
    adjust.apply(&mut positive);
    adjust.apply(&mut negative);
    println!(
        "After when().or_else() - positive: {}, negative: {}\n",
        positive, negative
    );

    // ========================================================================
    // Example 6: ArcMutator - Multi-threaded Sharing
    // ========================================================================
    println!("Example 6: ArcMutator - Multi-threaded Sharing");
    println!("{}", "-".repeat(50));

    let shared = ArcMutator::new(|x: &mut i32| *x *= 2);

    // Clone for another thread
    let shared_clone = shared.clone();
    let handle = thread::spawn(move || {
        let mut value = 5;
        let mutator = shared_clone;
        mutator.apply(&mut value);
        println!("In thread: 5 * 2 = {}", value);
        value
    });

    // Use in main thread
    let mut value = 3;
    let mutator = shared;
    mutator.apply(&mut value);
    println!("Main thread: 3 * 2 = {}", value);

    let thread_result = handle.join().expect("thread should not panic");
    println!("Thread result: {}\n", thread_result);

    // ========================================================================
    // Example 7: ArcMutator Composition (without consuming original mutator)
    // ========================================================================
    println!("Example 7: ArcMutator Composition (borrowing &self)");
    println!("{}", "-".repeat(50));

    let double = ArcMutator::new(|x: &mut i32| *x *= 2);
    let add_ten = ArcMutator::new(|x: &mut i32| *x += 10);

    // Composition doesn't consume the original mutator
    let pipeline1 = double.and_then(add_ten.clone());
    let pipeline2 = add_ten.and_then(double.clone());

    let mut value1 = 5;
    let p1 = pipeline1;
    p1.apply(&mut value1);
    println!("pipeline1 (double then add): 5 -> {}", value1);

    let mut value2 = 5;
    let p2 = pipeline2;
    p2.apply(&mut value2);
    println!("pipeline2 (add then double): 5 -> {}", value2);

    // double and add_ten are still available
    let mut value3 = 10;
    let d = double;
    d.apply(&mut value3);
    println!("Original double still available: 10 -> {}\n", value3);

    // ========================================================================
    // Example 8: RcMutator - Single-threaded Sharing
    // ========================================================================
    println!("Example 8: RcMutator - Single-threaded Sharing");
    println!("{}", "-".repeat(50));

    let rc_mutator = RcMutator::new(|x: &mut i32| *x *= 2);

    // Clone multiple copies
    let clone1 = rc_mutator.clone();
    let clone2 = rc_mutator.clone();

    let mut value1 = 5;
    let c1 = clone1;
    c1.apply(&mut value1);
    println!("clone1: 5 -> {}", value1);

    let mut value2 = 3;
    let c2 = clone2;
    c2.apply(&mut value2);
    println!("clone2: 3 -> {}", value2);

    let mut value3 = 7;
    let c3 = rc_mutator;
    c3.apply(&mut value3);
    println!("Original: 7 -> {}\n", value3);

    // ========================================================================
    // Example 9: RcMutator Composition (borrowing &self)
    // ========================================================================
    println!("Example 9: RcMutator Composition (borrowing &self)");
    println!("{}", "-".repeat(50));

    let double = RcMutator::new(|x: &mut i32| *x *= 2);
    let add_ten = RcMutator::new(|x: &mut i32| *x += 10);

    let pipeline1 = double.and_then(add_ten.clone());
    let pipeline2 = add_ten.and_then(double.clone());

    let mut value1 = 5;
    let p1 = pipeline1;
    p1.apply(&mut value1);
    println!("pipeline1 (double then add): 5 -> {}", value1);

    let mut value2 = 5;
    let p2 = pipeline2;
    p2.apply(&mut value2);
    println!("pipeline2 (add then double): 5 -> {}\n", value2);

    // ========================================================================
    // Example 10: Unified Mutator trait
    // ========================================================================
    println!("Example 10: Unified Mutator trait");
    println!("{}", "-".repeat(50));

    fn apply_to_all<M: Mutator<i32>>(mutator: &mut M, values: &mut [i32]) {
        for value in values.iter_mut() {
            mutator.apply(value);
        }
    }

    let mut values1 = vec![1, 2, 3, 4, 5];
    let mut box_mut = BoxMutator::new(|x: &mut i32| *x *= 2);
    println!("Using BoxMutator: {:?}", values1);
    apply_to_all(&mut box_mut, &mut values1);
    println!("Result: {:?}", values1);

    let mut values2 = vec![1, 2, 3, 4, 5];
    let mut arc_mut = ArcMutator::new(|x: &mut i32| *x *= 2);
    println!("Using ArcMutator: {:?}", values2);
    apply_to_all(&mut arc_mut, &mut values2);
    println!("Result: {:?}", values2);

    let mut values3 = vec![1, 2, 3, 4, 5];
    let mut rc_mut = RcMutator::new(|x: &mut i32| *x *= 2);
    println!("Using RcMutator: {:?}", values3);
    apply_to_all(&mut rc_mut, &mut values3);
    println!("Result: {:?}", values3);

    let mut values4 = vec![1, 2, 3, 4, 5];
    let mut closure = |x: &mut i32| *x *= 2;
    println!("Using closure: {:?}", values4);
    apply_to_all(&mut closure, &mut values4);
    println!("Result: {:?}\n", values4);

    // ========================================================================
    // Example 11: Complex Data Processing Pipeline
    // ========================================================================
    println!("Example 11: Complex Data Processing Pipeline");
    println!("{}", "-".repeat(50));

    let pipeline = BoxMutator::new(|x: &mut i32| {
        // Validation: clamp to 0-100
        *x = (*x).clamp(0, 100);
    })
    .and_then(|x: &mut i32| {
        // Normalization: scale to 0-10
        *x /= 10;
    })
    .and_then(|x: &mut i32| {
        // Transformation: square
        *x = *x * *x;
    });

    let mut value1 = -50;
    pipeline.apply(&mut value1);
    println!("-50 -> {}", value1);

    let mut value2 = 200;
    pipeline.apply(&mut value2);
    println!("200 -> {}", value2);

    let mut value3 = 30;
    pipeline.apply(&mut value3);
    println!("30 -> {}\n", value3);

    // ========================================================================
    // Example 12: String Processing
    // ========================================================================
    println!("Example 12: String Processing");
    println!("{}", "-".repeat(50));

    let string_processor = BoxMutator::new(|s: &mut String| s.retain(|c| !c.is_whitespace()))
        .and_then(|s: &mut String| *s = s.to_lowercase())
        .and_then(|s: &mut String| s.push_str("!!!"));

    let mut text = String::from("Hello World");
    println!("Original: {}", text);
    string_processor.apply(&mut text);
    println!("After processing: {}\n", text);

    // ========================================================================
    // Example 13: Type Conversion
    // ========================================================================
    println!("Example 13: Type Conversion");
    println!("{}", "-".repeat(50));

    // Closure -> BoxMutator
    let closure = |x: &mut i32| *x *= 2;
    let box_mut = BoxMutator::new(closure);
    let mut value = 5;
    box_mut.apply(&mut value);
    println!("Closure -> BoxMutator: 5 -> {}", value);

    // Closure -> RcMutator
    let closure = |x: &mut i32| *x *= 2;
    let rc_mut = RcMutator::new(closure);
    let mut value = 5;
    rc_mut.apply(&mut value);
    println!("Closure -> RcMutator: 5 -> {}", value);

    // Closure -> ArcMutator
    let closure = |x: &mut i32| *x *= 2;
    let arc_mut = ArcMutator::new(closure);
    let mut value = 5;
    arc_mut.apply(&mut value);
    println!("Closure -> ArcMutator: 5 -> {}", value);

    // BoxMutator -> RcMutator
    let box_mut = BoxMutator::new(|x: &mut i32| *x *= 2);
    let rc_mut = RcMutator::new(box_mut);
    let mut value = 5;
    rc_mut.apply(&mut value);
    println!("BoxMutator -> RcMutator: 5 -> {}", value);

    // RcMutator -> BoxMutator
    let rc_mut = RcMutator::new(|x: &mut i32| *x *= 2);
    let box_mut = BoxMutator::new(rc_mut);
    let mut value = 5;
    box_mut.apply(&mut value);
    println!("RcMutator -> BoxMutator: 5 -> {}\n", value);

    // ========================================================================
    // Example 14: Custom Types
    // ========================================================================
    println!("Example 14: Custom Types");
    println!("{}", "-".repeat(50));

    #[derive(Debug, Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    let processor = BoxMutator::new(|p: &mut Point| p.x *= 2)
        .and_then(|p: &mut Point| p.y *= 2)
        .and_then(|p: &mut Point| p.x += p.y);

    let mut point = Point { x: 3, y: 4 };
    println!("Original point: {:?}", point);
    processor.apply(&mut point);
    println!("After processing: {:?}\n", point);

    println!("=== All Examples Completed ===");
}
