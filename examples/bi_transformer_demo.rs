/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcBiTransformer, BiTransformer, BoxBiTransformer, RcBiTransformer};

fn main() {
    println!("=== BiTransformer Demo ===\n");

    // 1. BoxBiTransformer - Single ownership
    println!("1. BoxBiTransformer - Single ownership");
    let add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    println!("   add.apply(20, 22) = {}", add.apply(20, 22));

    let multiply = BoxBiTransformer::new(|x: i32, y: i32| x * y);
    println!("   multiply.apply(6, 7) = {}", multiply.apply(6, 7));

    // Constant bi-transformer
    let constant = BoxBiTransformer::constant("hello");
    println!("   constant.apply(1, 2) = {}", constant.apply(1, 2));
    println!();

    // 2. ArcBiTransformer - Thread-safe, cloneable
    println!("2. ArcBiTransformer - Thread-safe, cloneable");
    let arc_add = ArcBiTransformer::new(|x: i32, y: i32| x + y);
    let arc_add_clone = arc_add.clone();

    println!("   arc_add.apply(10, 15) = {}", arc_add.apply(10, 15));
    println!(
        "   arc_add_clone.apply(5, 8) = {}",
        arc_add_clone.apply(5, 8)
    );
    println!();

    // 3. RcBiTransformer - Single-threaded, cloneable
    println!("3. RcBiTransformer - Single-threaded, cloneable");
    let rc_multiply = RcBiTransformer::new(|x: i32, y: i32| x * y);
    let rc_multiply_clone = rc_multiply.clone();

    println!("   rc_multiply.apply(3, 4) = {}", rc_multiply.apply(3, 4));
    println!(
        "   rc_multiply_clone.apply(5, 6) = {}",
        rc_multiply_clone.apply(5, 6)
    );
    println!();

    // 4. Conditional BiTransformer
    println!("4. Conditional BiTransformer");
    let add_if_positive = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    let multiply_otherwise = BoxBiTransformer::new(|x: i32, y: i32| x * y);
    let conditional = add_if_positive
        .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
        .or_else(multiply_otherwise);

    println!(
        "   conditional.apply(5, 3) = {} (both positive, add)",
        conditional.apply(5, 3)
    );
    println!(
        "   conditional.apply(-5, 3) = {} (not both positive, multiply)",
        conditional.apply(-5, 3)
    );
    println!();

    // 5. Working with different types
    println!("5. Working with different types");
    let format =
        BoxBiTransformer::new(|name: String, age: i32| format!("{} is {} years old", name, age));
    println!(
        "   format.apply(\"Alice\", 30) = {}",
        format.apply("Alice".to_string(), 30)
    );
    println!();

    // 6. Closure as BiTransformer
    println!("6. Closure as BiTransformer");
    let subtract = |x: i32, y: i32| x - y;
    println!("   subtract.apply(42, 10) = {}", subtract.apply(42, 10));
    println!();

    // 7. Conversion between types
    println!("7. Conversion between types");
    let box_add = BoxBiTransformer::new(|x: i32, y: i32| x + y);
    let rc_add = box_add.into_rc();
    println!("   Converted BoxBiTransformer to RcBiTransformer");
    println!("   rc_add.apply(7, 8) = {}", rc_add.apply(7, 8));
    println!();

    // 8. Safe division with Option
    println!("8. Safe division with Option");
    let safe_divide =
        BoxBiTransformer::new(|x: i32, y: i32| if y == 0 { None } else { Some(x / y) });
    println!(
        "   safe_divide.apply(42, 2) = {:?}",
        safe_divide.apply(42, 2)
    );
    println!(
        "   safe_divide.apply(42, 0) = {:?}",
        safe_divide.apply(42, 0)
    );
    println!();

    // 9. String concatenation
    println!("9. String concatenation");
    let concat = BoxBiTransformer::new(|s1: String, s2: String| format!("{}{}", s1, s2));
    println!(
        "   concat.apply(\"Hello\", \"World\") = {}",
        concat.apply("Hello".to_string(), "World".to_string())
    );
    println!();

    println!("=== Demo Complete ===");
}
