/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcTransformer, BoxTransformer, RcTransformer, Transformer};
use std::collections::HashMap;
use std::thread;

fn main() {
    println!("=== Transformer Demo - Type Transformation (consumes T) ===\n");

    // ====================================================================
    // Part 1: BoxTransformer - Single ownership, reusable
    // ====================================================================
    println!("--- BoxTransformer ---");
    let double = BoxTransformer::new(|x: i32| x * 2);
    println!("double.transform(21) = {}", double.transform(21));
    println!("double.transform(42) = {}", double.transform(42));

    // Identity and constant
    let identity = BoxTransformer::<i32, i32>::identity();
    println!("identity.transform(42) = {}", identity.transform(42));

    let constant = BoxTransformer::constant("hello");
    println!("constant.transform(123) = {}", constant.transform(123));
    println!();

    // ====================================================================
    // Part 2: ArcTransformer - Thread-safe, cloneable
    // ====================================================================
    println!("--- ArcTransformer ---");
    let arc_double = ArcTransformer::new(|x: i32| x * 2);
    let arc_cloned = arc_double.clone();

    println!("arc_double.transform(21) = {}", arc_double.transform(21));
    println!("arc_cloned.transform(42) = {}", arc_cloned.transform(42));

    // Multi-threaded usage
    let for_thread = arc_double.clone();
    let handle = thread::spawn(move || for_thread.transform(100));
    println!(
        "In main thread: arc_double.transform(50) = {}",
        arc_double.transform(50)
    );
    println!("In child thread: result = {}", handle.join().unwrap());
    println!();

    // ====================================================================
    // Part 3: RcTransformer - Single-threaded, cloneable
    // ====================================================================
    println!("--- RcTransformer ---");
    let rc_double = RcTransformer::new(|x: i32| x * 2);
    let rc_cloned = rc_double.clone();

    println!("rc_double.transform(21) = {}", rc_double.transform(21));
    println!("rc_cloned.transform(42) = {}", rc_cloned.transform(42));
    println!();

    // ====================================================================
    // Part 4: Practical Examples
    // ====================================================================
    println!("=== Practical Examples ===\n");

    // Example 1: String transformation
    println!("--- String Transformation ---");
    let to_upper = BoxTransformer::new(|s: String| s.to_uppercase());
    println!(
        "to_upper.transform('hello') = {}",
        to_upper.transform("hello".to_string())
    );
    println!(
        "to_upper.transform('world') = {}",
        to_upper.transform("world".to_string())
    );
    println!();

    // Example 2: Type conversion pipeline
    println!("--- Type Conversion Pipeline ---");
    let parse_int = BoxTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0));
    let double_int = BoxTransformer::new(|x: i32| x * 2);
    let to_string = BoxTransformer::new(|x: i32| x.to_string());

    let pipeline = parse_int.and_then(double_int).and_then(to_string);
    println!(
        "pipeline.transform('21') = {}",
        pipeline.transform("21".to_string())
    );
    println!();

    // Example 3: Shared transformation logic
    println!("--- Shared Transformation Logic ---");
    let square = ArcTransformer::new(|x: i32| x * x);

    // Can be shared across different parts of the program
    let transformer1 = square.clone();
    let transformer2 = square.clone();

    println!("transformer1.transform(5) = {}", transformer1.transform(5));
    println!("transformer2.transform(7) = {}", transformer2.transform(7));
    println!("square.transform(3) = {}", square.transform(3));
    println!();

    // Example 4: Transformer registry
    println!("--- Transformer Registry ---");
    let mut transformers: HashMap<String, RcTransformer<i32, String>> = HashMap::new();

    transformers.insert(
        "double".to_string(),
        RcTransformer::new(|x: i32| format!("Doubled: {}", x * 2)),
    );
    transformers.insert(
        "square".to_string(),
        RcTransformer::new(|x: i32| format!("Squared: {}", x * x)),
    );

    if let Some(transformer) = transformers.get("double") {
        println!("Transformer 'double': {}", transformer.transform(7));
    }
    if let Some(transformer) = transformers.get("square") {
        println!("Transformer 'square': {}", transformer.transform(7));
    }
    println!();

    // ====================================================================
    // Part 5: Trait Usage
    // ====================================================================
    println!("=== Trait Usage ===\n");

    fn apply_transformer<F: Transformer<i32, String>>(f: &F, x: i32) -> String {
        f.transform(x)
    }

    let to_string = BoxTransformer::new(|x: i32| format!("Value: {}", x));
    println!("Via trait: {}", apply_transformer(&to_string, 42));

    println!("\n=== Demo Complete ===");
}
