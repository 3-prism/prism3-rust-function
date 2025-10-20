/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Demonstrates blanket implementations for Fn -> Transformer

use prism3_function::Transformer;

fn main() {
    println!("=== Testing Fn -> Transformer ===");
    test_transformer();
}

fn test_transformer() {
    // Test function pointer
    fn double(x: i32) -> i32 {
        x * 2
    }
    assert_eq!(double.apply(21), 42);
    println!("✓ Function pointer test passed: double.apply(21) = 42");

    // Test closure
    let triple = |x: i32| x * 3;
    assert_eq!(triple.apply(14), 42);
    println!("✓ Closure test passed: triple.apply(14) = 42");

    // Test conversion to BoxTransformer
    let quad = |x: i32| x * 4;
    let boxed = Transformer::into_box(quad);
    assert_eq!(boxed.apply(10), 40);
    println!("✓ into_box() test passed");

    // Test conversion to RcTransformer
    let times_five = |x: i32| x * 5;
    let rc = Transformer::into_rc(times_five);
    assert_eq!(rc.apply(8), 40);
    println!("✓ into_rc() test passed");

    // Test conversion to ArcTransformer
    let times_six = |x: i32| x * 6;
    let arc = Transformer::into_arc(times_six);
    assert_eq!(arc.apply(7), 42);
    println!("✓ into_arc() test passed");
}
