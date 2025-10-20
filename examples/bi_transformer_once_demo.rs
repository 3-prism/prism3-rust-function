/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! BiTransformerOnce usage examples

use prism3_function::{BiTransformerOnce, BoxBiTransformerOnce};

fn main() {
    println!("=== BiTransformerOnce Examples ===\n");

    // Example 1: Basic usage with closure
    println!("1. Basic usage with closure:");
    let add = |x: i32, y: i32| x + y;
    let result = add.apply(20, 22);
    println!("   20 + 22 = {}", result);

    // Example 2: BoxBiTransformerOnce with new
    println!("\n2. BoxBiTransformerOnce with new:");
    let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
    println!("   6 * 7 = {}", multiply.apply(6, 7));

    // Example 3: Constant transformer
    println!("\n3. Constant transformer:");
    let constant = BoxBiTransformerOnce::constant("hello");
    println!("   constant(123, 456) = {}", constant.apply(123, 456));

    // Example 4: Consuming owned values
    println!("\n4. Consuming owned values:");
    let concat = BoxBiTransformerOnce::new(|x: String, y: String| format!("{} {}", x, y));
    let s1 = String::from("hello");
    let s2 = String::from("world");
    let result = concat.apply(s1, s2);
    println!("   concat('hello', 'world') = {}", result);

    // Example 5: Conditional transformation with when/or_else
    println!("\n5. Conditional transformation (positive numbers):");
    let add = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let multiply = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
    let conditional = add
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply);
    println!("   conditional(5, 3) = {} (add)", conditional.apply(5, 3));

    println!("\n6. Conditional transformation (negative numbers):");
    let add2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let multiply2 = BoxBiTransformerOnce::new(|x: i32, y: i32| x * y);
    let conditional2 = add2
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply2);
    println!(
        "   conditional(-5, 3) = {} (multiply)",
        conditional2.apply(-5, 3)
    );

    // Example 7: Conditional with closure in or_else
    println!("\n7. Conditional with closure in or_else:");
    let add3 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let conditional3 = add3
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(|x: i32, y: i32| x * y);
    println!("   conditional(4, 6) = {}", conditional3.apply(4, 6));

    // Example 8: Merging vectors
    println!("\n8. Merging vectors:");
    let merge = BoxBiTransformerOnce::new(|mut x: Vec<i32>, y: Vec<i32>| {
        x.extend(y);
        x
    });
    let v1 = vec![1, 2, 3];
    let v2 = vec![4, 5, 6];
    let result = merge.apply(v1, v2);
    println!("   merge([1, 2, 3], [4, 5, 6]) = {:?}", result);

    // Example 9: Complex transformation with calculation
    println!("\n9. Complex transformation with calculation:");
    let calculate = BoxBiTransformerOnce::new(|x: i32, y: i32| {
        let sum = x + y;
        let product = x * y;
        (sum, product)
    });
    let (sum, product) = calculate.apply(5, 3);
    println!("   calculate(5, 3) = (sum: {}, product: {})", sum, product);

    // Example 10: String manipulation
    println!("\n10. String manipulation:");
    let process = BoxBiTransformerOnce::new(|x: String, y: String| {
        format!("{} {} {}", x.to_uppercase(), "and", y.to_lowercase())
    });
    println!(
        "   process('Hello', 'WORLD') = {}",
        process.apply("Hello".to_string(), "WORLD".to_string())
    );

    // Example 11: Converting to function
    println!("\n11. Converting to function:");
    let add4 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let f = add4.into_fn();
    println!("   f(10, 20) = {}", f(10, 20));

    // Example 12: Converting to box (zero-cost)
    println!("\n12. Converting to box (zero-cost):");
    let add5 = BoxBiTransformerOnce::new(|x: i32, y: i32| x + y);
    let boxed = add5.into_box();
    println!("   boxed(15, 25) = {}", boxed.apply(15, 25));

    println!("\n=== All examples completed successfully! ===");
}
