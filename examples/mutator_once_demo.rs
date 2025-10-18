/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MutatorOnce Demo
//!
//! Demonstrates various usage scenarios of MutatorOnce

use prism3_function::{BoxMutatorOnce, FnMutatorOnceOps, MutatorOnce};

fn main() {
    println!("=== MutatorOnce Examples ===\n");

    // 1. Basic usage: moving captured variables
    println!("1. Basic usage: moving captured variables");
    let data = vec![1, 2, 3];
    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   Adding data: {:?}", data);
        x.extend(data);
    });

    let mut target = vec![0];
    mutator.mutate(&mut target);
    println!("   Result: {:?}\n", target);

    // 2. Method chaining: combining multiple operations
    println!("2. Method chaining: combining multiple operations");
    let prefix = vec![1, 2];
    let middle = vec![3, 4];
    let suffix = vec![5, 6];

    let chained = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   Adding prefix: {:?}", prefix);
        x.extend(prefix);
    })
    .and_then(move |x: &mut Vec<i32>| {
        println!("   Adding middle: {:?}", middle);
        x.extend(middle);
    })
    .and_then(move |x: &mut Vec<i32>| {
        println!("   Adding suffix: {:?}", suffix);
        x.extend(suffix);
    });

    let mut result = vec![0];
    chained.mutate(&mut result);
    println!("   Result: {:?}\n", result);

    // 3. Initializer pattern
    println!("3. Initializer pattern");

    struct Initializer {
        name: String,
        on_complete: Option<BoxMutatorOnce<Vec<String>>>,
    }

    impl Initializer {
        fn new<F>(name: impl Into<String>, callback: F) -> Self
        where
            F: FnOnce(&mut Vec<String>) + 'static,
        {
            Self {
                name: name.into(),
                on_complete: Some(BoxMutatorOnce::new(callback)),
            }
        }

        fn run(mut self, data: &mut Vec<String>) {
            println!("   Initializer '{}' is running", self.name);
            data.push(format!("Initialized by {}", self.name));

            if let Some(callback) = self.on_complete.take() {
                println!("   Executing completion callback");
                callback.mutate(data);
            }
        }
    }

    let extra = vec!["extra1".to_string(), "extra2".to_string()];
    let init = Initializer::new("MainInit", move |values| {
        println!("   Adding extra data in callback: {:?}", extra);
        values.extend(extra);
    });

    let mut config = Vec::new();
    init.run(&mut config);
    println!("   Final config: {:?}\n", config);

    // 4. String builder pattern
    println!("4. String builder pattern");
    let greeting = String::from("Hello, ");
    let name = String::from("Alice");
    let punctuation = String::from("!");

    let builder = BoxMutatorOnce::new(move |s: &mut String| {
        println!("   Adding greeting: {}", greeting);
        s.insert_str(0, &greeting);
    })
    .and_then(move |s: &mut String| {
        println!("   Adding name: {}", name);
        s.push_str(&name);
    })
    .and_then(move |s: &mut String| {
        println!("   Adding punctuation: {}", punctuation);
        s.push_str(&punctuation);
    })
    .and_then(|s: &mut String| {
        println!("   Converting to uppercase");
        *s = s.to_uppercase();
    });

    let mut message = String::new();
    builder.mutate(&mut message);
    println!("   Final message: {}\n", message);

    // 5. Direct closure usage
    println!("5. Direct closure usage");
    let data1 = vec![10, 20];
    let data2 = vec![30, 40];

    let chained_closure = (move |x: &mut Vec<i32>| {
        println!("   Step 1: Adding {:?}", data1);
        x.extend(data1);
    })
    .and_then(move |x: &mut Vec<i32>| {
        println!("   Step 2: Adding {:?}", data2);
        x.extend(data2);
    })
    .and_then(|x: &mut Vec<i32>| {
        println!("   Step 3: Multiplying each element by 2");
        x.iter_mut().for_each(|n| *n *= 2);
    });

    let mut values = vec![0];
    chained_closure.mutate(&mut values);
    println!("   Result: {:?}\n", values);

    // 6. Resource transfer scenario
    println!("6. Resource transfer scenario");
    let large_data = vec![1; 10];
    println!(
        "   Preparing to transfer large data (length: {})",
        large_data.len()
    );

    let mutator = BoxMutatorOnce::new(move |x: &mut Vec<i32>| {
        println!("   Transferring data (moving, not cloning)");
        x.extend(large_data); // large_data is moved, not cloned
    });

    let mut container = Vec::new();
    mutator.mutate(&mut container);
    println!("   Data length in container: {}\n", container.len());

    // 7. Generic function usage
    println!("7. Generic function usage");

    fn apply_transformation<M: MutatorOnce<Vec<i32>>>(mutator: M, initial: Vec<i32>) -> Vec<i32> {
        let mut val = initial;
        mutator.mutate(&mut val);
        val
    }

    let data = vec![100, 200, 300];
    let result = apply_transformation(
        move |x: &mut Vec<i32>| {
            println!("   Adding in generic function: {:?}", data);
            x.extend(data);
        },
        vec![0],
    );
    println!("   Result: {:?}\n", result);

    // 8. Configuration builder
    println!("8. Configuration builder");

    struct Config {
        options: Vec<String>,
    }

    impl Config {
        fn new() -> Self {
            Self {
                options: Vec::new(),
            }
        }

        fn with_defaults(mut self) -> Self {
            println!("   Adding default options");
            self.options.push("default1".to_string());
            self.options.push("default2".to_string());
            self
        }

        fn customize<F>(mut self, customizer: F) -> Self
        where
            F: FnOnce(&mut Vec<String>) + 'static,
        {
            println!("   Applying custom configuration");
            customizer.mutate(&mut self.options);
            self
        }

        fn build(self) -> Self {
            println!("   Configuration build completed");
            self
        }
    }

    let custom_opts = vec!["custom1".to_string(), "custom2".to_string()];
    let config = Config::new()
        .with_defaults()
        .customize(move |opts| {
            println!("   Adding custom options: {:?}", custom_opts);
            opts.extend(custom_opts);
        })
        .build();

    println!("   Final options: {:?}\n", config.options);

    println!("=== Examples completed ===");
}
