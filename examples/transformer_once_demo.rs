/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! TransformerOnce demonstration
//!
//! This example demonstrates the usage of TransformerOnce types, including:
//! - BoxTransformerOnce: Single ownership, one-time use
//! - ArcTransformerOnce: Thread-safe shared ownership, reusable (cloneable)
//! - RcTransformerOnce: Single-threaded shared ownership, reusable
//!   (cloneable)

use prism3_function::{ArcTransformerOnce, BoxTransformerOnce, RcTransformerOnce, TransformerOnce};

fn main() {
    println!("=== TransformerOnce Demo ===\n");

    box_transformer_once_demo();
    arc_transformer_once_demo();
    rc_transformer_once_demo();
    composition_demo();
    blanket_impl_demo();
}

// ============================================================================
// BoxTransformerOnce Demonstration
// ============================================================================

fn box_transformer_once_demo() {
    println!("--- BoxTransformerOnce Demo ---");

    // Basic usage - consumes input
    let to_upper = BoxTransformerOnce::new(|s: String| s.to_uppercase());
    let input = String::from("hello world");
    println!(
        "to_upper.transform(\"hello world\") = {}",
        to_upper.transform(input)
    );
    // Note: input has been consumed and cannot be used anymore

    // Identity transformer
    let identity = BoxTransformerOnce::<i32>::identity();
    println!("identity.transform(42) = {}", identity.transform(42));

    // Constant transformer
    let constant = BoxTransformerOnce::constant(100);
    println!("constant.transform(42) = {}", constant.transform(42));

    // Capturing values by move
    let prefix = String::from("Hello, ");
    let add_prefix = BoxTransformerOnce::new(move |name: String| format!("{}{}", prefix, name));
    println!(
        "add_prefix.transform(\"Alice\") = {}",
        add_prefix.transform("Alice".to_string())
    );

    println!();
}

// ============================================================================
// ArcTransformerOnce Demonstration
// ============================================================================

fn arc_transformer_once_demo() {
    println!("--- ArcTransformerOnce Demo ---");

    // Basic usage
    let to_upper = ArcTransformerOnce::new(|s: String| s.to_uppercase());
    println!(
        "to_upper.transform(\"hello\") = {}",
        to_upper.transform("hello".to_string())
    );

    // Cloneable - can be reused
    let double = ArcTransformerOnce::new(|x: i32| x * 2);
    let cloned = double.clone();

    println!("double.transform(21) = {}", double.transform(21));
    println!("cloned.transform(10) = {}", cloned.transform(10));

    // Identity transformer
    let identity = ArcTransformerOnce::<i32>::identity();
    println!("identity.transform(42) = {}", identity.transform(42));

    // Constant transformer
    let constant = ArcTransformerOnce::constant(100);
    println!("constant.transform(42) = {}", constant.transform(42));

    // Composition with references (original remains usable)
    let add_one = ArcTransformerOnce::new(|x: i32| x + 1);
    let triple = ArcTransformerOnce::new(|x: i32| x * 3);
    let composed = add_one.and_then(&triple);

    println!("add_one.transform(5) = {}", add_one.transform(5));
    println!(
        "composed (add_one then triple).transform(5) = {}",
        composed.transform(5)
    ); // (5 + 1) * 3 = 18

    println!();
}

// ============================================================================
// RcTransformerOnce Demonstration
// ============================================================================

fn rc_transformer_once_demo() {
    println!("--- RcTransformerOnce Demo ---");

    // Basic usage
    let to_upper = RcTransformerOnce::new(|s: String| s.to_uppercase());
    println!(
        "to_upper.transform(\"hello\") = {}",
        to_upper.transform("hello".to_string())
    );

    // Cloneable - can be reused
    let double = RcTransformerOnce::new(|x: i32| x * 2);
    let cloned = double.clone();

    println!("double.transform(21) = {}", double.transform(21));
    println!("cloned.transform(10) = {}", cloned.transform(10));

    // Identity transformer
    let identity = RcTransformerOnce::<i32>::identity();
    println!("identity.transform(42) = {}", identity.transform(42));

    // Constant transformer
    let constant = RcTransformerOnce::constant(100);
    println!("constant.transform(42) = {}", constant.transform(42));

    // Composition with references (original remains usable)
    let add_one = RcTransformerOnce::new(|x: i32| x + 1);
    let triple = RcTransformerOnce::new(|x: i32| x * 3);
    let composed = add_one.and_then(&triple);

    println!("add_one.transform(5) = {}", add_one.transform(5));
    println!(
        "composed (add_one then triple).transform(5) = {}",
        composed.transform(5)
    ); // (5 + 1) * 3 = 18

    println!();
}

// ============================================================================
// Composition Demonstration
// ============================================================================

fn composition_demo() {
    println!("--- Composition Demo ---");

    // and_then: self -> after
    let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
    let triple = |x: i32| x * 3;
    let composed1 = add_one.and_then(triple);
    println!(
        "and_then (add_one then triple): 5 -> {}",
        composed1.transform(5)
    ); // (5 + 1) * 3 = 18

    // compose: before -> self
    let double = BoxTransformerOnce::new(|x: i32| x * 2);
    let add_one = |x: i32| x + 1;
    let composed2 = double.compose(add_one);
    println!(
        "compose (add_one then double): 5 -> {}",
        composed2.transform(5)
    ); // (5 + 1) * 2 = 12

    // Chain composition with ArcTransformerOnce
    let t1 = ArcTransformerOnce::new(|x: i32| x + 1);
    let t2 = ArcTransformerOnce::new(|x: i32| x * 2);
    let t3 = ArcTransformerOnce::new(|x: i32| x - 3);
    let chained = t1.and_then(&t2).and_then(&t3);

    println!("Original t1.transform(5) = {}", t1.transform(5));
    println!("chain (add 1, mul 2, sub 3): 5 -> {}", chained.transform(5)); // ((5 + 1) * 2) - 3 = 9

    println!();
}

// ============================================================================
// Blanket Implementation Demonstration
// ============================================================================

fn blanket_impl_demo() {
    println!("--- Blanket Implementation Demo ---");

    // Closure can be used as TransformerOnce directly
    let double = |x: i32| x * 2;
    println!(
        "Closure as TransformerOnce: double.transform(21) = {}",
        double.transform(21)
    );

    // Function pointer can be used as TransformerOnce directly
    fn triple(x: i32) -> i32 {
        x * 3
    }
    println!(
        "Function as TransformerOnce: triple.transform(14) = {}",
        triple.transform(14)
    );

    // Using with iterator methods
    let values = vec![1, 2, 3, 4, 5];
    let transformer = ArcTransformerOnce::new(|x: i32| x * 2);

    println!("\nUsing with iterator:");
    let results: Vec<i32> = values
        .into_iter()
        .map(|x| transformer.clone().transform(x))
        .collect();
    println!("Results: {:?}", results);

    println!();
}
