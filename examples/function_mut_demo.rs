/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_function::{ArcFunctionMut, BoxFunctionMut, FunctionMut, RcFunctionMut};

fn main() {
    println!("=== FunctionMut Demo - Mutable Transformation (borrows &mut T) ===\n");

    // ====================================================================
    // Part 1: BoxFunctionMut - Single ownership
    // ====================================================================
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

    // ====================================================================
    // Part 2: ArcFunctionMut - Thread-safe, cloneable
    // ====================================================================
    println!("--- ArcFunctionMut ---");
    let mut arc_increment = ArcFunctionMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });

    let mut arc_cloned_mut = arc_increment.clone();

    let mut val1 = 0;
    let mut val2 = 10;
    println!(
        "arc_increment.apply(&mut 0) = {}",
        arc_increment.apply(&mut val1)
    );
    println!(
        "arc_cloned_mut.apply(&mut 10) = {}",
        arc_cloned_mut.apply(&mut val2)
    );
    println!();

    // ====================================================================
    // Part 3: RcFunctionMut - Single-threaded, cloneable
    // ====================================================================
    println!("--- RcFunctionMut ---");
    let mut rc_increment = RcFunctionMut::new(|x: &mut i32| {
        *x += 1;
        *x
    });

    let mut rc_cloned_mut = rc_increment.clone();

    let mut val3 = 0;
    let mut val4 = 10;
    println!(
        "rc_increment.apply(&mut 0) = {}",
        rc_increment.apply(&mut val3)
    );
    println!(
        "rc_cloned_mut.apply(&mut 10) = {}",
        rc_cloned_mut.apply(&mut val4)
    );
    println!();

    // ====================================================================
    // Part 4: Practical Examples
    // ====================================================================
    println!("=== Practical Examples ===\n");

    // Example: String processing with mutation
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

    // ====================================================================
    // Part 5: Trait Usage
    // ====================================================================
    println!("=== Trait Usage ===\n");

    fn apply_function_mut<F: FunctionMut<i32, String>>(f: &mut F, x: &mut i32) -> String {
        f.apply(x)
    }

    let mut incrementer = BoxFunctionMut::new(|x: &mut i32| {
        *x += 1;
        format!("Incremented to: {}", x)
    });

    let mut val = 10;
    println!(
        "Via trait mut: {}",
        apply_function_mut(&mut incrementer, &mut val)
    );

    println!("\n=== Demo Complete ===");
}
