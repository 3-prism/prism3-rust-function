/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # TransformerOnce Demo
//!
//! Demonstrates TransformerOnce implementation for BoxTransformer, RcTransformer, and ArcTransformer
//!
//! # Author
//!
//! Haixing Hu

use prism3_function::{
    ArcTransformer,
    BoxTransformer,
    RcTransformer,
    TransformerOnce,
};
use std::sync::Arc;
use std::thread;

fn main() {
    println!("=== TransformerOnce Demo ===\n");

    // BoxTransformer TransformerOnce demonstration
    println!("1. BoxTransformer TransformerOnce demonstration:");
    let double = BoxTransformer::new(|x: i32| x * 2);
    let result = double.apply_once(21);
    println!("   double.apply_once(21) = {}", result);

    // Convert to BoxTransformerOnce
    let double = BoxTransformer::new(|x: i32| x * 2);
    let boxed = double.into_box_once();
    let result = boxed.apply_once(21);
    println!("   double.into_box_once().apply_once(21) = {}", result);

    // Convert to function
    let double = BoxTransformer::new(|x: i32| x * 2);
    let func = double.into_fn_once();
    let result = func(21);
    println!("   double.into_fn_once()(21) = {}", result);

    println!();

    // RcTransformer TransformerOnce demonstration
    println!("2. RcTransformer TransformerOnce demonstration:");
    let uppercase = RcTransformer::new(|s: String| s.to_uppercase());
    let result = uppercase.apply_once("hello".to_string());
    println!("   uppercase.apply_once(\"hello\") = {}", result);

    // Use after cloning
    let uppercase = RcTransformer::new(|s: String| s.to_uppercase());
    let uppercase_clone = uppercase.clone();
    let result1 = uppercase.apply_once("world".to_string());
    let result2 = uppercase_clone.apply_once("rust".to_string());
    println!("   uppercase.apply_once(\"world\") = {}", result1);
    println!("   uppercase_clone.apply_once(\"rust\") = {}", result2);

    println!();

    // ArcTransformer TransformerOnce demonstration
    println!("3. ArcTransformer TransformerOnce demonstration:");
    let parse_and_double = ArcTransformer::new(|s: String| s.parse::<i32>().unwrap_or(0) * 2);
    let result = parse_and_double.apply_once("21".to_string());
    println!("   parse_and_double.apply_once(\"21\") = {}", result);

    // Thread safety demonstration
    println!("4. ArcTransformer thread safety demonstration:");
    let double = ArcTransformer::new(|x: i32| x * 2);
    let double_arc = Arc::new(double);
    let _double_clone = Arc::clone(&double_arc);

    let handle = thread::spawn(move || {
        // Create a new transformer in the thread to demonstrate thread safety
        let new_double = ArcTransformer::new(|x: i32| x * 2);
        new_double.apply_once(21)
    });

    let result = handle.join().unwrap();
    println!(
        "   Executed in thread: new_double.apply_once(21) = {}",
        result
    );

    println!("\n=== Demo completed ===");
}
