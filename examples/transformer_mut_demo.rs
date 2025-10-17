/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! TransformerMut demonstration
//!
//! This example demonstrates the usage of TransformerMut types, including:
//! - BoxTransformerMut: Single ownership, reusable, can maintain state
//! - ArcTransformerMut: Thread-safe shared ownership, reusable, can maintain
//!   state
//! - RcTransformerMut: Single-threaded shared ownership, reusable, can
//!   maintain state

use prism3_function::{ArcTransformerMut, BoxTransformerMut, RcTransformerMut, TransformerMut};

fn main() {
    println!("=== TransformerMut Demo ===\n");

    box_transformer_mut_demo();
    arc_transformer_mut_demo();
    rc_transformer_mut_demo();
    composition_demo();
    stateful_demo();
    blanket_impl_demo();
}

// ============================================================================
// BoxTransformerMut Demonstration
// ============================================================================

fn box_transformer_mut_demo() {
    println!("--- BoxTransformerMut Demo ---");

    // Basic usage - modifies input and returns result
    let mut double_in_place = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    println!(
        "double_in_place.transform(&mut 21) = {}",
        double_in_place.transform(&mut value)
    );
    println!("value after transform = {}", value);

    // Identity transformer
    let mut identity = BoxTransformerMut::<i32>::identity();
    let mut val = 42;
    println!(
        "identity.transform(&mut 42) = {}",
        identity.transform(&mut val)
    );

    // Multiple calls
    let mut incrementer = BoxTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });

    let mut val1 = 10;
    println!(
        "First call: incrementer.transform(&mut 10) = {}",
        incrementer.transform(&mut val1)
    );

    let mut val2 = 20;
    println!(
        "Second call: incrementer.transform(&mut 20) = {}",
        incrementer.transform(&mut val2)
    );

    println!();
}

// ============================================================================
// ArcTransformerMut Demonstration
// ============================================================================

fn arc_transformer_mut_demo() {
    println!("--- ArcTransformerMut Demo ---");

    // Basic usage
    let mut double_in_place = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    println!(
        "double_in_place.transform(&mut 21) = {}",
        double_in_place.transform(&mut value)
    );
    println!("value after transform = {}", value);

    // Cloneable
    let double = ArcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let mut cloned = double.clone();

    let mut val1 = 10;
    println!(
        "cloned.transform(&mut 10) = {}",
        cloned.transform(&mut val1)
    );

    // Identity transformer
    let mut identity = ArcTransformerMut::<i32>::identity();
    let mut val = 42;
    println!(
        "identity.transform(&mut 42) = {}",
        identity.transform(&mut val)
    );

    println!();
}

// ============================================================================
// RcTransformerMut Demonstration
// ============================================================================

fn rc_transformer_mut_demo() {
    println!("--- RcTransformerMut Demo ---");

    // Basic usage
    let mut double_in_place = RcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut value = 21;
    println!(
        "double_in_place.transform(&mut 21) = {}",
        double_in_place.transform(&mut value)
    );
    println!("value after transform = {}", value);

    // Cloneable
    let double = RcTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let mut cloned = double.clone();

    let mut val1 = 10;
    println!(
        "cloned.transform(&mut 10) = {}",
        cloned.transform(&mut val1)
    );

    // Identity transformer
    let mut identity = RcTransformerMut::<i32>::identity();
    let mut val = 42;
    println!(
        "identity.transform(&mut 42) = {}",
        identity.transform(&mut val)
    );

    println!();
}

// ============================================================================
// Composition Demonstration
// ============================================================================

fn composition_demo() {
    println!("--- Composition Demo ---");

    // and_then: self -> after
    let double = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let add_one = BoxTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });
    let mut composed1 = double.and_then(add_one);

    let mut val1 = 5;
    println!(
        "and_then (double then add_one): 5 -> {}",
        composed1.transform(&mut val1)
    ); // 5 * 2 + 1 = 11

    // compose: before -> self
    let double = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });
    let add_one = BoxTransformerMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });
    let mut composed2 = double.compose(add_one);

    let mut val2 = 5;
    println!(
        "compose (add_one then double): 5 -> {}",
        composed2.transform(&mut val2)
    ); // (5 + 1) * 2 = 12

    println!();
}

// ============================================================================
// Stateful Demonstration
// ============================================================================

fn stateful_demo() {
    println!("--- Stateful Demo ---");

    // BoxTransformerMut can maintain internal state
    let mut counter = 0;
    let mut counting_double = BoxTransformerMut::new(move |x: &mut i32| {
        counter += 1;
        println!("  Call #{}: doubling {}", counter, *x);
        *x *= 2;
        *x
    });

    let mut val1 = 10;
    let result1 = counting_double.transform(&mut val1);
    println!("Result: {}\n", result1);

    let mut val2 = 5;
    let result2 = counting_double.transform(&mut val2);
    println!("Result: {}\n", result2);

    let mut val3 = 7;
    let result3 = counting_double.transform(&mut val3);
    println!("Result: {}", result3);

    println!();
}

// ============================================================================
// Blanket Implementation Demonstration
// ============================================================================

fn blanket_impl_demo() {
    println!("--- Blanket Implementation Demo ---");

    // Closure can be used as TransformerMut directly
    let mut counter = 0;
    let mut counting_double = |x: &mut i32| {
        counter += 1;
        *x *= 2;
        *x + counter
    };

    let mut val1 = 10;
    println!(
        "Closure as TransformerMut: transform(&mut 10) = {}",
        counting_double.transform(&mut val1)
    );

    let mut val2 = 5;
    println!(
        "Second call: transform(&mut 5) = {}",
        counting_double.transform(&mut val2)
    );

    // Using with collections
    let mut values = vec![1, 2, 3, 4, 5];
    let mut doubler = BoxTransformerMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    println!("\nTransforming vector elements:");
    println!("Original: {:?}", values);
    for val in values.iter_mut() {
        doubler.transform(val);
    }
    println!("After doubling: {:?}", values);

    println!();
}
