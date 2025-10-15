/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcFunction, BoxFunction, Function, RcFunction};
use std::collections::HashMap;
use std::thread;

fn main() {
    println!("=== Function Demo - Immutable Transformation (borrows &T) ===\n");

    // ====================================================================
    // Part 1: BoxFunction - Single ownership, reusable
    // ====================================================================
    println!("--- BoxFunction ---");
    let double = BoxFunction::new(|x: &i32| x * 2);
    let value = 21;
    println!("double.apply(&{}) = {}", value, double.apply(&value));
    println!("value is still usable: {}", value);
    println!("double.apply(&{}) = {}", value, double.apply(&value));

    // Identity and constant
    let identity = BoxFunction::<i32, i32>::identity();
    println!("identity.apply(&42) = {}", identity.apply(&42));

    let constant = BoxFunction::constant("hello");
    println!("constant.apply(&123) = {}", constant.apply(&123));
    println!();

    // ====================================================================
    // Part 2: ArcFunction - Thread-safe, cloneable
    // ====================================================================
    println!("--- ArcFunction ---");
    let arc_double = ArcFunction::new(|x: &i32| x * 2);
    let arc_cloned = arc_double.clone();

    println!("arc_double.apply(&21) = {}", arc_double.apply(&21));
    println!("arc_cloned.apply(&42) = {}", arc_cloned.apply(&42));

    // Multi-threaded usage
    let for_thread = arc_double.clone();
    let handle = thread::spawn(move || for_thread.apply(&100));
    println!("In main thread: arc_double.apply(&50) = {}", arc_double.apply(&50));
    println!("In child thread: result = {}", handle.join().unwrap());
    println!();

    // ====================================================================
    // Part 3: RcFunction - Single-threaded, cloneable
    // ====================================================================
    println!("--- RcFunction ---");
    let rc_double = RcFunction::new(|x: &i32| x * 2);
    let rc_cloned = rc_double.clone();

    println!("rc_double.apply(&21) = {}", rc_double.apply(&21));
    println!("rc_cloned.apply(&42) = {}", rc_cloned.apply(&42));
    println!();

    // ====================================================================
    // Part 4: Practical Examples
    // ====================================================================
    println!("=== Practical Examples ===\n");

    // Example 1: Data validation and transformation
    println!("--- Data Validation Pipeline ---");
    let validate_positive = BoxFunction::new(|x: &i32| {
        if *x > 0 {
            Some(*x)
        } else {
            None
        }
    });

    let num1 = 42;
    let num2 = -5;
    println!("validate_positive(&{}) = {:?}", num1, validate_positive.apply(&num1));
    println!("validate_positive(&{}) = {:?}", num2, validate_positive.apply(&num2));
    println!();

    // Example 2: Shared configuration
    println!("--- Shared Configuration ---");
    let multiplier = 10;
    let multiply_by_config = ArcFunction::new(move |x: &i32| x * multiplier);

    // Can be shared across different parts of the program
    let config1 = multiply_by_config.clone();
    let config2 = multiply_by_config.clone();

    println!("config1.apply(&5) = {}", config1.apply(&5));
    println!("config2.apply(&7) = {}", config2.apply(&7));
    println!("multiply_by_config.apply(&3) = {}", multiply_by_config.apply(&3));
    println!();

    // Example 3: Event handler registry
    println!("--- Event Handler Registry ---");
    let mut handlers: HashMap<String, RcFunction<i32, String>> = HashMap::new();

    handlers.insert(
        "double".to_string(),
        RcFunction::new(|x: &i32| format!("Doubled: {}", x * 2))
    );
    handlers.insert(
        "square".to_string(),
        RcFunction::new(|x: &i32| format!("Squared: {}", x * x))
    );

    let value = 7;
    if let Some(handler) = handlers.get("double") {
        println!("Handler 'double': {}", handler.apply(&value));
    }
    if let Some(handler) = handlers.get("square") {
        println!("Handler 'square': {}", handler.apply(&value));
    }
    println!();

    // ====================================================================
    // Part 5: Trait Usage
    // ====================================================================
    println!("=== Trait Usage ===\n");

    fn apply_function<F: Function<i32, String>>(f: &F, x: &i32) -> String {
        f.apply(x)
    }

    let to_string = BoxFunction::new(|x: &i32| format!("Value: {}", x));
    println!("Via trait: {}", apply_function(&to_string, &42));

    println!("\n=== Demo Complete ===");
}
