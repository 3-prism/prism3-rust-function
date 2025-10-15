/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{
    ArcFunction, ArcFunctionMut, ArcFunctionOnce, BoxFunction, BoxFunctionMut,
    BoxFunctionOnce, Function, FunctionMut, FunctionOnce, RcFunction,
    RcFunctionMut, RcFunctionOnce,
};
use std::thread;

fn main() {
    println!("=== Function Demo - Three Traits with Multiple Implementations ===\n");

    // ====================================================================
    // Part 1: Function - Immutable Transformation (borrows &T)
    // ====================================================================
    println!("=== 1. Function - Immutable Transformation ===\n");

    // BoxFunction - Single ownership, reusable
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

    // ArcFunction - Thread-safe, cloneable
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

    // RcFunction - Single-threaded, cloneable
    println!("--- RcFunction ---");
    let rc_double = RcFunction::new(|x: &i32| x * 2);
    let rc_cloned = rc_double.clone();

    println!("rc_double.apply(&21) = {}", rc_double.apply(&21));
    println!("rc_cloned.apply(&42) = {}", rc_cloned.apply(&42));
    println!();

    // ====================================================================
    // Part 2: FunctionMut - Mutable Transformation (borrows &mut T)
    // ====================================================================
    println!("=== 2. FunctionMut - Mutable Transformation ===\n");

    // BoxFunctionMut - Single ownership
    println!("--- BoxFunctionMut ---");
    let mut double_mut = BoxFunctionMut::new(|x: &mut i32| {
        *x *= 2;
        *x
    });

    let mut val = 21;
    let val_before = val;
    let result = double_mut.apply(&mut val);
    println!("double_mut.apply(&mut {}) = {}", val_before, result);
    println!("val after mutation: {}", val);

    // Stateful function
    let mut count = 0;
    let mut counter = BoxFunctionMut::new(move |x: &mut i32| {
        count += 1;
        *x += count;
        (*x, count)
    });

    let mut value = 10;
    println!("counter with state:");
    println!("  1st call: {:?}", counter.apply(&mut value));
    println!("  2nd call: {:?}", counter.apply(&mut value));
    println!("  3rd call: {:?}", counter.apply(&mut value));
    println!();

    // ArcFunctionMut - Thread-safe, cloneable
    println!("--- ArcFunctionMut ---");
    let mut arc_increment = ArcFunctionMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });

    let mut arc_cloned_mut = arc_increment.clone();

    let mut val1 = 0;
    let mut val2 = 10;
    println!("arc_increment.apply(&mut 0) = {}", arc_increment.apply(&mut val1));
    println!("arc_cloned_mut.apply(&mut 10) = {}", arc_cloned_mut.apply(&mut val2));
    println!();

    // RcFunctionMut - Single-threaded, cloneable
    println!("--- RcFunctionMut ---");
    let mut rc_increment = RcFunctionMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });

    let mut rc_cloned_mut = rc_increment.clone();

    let mut val3 = 0;
    let mut val4 = 10;
    println!("rc_increment.apply(&mut 0) = {}", rc_increment.apply(&mut val3));
    println!("rc_cloned_mut.apply(&mut 10) = {}", rc_cloned_mut.apply(&mut val4));
    println!();

    // ====================================================================
    // Part 3: FunctionOnce - Consuming Transformation (takes T)
    // ====================================================================
    println!("=== 3. FunctionOnce - Consuming Transformation ===\n");

    // BoxFunctionOnce - One-time use
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

    // ArcFunctionOnce - Reusable (cloneable)
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

    // RcFunctionOnce - Single-threaded, reusable
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
    println!("=== 4. Practical Examples ===\n");

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

    // Example 2: String processing with mutation
    println!("--- String Processing ---");
    let mut normalizer = BoxFunctionMut::new(|s: &mut String| {
        *s = s.trim().to_lowercase();
        s.len()
    });

    let mut text1 = "  HELLO World  ".to_string();
    println!("Before: {:?}", text1);
    let len = normalizer.apply(&mut text1);
    println!("After: {:?} (length: {})", text1, len);
    println!();

    // Example 3: Shared configuration
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

    // Example 4: Event handler registry
    println!("--- Event Handler Registry ---");
    use std::collections::HashMap;

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
    println!("=== 5. Trait Usage ===\n");

    fn apply_function<F: Function<i32, String>>(f: &F, x: &i32) -> String {
        f.apply(x)
    }

    let to_string = BoxFunction::new(|x: &i32| format!("Value: {}", x));
    println!("Via trait: {}", apply_function(&to_string, &42));

    fn apply_function_mut<F: FunctionMut<i32, String>>(
        f: &mut F,
        x: &mut i32,
    ) -> String {
        f.apply(x)
    }

    let mut incrementer = BoxFunctionMut::new(|x: &mut i32| {
        *x += 1;
        format!("Incremented to: {}", x)
    });

    let mut val = 10;
    println!("Via trait mut: {}", apply_function_mut(&mut incrementer, &mut val));

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
