/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{BoxTransformerOnce, TransformerOnce};

fn main() {
    println!("=== TransformerOnce Demo - One-Time Transformation ===\n");

    // ====================================================================
    // Part 1: BoxTransformerOnce - One-time use
    // ====================================================================
    println!("--- BoxTransformerOnce ---");
    let parse = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
    println!(
        "parse.transform(\"42\".to_string()) = {}",
        parse.transform("42".to_string())
    );
    // parse is consumed and cannot be used again

    // Composition
    let add_one = BoxTransformerOnce::new(|x: i32| x + 1);
    let double = BoxTransformerOnce::new(|x: i32| x * 2);
    let to_string = BoxTransformerOnce::new(|x: i32| x.to_string());
    let pipeline = add_one.and_then(double).and_then(to_string);
    println!("pipeline(5) = {} (expected: \"12\")", pipeline.transform(5));
    println!();

    // ====================================================================
    // Part 2: Practical Examples
    // ====================================================================
    println!("=== Practical Examples ===\n");

    // Example 1: String transformation pipeline
    println!("--- String Transformation Pipeline ---");
    let to_upper = BoxTransformerOnce::new(|s: String| s.to_uppercase());
    println!(
        "to_upper(\"hello\") = {}",
        to_upper.transform("hello".to_string())
    );
    println!();

    // Example 2: Type conversion with ownership transfer
    println!("--- Type Conversion ---");
    let into_bytes = BoxTransformerOnce::new(|s: String| s.into_bytes());
    let bytes = into_bytes.transform("hello".to_string());
    println!("into_bytes(\"hello\") = {:?}", bytes);
    println!();

    // Example 3: Complex pipeline
    println!("--- Complex Pipeline ---");
    let parser = BoxTransformerOnce::new(|s: String| s.parse::<i32>().unwrap_or(0));
    let doubler = BoxTransformerOnce::new(|x: i32| x * 2);
    let formatter = BoxTransformerOnce::new(|x: i32| format!("Result: {}", x));
    let parse_and_process = parser.and_then(doubler).and_then(formatter);

    println!(
        "parse_and_process(\"21\") = {}",
        parse_and_process.transform("21".to_string())
    );
    println!();

    // ====================================================================
    // Part 3: Trait Usage
    // ====================================================================
    println!("=== Trait Usage ===\n");

    fn apply_transformer_once<F: TransformerOnce<String, usize>>(f: F, x: String) -> usize {
        f.transform(x)
    }

    let length = BoxTransformerOnce::new(|s: String| s.len());
    println!(
        "Via trait once: {}",
        apply_transformer_once(length, "hello".to_string())
    );

    println!("\n=== Demo Complete ===");
}
