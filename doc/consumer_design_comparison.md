# Consumer Design Comparison Analysis

## Overview

This document provides a detailed analysis of three different design approaches for implementing the Consumer type in Rust, comparing their advantages, disadvantages, applicable scenarios, and implementation details.

The core functionality of a Consumer is to accept a value and perform operations on it (typically with side effects) without returning a result, similar to Java's `Consumer<T>` interface. In Rust, we need to make trade-offs in the following aspects:

- **Type Expression**: Type alias vs Struct vs Trait
- **Mutability**: `FnMut` allows modifying captured environment and input values
- **Ownership Model**: Box (single ownership) vs Arc (shared ownership) vs Rc (single-threaded shared)
- **Invocation Style**: Direct call vs method call
- **Composition Capability**: `and_then` method chaining
- **Extensibility**: Whether metadata can be added, whether other traits can be implemented

---

## Approach 1: Type Alias + Static Composition Methods

### Design Overview

Define Consumer types using type aliases and provide composition methods through static utility classes. This is the simplest and most straightforward implementation approach.

### Core Design

```rust
// Type alias definition
pub type Consumer<T> = Box<dyn FnMut(&mut T)>;
pub type SharedConsumer<T> = Arc<Mutex<dyn FnMut(&mut T) + Send>>;

// Static composition utility class
pub struct Consumers;

impl Consumers {
    /// Create AND_THEN composition
    pub fn and_then<T, F1, F2>(first: F1, second: F2) -> Consumer<T>
    where
        T: 'static,
        F1: FnMut(&mut T) + 'static,
        F2: FnMut(&mut T) + 'static,
    {
        let mut first = first;
        let mut second = second;
        Box::new(move |t| {
            first(t);
            second(t);
        })
    }

    /// Create no-op consumer
    pub fn noop<T>() -> Consumer<T>
    where
        T: 'static,
    {
        Box::new(|_| {})
    }

    /// Create conditional consumer
    pub fn if_then<T, P, C>(predicate: P, consumer: C) -> Consumer<T>
    where
        T: 'static,
        P: FnMut(&T) -> bool + 'static,
        C: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        Box::new(move |t| {
            if pred(t) {
                cons(t);
            }
        })
    }
}

// Composition utility class for SharedConsumer
pub struct SharedConsumers;

impl SharedConsumers {
    pub fn and_then<T>(
        first: SharedConsumer<T>,
        second: SharedConsumer<T>,
    ) -> SharedConsumer<T>
    where
        T: 'static,
    {
        Arc::new(Mutex::new(move |t: &mut T| {
            first.lock().unwrap()(t);
            second.lock().unwrap()(t);
        }))
    }

    // ... other methods similar
}
```

### Usage Examples

```rust
// Create consumer
let mut consumer: Consumer<i32> = Box::new(|x| *x *= 2);

// Direct invocation
let mut value = 5;
consumer(&mut value);
assert_eq!(value, 10);

// Compose consumers (passing closures)
let mut chained = Consumers::and_then(
    |x: &mut i32| *x *= 2,
    |x: &mut i32| *x += 10,
);

let mut value = 5;
chained(&mut value);
assert_eq!(value, 20); // (5 * 2) + 10

// Complex composition
let mut complex = Consumers::and_then(
    Consumers::if_then(|x: &i32| *x > 0, |x| *x *= 2),
    |x| *x += 1,
);

let mut value = 5;
complex(&mut value);
assert_eq!(value, 11); // (5 * 2) + 1

// Using SharedConsumer (requires Mutex protection for mutability)
let shared: SharedConsumer<i32> = Arc::new(Mutex::new(|x| *x *= 2));
let cloned = Arc::clone(&shared);

// Use in multiple places
let mut value1 = 5;
shared.lock().unwrap()(&mut value1);
assert_eq!(value1, 10);

let mut value2 = 7;
cloned.lock().unwrap()(&mut value2);
assert_eq!(value2, 14);
```

### Using as Function Parameters

```rust
// Define function accepting consumer parameter
fn for_each<T, F>(values: &mut [T], mut consumer: F)
where
    F: FnMut(&mut T),
{
    for value in values.iter_mut() {
        consumer(value);
    }
}

// Usage examples
let mut values = vec![1, 2, 3, 4, 5];

// 1. Pass closure
for_each(&mut values, |x: &mut i32| *x *= 2);
assert_eq!(values, vec![2, 4, 6, 8, 10]);

// 2. Pass Consumer object (note: transfers ownership)
let mut consumer: Consumer<i32> = Box::new(|x| *x += 1);
for_each(&mut values, consumer); // consumer is moved
assert_eq!(values, vec![3, 5, 7, 9, 11]);
// consumer is no longer available here

// 3. Pass composed consumer
let mut combined = Consumers::and_then(|x: &mut i32| *x *= 2, |x| *x += 1);
let mut values = vec![1, 2, 3];
for_each(&mut values, combined);
assert_eq!(values, vec![3, 5, 7]); // (x * 2) + 1
```

### Advantages

#### 1. **Minimalist API and User Experience**
- ✅ **Direct invocation**: `consumer(&mut value)` instead of `consumer.accept(&mut value)`
- ✅ **Zero cognitive load**: Type alias is completely transparent, users can directly use `Box<dyn FnMut>`
- ✅ **Perfect integration with standard library**: Can be used directly in methods like `for_each`, `iter_mut`

```rust
// Very natural use with standard library
vec![1, 2, 3]
    .iter_mut()
    .for_each(|x| consumer(x)); // ✅ Used directly as closure
```

#### 2. **Perfect Generic Support**
- ✅ **Unified FnMut trait**: Closures and Consumers are unified through `FnMut(&mut T)`
- ✅ **No conversion needed**: All callable types can be passed directly to composition methods
- ✅ **Type inference friendly**: Compiler can automatically infer closure types

```rust
// Supports all callable types
let c1 = Consumers::and_then(|x| *x *= 2, |x| *x += 1);     // Closures
let c2 = Consumers::and_then(multiply_fn, add_fn);          // Function pointers
let c3 = Consumers::and_then(c1, |x| println!("{}", x));    // Consumer + closure
```

#### 3. **Zero-Cost Abstraction**
- ✅ **Single boxing**: Each closure is boxed only once
- ✅ **Inline optimization**: Compiler can optimize closure calls
- ✅ **No extra indirection**: Direct invocation through `Box::call_mut()`

#### 4. **Simple Implementation**
- ✅ **Less code**: No need to define complex structs or traits
- ✅ **Low maintenance cost**: Type aliases are easy to understand and maintain
- ✅ **Concise documentation**: Users only need to understand function signatures

### Disadvantages

#### 1. **Cannot Extend**
- ❌ **Cannot add fields**: Cannot add metadata like name, statistics to Consumer
- ❌ **Cannot implement traits**: Type aliases cannot implement `Display`, `Debug` and other traits
- ❌ **Cannot add methods**: Cannot add instance methods to Consumer

```rust
// ❌ Cannot implement
impl<T> Display for Consumer<T> {  // Compile error: type alias cannot have impl
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Consumer")
    }
}
```

#### 2. **Low Type Distinctiveness**
- ❌ **Cannot distinguish at type system level**: `Consumer<T>` and `Box<dyn FnMut(&mut T)>` are completely equivalent
- ❌ **Easy to confuse**: Users might directly use `Box::new()` instead of going through `Consumers`
- ❌ **Semantics not clear enough**: Type name cannot convey more information

#### 3. **Two Parallel APIs**
- ⚠️ **Consumer vs SharedConsumer**: Need to maintain two sets of types and composition methods
- ⚠️ **SharedConsumer needs Mutex**: Due to `FnMut` requiring mutability, Arc must be used with Mutex
- ⚠️ **Performance overhead**: SharedConsumer requires locking on every call
- ⚠️ **Missing Rc support**: No Rc implementation provided for single-threaded scenarios

```rust
// Two parallel APIs
struct Consumers;           // Provides composition methods for Consumer
struct SharedConsumers;     // Provides composition methods for SharedConsumer

// Performance issue with SharedConsumer
let shared: SharedConsumer<i32> = Arc::new(Mutex::new(|x| *x *= 2));
// Requires locking on every call
shared.lock().unwrap()(&mut value); // ⚠️ Lock overhead
```

#### 4. **Cannot Implement Method Chains**
- ❌ **Only nested calls**: Deep nesting for complex compositions
- ❌ **Poor readability**: Multi-level nesting is less clear than method chaining

```rust
// Complex composition requires nesting
let complex = Consumers::and_then(
    Consumers::if_then(
        is_positive,
        multiply_by_two
    ),
    add_one
);

// Cannot use method chains (ideal form):
// let complex = multiply_by_two.if_then(is_positive).and_then(add_one);
```

### Applicable Scenarios

✅ **Best suited for:**

1. **Simple operation composition**: No need for complex metadata or method chains
2. **Pursuing minimalist API**: Want code to be as concise as possible
3. **Deep standard library integration**: Need to use directly in methods like `for_each`
4. **One-time use**: Consumer is created and doesn't need to be reused multiple times
5. **Rapid prototyping**: Quickly implement functionality without considering long-term extension

❌ **Not suitable for:**

1. Need to add name, debug information and other metadata to consumer
2. Need to implement traits like `Display`, `Debug`
3. Need complex method chaining
4. Need frequent use in multi-threaded environments (SharedConsumer's lock overhead)

---

## Approach 2: Struct Wrapper + Instance Methods

### Design Overview

Define Consumer as a struct wrapping `Box<dyn FnMut>`, provide composition capabilities through instance methods, supporting method chaining. This is the approach currently adopted by `prism3-rust-function`.

### Core Design

```rust
// Struct definition
pub struct Consumer<T> {
    func: Box<dyn FnMut(&mut T)>,
}

impl<T> Consumer<T>
where
    T: 'static,
{
    /// Create new Consumer
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        Consumer { func: Box::new(f) }
    }

    /// Execute consumer
    pub fn accept(&mut self, value: &mut T) {
        (self.func)(value)
    }

    /// Chain composition (consumes self)
    pub fn and_then<F>(self, next: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        let mut first = self.func;
        let mut second = next;

        Consumer {
            func: Box::new(move |t| {
                first(t);
                second(t);
            }),
        }
    }

    /// Compose with another Consumer instance
    pub fn and_then_consumer(self, next: Consumer<T>) -> Self {
        let mut first = self.func;
        let mut second = next.func;

        Consumer {
            func: Box::new(move |t| {
                first(t);
                second(t);
            }),
        }
    }

    /// Create no-op consumer
    pub fn noop() -> Self {
        Consumer::new(|_| {})
    }

    /// Create print consumer
    pub fn print() -> Self
    where
        T: std::fmt::Debug,
    {
        Consumer::new(|t| {
            println!("{:?}", t);
        })
    }

    /// Create conditional consumer
    pub fn if_then<P, C>(predicate: P, consumer: C) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut cons = consumer;
        Consumer::new(move |t| {
            if pred(t) {
                cons(t);
            }
        })
    }

    /// Create conditional branch consumer
    pub fn if_then_else<P, C1, C2>(
        predicate: P,
        then_consumer: C1,
        else_consumer: C2,
    ) -> Self
    where
        P: FnMut(&T) -> bool + 'static,
        C1: FnMut(&mut T) + 'static,
        C2: FnMut(&mut T) + 'static,
    {
        let mut pred = predicate;
        let mut then_cons = then_consumer;
        let mut else_cons = else_consumer;
        Consumer::new(move |t| {
            if pred(t) {
                then_cons(t);
            } else {
                else_cons(t);
            }
        })
    }
}

// SharedConsumer (based on Arc + Mutex)
pub struct SharedConsumer<T> {
    func: Arc<Mutex<dyn FnMut(&mut T) + Send>>,
}

impl<T> SharedConsumer<T>
where
    T: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + Send + 'static,
    {
        SharedConsumer {
            func: Arc::new(Mutex::new(f)),
        }
    }

    pub fn accept(&self, value: &mut T) {
        (self.func.lock().unwrap())(value)
    }

    pub fn and_then(&self, next: &SharedConsumer<T>) -> Self {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        SharedConsumer {
            func: Arc::new(Mutex::new(move |t: &mut T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
        }
    }
}

// Implement Clone (Arc can be cloned)
impl<T> Clone for SharedConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}
```

### Usage Examples

```rust
// Create Consumer
let mut consumer = Consumer::new(|x: &mut i32| *x *= 2);

// Invocation requires using .accept()
let mut value = 5;
consumer.accept(&mut value);
assert_eq!(value, 10);

// Method chaining
let mut chained = Consumer::new(|x: &mut i32| *x *= 2)
    .and_then(|x| *x += 10)
    .and_then(|x| println!("Result: {}", x));

let mut value = 5;
chained.accept(&mut value); // Prints: Result: 20
assert_eq!(value, 20);

// Using factory methods
let mut print = Consumer::<i32>::print();
let mut value = 42;
print.accept(&mut value); // Prints: 42

// Conditional consumer
let mut conditional = Consumer::if_then(
    |x: &i32| *x > 0,
    |x: &mut i32| *x += 1,
);

let mut positive = 5;
conditional.accept(&mut positive);
assert_eq!(positive, 6);

let mut negative = -5;
conditional.accept(&mut negative);
assert_eq!(negative, -5); // Unchanged

// SharedConsumer can be cloned
let shared = SharedConsumer::new(|x: &mut i32| *x *= 2);
let cloned1 = shared.clone();
let cloned2 = shared.clone();

// Can be used in multiple places
let mut value1 = 5;
shared.accept(&mut value1);
assert_eq!(value1, 10);

let mut value2 = 7;
cloned1.accept(&mut value2);
assert_eq!(value2, 14);
```

### Using as Function Parameters

Approach 2 requires defining helper traits to uniformly accept different types of parameters:

```rust
// Method 1: Define Consumable trait (recommended)
pub trait Consumable<T> {
    fn accept(&mut self, value: &mut T);
}

// Implement Consumable for closures
impl<T, F> Consumable<T> for F
where
    F: FnMut(&mut T),
{
    fn accept(&mut self, value: &mut T) {
        self(value)
    }
}

// Implement Consumable for Consumer
impl<T> Consumable<T> for Consumer<T> {
    fn accept(&mut self, value: &mut T) {
        self.accept(value)
    }
}

// Define function accepting consumer parameter
fn for_each<T, C>(values: &mut [T], consumer: &mut C)
where
    C: Consumable<T>,
{
    for value in values.iter_mut() {
        consumer.accept(value);
    }
}

// Usage examples
let mut values = vec![1, 2, 3, 4, 5];

// 1. Pass closure reference
let mut closure = |x: &mut i32| *x *= 2;
for_each(&mut values, &mut closure);
assert_eq!(values, vec![2, 4, 6, 8, 10]);

// 2. Pass Consumer object reference
let mut consumer = Consumer::new(|x: &mut i32| *x += 1);
for_each(&mut values, &mut consumer);
assert_eq!(values, vec![3, 5, 7, 9, 11]);
// consumer is still available (only borrowed)

// 3. Pass composed consumer
let mut combined = Consumer::new(|x: &mut i32| *x *= 2)
    .and_then(|x| *x += 1);
let mut values = vec![1, 2, 3];
for_each(&mut values, &mut combined);
assert_eq!(values, vec![3, 5, 7]); // (x * 2) + 1
```

### Advantages

#### 1. **Elegant Method Chains**
- ✅ **Fluent API**: Method chaining with `.and_then()` is more natural
- ✅ **Good readability**: Complex compositions are clearer and easier to read
- ✅ **Fits OOP habits**: Similar to style in languages like Java, C#

```rust
// Method chains are clearer than nested calls
let mut complex = Consumer::new(|x| *x *= 2)
    .and_then(|x| *x += 10)
    .and_then(|x| println!("Result: {}", x));
```

#### 2. **Strong Extensibility**
- ✅ **Can add fields**: Can add metadata like name, statistics to Consumer
- ✅ **Can implement traits**: Display, Debug, Serialize, etc.
- ✅ **Can add methods**: Any custom instance methods and factory methods

```rust
pub struct Consumer<T> {
    func: Box<dyn FnMut(&mut T)>,
    name: Option<String>,           // Name
    call_count: Arc<AtomicUsize>,   // Call statistics
}

impl<T> Consumer<T> {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }
}
```

#### 3. **Type Safety**
- ✅ **Independent type**: `Consumer<T>` is a distinct type, won't be confused with `Box<dyn FnMut>`
- ✅ **Better type checking**: Compiler can provide better error messages
- ✅ **Clear type semantics**: Type name directly reflects purpose

#### 4. **Rich Factory Methods**
- ✅ **Convenient constructors**: `noop()`, `print()`, `if_then()`, etc.
- ✅ **Improved development efficiency**: Common patterns work out of the box
- ✅ **Code reuse**: Avoid repeatedly writing same logic

```rust
// Convenient factory methods
let mut noop = Consumer::<i32>::noop();
let mut print = Consumer::<i32>::print();
let mut conditional = Consumer::if_then(|x| *x > 0, |x| *x += 1);
```

### Disadvantages

#### 1. **Cannot Call Directly**
- ❌ **Must use `.accept()`**: `consumer.accept(&mut value)` instead of `consumer(&mut value)`
- ❌ **Less natural standard library integration**: Requires extra adaptation in `for_each`
- ❌ **Code slightly verbose**: Every call has an extra `.accept()`

```rust
// Cannot call directly
let mut consumer = Consumer::new(|x: &mut i32| *x *= 2);
// consumer(&mut value);  // ❌ Compile error

// Must do this
consumer.accept(&mut value);  // ✅

// Slightly awkward use with standard library
values.iter_mut().for_each(|x| consumer.accept(x)); // ⚠️ But consumer's mutable borrow will have issues
```

#### 2. **Still Need Multiple Implementations**
- ⚠️ **Box and Arc need separate implementations**: `Consumer` and `SharedConsumer`
- ⚠️ **Code duplication**: Methods like `and_then` need to be implemented twice in both structs
- ⚠️ **SharedConsumer must use Mutex**: Due to `FnMut` requiring mutability, Arc must be used with Mutex
- ⚠️ **Increased maintenance cost**: Modifying one requires modifying the other

```rust
// Need to implement same logic twice
impl<T> Consumer<T> {
    pub fn and_then(self, other: ...) -> Self { /* implementation */ }
}

impl<T> SharedConsumer<T> {
    pub fn and_then(&self, other: ...) -> Self { /* similar implementation, but needs to handle locks */ }
}
```

#### 3. **Ownership Issues**
- ⚠️ **Method chains consume self**: Each call moves ownership
- ⚠️ **Cannot reuse intermediate results**: Consumer cannot be cloned (`Box<dyn FnMut>` cannot be cloned)
- ⚠️ **SharedConsumer needs explicit clone**: Even with shared ownership, needs `.clone()`

```rust
let consumer = Consumer::new(|x: &mut i32| *x *= 2);
let combined = consumer.and_then(|x| *x += 1);
// consumer has been moved, cannot be used again

// SharedConsumer needs explicit clone
let shared = SharedConsumer::new(|x: &mut i32| *x *= 2);
let combined1 = shared.clone().and_then(...);
let combined2 = shared.clone().and_then(...);
```

#### 4. **SharedConsumer Performance Overhead**
- ⚠️ **Locking on every call**: Mutex lock overhead cannot be avoided
- ⚠️ **May cause lock contention**: Performance may be affected in multi-threaded scenarios
- ⚠️ **Complex error handling**: `lock().unwrap()` may cause panic

```rust
// Requires locking on every call
let shared = SharedConsumer::new(|x: &mut i32| *x *= 2);
shared.accept(&mut value); // Internally needs lock().unwrap()

// Nested locks during composition, may cause deadlock or performance issues
let combined = shared1.and_then(&shared2); // Internally creates nested Mutex
```

#### 5. **Mutable Borrow Limitations**
- ⚠️ **accept needs &mut self**: Makes it difficult to use in certain scenarios
- ⚠️ **Cannot use directly in iterators**: Because requires mutable borrow

```rust
let mut consumer = Consumer::new(|x: &mut i32| *x *= 2);

// ❌ Compile error: cannot mutably borrow consumer in closure
// values.iter_mut().for_each(|x| consumer.accept(x));

// ✅ Must use manual loop
for value in values.iter_mut() {
    consumer.accept(value);
}
```

### Applicable Scenarios

✅ **Best suited for:**

1. **Need method chains**: Complex operation composition, want to use fluent API
2. **Need metadata**: Add name, statistics, debug information to consumer
3. **Need to implement traits**: Display, Debug, Serialize, etc.
4. **Object-oriented style**: Team is more familiar with OOP-style API
5. **Rich factory methods**: Convenient constructors like noop, print, if_then

❌ **Not suitable for:**

1. Pursuing minimalist API, don't need extra features
2. Need direct invocation (like `consumer(&mut value)`)
3. Need to use in iterator method chains
4. High-frequency multi-threaded calls (SharedConsumer's lock overhead)

---

## Approach 3: Trait Abstraction + Multiple Implementations

### Design Overview

This is the most flexible and elegant approach, similar to the Approach 3 design for Predicate. It combines the unified abstraction capability of traits with the concrete implementation capability of structs.

**Core Idea**:
1. **Define minimal `Consumer<T>` Trait**: Only contains core `accept(&mut self, &mut T)` method and `into_*` type conversion methods
2. **Provide three concrete Struct implementations**:
   - `BoxConsumer<T>`: Based on `Box`, for single ownership scenarios
   - `ArcConsumer<T>`: Based on `Arc<Mutex<>>`, for thread-safe shared ownership scenarios
   - `RcConsumer<T>`: Based on `Rc<RefCell<>>`, for single-threaded shared ownership scenarios
3. **Implement specialized composition methods on Structs**: Each Struct implements its own inherent methods like `and_then`
4. **Provide extension Trait for closures**: Through extension traits, provide methods like `.and_then()` for all closures
5. **Uniformly implement `Consumer<T>` Trait**: All closures and three Structs implement `Consumer<T>` Trait

### Core Design

```rust
// ============================================================================
// 1. Define minimal Consumer trait
// ============================================================================

/// Consumer trait - unified consumer interface
pub trait Consumer<T> {
    /// Execute consume operation
    fn accept(&mut self, value: &mut T);

    /// Convert to BoxConsumer
    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Convert to RcConsumer
    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static;

    /// Convert to ArcConsumer
    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static;
}

// ============================================================================
// 2. Implement Consumer trait for closures
// ============================================================================

/// Implement Consumer for all FnMut(&mut T)
impl<T, F> Consumer<T> for F
where
    F: FnMut(&mut T),
{
    fn accept(&mut self, value: &mut T) {
        self(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        BoxConsumer::new(self)
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        Self: Sized + 'static,
        T: 'static,
    {
        RcConsumer::new(self)
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        Self: Sized + Send + 'static,
        T: Send + 'static,
    {
        ArcConsumer::new(self)
    }
}

// ============================================================================
// 3. Extension trait providing logical composition methods for closures
// ============================================================================

/// Extension trait providing composition methods for closures
pub trait FnConsumerOps<T>: FnMut(&mut T) + Sized {
    /// AND_THEN composition - consumes closure, returns BoxConsumer
    fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        Self: 'static,
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let mut first = self;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first.accept(t);
            second.accept(t);
        })
    }
}

/// Implement FnConsumerOps for all closure types
impl<T, F> FnConsumerOps<T> for F where F: FnMut(&mut T) {}

// ============================================================================
// 4. BoxConsumer - single ownership implementation
// ============================================================================

pub struct BoxConsumer<T> {
    func: Box<dyn FnMut(&mut T)>,
}

impl<T> BoxConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        BoxConsumer { func: Box::new(f) }
    }

    /// AND_THEN composition - consumes self, returns BoxConsumer
    pub fn and_then<C>(self, next: C) -> BoxConsumer<T>
    where
        C: Consumer<T> + 'static,
        T: 'static,
    {
        let mut first = self.func;
        let mut second = next;
        BoxConsumer::new(move |t| {
            first(t);
            second.accept(t);
        })
    }

    // Factory methods
    pub fn noop() -> Self
    where
        T: 'static,
    {
        BoxConsumer::new(|_| {})
    }

    pub fn print() -> Self
    where
        T: std::fmt::Debug + 'static,
    {
        BoxConsumer::new(|t| println!("{:?}", t))
    }
}

impl<T> Consumer<T> for BoxConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func)(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        RcConsumer::new(move |t| self.func(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        ArcConsumer::new(move |t| self.func(t))
    }
}

// ============================================================================
// 5. ArcConsumer - thread-safe shared ownership implementation
// ============================================================================

pub struct ArcConsumer<T> {
    func: Arc<Mutex<dyn FnMut(&mut T) + Send>>,
}

impl<T> ArcConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + Send + 'static,
    {
        ArcConsumer {
            func: Arc::new(Mutex::new(f)),
        }
    }

    /// AND_THEN composition - borrows &self, returns ArcConsumer
    pub fn and_then(&self, next: &ArcConsumer<T>) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        let first = Arc::clone(&self.func);
        let second = Arc::clone(&next.func);
        ArcConsumer {
            func: Arc::new(Mutex::new(move |t: &mut T| {
                first.lock().unwrap()(t);
                second.lock().unwrap()(t);
            })),
        }
    }
}

impl<T> Consumer<T> for ArcConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func.lock().unwrap())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        BoxConsumer::new(move |t| self.func.lock().unwrap()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        RcConsumer::new(move |t| self.func.lock().unwrap()(t))
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        self
    }
}

impl<T> Clone for ArcConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// ============================================================================
// 6. RcConsumer - single-threaded shared ownership implementation
// ============================================================================

pub struct RcConsumer<T> {
    func: Rc<RefCell<dyn FnMut(&mut T)>>,
}

impl<T> RcConsumer<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&mut T) + 'static,
    {
        RcConsumer {
            func: Rc::new(RefCell::new(f)),
        }
    }

    /// AND_THEN composition - borrows &self, returns RcConsumer
    pub fn and_then(&self, next: &RcConsumer<T>) -> RcConsumer<T>
    where
        T: 'static,
    {
        let first = Rc::clone(&self.func);
        let second = Rc::clone(&next.func);
        RcConsumer {
            func: Rc::new(RefCell::new(move |t: &mut T| {
                first.borrow_mut()(t);
                second.borrow_mut()(t);
            })),
        }
    }
}

impl<T> Consumer<T> for RcConsumer<T> {
    fn accept(&mut self, value: &mut T) {
        (self.func.borrow_mut())(value)
    }

    fn into_box(self) -> BoxConsumer<T>
    where
        T: 'static,
    {
        BoxConsumer::new(move |t| self.func.borrow_mut()(t))
    }

    fn into_rc(self) -> RcConsumer<T>
    where
        T: 'static,
    {
        self
    }

    fn into_arc(self) -> ArcConsumer<T>
    where
        T: Send + 'static,
    {
        panic!("Cannot convert RcConsumer to ArcConsumer (not Send)")
    }
}

impl<T> Clone for RcConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}
```

### Usage Examples

```rust
// ============================================================================
// 1. Closures automatically have .accept() and logical composition methods
// ============================================================================

let mut closure = |x: &mut i32| *x *= 2;
let mut value = 5;
closure.accept(&mut value); // Directly use .accept()
assert_eq!(value, 10);

// Closures use method chains, returns BoxConsumer
let mut chained = (|x: &mut i32| *x *= 2).and_then(|x| *x += 10);
let mut value = 5;
chained.accept(&mut value);
assert_eq!(value, 20);

// ============================================================================
// 2. BoxConsumer - one-time use scenarios, consumes self
// ============================================================================

let consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);
let mut combined = consumer.and_then(|x| *x += 10); // consumer is consumed
let mut value = 5;
combined.accept(&mut value);
assert_eq!(value, 20);

// ============================================================================
// 3. ArcConsumer - multi-threaded sharing scenario, borrows &self
// ============================================================================

let shared = ArcConsumer::new(|x: &mut i32| *x *= 2);

// ✅ Use method chain composition, no need for explicit clone
let combined = shared.and_then(&ArcConsumer::new(|x| *x += 10));

// ✅ shared is still available, can continue composing
let another_combined = shared.and_then(&ArcConsumer::new(|x| *x -= 5));

let mut value = 5;
let mut shared_clone = shared.clone();
shared_clone.accept(&mut value);
assert_eq!(value, 10);

// ✅ Composition result is still ArcConsumer, can be cloned and used across threads
let combined_clone = combined.clone();
use std::thread;
let handle = thread::spawn(move || {
    let mut val = 5;
    let mut c = combined_clone;
    c.accept(&mut val);
    val
});
assert_eq!(handle.join().unwrap(), 20);

// ============================================================================
// 4. RcConsumer - single-threaded reuse scenario, borrows &self
// ============================================================================

let rc_consumer = RcConsumer::new(|x: &mut i32| *x *= 2);

// ✅ Use method chains, no need for explicit clone
let combined1 = rc_consumer.and_then(&RcConsumer::new(|x| *x += 10));
let combined2 = rc_consumer.and_then(&RcConsumer::new(|x| *x -= 5));

// ✅ Original consumer is still available
let mut value = 5;
let mut rc_clone = rc_consumer.clone();
rc_clone.accept(&mut value);
assert_eq!(value, 10);

// ============================================================================
// 5. Unified interface - all types implement Consumer trait
// ============================================================================

fn apply_consumer<C: Consumer<i32>>(consumer: &mut C, value: i32) -> i32 {
    let mut val = value;
    consumer.accept(&mut val);
    val
}

// All types can be passed in
let mut box_con = BoxConsumer::new(|x: &mut i32| *x *= 2);
assert_eq!(apply_consumer(&mut box_con, 5), 10);

let mut arc_con = ArcConsumer::new(|x: &mut i32| *x *= 2);
assert_eq!(apply_consumer(&mut arc_con, 5), 10);

let mut rc_con = RcConsumer::new(|x: &mut i32| *x *= 2);
assert_eq!(apply_consumer(&mut rc_con, 5), 10);

let mut closure = |x: &mut i32| *x *= 2;
assert_eq!(apply_consumer(&mut closure, 5), 10);
```

### Using as Function Parameters

Approach 3's unified trait interface makes function parameter usage very natural:

```rust
// Define function accepting consumer parameter (via mutable borrow)
fn for_each<T, C>(values: &mut [T], consumer: &mut C)
where
    C: Consumer<T>,
{
    for value in values.iter_mut() {
        consumer.accept(value);
    }
}

// Usage examples
let mut values = vec![1, 2, 3, 4, 5];

// 1. Pass closure reference
let mut closure = |x: &mut i32| *x *= 2;
for_each(&mut values, &mut closure);
assert_eq!(values, vec![2, 4, 6, 8, 10]);

// 2. Pass BoxConsumer object reference
let mut box_con = BoxConsumer::new(|x: &mut i32| *x += 1);
for_each(&mut values, &mut box_con);
assert_eq!(values, vec![3, 5, 7, 9, 11]);

// 3. Pass ArcConsumer object reference
let mut arc_con = ArcConsumer::new(|x: &mut i32| *x *= 2);
for_each(&mut values, &mut arc_con);
assert_eq!(values, vec![6, 10, 14, 18, 22]);

// 4. Pass composed consumer
let mut combined = (|x: &mut i32| *x *= 2).and_then(|x| *x += 1);
let mut values = vec![1, 2, 3];
for_each(&mut values, &mut combined);
assert_eq!(values, vec![3, 5, 7]); // (x * 2) + 1
```

### Advantages

#### 1. **Perfect Semantic Clarity**

- ✅ **Name is documentation**: `BoxConsumer`, `ArcConsumer`, `RcConsumer` directly express underlying implementation and ownership model
- ✅ **Symmetric design**: Three types are functionally symmetric, easy to understand and use
- ✅ **Consistent with standard library**: Naming pattern consistent with Rust standard library's smart pointers `Box`, `Arc`, `Rc`

#### 2. **Unified Trait Interface**

- ✅ **Unified abstraction**: All types unified through `Consumer<T>` trait, all can use `.accept()`
- ✅ **Polymorphism support**: Can write generic functions accepting `&mut dyn Consumer<T>` or `impl Consumer<T>`
- ✅ **Automatic closure support**: All closures automatically implement `Consumer<T>`, no conversion needed

#### 3. **Complete Ownership Model Coverage**

Three implementations correspond to three typical scenarios:

| Type | Ownership | Clone | Thread-Safe | Interior Mutability | API | Use Case |
|:---|:---|:---|:---:|:---:|:---|:---|
| `BoxConsumer` | Single | ❌ | ❌ | FnMut | `self` | One-time use, builder pattern |
| `ArcConsumer` | Shared | ✅ | ✅ | Arc<Mutex<>> | `&self` | Multi-threaded sharing, concurrent tasks |
| `RcConsumer` | Shared | ✅ | ❌ | Rc<RefCell<>> | `&self` | Single-threaded reuse, event handling |

#### 4. **Specialization Brings Type Preservation and Elegant API**

This is the core advantage of this approach:

- ✅ **Type preservation**: `ArcConsumer`'s composition methods return `ArcConsumer`, preserving its cloneable and thread-safe properties
- ✅ **Elegant API**: `ArcConsumer` and `RcConsumer`'s composition methods use `&self`, no need for explicit `.clone()` when calling
- ✅ **No static composition methods needed**: All operations completed through method chains, API more cohesive and concise

```rust
// ArcConsumer → ArcConsumer (borrows &self, reusable)
let arc_con = ArcConsumer::new(|x| *x *= 2);
let arc_result = arc_con.and_then(&another_arc);   // ✅ No need to clone
let arc_result2 = arc_con.and_then(&third_arc);    // ✅ arc_con still available
let cloned = arc_result.clone();                   // ✅ Composition result also cloneable

// BoxConsumer → BoxConsumer (consumes ownership, uses self)
let box_con = BoxConsumer::new(|x| *x *= 2);
let box_result = box_con.and_then(another);        // ⚠️ box_con is moved, cannot use again
```

#### 5. **Solves Interior Mutability Issues**

- ✅ **ArcConsumer uses Arc<Mutex<>>**: Thread-safe interior mutability
- ✅ **RcConsumer uses Rc<RefCell<>>**: Single-threaded interior mutability, no lock overhead
- ✅ **Clear semantics**: Type names clearly express interior mutability implementation

#### 6. **Strongest Extensibility**

- ✅ **Can add new implementations**: Easy to add new consumer types in the future
- ✅ **Can add fields**: Each implementation can have its own metadata (name, statistics, etc.)
- ✅ **Can implement traits**: `Display`, `Debug`, `Serialize`, etc.

#### 7. **Consistent with Rust Standard Library Design Philosophy**

This design pattern (one trait + multiple struct implementations) is completely consistent with smart pointer design in Rust standard library, aligns with Rust's design philosophy.

### Disadvantages

#### 1. **Still Cannot Call Directly**

Same as Approach 2, this is an inconvenience in usage.

```rust
let mut consumer = BoxConsumer::new(|x: &mut i32| *x *= 2);

// ❌ Cannot call directly
// consumer(&mut value);

// ✅ Must use .accept()
consumer.accept(&mut value);
```

#### 2. **Slightly Higher Learning Curve**

Users need to understand:
- ⚠️ `Consumer` trait as unified interface
- ⚠️ Differences and applicable scenarios of `BoxConsumer`, `ArcConsumer`, `RcConsumer`
- ⚠️ Closure composition defaults to returning `BoxConsumer`
- ⚠️ Why `BoxConsumer`'s composition methods consume `self`, while `Arc/RcConsumer` use `&self`
- ⚠️ Reasons why `ArcConsumer` uses `Mutex`, `RcConsumer` uses `RefCell`

**Mitigation**: Provide clear documentation and usage guides (which is the purpose of this document).

#### 3. **Implementation Cost**

- ⚠️ Need to implement all methods separately for three Structs, more code
- ⚠️ But due to clear architecture and strong logic repetition, long-term maintenance cost is actually lower

#### 4. **Interior Mutability Overhead**

- ⚠️ **ArcConsumer**: Mutex lock overhead (but necessary for multi-threaded sharing)
- ⚠️ **RcConsumer**: RefCell runtime borrow checking overhead (but lighter than Mutex)

#### 5. **Trait Object Limitations**

The `Consumer<T>` trait itself is not object-safe (if it includes `into_*` methods with `where Self: Sized` constraints), meaning you cannot create `Box<dyn Consumer<T>>`.

```rust
// ❌ May compile error: trait is not object-safe
// let consumers: Vec<Box<dyn Consumer<i32>>> = vec![...];

// ✅ Solution: Use concrete types or Enum wrapper
// Solution A: Use concrete type
let consumers: Vec<BoxConsumer<i32>> = vec![...];

// Solution B: Use Enum wrapper
enum AnyConsumer<T> {
    Box(BoxConsumer<T>),
    Arc(ArcConsumer<T>),
    Rc(RcConsumer<T>),
}
```

### Applicable Scenarios

✅ **Best suited for:**

1. **Library development**: Provide clear, flexible, powerful API for users
2. **Large projects**: Large codebase scale, need clear architecture to ensure maintainability
3. **Team collaboration**: Provide unified interface specification and clear semantics
4. **Multi-scenario support**: Simultaneously have one-time use, single-threaded reuse, multi-threaded sharing and other scenarios
5. **Need interior mutability**: Need to choose appropriate interior mutability implementation in different scenarios

✅ **Strongly recommended for foundational library projects like `prism3-rust-function`.**

---

## Summary Comparison of Three Approaches

### Core Feature Comparison Table

| Feature | Approach 1: Type Alias | Approach 2: Struct Wrapper | Approach 3: Trait + Multiple Impls |
|:---|:---|:---|:---|
| **Invocation Style** | `consumer(&mut x)` ✅ | `consumer.accept(&mut x)` ❌ | `consumer.accept(&mut x)` ❌ |
| **Semantic Clarity** | 🟡 Medium | 🟢 Good | 🟢 **Excellent** ✨ |
| **Ownership Model** | Box + Arc<Mutex> (two) | Box + Arc<Mutex> (two) | Box + Arc<Mutex> + Rc<RefCell> (three) ✅ |
| **Type Names** | Consumer / SharedConsumer | Consumer / SharedConsumer | BoxConsumer / ArcConsumer / RcConsumer ✅ |
| **Unified Interface** | ❌ Two independent APIs | ❌ Two independent structs | ✅ **Unified Consumer trait** |
| **Method Chains** | ❌ Only nesting | ✅ Supported | ✅ **Supported (with type preservation)** ✨ |
| **Extensibility** | ❌ Cannot extend | ✅ Extensible | ✅ **Highly extensible** |
| **Metadata Support** | ❌ Not supported | ✅ Supported | ✅ Supported |
| **Factory Methods** | 🟡 Can add static methods | ✅ Rich factory methods | ✅ Rich factory methods |
| **Generic Support** | ✅ Perfect (FnMut trait) | 🟡 Medium (needs extra abstraction) | ✅ **Perfect (Consumer trait)** |
| **Interior Mutability** | ⚠️ SharedConsumer must use Mutex | ⚠️ SharedConsumer must use Mutex | ✅ **Three ways (none/Mutex/RefCell)** |
| **Code Conciseness** | ✅ Minimalist | 🟡 Medium | 🟡 Slightly complex |
| **Learning Curve** | ✅ Lowest | 🟡 Medium | 🟡 Slightly higher |
| **Maintenance Cost** | 🟡 Medium (two APIs) | 🟡 Medium (code duplication) | ✅ **Low (clear architecture)** |
| **Standard Library Consistency** | 🟡 Medium | 🟡 Medium | ✅ **Perfect** ✨ |

### Usage Scenario Comparison

| Scenario | Approach 1 | Approach 2 | Approach 3 |
|:---|:---|:---|:---|
| **Rapid prototyping** | ✅ Best | 🟡 Okay | 🟡 Okay |
| **Simple operation composition** | ✅ Best | 🟡 Okay | 🟡 Okay |
| **Complex method chains** | ❌ Not suitable | ✅ Suitable | ✅ **Best** |
| **Need metadata/debugging** | ❌ Not supported | ✅ Supported | ✅ **Best** |
| **Multi-threaded sharing** | 🟡 SharedConsumer (with locks) | 🟡 SharedConsumer (with locks) | ✅ **ArcConsumer (clear)** |
| **Single-threaded reuse** | ❌ Not supported | ❌ Not supported | ✅ **RcConsumer (lock-free)** |
| **Library development** | 🟡 Okay | ✅ Suitable | ✅ **Best** |
| **Large projects** | 🟡 Okay | ✅ Suitable | ✅ **Best** |
| **Long-term maintenance** | 🟡 Medium | 🟡 Medium | ✅ **Best** |

### Key Differences Between Consumer and Predicate

| Difference | Predicate | Consumer |
|:---|:---|:---|
| **Function Signature** | `Fn(&T) -> bool` | `FnMut(&mut T)` |
| **Mutability** | Immutable (Fn) | Mutable (FnMut) |
| **Shared Ownership** | Arc can share directly | Arc must use with Mutex/RefCell |
| **Composition** | `and`/`or`/`not` (logical ops) | `and_then` (sequential execution) |
| **Return Value** | Has return value (bool) | No return value (side effects) |
| **Concurrency Difficulty** | Low (no mutability) | High (needs interior mutability) |

---

## Conclusion

### Consumer's Special Challenges

Compared to Predicate, Consumer implementation faces additional challenges:

1. **Mutability requirement**: Consumer needs `FnMut`, meaning must handle interior mutability
2. **Sharing difficulty**: Due to mutability, shared ownership must use `Mutex` (multi-threaded) or `RefCell` (single-threaded)
3. **Performance trade-offs**: Need to choose between safety and performance

### Approach Selection Recommendations

For library projects like `prism3-rust-function`:

#### Current Approach (Approach 2)
The current implementation using Approach 2 is a **reasonable middle choice**:
- ✅ Provides method chains and extensibility
- ✅ Has rich factory methods
- ⚠️ But lacks unified trait abstraction
- ⚠️ Needs to maintain two independent implementations

#### Recommended Approach (Approach 3)
If aiming for architectural elegance equal to Predicate, **strongly recommend upgrading to Approach 3**:
- ✅ Unified `Consumer` trait interface
- ✅ Three clear implementations covering all scenarios
- ✅ `RcConsumer` provides single-threaded lock-free sharing (missing in Approach 2)
- ✅ Type names with clear semantics (`BoxConsumer`/`ArcConsumer`/`RcConsumer`)
- ✅ Consistent design with `Predicate`, reducing learning curve

#### Implementation Path

If deciding to upgrade to Approach 3, recommend progressive migration:

1. **Step 1**: Keep current `Consumer` structure, rename it to `BoxConsumer`
2. **Step 2**: Add `ArcConsumer` and `RcConsumer` implementations
3. **Step 3**: Introduce unified `Consumer` trait
4. **Step 4**: Implement trait and composition methods for all types
5. **Step 5**: Update documentation and examples

This maintains backward compatibility while progressively introducing new architecture.

### Final Recommendation

For Consumer implementation:

- **Rapid development/prototype projects**: Choose Approach 1 or keep current Approach 2
- **Long-term maintained library projects**: **Strongly recommend Approach 3**, for the following reasons:
  - Provides clearest architecture and semantics
  - Complete coverage of three ownership models (especially RcConsumer)
  - Consistent design with Predicate
  - Lowest long-term maintenance cost
  - Provides most flexible and powerful API for users

Although Approach 3 has higher implementation cost, the structural advantages and elegant API design it brings are completely worth the investment, especially for foundational library projects like `prism3-rust-function`.

