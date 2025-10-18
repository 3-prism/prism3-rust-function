/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # FnBiTransformerOps Demo
//!
//! Demonstrates how to use the `FnBiTransformerOps` trait to provide `and_then` and `when` methods for closures.

use prism3_function::{BiTransformer, FnBiTransformerOps};

fn main() {
    println!("=== FnBiTransformerOps Demo ===\n");

    // Example 1: Basic and_then composition
    println!("1. Basic and_then composition:");
    let add = |x: i32, y: i32| x + y;
    let double = |x: i32| x * 2;

    let composed = add.and_then(double);
    let result = composed.transform(3, 5);
    println!("   (3 + 5) * 2 = {}", result);
    println!();

    // Example 2: Type conversion and_then
    println!("2. Type conversion and_then:");
    let multiply = |x: i32, y: i32| x * y;
    let to_string = |x: i32| format!("Result: {}", x);

    let composed = multiply.and_then(to_string);
    let result = composed.transform(6, 7);
    println!("   6 * 7 = {}", result);
    println!();

    // Example 3: Conditional execution - when
    println!("3. Conditional execution - when:");
    let add = |x: i32, y: i32| x + y;
    let multiply = |x: i32, y: i32| x * y;

    let conditional = add
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply);

    println!("   When both numbers are positive, perform addition, otherwise multiplication:");
    println!("   conditional(5, 3) = {}", conditional.transform(5, 3));
    println!("   conditional(-5, 3) = {}", conditional.transform(-5, 3));
    println!();

    // Example 4: Complex conditional logic
    println!("4. Complex conditional logic:");
    let add = |x: i32, y: i32| x + y;
    let subtract = |x: i32, y: i32| x - y;

    let conditional = add
        .when(|x: &i32, y: &i32| (*x + *y) < 100)
        .or_else(subtract);

    println!("   When sum is less than 100, perform addition, otherwise subtraction:");
    println!("   conditional(30, 40) = {}", conditional.transform(30, 40));
    println!("   conditional(60, 50) = {}", conditional.transform(60, 50));
    println!();

    // Example 5: String operations
    println!("5. String operations:");
    let concat = |x: String, y: String| format!("{}-{}", x, y);
    let uppercase = |s: String| s.to_uppercase();

    let composed = concat.and_then(uppercase);
    let result = composed.transform("hello".to_string(), "world".to_string());
    println!("   concat + uppercase: {}", result);
    println!();

    // Example 6: Function pointers can also be used
    println!("6. Function pointers can also be used:");
    fn add_fn(x: i32, y: i32) -> i32 {
        x + y
    }
    fn triple(x: i32) -> i32 {
        x * 3
    }

    let composed = add_fn.and_then(triple);
    let result = composed.transform(4, 6);
    println!("   (4 + 6) * 3 = {}", result);
    println!();

    // Example 7: Real application - Calculator
    println!("7. Real application - Simple calculator:");
    let calculate = |x: i32, y: i32| x + y;
    let format_result = |result: i32| {
        if result >= 0 {
            format!("✓ Result: {}", result)
        } else {
            format!("✗ Negative result: {}", result)
        }
    };

    let calculator = calculate.and_then(format_result);
    println!("   10 + 5 = {}", calculator.transform(10, 5));
    println!("   -10 + 3 = {}", calculator.transform(-10, 3));
    println!();

    // Example 8: Combining multiple operations
    println!("8. Combining multiple operations:");
    let add = |x: i32, y: i32| x + y;

    // First calculate the sum, then choose different formatting based on whether it's even
    let sum_and_format = add.and_then(|n| {
        if n % 2 == 0 {
            format!("{} is even", n)
        } else {
            format!("{} is odd", n)
        }
    });

    println!("   3 + 5 = {}", sum_and_format.transform(3, 5));
    println!("   4 + 6 = {}", sum_and_format.transform(4, 6));

    println!("\n=== Demo completed ===");
}
