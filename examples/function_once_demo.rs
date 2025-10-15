/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcFunctionOnce, BoxFunctionOnce, FunctionOnce, RcFunctionOnce};
use std::thread;

fn main() {
    println!("=== FunctionOnce Demo - Consuming Transformation (takes T) ===\n");

    // ====================================================================
    // Part 1: BoxFunctionOnce - One-time use
    // ====================================================================
    println!("--- BoxFunctionOnce ---");
    let parse = BoxFunctionOnce::new(|s: String| {
        s.parse::<i32>().unwrap_or(0)
    });
    println!("parse.apply(\"42\".to_string()) = {}", parse.apply("42".to_string()));
    // parse is consumed and cannot be used again

    // Composition
    let add_one = BoxFunctionOnce::new(|x: i32| x + 1);
    let pipeline = add_one
        .and_then(|x| x * 2)
        .and_then(|x| x.to_string());
    println!("pipeline(5) = {} (expected: \"12\")", pipeline.apply(5));
    println!();

    // ====================================================================
    // Part 2: ArcFunctionOnce - Reusable (cloneable)
    // ====================================================================
    println!("--- ArcFunctionOnce ---");
    let arc_parse = ArcFunctionOnce::new(|s: String| {
        s.parse::<i32>().unwrap_or(0)
    });

    let arc_parse_clone = arc_parse.clone();

    // Both can be used (but each consumes its String input)
    println!("arc_parse.apply(\"42\") = {}", arc_parse.apply("42".to_string()));
    println!("arc_parse_clone.apply(\"21\") = {}", arc_parse_clone.apply("21".to_string()));

    // Thread-safe
    let arc_parse2 = ArcFunctionOnce::new(|s: String| s.to_uppercase());
    let for_thread2 = arc_parse2.clone();
    let handle2 = thread::spawn(move || {
        for_thread2.apply("hello".to_string())
    });
    println!("In main: arc_parse2(\"world\") = {}", arc_parse2.apply("world".to_string()));
    println!("In thread: result = {}", handle2.join().unwrap());
    println!();

    // ====================================================================
    // Part 3: RcFunctionOnce - Single-threaded, reusable
    // ====================================================================
    println!("--- RcFunctionOnce ---");
    let rc_parse = RcFunctionOnce::new(|s: String| {
        s.parse::<i32>().unwrap_or(0)
    });

    let rc_parse_clone = rc_parse.clone();

    println!("rc_parse.apply(\"42\") = {}", rc_parse.apply("42".to_string()));
    println!("rc_parse_clone.apply(\"21\") = {}", rc_parse_clone.apply("21".to_string()));
    println!();

    // ====================================================================
    // Part 4: Practical Examples
    // ====================================================================
    println!("=== Practical Examples ===\n");

    // Example: String transformation pipeline
    println!("--- String Transformation Pipeline ---");
    let to_upper = ArcFunctionOnce::new(|s: String| s.to_uppercase());
    let add_prefix = ArcFunctionOnce::new(|s: String| format!(">>> {}", s));
    let pipeline = to_upper.and_then(&add_prefix);

    println!("pipeline(\"hello\") = {}", pipeline.apply("hello".to_string()));
    println!();

    // ====================================================================
    // Part 5: Trait Usage
    // ====================================================================
    println!("=== Trait Usage ===\n");

    fn apply_function_once<F: FunctionOnce<String, usize>>(
        f: F,
        x: String,
    ) -> usize {
        f.apply(x)
    }

    let length = BoxFunctionOnce::new(|s: String| s.len());
    println!("Via trait once: {}", apply_function_once(length, "hello".to_string()));

    println!("\n=== Demo Complete ===");
}

