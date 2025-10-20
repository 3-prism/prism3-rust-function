/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Demonstrates the usage of FnTransformerOnceOps extension trait
//!
//! This example shows how to directly use and_then, compose, and when methods on FnOnce closures,
//! without explicitly wrapping them in BoxTransformerOnce.

use prism3_function::{FnTransformerOnceOps, TransformerOnce};

fn main() {
    println!("=== FnTransformerOnceOps Example ===\n");

    // 1. Basic and_then composition
    println!("1. Basic and_then composition:");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let double = |x: i32| x * 2;
    let composed = parse.and_then(double);
    println!(
        "   parse.and_then(double).apply(\"21\") = {}",
        composed.apply("21".to_string())
    );
    println!();

    // 2. Chained and_then composition
    println!("2. Chained and_then composition:");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let add_one = |x: i32| x + 1;
    let double = |x: i32| x * 2;
    let chained = parse.and_then(add_one).and_then(double);
    println!(
        "   parse.and_then(add_one).and_then(double).apply(\"5\") = {}",
        chained.apply("5".to_string())
    ); // (5 + 1) * 2 = 12
    println!();

    // 3. compose reverse composition
    println!("3. compose reverse composition:");
    let double = |x: i32| x * 2;
    let to_string = |x: i32| x.to_string();
    let composed = to_string.compose(double);
    println!(
        "   to_string.compose(double).apply(21) = {}",
        composed.apply(21)
    ); // (21 * 2).to_string() = "42"
    println!();

    // 4. Conditional transformation when
    println!("4. Conditional transformation when:");
    let double = |x: i32| x * 2;
    let conditional = double.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    println!("   double.when(x > 0).or_else(negate):");
    println!("     transform(5) = {}", conditional.apply(5)); // 10

    let double2 = |x: i32| x * 2;
    let conditional2 = double2.when(|x: &i32| *x > 0).or_else(|x: i32| -x);
    println!("     transform(-5) = {}", conditional2.apply(-5)); // 5
    println!();

    // 5. Complex composition
    println!("5. Complex composition:");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let double = |x: i32| x * 2;
    let triple = |x: i32| x * 3;
    let to_string = |x: i32| x.to_string();

    let complex = parse
        .and_then(double.when(|x: &i32| *x > 5).or_else(triple))
        .and_then(to_string);

    println!("   parse.and_then(double.when(x > 5).or_else(triple)).and_then(to_string):");
    println!(
        "     transform(\"3\") = {}",
        complex.apply("3".to_string())
    ); // 3 <= 5, so 3 * 3 = 9

    let parse2 = |s: String| s.parse::<i32>().unwrap_or(0);
    let double2 = |x: i32| x * 2;
    let triple2 = |x: i32| x * 3;
    let to_string2 = |x: i32| x.to_string();
    let complex2 = parse2
        .and_then(double2.when(|x: &i32| *x > 5).or_else(triple2))
        .and_then(to_string2);
    println!(
        "     transform(\"10\") = {}",
        complex2.apply("10".to_string())
    ); // 10 > 5, so 10 * 2 = 20
    println!();

    // 6. Type conversion
    println!("6. Type conversion:");
    let to_string = |x: i32| x.to_string();
    let get_length = |s: String| s.len();
    let length_transformer = to_string.and_then(get_length);
    println!(
        "   to_string.and_then(get_length).apply(12345) = {}",
        length_transformer.apply(12345)
    ); // 5
    println!();

    // 7. Closures that capture environment
    println!("7. Closures that capture environment:");
    let multiplier = 3;
    let multiply = move |x: i32| x * multiplier;
    let add_ten = |x: i32| x + 10;
    let with_capture = multiply.and_then(add_ten);
    println!(
        "   multiply(3).and_then(add_ten).apply(5) = {}",
        with_capture.apply(5)
    ); // 5 * 3 + 10 = 25
    println!();

    // 8. Function pointers
    println!("8. Function pointers:");
    fn parse_fn(s: String) -> i32 {
        s.parse().unwrap_or(0)
    }
    fn double_fn(x: i32) -> i32 {
        x * 2
    }
    let fn_composed = parse_fn.and_then(double_fn);
    println!(
        "   parse_fn.and_then(double_fn).apply(\"21\") = {}",
        fn_composed.apply("21".to_string())
    ); // 42
    println!();

    // 9. String operations that consume ownership
    println!("9. String operations that consume ownership:");
    let owned = String::from("hello");
    let append = move |s: String| format!("{} {}", s, owned);
    let uppercase = |s: String| s.to_uppercase();
    let composed = append.and_then(uppercase);
    println!(
        "   append.and_then(uppercase).apply(\"world\") = {}",
        composed.apply("world".to_string())
    ); // "WORLD HELLO"
    println!();

    // 10. Parsing and validation
    println!("10. Parsing and validation:");
    let parse = |s: String| s.parse::<i32>().unwrap_or(0);
    let validate = |x: i32| if x > 0 { x } else { 1 };
    let composed = parse.and_then(validate);
    println!(
        "   parse.and_then(validate).apply(\"42\") = {}",
        composed.apply("42".to_string())
    ); // 42

    let parse2 = |s: String| s.parse::<i32>().unwrap_or(0);
    let validate2 = |x: i32| if x > 0 { x } else { 1 };
    let composed2 = parse2.and_then(validate2);
    println!(
        "   parse.and_then(validate).apply(\"-5\") = {}",
        composed2.apply("-5".to_string())
    ); // 1
    println!();

    println!("=== Example completed ===");
}
