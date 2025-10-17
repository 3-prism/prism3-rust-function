/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Transformer demonstration
//!
//! This example demonstrates the usage of Transformer types, including:
//! - BoxTransformer: Single ownership, reusable
//! - ArcTransformer: Thread-safe shared ownership, reusable
//! - RcTransformer: Single-threaded shared ownership, reusable

use prism3_function::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};

fn main() {
    println!("=== Transformer Demo ===\n");

    box_transformer_demo();
    arc_transformer_demo();
    rc_transformer_demo();
    composition_demo();
    blanket_impl_demo();
}

// ============================================================================
// BoxTransformer Demonstration
// ============================================================================

fn box_transformer_demo() {
    println!("--- BoxTransformer Demo ---");

    // Basic usage
    let double = BoxTransformer::new(|x: &i32| x * 2);
    println!("double.transform(&21) = {}", double.transform(&21));
    println!("double.transform(&10) = {}", double.transform(&10));

    // Identity transformer
    let identity = BoxTransformer::<i32>::identity();
    println!("identity.transform(&42) = {}", identity.transform(&42));

    // Constant transformer
    let constant = BoxTransformer::constant(100);
    println!("constant.transform(&1) = {}", constant.transform(&1));
    println!("constant.transform(&99) = {}", constant.transform(&99));

    // Composition
    let add_one = BoxTransformer::new(|x: &i32| x + 1);
    let triple = BoxTransformer::new(|x: &i32| x * 3);
    let composed = add_one.and_then(triple);
    println!(
        "composed (add_one then triple).transform(&5) = {}",
        composed.transform(&5)
    ); // (5 + 1) * 3 = 18

    println!();
}

// ============================================================================
// ArcTransformer Demonstration
// ============================================================================

fn arc_transformer_demo() {
    println!("--- ArcTransformer Demo ---");

    // Basic usage
    let double = ArcTransformer::new(|x: &i32| x * 2);
    println!("double.transform(&21) = {}", double.transform(&21));

    // Cloneable
    let cloned = double.clone();
    println!("cloned.transform(&10) = {}", cloned.transform(&10));
    println!(
        "original still usable: double.transform(&5) = {}",
        double.transform(&5)
    );

    // Identity transformer
    let identity = ArcTransformer::<i32>::identity();
    println!("identity.transform(&42) = {}", identity.transform(&42));

    // Constant transformer
    let constant = ArcTransformer::constant(100);
    println!("constant.transform(&1) = {}", constant.transform(&1));

    // Composition with references (original remains usable)
    let add_one = ArcTransformer::new(|x: &i32| x + 1);
    let triple = ArcTransformer::new(|x: &i32| x * 3);
    let composed = add_one.and_then(&triple);

    println!("add_one.transform(&5) = {}", add_one.transform(&5));
    println!("composed.transform(&5) = {}", composed.transform(&5));

    println!();
}

// ============================================================================
// RcTransformer Demonstration
// ============================================================================

fn rc_transformer_demo() {
    println!("--- RcTransformer Demo ---");

    // Basic usage
    let double = RcTransformer::new(|x: &i32| x * 2);
    println!("double.transform(&21) = {}", double.transform(&21));

    // Cloneable
    let cloned = double.clone();
    println!("cloned.transform(&10) = {}", cloned.transform(&10));
    println!(
        "original still usable: double.transform(&5) = {}",
        double.transform(&5)
    );

    // Identity transformer
    let identity = RcTransformer::<i32>::identity();
    println!("identity.transform(&42) = {}", identity.transform(&42));

    // Constant transformer
    let constant = RcTransformer::constant(100);
    println!("constant.transform(&1) = {}", constant.transform(&1));

    // Composition with references (original remains usable)
    let add_one = RcTransformer::new(|x: &i32| x + 1);
    let triple = RcTransformer::new(|x: &i32| x * 3);
    let composed = add_one.and_then(&triple);

    println!("add_one.transform(&5) = {}", add_one.transform(&5));
    println!("composed.transform(&5) = {}", composed.transform(&5));

    println!();
}

// ============================================================================
// Composition Demonstration
// ============================================================================

fn composition_demo() {
    println!("--- Composition Demo ---");

    // and_then: self -> after
    let double = BoxTransformer::new(|x: &i32| x * 2);
    let add_one = BoxTransformer::new(|x: &i32| x + 1);
    let composed1 = double.and_then(add_one);
    println!(
        "and_then (double then add_one): 5 -> {}",
        composed1.transform(&5)
    ); // 5 * 2 + 1 = 11

    // compose: before -> self
    let double = BoxTransformer::new(|x: &i32| x * 2);
    let add_one = BoxTransformer::new(|x: &i32| x + 1);
    let composed2 = double.compose(add_one);
    println!(
        "compose (add_one then double): 5 -> {}",
        composed2.transform(&5)
    ); // (5 + 1) * 2 = 12

    // Chain composition
    let t1 = ArcTransformer::new(|x: &i32| x + 1);
    let t2 = ArcTransformer::new(|x: &i32| x * 2);
    let t3 = ArcTransformer::new(|x: &i32| x - 3);
    let chained = t1.and_then(&t2).and_then(&t3);
    println!(
        "chain (add 1, mul 2, sub 3): 5 -> {}",
        chained.transform(&5)
    ); // ((5 + 1) * 2) - 3 = 9

    println!();
}

// ============================================================================
// Blanket Implementation Demonstration
// ============================================================================

fn blanket_impl_demo() {
    println!("--- Blanket Implementation Demo ---");

    // Closure can be used as Transformer directly
    let double = |x: &i32| x * 2;
    println!(
        "Closure as Transformer: double.transform(&21) = {}",
        double.transform(&21)
    );

    // Function pointer can be used as Transformer directly
    fn triple(x: &i32) -> i32 {
        x * 3
    }
    println!(
        "Function as Transformer: triple.transform(&14) = {}",
        triple.transform(&14)
    );

    // Using with iterator methods
    let values = vec![1, 2, 3, 4, 5];
    let transformer = BoxTransformer::new(|x: &i32| x * 2);
    let results: Vec<i32> = values.iter().map(|x| transformer.transform(x)).collect();
    println!("Map with transformer: {:?} -> {:?}", values, results);

    println!();
}
