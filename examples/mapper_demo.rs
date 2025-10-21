/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Demonstrates the usage of Mapper types
//!
//! This example shows how to use BoxMapper, RcMapper, and ArcMapper
//! for stateful value transformation.

use prism3_function::{ArcMapper, BoxMapper, FnMapperOps, Mapper, RcMapper};

fn main() {
    println!("=== Mapper Demo ===\n");

    // 1. Basic BoxMapper with state
    println!("1. BoxMapper with stateful counter:");
    let mut counter = 0;
    let mut mapper = BoxMapper::new(move |x: i32| {
        counter += 1;
        format!("Item #{}: {}", counter, x)
    });

    println!("  {}", mapper.apply(100)); // Item #1: 100
    println!("  {}", mapper.apply(200)); // Item #2: 200
    println!("  {}", mapper.apply(300)); // Item #3: 300

    // 2. Composing mappers with and_then
    println!("\n2. Composing mappers with and_then:");
    let mut counter1 = 0;
    let mapper1 = BoxMapper::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let mapper2 = BoxMapper::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = mapper1.and_then(mapper2);
    println!("  First call:  {}", composed.apply(10)); // (10 + 1) * 1 = 11
    println!("  Second call: {}", composed.apply(10)); // (10 + 2) * 2 = 24
    println!("  Third call:  {}", composed.apply(10)); // (10 + 3) * 3 = 39

    // 3. Conditional mapping with when/or_else
    println!("\n3. Conditional mapping:");
    let mut high_count = 0;
    let mut low_count = 0;

    let mut conditional = BoxMapper::new(move |x: i32| {
        high_count += 1;
        format!("High[{}]: {} * 2 = {}", high_count, x, x * 2)
    })
    .when(|x: &i32| *x >= 10)
    .or_else(move |x| {
        low_count += 1;
        format!("Low[{}]: {} + 1 = {}", low_count, x, x + 1)
    });

    println!("  {}", conditional.apply(15)); // High[1]: 15 * 2 = 30
    println!("  {}", conditional.apply(5)); // Low[1]: 5 + 1 = 6
    println!("  {}", conditional.apply(20)); // High[2]: 20 * 2 = 40

    // 4. RcMapper for cloneable mappers
    println!("\n4. RcMapper (cloneable, single-threaded):");
    let mut counter = 0;
    let mapper = RcMapper::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut mapper1 = mapper.clone();
    let mut mapper2 = mapper.clone();

    println!("  mapper1: {}", mapper1.apply(10)); // 11
    println!("  mapper2: {}", mapper2.apply(10)); // 12
    println!("  mapper1: {}", mapper1.apply(10)); // 13

    // 5. ArcMapper for thread-safe mappers
    println!("\n5. ArcMapper (thread-safe):");
    let mut counter = 0;
    let mapper = ArcMapper::new(move |x: i32| {
        counter += 1;
        format!("Result[{}]: {}", counter, x * 2)
    });

    let mut mapper_clone = mapper.clone();
    println!("  Original: {}", mapper_clone.apply(5)); // Result[1]: 10
    println!("  Clone:    {}", mapper_clone.apply(7)); // Result[2]: 14

    // 6. Using FnMapperOps extension trait
    println!("\n6. Using FnMapperOps extension trait:");
    let mut count = 0;
    let mut mapper = (move |x: i32| {
        count += 1;
        x + count
    })
    .and_then(|x| x * 2);

    println!("  {}", mapper.apply(10)); // (10 + 1) * 2 = 22
    println!("  {}", mapper.apply(10)); // (10 + 2) * 2 = 24

    // 7. Building a complex pipeline
    println!("\n7. Complex processing pipeline:");
    let mut step1_count = 0;
    let step1 = BoxMapper::new(move |x: i32| {
        step1_count += 1;
        format!("Step1[{}]: {}", step1_count, x)
    });

    let mut step2_count = 0;
    let step2 = BoxMapper::new(move |s: String| {
        step2_count += 1;
        format!("{} -> Step2[{}]", s, step2_count)
    });

    let mut step3_count = 0;
    let step3 = BoxMapper::new(move |s: String| {
        step3_count += 1;
        format!("{} -> Step3[{}]", s, step3_count)
    });

    let mut pipeline = step1.and_then(step2).and_then(step3);

    println!("  {}", pipeline.apply(100));
    println!("  {}", pipeline.apply(200));

    println!("\n=== Demo Complete ===");
}
