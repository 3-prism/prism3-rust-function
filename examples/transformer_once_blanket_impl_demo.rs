/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Demonstrates FnOnce -> TransformerOnce blanket implementations

use prism3_function::TransformerOnce;

fn main() {
    println!("=== Testing FnOnce -> TransformerOnce ===");
    test_transformer_once();
}

fn test_transformer_once() {
    // Test function pointer
    fn parse(s: String) -> i32 {
        s.parse().unwrap_or(0)
    }
    assert_eq!(parse.apply("42".to_string()), 42);
    println!("✓ Function pointer test passed: parse.apply(\"42\") = 42");

    // Test closure that consumes ownership
    let owned_value = String::from("hello");
    let consume = |s: String| format!("{} world", s);
    assert_eq!(consume.apply(owned_value), "hello world");
    println!("✓ Closure that consumes ownership test passed");

    // Test conversion to BoxTransformerOnce
    let transform = |s: String| s.to_uppercase();
    let boxed = transform.into_box();
    assert_eq!(boxed.apply("hello".to_string()), "HELLO");
    println!("✓ into_box() test passed");

    // Test into_fn
    let transform2 = |s: String| s.len();
    let func = transform2.into_fn();
    assert_eq!(func("hello".to_string()), 5);
    println!("✓ into_fn() test passed");
}
