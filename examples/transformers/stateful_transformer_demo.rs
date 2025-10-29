/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Demonstrates the usage of StatefulTransformer types
//!
//! This example shows how to use BoxStatefulTransformer, RcStatefulTransformer, and ArcStatefulTransformer
//! for stateful value transformation.

use prism3_function::{
    ArcStatefulTransformer,
    BoxStatefulTransformer,
    FnStatefulTransformerOps,
    RcStatefulTransformer,
    StatefulTransformer,
    TransformerOnce,
};

fn main() {
    println!("=== StatefulTransformer Demo ===\n");

    // 1. Basic BoxStatefulTransformer with state
    println!("1. BoxStatefulTransformer with stateful counter:");
    let mut counter = 0;
    let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
        counter += 1;
        format!("Item #{}: {}", counter, x)
    });

    println!("  {}", mapper.apply(100)); // Item #1: 100
    println!("  {}", mapper.apply(200)); // Item #2: 200
    println!("  {}", mapper.apply(300)); // Item #3: 300

    // 2. Composing mappers with and_then
    println!("\n2. Composing mappers with and_then:");
    let mut counter1 = 0;
    let mapper1 = BoxStatefulTransformer::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let mapper2 = BoxStatefulTransformer::new(move |x: i32| {
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

    let mut conditional = BoxStatefulTransformer::new(move |x: i32| {
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

    // 4. RcStatefulTransformer for cloneable mappers
    println!("\n4. RcStatefulTransformer (cloneable, single-threaded):");
    let mut counter = 0;
    let mapper = RcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut mapper1 = mapper.clone();
    let mut mapper2 = mapper.clone();

    println!("  mapper1: {}", mapper1.apply(10)); // 11
    println!("  mapper2: {}", mapper2.apply(10)); // 12
    println!("  mapper1: {}", mapper1.apply(10)); // 13

    // 5. ArcStatefulTransformer for thread-safe mappers
    println!("\n5. ArcStatefulTransformer (thread-safe):");
    let mut counter = 0;
    let mapper = ArcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        format!("Result[{}]: {}", counter, x * 2)
    });

    let mut mapper_clone = mapper.clone();
    println!("  Original: {}", mapper_clone.apply(5)); // Result[1]: 10
    println!("  Clone:    {}", mapper_clone.apply(7)); // Result[2]: 14

    // 6. Using FnStatefulTransformerOps extension trait
    println!("\n6. Using FnStatefulTransformerOps extension trait:");
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
    let step1 = BoxStatefulTransformer::new(move |x: i32| {
        step1_count += 1;
        format!("Step1[{}]: {}", step1_count, x)
    });

    let mut step2_count = 0;
    let step2 = BoxStatefulTransformer::new(move |s: String| {
        step2_count += 1;
        format!("{} -> Step2[{}]", s, step2_count)
    });

    let mut step3_count = 0;
    let step3 = BoxStatefulTransformer::new(move |s: String| {
        step3_count += 1;
        format!("{} -> Step3[{}]", s, step3_count)
    });

    let mut pipeline = step1.and_then(step2).and_then(step3);

    println!("  {}", pipeline.apply(100));
    println!("  {}", pipeline.apply(200));

    // 7. TransformerOnce implementation - consuming transformers
    println!("\n7. TransformerOnce implementation - consuming StatefulTransformers:");

    // BoxStatefulTransformer can be consumed as TransformerOnce
    let mut counter = 0;
    let box_mapper = BoxStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x * counter
    });
    println!(
        "  BoxStatefulTransformer consumed once: {}",
        box_mapper.apply_once(10)
    ); // 10 * 1 = 10

    // RcStatefulTransformer can be consumed as TransformerOnce
    let mut counter = 0;
    let rc_mapper = RcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });
    let rc_clone = rc_mapper.clone(); // Clone before consuming
    println!(
        "  RcStatefulTransformer consumed once: {}",
        rc_mapper.apply_once(10)
    ); // 10 + 1 = 11
    println!(
        "  RcStatefulTransformer clone still works: {}",
        rc_clone.clone().apply(10)
    ); // 10 + 2 = 12

    // ArcStatefulTransformer can be consumed as TransformerOnce
    let mut counter = 0;
    let arc_mapper = ArcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x * counter
    });
    let arc_clone = arc_mapper.clone(); // Clone before consuming
    println!(
        "  ArcStatefulTransformer consumed once: {}",
        arc_mapper.apply_once(10)
    ); // 10 * 1 = 10
    println!(
        "  ArcStatefulTransformer clone still works: {}",
        arc_clone.clone().apply(10)
    ); // 10 * 2 = 20

    // 8. Converting to BoxTransformerOnce
    println!("\n8. Converting StatefulTransformers to BoxTransformerOnce:");

    let mut counter = 0;
    let mapper = BoxStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x * counter
    });
    let once_mapper = mapper.into_box_once();
    println!(
        "  BoxStatefulTransformer->BoxTransformerOnce: {}",
        once_mapper.apply_once(5)
    ); // 5 * 1 = 5

    // RcStatefulTransformer can use to_box_once() to preserve original
    let mut counter = 0;
    let rc_mapper = RcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x * counter
    });
    let once_mapper = rc_mapper.to_box_once();
    println!(
        "  RcStatefulTransformer->BoxTransformerOnce: {}",
        once_mapper.apply_once(5)
    ); // 5 * 1 = 5
    println!(
        "  Original RcStatefulTransformer still works: {}",
        rc_mapper.clone().apply(5)
    ); // 5 * 2 = 10

    println!("\n=== Demo Complete ===");
}
