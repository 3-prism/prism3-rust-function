/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! BoxBiTransformerOnce and_then method example
//!
//! Demonstrates how to use BoxBiTransformerOnce's and_then method for chained composition.

use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce};

fn main() {
    println!("=== BoxBiTransformerOnce and_then Method Example ===\n");

    // Example 1: Basic and_then usage
    println!("Example 1: Basic and_then usage");
    let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let double = |x: i32| x * 2;
    let composed = add.and_then(double);
    let result = composed.apply_once(3, 5);
    println!("  (3 + 5) * 2 = {}", result);
    assert_eq!(result, 16);
    println!();

    // Example 2: Type conversion
    println!("Example 2: Type conversion");
    let add2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let to_string = |x: i32| x.to_string();
    let composed2 = add2.and_then(to_string);
    let result2 = composed2.apply_once(20, 22);
    println!("  (20 + 22).to_string() = \"{}\"", result2);
    assert_eq!(result2, "42");
    println!();

    // Example 3: Multi-level chained composition
    println!("Example 3: Multi-level chained composition");
    let add3 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let double3 = |x: i32| x * 2;
    let to_string3 = |x: i32| format!("Result: {}", x);
    let composed3 = add3.and_then(double3).and_then(to_string3);
    let result3 = composed3.apply_once(3, 5);
    println!("  (3 + 5) * 2 -> \"{}\"", result3);
    assert_eq!(result3, "Result: 16");
    println!();

    // Example 4: String operations
    println!("Example 4: String operations");
    let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{} {}", x, y));
    let uppercase = |s: String| s.to_uppercase();
    let composed4 = concat.and_then(uppercase);
    let result4 = composed4.apply_once("hello".to_string(), "world".to_string());
    println!("  \"hello\" + \"world\" -> uppercase = \"{}\"", result4);
    assert_eq!(result4, "HELLO WORLD");
    println!();

    // Example 5: Mathematical calculation chain
    println!("Example 5: Mathematical calculation chain");
    let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
    let to_float = |x: i32| x as f64 / 2.0;
    let composed5 = multiply.and_then(to_float);
    let result5 = composed5.apply_once(6, 7);
    println!("  (6 * 7) / 2.0 = {}", result5);
    assert!((result5 - 21.0).abs() < 1e-10);
    println!();

    // Example 6: Complex business logic
    println!("Example 6: Complex business logic");
    let calculate_total =
        BoxBiTransformerOnce::new(|price: f64, quantity: i32| price * quantity as f64);
    let apply_discount = |total: f64| {
        if total > 100.0 {
            total * 0.9 // 10% discount
        } else {
            total
        }
    };
    let format_price = |total: f64| format!("${:.2}", total);
    let composed6 = calculate_total
        .and_then(apply_discount)
        .and_then(format_price);
    let result6 = composed6.apply_once(15.5, 8);
    println!("  Price: $15.5, Quantity: 8");
    println!("  Total price (with discount): {}", result6);
    assert_eq!(result6, "$111.60");
    println!();

    println!("=== All examples executed successfully! ===");
}
