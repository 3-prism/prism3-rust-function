/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # TransformerOnce Specialized Methods Demo
//!
//! Demonstrates the specialized `into_xxx` and `to_xxx` methods for
//! `ArcTransformer` and `RcTransformer` when implementing `TransformerOnce`.
//!
//! # Author
//!
//! Hu Haixing

use prism3_function::{ArcTransformer, RcTransformer, Transformer, TransformerOnce};

fn main() {
    println!("=== TransformerOnce Specialized Methods Demo ===\n");

    // ============================================================================
    // ArcTransformer TransformerOnce specialized methods
    // ============================================================================

    println!("1. ArcTransformer TransformerOnce specialized methods:");

    let arc_double = ArcTransformer::new(|x: i32| x * 2);

    // Test into_box_once - consumes self
    let boxed_once = arc_double.clone().into_box_once();
    println!(
        "   ArcTransformer::into_box_once(): {}",
        boxed_once.apply_once(21)
    );

    // Test into_fn_once - consumes self
    let fn_once = arc_double.clone().into_fn_once();
    println!("   ArcTransformer::into_fn_once(): {}", fn_once(21));

    // Test to_box_once - borrows self
    let boxed_once_borrowed = arc_double.to_box_once();
    println!(
        "   ArcTransformer::to_box_once(): {}",
        boxed_once_borrowed.apply_once(21)
    );

    // Test to_fn_once - borrows self
    let fn_once_borrowed = arc_double.to_fn_once();
    println!("   ArcTransformer::to_fn_once(): {}", fn_once_borrowed(21));

    // Original transformer still usable after to_xxx methods
    println!(
        "   Original ArcTransformer still works: {}",
        arc_double.apply(21)
    );

    println!();

    // ============================================================================
    // RcTransformer TransformerOnce specialized methods
    // ============================================================================

    println!("2. RcTransformer TransformerOnce specialized methods:");

    let rc_triple = RcTransformer::new(|x: i32| x * 3);

    // Test into_box_once - consumes self
    let boxed_once = rc_triple.clone().into_box_once();
    println!(
        "   RcTransformer::into_box_once(): {}",
        boxed_once.apply_once(14)
    );

    // Test into_fn_once - consumes self
    let fn_once = rc_triple.clone().into_fn_once();
    println!("   RcTransformer::into_fn_once(): {}", fn_once(14));

    // Test to_box_once - borrows self
    let boxed_once_borrowed = rc_triple.to_box_once();
    println!(
        "   RcTransformer::to_box_once(): {}",
        boxed_once_borrowed.apply_once(14)
    );

    // Test to_fn_once - borrows self
    let fn_once_borrowed = rc_triple.to_fn_once();
    println!("   RcTransformer::to_fn_once(): {}", fn_once_borrowed(14));

    // Original transformer still usable after to_xxx methods
    println!(
        "   Original RcTransformer still works: {}",
        rc_triple.apply(14)
    );

    println!();

    // ============================================================================
    // Comparison with default implementations
    // ============================================================================

    println!("3. Performance comparison (specialized vs default):");

    let arc_square = ArcTransformer::new(|x: i32| x * x);

    // Using specialized method (more efficient)
    let specialized_box = arc_square.clone().into_box_once();
    println!(
        "   Specialized into_box_once: {}",
        specialized_box.apply_once(5)
    );

    // Using default implementation (less efficient)
    let default_box = arc_square.clone().into_box_once();
    println!("   Default into_box_once: {}", default_box.apply_once(5));

    println!();

    // ============================================================================
    // Thread safety demonstration for ArcTransformer
    // ============================================================================

    println!("4. Thread safety with ArcTransformer:");

    let arc_shared = ArcTransformer::new(|x: i32| x + 100);

    // Clone for thread safety
    let arc_clone = arc_shared.clone();

    // Use in different thread context (simulated)
    let handle = std::thread::spawn(move || {
        let boxed = arc_clone.into_box_once();
        boxed.apply_once(50)
    });

    let result = handle.join().unwrap();
    println!("   Thread-safe ArcTransformer result: {}", result);

    // Original still usable
    println!(
        "   Original ArcTransformer still works: {}",
        arc_shared.apply(50)
    );

    println!();

    // ============================================================================
    // String transformation example
    // ============================================================================

    println!("5. String transformation with specialized methods:");

    let arc_uppercase = ArcTransformer::new(|s: String| s.to_uppercase());

    // Test with string input
    let test_string = "hello world".to_string();

    // Using specialized methods
    let boxed_upper = arc_uppercase.clone().into_box_once();
    let result = boxed_upper.apply_once(test_string.clone());
    println!(
        "   String transformation: '{}' -> '{}'",
        test_string, result
    );

    // Using to_xxx methods (borrowing)
    let fn_upper = arc_uppercase.to_fn_once();
    let result2 = fn_upper(test_string.clone());
    println!(
        "   String transformation (borrowed): '{}' -> '{}'",
        test_string, result2
    );

    // Original still usable
    println!(
        "   Original ArcTransformer still works: '{}'",
        arc_uppercase.apply(test_string)
    );

    println!("\n=== Demo completed successfully! ===");
}
