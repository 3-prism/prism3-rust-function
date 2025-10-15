# Predicate Design Comparison Analysis

## Overview

This document provides a detailed analysis of three different design approaches for implementing the Predicate type in Rust, comparing their pros and cons, applicable scenarios, and implementation details.

The core functionality of a Predicate is to test whether a value satisfies a specific condition, similar to the `Predicate<T>` interface in Java. In Rust, we need to make trade-offs in several aspects:

- **Type Expression**: Type alias vs Struct vs Trait
- **Ownership Model**: Box (single ownership) vs Arc (shared ownership) vs Rc (single-threaded shared)
- **Invocation Style**: Direct call vs Method call
- **Composition Ability**: Static methods vs Instance methods vs Trait methods
- **Extensibility**: Whether metadata can be added, whether other traits can be implemented

---

## Approach 1: Type Alias + Static Composition Methods

### Design Overview

Define the Predicate type using type aliases and provide composition methods through static utility classes. This is the simplest and most straightforward implementation approach.

### Core Design

```rust
// Type alias definitions
pub type Predicate<T> = Box<dyn Fn(&T) -> bool>;
pub type SharedPredicate<T> = Arc<dyn Fn(&T) -> bool + Send + Sync>;

// Static composition utility class
pub struct Predicates;

impl Predicates {
    /// Create AND combination
    pub fn and<T, F1, F2>(first: F1, second: F2) -> Predicate<T>
    where
        T: 'static,
        F1: Fn(&T) -> bool + 'static,
        F2: Fn(&T) -> bool + 'static,
    {
        Box::new(move |t| first(t) && second(t))
    }

    /// Create OR combination
    pub fn or<T, F1, F2>(first: F1, second: F2) -> Predicate<T>
    where
        T: 'static,
        F1: Fn(&T) -> bool + 'static,
        F2: Fn(&T) -> bool + 'static,
    {
        Box::new(move |t| first(t) || second(t))
    }

    /// Create NOT combination
    pub fn not<T, F>(predicate: F) -> Predicate<T>
    where
        T: 'static,
        F: Fn(&T) -> bool + 'static,
    {
        Box::new(move |t| !predicate(t))
    }
}

// SharedPredicate composition utility class
pub struct SharedPredicates;

impl SharedPredicates {
    pub fn and<T>(first: SharedPredicate<T>, second: SharedPredicate<T>) -> SharedPredicate<T>
    where
        T: 'static,
    {
        Arc::new(move |t| first(t) && second(t))
    }

    pub fn or<T>(first: SharedPredicate<T>, second: SharedPredicate<T>) -> SharedPredicate<T>
    where
        T: 'static,
    {
        Arc::new(move |t| first(t) || second(t))
    }

    pub fn not<T>(predicate: SharedPredicate<T>) -> SharedPredicate<T>
    where
        T: 'static,
    {
        Arc::new(move |t| !predicate(t))
    }
}
```

### Usage Examples

```rust
// Create predicates
let is_positive: Predicate<i32> = Box::new(|x| *x > 0);
let is_even: Predicate<i32> = Box::new(|x| x % 2 == 0);

// Direct invocation
assert!(is_positive(&5));
assert!(is_even(&4));

// Combine predicates (passing closures)
let is_positive_even = Predicates::and(
    |x: &i32| *x > 0,
    |x: &i32| x % 2 == 0,
);

// Direct invocation of combined predicate
assert!(is_positive_even(&4));
assert!(!is_positive_even(&3));

// Complex combination
let complex = Predicates::or(
    Predicates::and(|x: &i32| *x > 0, |x: &i32| x % 2 == 0),
    |x: &i32| *x > 100,
);
assert!(complex(&4));   // positive and even
assert!(complex(&150)); // large

// Using SharedPredicate (cloneable, thread-safe)
let shared_pred: SharedPredicate<i32> = Arc::new(|x| *x > 0);
let cloned = Arc::clone(&shared_pred);

// Use in multiple places
assert!(shared_pred(&5));
assert!(cloned(&10));
```

### Using as Function Parameters

```rust
// Define function accepting predicate parameters
fn filter_values<T, F>(values: Vec<T>, predicate: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    values.into_iter().filter(|v| predicate(v)).collect()
}

// Usage examples
let values = vec![1, -2, 3, -4, 5];

// 1. Pass closure
let result = filter_values(values.clone(), |x: &i32| *x > 0);
assert_eq!(result, vec![1, 3, 5]);

// 2. Pass function pointer
fn is_positive(x: &i32) -> bool { *x > 0 }
let result = filter_values(values.clone(), is_positive);
assert_eq!(result, vec![1, 3, 5]);

// 3. Pass Predicate object (note: transfers ownership)
let pred: Predicate<i32> = Box::new(|x| *x > 0);
let result = filter_values(values.clone(), pred);  // pred is moved
assert_eq!(result, vec![1, 3, 5]);
// pred is no longer available here

// 4. Pass combined predicate
let combined = Predicates::and(|x: &i32| *x > 0, |x: &i32| x % 2 == 0);
let result = filter_values(values, combined);
assert_eq!(result, vec![]);

// Note: Since Predicate<T> is just Box<dyn Fn(&T) -> bool>,
// it automatically implements the Fn trait, so it can be passed directly
```

### Advantages

#### 1. **Minimal API and User Experience**
- ✅ **Direct invocation**: `pred(&value)` instead of `pred.test(&value)`
- ✅ **Zero mental overhead**: Type alias is completely transparent, users can directly use `Box<dyn Fn>`
- ✅ **Perfect integration with standard library**: Can be used directly in `filter`, `find`, and other methods

```rust
// Very natural usage in standard library
let result: Vec<i32> = vec![1, -2, 3, 4]
    .into_iter()
    .filter(|x| pred(x))  // ✅ Used directly as closure
    .collect();
```

#### 2. **Perfect Generic Support**
- ✅ **Unified Fn trait**: Closures, function pointers, and Predicates are all unified through `Fn(&T) -> bool`
- ✅ **No conversion needed**: All callable types can be passed directly to composition methods
- ✅ **Type inference friendly**: Compiler can automatically infer closure types

```rust
// Supports all callable types
let pred1 = Predicates::and(|x| *x > 0, |x| x % 2 == 0);           // closures
let pred2 = Predicates::and(is_positive_fn, is_even_fn);           // function pointers
let pred3 = Predicates::and(pred1, |x| *x < 100);                  // Predicate + closure
```

#### 3. **Zero-Cost Abstraction**
- ✅ **Single boxing**: Each closure is boxed only once
- ✅ **Inline optimization**: Compiler can optimize closure invocations
- ✅ **No extra indirection**: Direct invocation through `Box::call()`

#### 4. **Simple Implementation**
- ✅ **Less code**: No need to define complex structs or traits
- ✅ **Low maintenance cost**: Type aliases are easy to understand and maintain
- ✅ **Concise documentation**: Users only need to understand function signatures

#### 5. **Native Trait Object Support**
```rust
// Can be stored directly in containers
let predicates: Vec<Predicate<i32>> = vec![
    Box::new(|x| *x > 0),
    Box::new(|x| x % 2 == 0),
];

// Can be passed as trait object
fn use_predicate(pred: &dyn Fn(&i32) -> bool) {
    assert!(pred(&5));
}
```

### Disadvantages

#### 1. **Cannot Extend**
- ❌ **Cannot add fields**: Cannot add metadata like name, statistics, etc. to Predicate
- ❌ **Cannot implement traits**: Type alias cannot implement `Display`, `Debug`, etc.
- ❌ **Cannot add methods**: Cannot add instance methods to Predicate

```rust
// ❌ Cannot implement
impl<T> Display for Predicate<T> {  // Compile error: type alias cannot have impl
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Predicate")
    }
}
```

#### 2. **Low Type Distinction**
- ❌ **Cannot distinguish at type system level**: `Predicate<T>` and `Box<dyn Fn(&T) -> bool>` are completely equivalent
- ❌ **Easy to confuse**: Users might directly use `Box::new()` instead of going through `Predicates`
- ❌ **Semantics not clear enough**: Type name cannot convey more information

#### 3. **Two Parallel APIs**
- ⚠️ **Predicate vs SharedPredicate**: Need to maintain two sets of types and composition methods
- ⚠️ **Naming not clear enough**: "Shared" doesn't clearly indicate whether it's Arc or Rc
- ⚠️ **Lack of Rc support**: No Rc implementation provided for single-threaded scenarios

```rust
// Two parallel APIs
struct Predicates;           // Provides composition methods for Predicate
struct SharedPredicates;     // Provides composition methods for SharedPredicate

// Users need to remember which one to use
let pred1 = Predicates::and(...);           // Box version
let pred2 = SharedPredicates::and(...);     // Arc version
```

#### 4. **Cannot Implement Method Chaining**
- ❌ **Only nested calls**: Complex combinations require deep nesting
- ❌ **Poor readability**: Multiple nesting levels are less clear than method chaining

```rust
// Complex combination requires nesting
let complex = Predicates::or(
    Predicates::and(
        Predicates::not(is_negative),
        is_even
    ),
    is_large
);

// Cannot use method chaining (ideal form):
// let complex = is_negative.not().and(is_even).or(is_large);
```

### Applicable Scenarios

✅ **Best suited for:**

1. **Simple predicate composition**: Don't need complex metadata or method chaining
2. **Pursuing minimal API**: Want code to be as concise as possible
3. **Deep integration with standard library**: Need to use directly in `filter`, `find`, etc.
4. **One-time use**: Predicates don't need to be reused multiple times after creation
5. **Rapid prototyping**: Quickly implement features without considering long-term extension

❌ **Not suited for:**

1. Need to add metadata like name, debug information, etc. to predicates
2. Need to implement `Display`, `Debug`, and other traits
3. Need complex method chaining
4. Need to enforce distinction between different predicate types at the type system level

---

## Approach 2: Struct Encapsulation + Instance Methods

### Design Overview

Define Predicate as a struct that wraps `Box<dyn Fn>` internally, providing composition capabilities through instance methods and supporting method chaining.

### Core Design

```rust
// Struct definition
pub struct Predicate<T> {
    inner: Box<dyn Fn(&T) -> bool>,
    name: Option<String>,  // Can add metadata
}

impl<T> Predicate<T> {
    /// Create new Predicate
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Self {
            inner: Box::new(f),
            name: None,
        }
    }

    /// Add name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Test value
    pub fn test(&self, value: &T) -> bool {
        (self.inner)(value)
    }

    /// AND combination (consumes self)
    pub fn and<F>(self, other: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let inner = self.inner;
        Self {
            inner: Box::new(move |t| inner(t) && other(t)),
            name: None,
        }
    }

    /// OR combination (consumes self)
    pub fn or<F>(self, other: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
        T: 'static,
    {
        let inner = self.inner;
        Self {
            inner: Box::new(move |t| inner(t) || other(t)),
            name: None,
        }
    }

    /// NOT combination (consumes self)
    pub fn not(self) -> Self
    where
        T: 'static,
    {
        let inner = self.inner;
        Self {
            inner: Box::new(move |t| !inner(t)),
            name: None,
        }
    }
}

// Implement Display trait
impl<T> std::fmt::Display for Predicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Predicate({})", self.name.as_deref().unwrap_or("unnamed"))
    }
}

// Implement Debug trait
impl<T> std::fmt::Debug for Predicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Predicate")
            .field("name", &self.name)
            .finish()
    }
}

// SharedPredicate (based on Arc)
pub struct SharedPredicate<T> {
    inner: Arc<dyn Fn(&T) -> bool + Send + Sync>,
    name: Option<String>,
}

impl<T> SharedPredicate<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(f),
            name: None,
        }
    }

    pub fn test(&self, value: &T) -> bool {
        (self.inner)(value)
    }

    pub fn and(self, other: Self) -> Self
    where
        T: 'static,
    {
        let first = self.inner;
        let second = other.inner;
        Self {
            inner: Arc::new(move |t| first(t) && second(t)),
            name: None,
        }
    }

    // ... other methods similar
}

// Implement Clone (Arc can be cloned)
impl<T> Clone for SharedPredicate<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            name: self.name.clone(),
        }
    }
}
```

### Usage Examples

```rust
// Create Predicate
let pred = Predicate::new(|x: &i32| *x > 0)
    .with_name("is_positive");

// Invocation requires using .test()
assert!(pred.test(&5));
assert!(!pred.test(&-3));

// Method chaining
let complex = Predicate::new(|x: &i32| *x > 0)
    .with_name("positive")
    .and(|x: &i32| x % 2 == 0)
    .or(|x: &i32| *x > 100);

assert!(complex.test(&4));
assert!(complex.test(&150));

// Can print and debug
println!("Predicate: {}", pred);
println!("Debug: {:?}", pred);

// SharedPredicate can be cloned
let shared = SharedPredicate::new(|x: &i32| *x > 0);
let cloned1 = shared.clone();
let cloned2 = shared.clone();

// Can be used in multiple places
assert!(shared.test(&5));
assert!(cloned1.test(&10));
assert!(cloned2.test(&15));
```

### Using as Function Parameters

Approach 2 requires defining helper traits to uniformly accept different parameter types:

```rust
// Method 1: Define Testable trait (recommended)
pub trait Testable<T> {
    fn test(&self, value: &T) -> bool;
}

// Implement Testable for closures
impl<T, F> Testable<T> for F
where
    F: Fn(&T) -> bool,
{
    fn test(&self, value: &T) -> bool {
        self(value)
    }
}

// Implement Testable for Predicate
impl<T> Testable<T> for Predicate<T> {
    fn test(&self, value: &T) -> bool {
        self.test(value)
    }
}

// Define function accepting predicate parameters
fn filter_values<T, P>(values: Vec<T>, predicate: &P) -> Vec<T>
where
    T: Clone,
    P: Testable<T>,
{
    values.into_iter().filter(|v| predicate.test(v)).collect()
}

// Usage examples
let values = vec![1, -2, 3, -4, 5];

// 1. Pass closure reference
let closure = |x: &i32| *x > 0;
let result = filter_values(values.clone(), &closure);
assert_eq!(result, vec![1, 3, 5]);

// 2. Pass function pointer
fn is_positive(x: &i32) -> bool { *x > 0 }
let result = filter_values(values.clone(), &is_positive);
assert_eq!(result, vec![1, 3, 5]);

// 3. Pass Predicate object reference
let pred = Predicate::new(|x: &i32| *x > 0);
let result = filter_values(values.clone(), &pred);
assert_eq!(result, vec![1, 3, 5]);
// pred is still available (only borrowed)

// 4. Pass combined predicate
let combined = Predicate::new(|x: &i32| *x > 0).and(|x: &i32| x % 2 == 0);
let result = filter_values(values, &combined);
assert_eq!(result, vec![]);

// Method 2: Use Into<Predicate> (has performance issues, not recommended)
impl<T, F> From<F> for Predicate<T>
where
    F: Fn(&T) -> bool + 'static,
{
    fn from(f: F) -> Self {
        Predicate::new(f)
    }
}

fn filter_values_v2<T, P>(values: Vec<T>, predicate: P) -> Vec<T>
where
    T: 'static,
    P: Into<Predicate<T>>,
{
    let pred = predicate.into();
    values.into_iter().filter(|v| pred.test(v)).collect()
}

// Note: This approach causes Predicate objects to be double-boxed
```

### Advantages

#### 1. **Elegant Method Chaining**
- ✅ **Fluent API**: `.and().or().not()` chaining is more natural
- ✅ **Good readability**: Complex compositions are clearer and easier to read
- ✅ **Fits object-oriented habits**: Similar to styles in Java, C#, and other languages

```rust
// Method chaining is clearer than nested calls
let complex = is_positive
    .and(is_even)
    .or(is_large)
    .not();
```

#### 2. **Powerful Extensibility**
- ✅ **Can add fields**: Name, statistics, creation time, etc.
- ✅ **Can implement traits**: Display, Debug, Serialize, etc.
- ✅ **Can add methods**: Any custom instance methods

```rust
pub struct Predicate<T> {
    inner: Box<dyn Fn(&T) -> bool>,
    name: Option<String>,
    call_count: Arc<AtomicUsize>,  // Call statistics
    created_at: SystemTime,        // Creation time
}

impl<T> Predicate<T> {
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }

    pub fn test(&self, value: &T) -> bool {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        (self.inner)(value)
    }
}
```

#### 3. **Type Safety**
- ✅ **Independent type**: `Predicate<T>` is a distinct type, won't be confused with `Box<dyn Fn>`
- ✅ **Better type checking**: Compiler can provide better error messages
- ✅ **Clear type semantics**: Type name directly reflects its purpose

#### 4. **Generic Parameter Support**

By defining a unified trait or using `Into` conversions, multiple input types can be supported:

```rust
// Method 1: Use Into<Predicate>
impl<T, F> From<F> for Predicate<T>
where
    F: Fn(&T) -> bool + 'static,
{
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

pub fn and<T, P1, P2>(first: P1, second: P2) -> Predicate<T>
where
    P1: Into<Predicate<T>>,
    P2: Into<Predicate<T>>,
{
    let pred1 = first.into();
    let pred2 = second.into();
    // ... composition logic
}

// Method 2: Define Testable trait
pub trait Testable<T> {
    fn test(&self, value: &T) -> bool;
}

impl<T, F> Testable<T> for F
where
    F: Fn(&T) -> bool,
{
    fn test(&self, value: &T) -> bool {
        self(value)
    }
}

pub fn and<T, P1, P2>(first: P1, second: P2) -> Predicate<T>
where
    P1: Testable<T> + 'static,
    P2: Testable<T> + 'static,
{
    Predicate::new(move |t| first.test(t) && second.test(t))
}
```

### Disadvantages

#### 1. **Cannot Call Directly**
- ❌ **Must use `.test()`**: `pred.test(&value)` instead of `pred(&value)`
- ❌ **Not natural integration with standard library**: Extra method calls needed in `filter`
- ❌ **Slightly verbose code**: Every invocation has an extra `.test()`

```rust
// Cannot call directly
// assert!(pred(&5));  // ❌ Compile error

// Must do this
assert!(pred.test(&5));  // ✅

// Usage in filter
let result: Vec<i32> = vec![1, -2, 3, 4]
    .into_iter()
    .filter(|x| pred.test(x))  // ⚠️ Not as natural as direct invocation
    .collect();
```

#### 2. **Still Need Multiple Implementations**
- ⚠️ **Box and Arc need separate implementations**: `Predicate` and `SharedPredicate`
- ⚠️ **Code duplication**: Methods like `and`, `or`, `not` need to be implemented in both structs
- ⚠️ **Increased maintenance cost**: Modifying one requires modifying the other

```rust
// Need to implement the same logic twice
impl<T> Predicate<T> {
    pub fn and(self, other: ...) -> Self { /* implementation */ }
    pub fn or(self, other: ...) -> Self { /* implementation */ }
    pub fn not(self) -> Self { /* implementation */ }
}

impl<T> SharedPredicate<T> {
    pub fn and(self, other: ...) -> Self { /* almost same implementation */ }
    pub fn or(self, other: ...) -> Self { /* almost same implementation */ }
    pub fn not(self) -> Self { /* almost same implementation */ }
}
```

#### 3. **Ownership Issues**
- ⚠️ **Method chaining consumes self**: Each call moves ownership
- ⚠️ **Cannot reuse intermediate results**: Unless Clone is implemented (but Box<dyn Fn> cannot be cloned)
- ⚠️ **Need explicit clone for SharedPredicate**: Even with shared ownership, still need `.clone()`

```rust
let pred = Predicate::new(|x: &i32| *x > 0);
let combined1 = pred.and(|x: &i32| x % 2 == 0);
// pred has been moved, cannot be used anymore

// SharedPredicate requires explicit cloning
let shared = SharedPredicate::new(|x: &i32| *x > 0);
let combined1 = shared.clone().and(...);
let combined2 = shared.clone().or(...);
```

#### 4. **Trait Object Limitations**
- ❌ **Cannot store different types using trait objects**: Because Predicate is a concrete struct

```rust
// Can store same type
let predicates: Vec<Predicate<i32>> = vec![
    Predicate::new(|x| *x > 0),
    Predicate::new(|x| x % 2 == 0),
];

// But cannot store mixed types (if wanting to store both Predicate and SharedPredicate)
// Need to define a unified trait
```

#### 5. **Potential Performance Issues (depending on implementation)**

Using `Into<Predicate>` may lead to double boxing:

```rust
// If using Into conversion
pub fn and<P1, P2>(first: P1, second: P2) -> Predicate<T>
where
    P1: Into<Predicate<T>>,
    P2: Into<Predicate<T>>,
{
    let pred1 = first.into();  // If first is already Predicate, boxes again
    let pred2 = second.into();
    // Composition boxes again
}
```

### Applicable Scenarios

✅ **Best suited for:**

1. **Need method chaining**: Complex predicate compositions, want to use fluent API
2. **Need metadata**: Add name, statistics, debug information to predicates
3. **Need to implement traits**: Display, Debug, Serialize, etc.
4. **Object-oriented style**: Team is more familiar with OOP-style APIs
5. **High type safety requirements**: Want to distinguish different predicate types at the type system level

❌ **Not suited for:**

1. Pursuing minimal API, don't need extra features
2. Need direct invocation (like `pred(&value)`)
3. Need deep integration with standard library
4. Don't want `.test()` everywhere in the code

---

## Approach 3: Trait Abstraction + Multiple Implementations

### Design Overview

This is the most flexible and elegant approach, and is the final approach adopted by the current library. It combines the unified abstraction capability of traits and the concrete implementation capability of structs, achieving a balance of semantic clarity, type safety, and API flexibility.

**Core Idea**:
1.  **Define a minimal `Predicate<T>` Trait**: This trait only contains the core `test(&self, &T) -> bool` method and `into_*` type conversion methods. It does not include logical composition methods like `and`/`or`.
2.  **Provide three concrete Struct implementations**:
    -   `BoxPredicate<T>`: Based on `Box`, for single ownership scenarios.
    -   `ArcPredicate<T>`: Based on `Arc`, for thread-safe shared ownership scenarios.
    -   `RcPredicate<T>`: Based on `Rc`, for single-threaded shared ownership scenarios.
3.  **Implement specialized composition methods on Structs**: Each Struct implements its own `and`/`or`/`not` and other **inherent methods**. This allows composition methods to return **concrete types**, thus maintaining their respective characteristics (e.g., `ArcPredicate` compositions still return `ArcPredicate`, remaining cloneable and thread-safe).
4.  **Provide extension Trait for closures**: Through the `FnPredicateOps<T>` extension trait, provide `.and()`, `.or()`, and other methods to all closures and function pointers, which uniformly return `BoxPredicate<T>` after composition, thus enabling method chaining.
5.  **Uniformly implement `Predicate<T>` Trait**: All closures and the three Structs implement the `Predicate<T>` Trait, allowing them to be uniformly handled by generic functions.

This design perfectly separates "what it is" (`Predicate` trait) from "how it works" (specific implementations of each Struct).

### Core Design

```rust
// ============================================================================
// 1. Define minimal Predicate trait
// ============================================================================

/// Predicate trait - unified predicate interface
///
/// Only defines test and into_* methods, does not include logical composition.
pub trait Predicate<T> {
    /// Test if value satisfies the predicate condition
    fn test(&self, value: &T) -> bool;

    /// Convert to BoxPredicate
    fn into_box(self) -> BoxPredicate<T> where Self: Sized + 'static, T: 'static;

    /// Convert to RcPredicate
    fn into_rc(self) -> RcPredicate<T> where Self: Sized + 'static, T: 'static;

    /// Convert to ArcPredicate
    fn into_arc(self) -> ArcPredicate<T> where Self: Sized + Send + Sync + 'static, T: Send + Sync + 'static;
}

// ============================================================================
// 2. Implement Predicate trait and FnPredicateOps extension for closures
// ============================================================================

/// Implement Predicate for all Fn(&T) -> bool
impl<T, F> Predicate<T> for F where F: Fn(&T) -> bool {
    fn test(&self, value: &T) -> bool { self(value) }
    // ... implementation of into_* methods ...
}

/// Extension trait providing logical composition methods for closures
pub trait FnPredicateOps<T>: Fn(&T) -> bool + Sized {
    /// AND combination - consumes closure, returns BoxPredicate
    fn and<P>(self, other: P) -> BoxPredicate<T>
    where
        Self: 'static,
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self(t) && other.test(t))
    }
    // ... or, not, etc. methods similar, all return BoxPredicate ...
}

/// Implement FnPredicateOps for all closure types
impl<T, F> FnPredicateOps<T> for F where F: Fn(&T) -> bool {}


// ============================================================================
// 3. BoxPredicate - single ownership implementation
// ============================================================================

pub struct BoxPredicate<T> { /* ... */ }

impl<T> BoxPredicate<T> {
    /// AND combination - consumes self, returns BoxPredicate
    pub fn and<P>(self, other: P) -> BoxPredicate<T>
    where
        P: Predicate<T> + 'static,
        T: 'static,
    {
        BoxPredicate::new(move |t| self.test(t) && other.test(t))
    }
    // ... or, not, etc. methods similar ...
}

// ============================================================================
// 4. ArcPredicate - thread-safe shared ownership implementation
// ============================================================================

pub struct ArcPredicate<T> { /* ... */ }

impl<T> ArcPredicate<T> {
    /// AND combination - borrows &self, returns ArcPredicate
    pub fn and(&self, other: &ArcPredicate<T>) -> ArcPredicate<T>
    where
        T: Send + Sync + 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        ArcPredicate::new(move |t| self_clone.test(t) && other_clone.test(t))
    }
    // ... or, not, etc. methods similar ...
}

// ============================================================================
// 5. RcPredicate - single-threaded shared ownership implementation
// ============================================================================

pub struct RcPredicate<T> { /* ... */ }

impl<T> RcPredicate<T> {
    /// AND combination - borrows &self, returns RcPredicate
    pub fn and(&self, other: &RcPredicate<T>) -> RcPredicate<T>
    where
        T: 'static,
    {
        let self_clone = self.clone();
        let other_clone = other.clone();
        RcPredicate::new(move |t| self_clone.test(t) && other_clone.test(t))
    }
    // ... or, not, etc. methods similar ...
}
```

### Usage Examples

```rust
// ============================================================================
// 1. Closures automatically have .test() and logical composition methods
// ============================================================================

let is_positive = |x: &i32| *x > 0;
assert!(is_positive.test(&5)); // Directly use .test()

// Closure method chaining, returns BoxPredicate
let positive_even = is_positive.and(|x: &i32| x % 2 == 0); // is_positive is consumed
assert!(positive_even.test(&4));
// positive_even is BoxPredicate, not cloneable

// ============================================================================
// 2. BoxPredicate - one-time use scenario, consumes self
// ============================================================================

let pred = BoxPredicate::new(|x: &i32| *x > 0);
let combined = pred.and(|x: &i32| x % 2 == 0); // pred is consumed
assert!(combined.test(&4));

// ============================================================================
// 3. ArcPredicate - multi-threaded sharing scenario, borrows &self
// ============================================================================

let shared = ArcPredicate::new(|x: &i32| *x > 0);

// ✅ Composition using method chaining, no need to explicitly clone (uses &self)
let combined = shared.and(&ArcPredicate::new(|x| x % 2 == 0));

// ✅ shared is still usable, can continue composing
let another_combined = shared.or(&ArcPredicate::new(|x| *x < -100));
assert!(shared.test(&5));

// ✅ Composition result is still ArcPredicate, can be cloned and used across threads
let combined_clone = combined.clone();
use std::thread;
let handle = thread::spawn(move || combined_clone.test(&4));
assert!(handle.join().unwrap());


// ============================================================================
// 4. RcPredicate - single-threaded reuse scenario, borrows &self
// ============================================================================

let rc_pred = RcPredicate::new(|x: &i32| *x > 0);

// ✅ Method chaining, no need to explicitly clone
let combined1 = rc_pred.and(&RcPredicate::new(|x| x % 2 == 0));
let combined2 = rc_pred.or(&RcPredicate::new(|x| *x > 100));

// ✅ Original predicate is still usable
assert!(rc_pred.test(&7));

// ============================================================================
// 5. Unified interface - all types implement Predicate trait
// ============================================================================

fn use_any_predicate<P: Predicate<i32>>(pred: &P, value: i32) -> bool {
    pred.test(&value)
}

// All types can be passed in
assert!(use_any_predicate(&positive_even, 4));
assert!(use_any_predicate(&shared, 5));
assert!(use_any_predicate(&rc_pred, 6));
assert!(use_any_predicate(&(|x: &i32| *x < 0), -1));
```

### Using as Function Parameters

Approach 3's unified trait interface makes function parameter usage very natural:

```rust
// Define function accepting predicate parameters (by borrowing)
fn filter_values<T, P>(values: Vec<T>, predicate: &P) -> Vec<T>
where
    T: Clone,
    P: Predicate<T> + ?Sized, // ?Sized allows passing trait object
{
    values.into_iter().filter(|v| predicate.test(v)).collect()
}

// Usage examples
let values = vec![1, -2, 3, -4, 5];

// 1. Pass closure reference
let closure = |x: &i32| *x > 0;
let result = filter_values(values.clone(), &closure);
assert_eq!(result, vec![1, 3, 5]);

// 2. Pass BoxPredicate object reference
let box_pred = BoxPredicate::new(|x: &i32| *x > 0);
let result = filter_values(values.clone(), &box_pred);
assert_eq!(result, vec![1, 3, 5]);

// 3. Pass ArcPredicate object reference
let arc_pred = ArcPredicate::new(|x: &i32| *x > 0);
let result = filter_values(values.clone(), &arc_pred);
assert_eq!(result, vec![1, 3, 5]);

// 4. Pass combined predicate
let combined = (|x: &i32| *x > 0).and(|x: &i32| x % 2 == 0);
let result = filter_values(values, &combined);
assert_eq!(result, vec![]);
```

### Advantages

#### 1. **Perfect Semantic Clarity**

- ✅ **Names are documentation**: `BoxPredicate`, `ArcPredicate`, `RcPredicate` directly express the underlying implementation and ownership model.
- ✅ **Symmetric design**: Three types with symmetric functionality, easy to understand and use.
- ✅ **Consistent with standard library**: Naming pattern consistent with Rust standard library's smart pointers `Box`, `Arc`, `Rc`.

#### 2. **Unified Trait Interface**

- ✅ **Unified abstraction**: All types unified through `Predicate<T>` trait, all can use `.test()`.
- ✅ **Polymorphism support**: Can write generic functions accepting `&dyn Predicate<T>` or `impl Predicate<T>`.
- ✅ **Automatic closure support**: All closures automatically implement `Predicate<T>`, no conversion needed.

#### 3. **Complete Ownership Model Coverage**

Three implementations correspond to three typical scenarios:

| Type | Ownership | Clone | Thread-Safe | API | Use Case |
|:---|:---|:---|:---:|:---|:---|
| `BoxPredicate` | Single | ❌ | ❌ | `self` | One-time use, builder pattern |
| `ArcPredicate` | Shared | ✅ | ✅ | `&self` | Multi-thread sharing, config center |
| `RcPredicate` | Shared | ✅ | ❌ | `&self` | Single-thread reuse, UI validation |

#### 4. **Type Preservation and Elegant API through Specialization**

This is the core advantage of this approach. By **implementing composition methods on concrete Structs** instead of defining them in the Trait:

- ✅ **Type preservation**: `ArcPredicate` composition methods still return `ArcPredicate`, maintaining its cloneable and thread-safe characteristics. Same for `RcPredicate`.
- ✅ **Elegant API**: `ArcPredicate` and `RcPredicate` composition methods use `&self`, no need for explicit `.clone()` when calling, very natural user experience, also consistent with reference-counted type design conventions.
- ✅ **No need for static composition methods**: All operations through method chaining, API is more cohesive and concise.

```rust
// ArcPredicate → ArcPredicate (borrows &self, reusable)
let arc_pred = ArcPredicate::new(|x| *x > 0);
let arc_result = arc_pred.and(&another_arc_pred);   // ✅ No clone needed, direct use
let arc_result2 = arc_pred.or(&third_arc_pred);     // ✅ arc_pred still usable
let cloned = arc_result.clone();                    // ✅ Composition result can also be cloned

// BoxPredicate → BoxPredicate (consumes ownership, uses self)
let box_pred = BoxPredicate::new(|x| *x > 0);
let box_result = box_pred.and(another);             // ⚠️ box_pred moved, cannot be used again
```

#### 5. **Strongest Extensibility**

- ✅ **Can add new implementations**: Future can easily add new predicate types (like `CowPredicate`).
- ✅ **Can add fields**: Each implementation can have its own metadata (name, statistics, etc.).
- ✅ **Can implement traits**: `Display`, `Debug`, `Serialize`, etc.

#### 6. **Consistent with Rust Standard Library Design Philosophy**

This design pattern (one trait + multiple struct implementations) is very similar to the relationship between the `Deref` trait and `Box/Rc/Arc` in the Rust standard library, aligning with Rust's design philosophy.

### Disadvantages

#### 1. **Cannot Call Directly**

Same as Approach 2, this is the biggest inconvenience in usage.

```rust
let pred = BoxPredicate::new(|x: &i32| *x > 0);

// ❌ Cannot call directly
// assert!(pred(&5));

// ✅ Must use .test()
assert!(pred.test(&5));
```

#### 2. **Slightly Higher Learning Curve**

Users need to understand:
- ⚠️ `Predicate` trait as a unified interface.
- ⚠️ Differences and applicable scenarios for `BoxPredicate`, `ArcPredicate`, `RcPredicate`.
- ⚠️ Closure compositions default to returning `BoxPredicate`.
- ⚠️ Why `BoxPredicate` composition methods consume `self`, while `Arc/RcPredicate` use `&self`.

**Mitigation**: Provide clear documentation and usage guides (which is the purpose of this document).

#### 3. **Implementation Cost**

- ⚠️ Need to separately implement all logical composition methods (`and`, `or`, `not`, `xor`, etc.) for three Structs, larger code volume.
- ⚠️ But due to clear architecture and strong logic repetition, long-term maintenance cost is actually lower.

#### 4. **Trait Object Limitations**

The `Predicate<T>` trait itself is not object-safe because its `into_*` methods have `where Self: Sized` constraints. This means you cannot create `Box<dyn Predicate<T>>`.

```rust
// ❌ Compile error: trait is not object-safe
// let predicates: Vec<Box<dyn Predicate<i32>>> = vec![...];

// ✅ Solutions: Use concrete types or Enum wrapper
// Solution A: Use concrete type
let predicates: Vec<BoxPredicate<i32>> = vec![...];
// Solution B: Use Enum wrapper
enum AnyPredicate<T> {
    Box(BoxPredicate<T>),
    Arc(ArcPredicate<T>),
    Rc(RcPredicate<T>),
}
```
In most scenarios, directly using `BoxPredicate` or `ArcPredicate` as collection element types is usually sufficient.

### Applicable Scenarios

✅ **Best suited for:**

1. **Library development**: Provide users with clear, flexible, and powerful APIs.
2. **Large projects**: Large codebase scale, need clear architecture to ensure maintainability.
3. **Team collaboration**: Provide unified interface specifications and clear semantics.
4. **Multi-scenario support**: Simultaneously have one-time use, single-thread reuse, multi-thread sharing scenarios.

✅ **Strongly recommended for foundational library projects like `prism3-rust-function`.**

---

## Summary Comparison of Three Approaches

### Core Feature Comparison Table

| Feature | Approach 1: Type Alias | Approach 2: Struct Encapsulation | Approach 3: Trait + Multiple Implementations |
|:---|:---|:---|:---|
| **Invocation Style** | `pred(&x)` ✅ | `pred.test(&x)` ❌ | `pred.test(&x)` ❌ |
| **Semantic Clarity** | 🟡 Medium | 🟢 Good | 🟢 **Excellent** ✨ |
| **Ownership Model** | Box + Arc (two) | Box + Arc (two) | Box + Arc + Rc (three) ✅ |
| **Type Names** | Predicate / SharedPredicate | Predicate / SharedPredicate | BoxPredicate / ArcPredicate / RcPredicate ✅ |
| **Unified Interface** | ❌ Two separate APIs | ❌ Two separate structs | ✅ **Unified Predicate trait** |
| **Method Chaining** | ❌ Only nesting | ✅ Supported | ✅ **Supported (with type preservation)** ✨ |
| **Extensibility** | ❌ Cannot extend | ✅ Extensible | ✅ **Highly extensible** |
| **Metadata Support**| ❌ Not supported | ✅ Supported | ✅ Supported |
| **Generic Support** | ✅ Perfect (Fn trait) | 🟡 Medium (needs extra abstraction) | ✅ **Perfect (Predicate trait)** |
| **Code Conciseness** | ✅ Minimal | 🟡 Medium | 🟡 Slightly complex |
| **Learning Curve** | ✅ Lowest | 🟡 Medium | 🟡 Slightly high |
| **Maintenance Cost** | 🟡 Medium (two APIs) | 🟡 Medium (code duplication) | ✅ **Low (clear architecture)** |
| **Standard Library Consistency** | 🟡 Medium | 🟡 Medium | ✅ **Perfect** ✨ |

### Usage Scenario Comparison

| Scenario | Approach 1 | Approach 2 | Approach 3 |
|:---|:---|:---|:---|
| **Rapid Prototyping** | ✅ Best | 🟡 Okay | 🟡 Okay |
| **Simple Predicate Composition** | ✅ Best | 🟡 Okay | 🟡 Okay |
| **Complex Method Chaining** | ❌ Not suitable | ✅ Suitable | ✅ **Best** |
| **Need Metadata/Debugging** | ❌ Not supported | ✅ Supported | ✅ **Best** |
| **Multi-thread Sharing** | ✅ SharedPredicate | ✅ SharedPredicate | ✅ **ArcPredicate** |
| **Single-thread Reuse** | ❌ Not supported | ❌ Not supported | ✅ **RcPredicate** |
| **Library Development** | 🟡 Okay | 🟡 Okay | ✅ **Best** |
| **Large Projects** | 🟡 Okay | ✅ Suitable | ✅ **Best** |
| **Long-term Maintenance** | 🟡 Medium | 🟡 Medium | ✅ **Best** |

---

## Conclusion

For library projects like `prism3-rust-function`, **the final choice of Approach 3 is absolutely correct**. It provides unparalleled semantic clarity, architectural flexibility, and long-term maintainability, perfectly aligning with Rust's design philosophy. While there is some cost in implementation and learning, the structural advantages and elegant API design it brings are completely worth the investment.

