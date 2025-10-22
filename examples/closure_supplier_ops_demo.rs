/**
 * Demonstrates various operations on closures using Supplier trait.
 *
 * This example showcases how to use the Supplier trait with closures,
 * including mapping, filtering, zipping, and memoization.
 */

use prism3_function::{FnSupplierOps, Supplier};

fn main() {
    println!("=== Closure Supplier Operations Demo ===\n");

    // 1. FnMut closure using map
    println!("1. FnMut closure using map:");
    let mut counter = 0;
    let mut mapped = (move || {
        counter += 1;
        counter
    })
    .map(|x| x * 2);

    println!("   First call: {}", mapped.get());
    println!("   Second call: {}\n", mapped.get());

    // 2. FnMut closure using filter
    println!("2. FnMut closure using filter:");
    let mut counter2 = 0;
    let mut filtered = (move || {
        counter2 += 1;
        counter2
    })
    .filter(|x| x % 2 == 0);

    println!("   First call (odd number): {:?}", filtered.get());
    println!("   Second call (even number): {:?}\n", filtered.get());

    // 3. FnMut closure using memoize
    println!("3. FnMut closure using memoize:");
    let mut call_count = 0;
    let mut memoized = (move || {
        call_count += 1;
        println!("   Underlying function called {} times", call_count);
        42
    })
    .memoize();

    println!("   First call: {}", memoized.get());
    println!("   Second call: {}", memoized.get());
    println!("   Third call: {}\n", memoized.get());

    // 4. Fn closure using map (Fn also implements FnMut, so can use FnSupplierOps)
    println!("4. Fn closure using map:");
    let mut mapped_readonly = (|| 10).map(|x| x * 3).map(|x| x + 5);
    println!("   Result: {}\n", mapped_readonly.get());

    // 5. Fn closure using filter (Fn also implements FnMut, so can use FnSupplierOps)
    println!("5. Fn closure using filter:");
    let mut filtered_readonly = (|| 42).filter(|x| x % 2 == 0);
    println!("   Filtered even number: {:?}\n", filtered_readonly.get());

    // 6. Chained operations
    println!("6. Chained operations:");
    let mut counter3 = 0;
    let mut chained = (move || {
        counter3 += 1;
        counter3
    })
    .map(|x| x * 2)
    .filter(|x| *x > 5)
    .map(|opt: Option<i32>| opt.unwrap_or(0));

    println!("   First call: {}", chained.get()); // 2, filtered out
    println!("   Second call: {}", chained.get()); // 4, filtered out
    println!("   Third call: {}", chained.get()); // 6, passed
    println!("   Fourth call: {}\n", chained.get()); // 8, passed

    println!("=== Demo completed ===");
}
