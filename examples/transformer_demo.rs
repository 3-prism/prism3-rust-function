/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Transformer usage examples
//!
//! This example demonstrates the four types of transformers:
//! - BoxTransformer: One-time use, single ownership
//! - BoxFnTransformer: Reusable, single ownership
//! - ArcFnTransformer: Reusable, multi-threaded sharing
//! - RcFnTransformer: Reusable, single-threaded sharing

use prism3_function::{ArcFnTransformer, BoxFnTransformer, BoxTransformer, RcFnTransformer};

fn main() {
    println!("=== Transformer Examples ===\n");

    // ========================================================================
    // 1. BoxTransformer - One-time use
    // ========================================================================
    println!("1. BoxTransformer (one-time use):");

    let double = BoxTransformer::new(|x: i32| x * 2);
    println!("   double(21) = {}", double.transform(21));
    // double cannot be used again

    // Identity transformer
    let identity = BoxTransformer::<i32>::identity();
    println!("   identity(42) = {}", identity.transform(42));

    // Constant transformer
    let always_hundred = BoxTransformer::constant(100);
    println!("   constant(42) = {}", always_hundred.transform(42));

    // Composition
    let add_one = BoxTransformer::new(|x: i32| x + 1);
    let pipeline = add_one.then(|x| x * 2).then(|x| x - 3);
    println!("   ((5 + 1) * 2) - 3 = {}", pipeline.transform(5));

    // Conditional transformation
    let conditional = BoxTransformer::when(|x: &i32| *x > 0, |x| x * 2);
    println!("   when(5 > 0, x * 2) = {}", conditional.transform(5));

    let conditional2 = BoxTransformer::when(|x: &i32| *x > 0, |x| x * 2);
    println!("   when(-5 > 0, x * 2) = {}", conditional2.transform(-5));

    // Repeat transformation
    let add_one_fn = |x: i32| x + 1;
    let add_five = BoxTransformer::repeat(add_one_fn, 5);
    println!("   repeat(+1, 5 times)(10) = {}", add_five.transform(10));

    println!();

    // ========================================================================
    // 2. BoxFnTransformer - Reusable, single ownership
    // ========================================================================
    println!("2. BoxFnTransformer (reusable, single ownership):");

    let triple = BoxFnTransformer::new(|x: i32| x * 3);
    println!("   triple(10) = {}", triple.transform(10));
    println!("   triple(20) = {}", triple.transform(20));
    println!("   triple(30) = {}", triple.transform(30));

    // String transformation
    let uppercase = BoxFnTransformer::new(|s: String| s.to_uppercase());
    println!(
        "   uppercase('hello') = {}",
        uppercase.transform("hello".to_string())
    );
    println!(
        "   uppercase('world') = {}",
        uppercase.transform("world".to_string())
    );

    println!();

    // ========================================================================
    // 3. ArcFnTransformer - Multi-threaded sharing
    // ========================================================================
    println!("3. ArcFnTransformer (multi-threaded sharing):");

    let square = ArcFnTransformer::new(|x: i32| x * x);

    // Can be cloned and used in multiple threads
    let square_clone1 = square.clone();
    let square_clone2 = square.clone();

    println!("   square(5) = {}", square.transform(5));
    println!("   square(10) = {}", square_clone1.transform(10));
    println!("   square(15) = {}", square_clone2.transform(15));

    // Composition preserves original transformers
    let add_two = ArcFnTransformer::new(|x: i32| x + 2);
    let composed = square.then(&add_two);

    println!("   (5^2) + 2 = {}", composed.transform(5));
    println!(
        "   Original square still works: square(6) = {}",
        square.transform(6)
    );
    println!(
        "   Original add_two still works: add_two(10) = {}",
        add_two.transform(10)
    );

    // Multi-threaded usage
    use std::thread;
    let multiplier = ArcFnTransformer::new(|x: i32| x * 4);

    let mut handles = vec![];
    for i in 1..=3 {
        let t = multiplier.clone();
        let handle = thread::spawn(move || {
            let result = t.transform(i);
            println!(
                "   Thread {:?}: transform({}) = {}",
                thread::current().id(),
                i,
                result
            );
            result
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    println!();

    // ========================================================================
    // 4. RcFnTransformer - Single-threaded sharing
    // ========================================================================
    println!("4. RcFnTransformer (single-threaded sharing):");

    let negate = RcFnTransformer::new(|x: i32| -x);
    let negate_clone = negate.clone();

    println!("   negate(42) = {}", negate.transform(42));
    println!("   negate(-10) = {}", negate_clone.transform(-10));

    // Composition
    let abs = RcFnTransformer::new(|x: i32| x.abs());
    let abs_then_negate = abs.then(&negate);

    println!("   abs(-5) then negate = {}", abs_then_negate.transform(-5));
    println!(
        "   Original abs still works: abs(-5) = {}",
        abs.transform(-5)
    );

    // Shared state example
    use std::cell::RefCell;
    use std::rc::Rc;

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);

    let counting_transformer = RcFnTransformer::new(move |x: i32| {
        *counter_clone.borrow_mut() += 1;
        x * 2
    });

    let t1 = counting_transformer.clone();
    let t2 = counting_transformer.clone();

    println!("   First call: {}", t1.transform(10));
    println!("   Second call: {}", t2.transform(20));
    println!("   Third call: {}", counting_transformer.transform(30));
    println!("   Total calls: {}", *counter.borrow());

    println!();

    // ========================================================================
    // 5. Option and Result transformations
    // ========================================================================
    println!("5. Option and Result transformations:");

    let double_opt = BoxTransformer::map_option(|x: i32| x * 2);
    println!(
        "   map_option(Some(21)) = {:?}",
        double_opt.transform(Some(21))
    );

    let double_opt2 = BoxTransformer::map_option(|x: i32| x * 2);
    println!("   map_option(None) = {:?}", double_opt2.transform(None));

    let double_result = BoxTransformer::map_result(|x: i32| x * 2);
    println!(
        "   map_result(Ok(21)) = {:?}",
        double_result.transform(Ok::<i32, String>(21))
    );

    let double_result2 = BoxTransformer::map_result(|x: i32| x * 2);
    println!(
        "   map_result(Err('error')) = {:?}",
        double_result2.transform(Err::<i32, String>("error".to_string()))
    );

    println!();

    // ========================================================================
    // 6. String processing pipeline
    // ========================================================================
    println!("6. String processing pipeline:");

    let trim = BoxTransformer::new(|s: String| s.trim().to_string());
    let uppercase = |s: String| s.to_uppercase();
    let add_exclamation = |s: String| format!("{}!", s);

    let pipeline = trim.then(uppercase).then(add_exclamation);

    let result = pipeline.transform("  hello world  ".to_string());
    println!("   '  hello world  ' -> '{}'", result);

    println!();

    // ========================================================================
    // 7. Complex conditional transformation
    // ========================================================================
    println!("7. Complex conditional transformation:");

    let process = BoxTransformer::if_else(
        |x: &i32| *x > 0,
        |x| x * 2,        // if positive, double
        |x| x.abs() + 10, // if negative or zero, abs + 10
    );

    println!("   process(5) = {}", process.transform(5));

    let process2 = BoxTransformer::if_else(|x: &i32| *x > 0, |x| x * 2, |x| x.abs() + 10);
    println!("   process(-5) = {}", process2.transform(-5));

    let process3 = BoxTransformer::if_else(|x: &i32| *x > 0, |x| x * 2, |x| x.abs() + 10);
    println!("   process(0) = {}", process3.transform(0));

    println!("\n=== End of Examples ===");
}
