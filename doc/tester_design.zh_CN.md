# Tester 设计文档

## 概述

Tester 是一个函数式抽象，用于测试状态或条件是否成立。它不接受参数，返回布尔值，表示某个状态或条件的测试结果。

## 核心语义

### Tester 的定义

```rust
/// 测试器 - 测试状态或条件是否成立
///
/// Tester 封装了"测试某个状态或条件"的逻辑，不接受参数，返回布尔值。
/// 它通过闭包捕获需要测试的上下文信息。
pub trait Tester {
    /// 执行测试，返回测试结果
    ///
    /// # Returns
    ///
    /// 如果条件成立返回 `true`，否则返回 `false`
    fn test(&self) -> bool;
}
```

### 核心特征

- **无输入参数**：通过闭包捕获上下文
- **返回布尔值**：表示测试结果
- **使用 `&self`**：不修改自身状态，只读取外部状态
- **可重复调用**：同一个 Tester 可以多次调用 `test()`

### Tester 与其他函数式抽象的对比

| 抽象 | 输入 | 输出 | self 签名 | 核心语义 | 闭包签名 |
|------|------|------|----------|---------|---------|
| **Tester** | 无 | `bool` | `&self` | 测试状态/条件 | `Fn() -> bool` |
| **Predicate** | `&T` | `bool` | `&self` | 判断值 | `Fn(&T) -> bool` |
| **Supplier** | 无 | `T` | `&mut self` | 生成值 | `FnMut() -> T` |
| **Consumer** | `T` | 无 | `&mut self` | 消费值 | `FnMut(T)` |

**关键洞察**：
- **Tester 与 Predicate 类似**：都是"判断/测试"类抽象，使用 `&self`，不修改自身状态
- **Tester 与 Supplier 不同**：虽然都不接受输入，但 Supplier 可能生成不同的值，需要 `&mut self`；而 Tester 只是读取状态判断条件，使用 `&self`
- **状态管理是调用方的职责**：Tester 只负责"判断"，不负责"管理状态"

### Tester 的主要用途

Tester 类型的核心价值在于：

1. **封装条件判断逻辑**
   ```rust
   let ready = Arc::new(AtomicBool::new(false));
   let ready_clone = Arc::clone(&ready);
   let tester = BoxTester::new(move || {
       ready_clone.load(Ordering::Acquire)
   });
   ```

2. **支持依赖注入**
   ```rust
   struct Executor {
       precondition: BoxTester,
   }

   impl Executor {
       fn execute(&self) {
           if self.precondition.test() {
               // 执行任务
           }
       }
   }
   ```

3. **抽象重试逻辑**
   ```rust
   fn wait_until(tester: &dyn Tester, timeout: Duration) -> bool {
       let start = Instant::now();
       while !tester.test() {
           if start.elapsed() > timeout {
               return false;
           }
           thread::sleep(Duration::from_millis(100));
       }
       true
   }
   ```

4. **组合条件判断**
   ```rust
   let check1 = BoxTester::new(|| database_alive());
   let check2 = BoxTester::new(|| cache_ready());
   let combined = check1.and(check2);
   ```

## 核心设计决策

### 1. self 的可变性：使用 `&self`

**核心问题**：Tester 应该使用 `&self` 还是 `&mut self`？

**答案**：**应该使用 `&self`**

#### 设计哲学

Tester 的职责是"测试判断"，不是"状态管理"：

```rust
// ✅ 正确理解：Tester 只负责判断
pub trait Tester {
    fn test(&self) -> bool;  // 只读状态，不修改自身
}

// ✅ 状态管理是调用方的职责
let count = Arc::new(AtomicUsize::new(0));
let count_clone = Arc::clone(&count);
let tester = BoxTester::new(move || {
    count_clone.load(Ordering::Relaxed) < 3  // 只读状态
});

// 调用方根据测试结果管理状态
loop {
    if !tester.test() {
        break;
    }
    if execute_task().is_ok() {
        count.fetch_add(1, Ordering::Relaxed);  // 调用方更新状态
    }
}
```

#### 与 Predicate 的一致性

| 特性 | Predicate | Tester |
|------|-----------|--------|
| **输入** | 接受参数 `&T` | 不接受参数 |
| **输出** | `bool` | `bool` |
| **self 签名** | `&self` | `&self` |
| **核心语义** | 判断一个值 | 测试一个状态 |
| **修改自身？** | ❌ 否 | ❌ 否 |
| **典型场景** | 过滤、验证 | 状态检查、条件等待 |

**关键洞察**：Predicate 和 Tester 都是"判断/测试"类抽象，语义上都不应该修改自身状态。

#### 实际场景分析

让我们看看典型场景中状态应该由谁管理：

**场景 1：重试限制**
```rust
// ❌ 错误：Tester 管理状态
let mut count = 0;
let mut tester = BoxTester::new(move || {
    count += 1;  // 问题：test() 的副作用
    count <= 3
});

// ✅ 正确：调用方管理状态
let max_attempts = 3;
let mut attempts = 0;
let tester = BoxTester::new(move || attempts < max_attempts);

while tester.test() {
    if execute_task().is_ok() {
        break;
    }
    attempts += 1;  // 调用方控制何时计数
}
```

**场景 2：缓存策略**
```rust
// ❌ 错误：Tester 管理缓存
let mut cached = None;
let mut tester = BoxTester::new(move || {
    if cached.is_none() {
        cached = Some(expensive_check());  // 问题：缓存策略固定
    }
    cached.unwrap()
});

// ✅ 正确：专门的结构体管理缓存
struct CachedChecker {
    cached: Option<bool>,
    last_check: Instant,
    ttl: Duration,
}

impl CachedChecker {
    fn check(&mut self) -> bool {
        if self.should_refresh() {
            self.cached = Some(expensive_check());
            self.last_check = Instant::now();
        }
        self.cached.unwrap()
    }
}

// Tester 只是接口
let checker = Arc::new(Mutex::new(CachedChecker::new()));
let checker_clone = Arc::clone(&checker);
let tester = BoxTester::new(move || {
    checker_clone.lock().unwrap().check()
});
```

**场景 3：条件等待（典型场景）**
```rust
// ✅ Tester 只负责判断条件
let ready = Arc::new(AtomicBool::new(false));
let ready_clone = Arc::clone(&ready);
let tester = BoxTester::new(move || {
    ready_clone.load(Ordering::Acquire)
});

// 调用方控制等待策略
let timeout = Duration::from_secs(30);
let start = Instant::now();
while !tester.test() {
    if start.elapsed() > timeout {
        return Err(TimeoutError);
    }
    thread::sleep(Duration::from_millis(100));
}
```

**场景 4：DoubleCheckedLockExecutor**
```rust
// ✅ 使用 &self 语义正确
struct DoubleCheckedLockExecutor {
    tester: BoxTester,
}

impl DoubleCheckedLockExecutor {
    pub fn execute<T, F>(&self, mutex: &Mutex<T>, task: F)
    where F: FnOnce(&mut T) -> Result<(), Error>
    {
        // 第一次检查（无锁）
        if !self.tester.test() {
            return Err(PreconditionFailed);
        }

        // 获取锁
        let mut guard = mutex.lock().unwrap();

        // 第二次检查（有锁）
        if !self.tester.test() {
            return Err(PreconditionFailed);
        }

        // 执行任务
        task(&mut guard)
    }
}
```

#### 最终结论

**使用 `&self` 的理由**：

1. **职责清晰**：Tester 只负责"判断"，状态管理是调用方的职责
2. **与 Predicate 一致**：两者都是判断类抽象，都使用 `&self`
3. **语义正确**：包含 Tester 的结构体方法可以是 `&self`（如 `Executor::execute(&self)`）
4. **可重复调用**：同一个 Tester 可以多次调用而不产生副作用
5. **与 Java 一致**：对应 Java 的 `BooleanSupplier`（`Fn() -> bool`）
6. **性能优势**：使用 `Fn()` 不需要 `Mutex`/`RefCell`，可以并发调用

**不使用 `&mut self` 的理由**：

1. **违背单一职责**：让 Tester 既判断又管理状态
2. **隐式副作用**：调用 `test()` 就改变状态，调用方无法控制
3. **难以组合**：无法在不可变上下文中使用
4. **限制并发**：无法在多线程中共享调用

### 2. TesterOnce 的必要性

**关键问题**：是否需要 `TesterOnce` trait？

```rust
pub trait TesterOnce {
    fn test(self) -> bool;  // 消费 self
}
```

#### 可能的使用场景

1. **一次性资源检查**：检查资源并消耗它
2. **延迟的布尔计算**：延迟计算一个昂贵的布尔值

```rust
// 场景示例
let resource = ExpensiveResource::new();
let once_tester = BoxTesterOnce::new(move || {
    resource.validate()  // 消费 resource
});

// 只能调用一次
let result = once_tester.test();
```

#### 分析

**问题**：
1. **使用场景罕见**：大多数测试需要多次调用
2. **替代方案更好**：
   ```rust
   // 不用 TesterOnce，直接用闭包
   let check = || expensive_resource.validate();
   let result = check();

   // 或者直接调用
   let result = expensive_resource.validate();
   ```
3. **与典型用途不符**：Tester 的典型场景（条件等待、重试）都需要多次调用

#### 结论

**不实现 TesterOnce**

**理由**：
1. ❌ 使用场景极少
2. ❌ 直接用闭包或函数调用更简单
3. ❌ 与 Tester 的核心用途（可重复测试）不符
4. ✅ 保持 API 简洁

## 三种实现方案对比

### 方案一：类型别名 + 静态组合方法

使用类型别名定义 Tester 类型，并通过静态工具类提供辅助方法。

```rust
// 类型别名定义
pub type Tester = Box<dyn Fn() -> bool>;

// 静态工具类提供辅助方法
pub struct Testers;

impl Testers {
    pub fn from<F>(func: F) -> Tester
    where
        F: Fn() -> bool + 'static,
    {
        Box::new(func)
    }

    pub fn and(t1: Tester, t2: Tester) -> Tester {
        Box::new(move || t1() && t2())
    }

    pub fn or(t1: Tester, t2: Tester) -> Tester {
        Box::new(move || t1() || t2())
    }

    pub fn not(t: Tester) -> Tester {
        Box::new(move || !t())
    }
}

// 使用示例
let test1 = Testers::from(|| check_condition());
let test2 = Testers::from(|| another_check());
let combined = Testers::and(test1, test2);
```

**优点**：
- ✅ 简单直接，易于理解
- ✅ 与闭包完全兼容
- ✅ 无运行时开销

**缺点**：
- ❌ 组合方法消费原 Tester，不能链式调用
- ❌ API 不够流畅
- ❌ 难以扩展到 Rc/Arc 版本

**评估**：适合简单场景，但不够灵活。

---

### 方案二：纯 Trait（无具体实现）

只提供 trait 定义，用户自己实现或直接使用闭包。

```rust
pub trait Tester {
    fn test(&self) -> bool;
}

// 为闭包实现 Tester
impl<F> Tester for F
where
    F: Fn() -> bool,
{
    fn test(&self) -> bool {
        self()
    }
}

// 使用示例
fn wait_until(tester: &dyn Tester) {
    while !tester.test() {
        thread::sleep(Duration::from_millis(100));
    }
}

// 直接传闭包
wait_until(&|| check_condition());

// 或自定义实现
struct MyTester;
impl Tester for MyTester {
    fn test(&self) -> bool {
        check_condition()
    }
}
```

**优点**：
- ✅ 最大灵活性
- ✅ 与闭包完全兼容
- ✅ 无额外抽象

**缺点**：
- ❌ 没有便利的包装类型（Box/Rc/Arc）
- ❌ 没有组合方法
- ❌ 用户需要自己处理所有权

**评估**：过于简单，缺少常用工具。

---

### 方案三：Trait 抽象 + 多种实现（推荐）⭐

提供 trait 定义和多种具体实现（BoxTester, RcTester, ArcTester），每种实现都提供组合方法。

```rust
// Trait 定义
pub trait Tester {
    fn test(&self) -> bool;
}

// Box 实现（单一所有权）
pub struct BoxTester {
    func: Box<dyn Fn() -> bool>,
}

impl BoxTester {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        Self {
            func: Box::new(func),
        }
    }

    pub fn and(self, other: BoxTester) -> BoxTester {
        let func1 = self.func;
        let func2 = other.func;
        BoxTester::new(move || func1() && func2())
    }

    pub fn or(self, other: BoxTester) -> BoxTester {
        let func1 = self.func;
        let func2 = other.func;
        BoxTester::new(move || func1() || func2())
    }

    pub fn not(self) -> BoxTester {
        let func = self.func;
        BoxTester::new(move || !func())
    }

    // 转换方法
    pub fn into_rc(self) -> RcTester { /* ... */ }
    pub fn into_arc(self) -> ArcTester { /* ... */ }
}

impl Tester for BoxTester {
    fn test(&self) -> bool {
        (self.func)()
    }
}

// Rc 实现（单线程共享所有权）
pub struct RcTester {
    func: Rc<dyn Fn() -> bool>,
}

impl RcTester {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        Self {
            func: Rc::new(func),
        }
    }

    // 类似的 and, or, not 方法
    // 转换方法
}

impl Tester for RcTester {
    fn test(&self) -> bool {
        (self.func)()
    }
}

impl Clone for RcTester {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func),
        }
    }
}

// Arc 实现（多线程共享所有权）
pub struct ArcTester {
    func: Arc<dyn Fn() -> bool + Send + Sync>,
}

impl ArcTester {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Self {
            func: Arc::new(func),
        }
    }

    // 类似的 and, or, not 方法
    // 转换方法
}

impl Tester for ArcTester {
    fn test(&self) -> bool {
        (self.func)()
    }
}

impl Clone for ArcTester {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

// 为闭包实现 Tester
impl<F> Tester for F
where
    F: Fn() -> bool,
{
    fn test(&self) -> bool {
        self()
    }
}
```

**使用示例**：

```rust
// 单一所有权
let test1 = BoxTester::new(|| check_db());
let test2 = BoxTester::new(|| check_cache());
let combined = test1.and(test2);

if combined.test() {
    execute_task();
}

// 单线程共享
let test = RcTester::new(|| system_ready());
let test_clone = test.clone();

thread_1(test);
thread_2(test_clone);

// 多线程共享
let test = ArcTester::new(|| atomic_flag.load(Ordering::Acquire));
let test_clone = test.clone();

thread::spawn(move || {
    while !test_clone.test() {
        thread::sleep(Duration::from_millis(100));
    }
});

// 直接使用闭包（通过 Tester trait）
fn wait_until(tester: &dyn Tester) {
    while !tester.test() {
        thread::sleep(Duration::from_millis(100));
    }
}

wait_until(&|| check_condition());
```

**优点**：
- ✅ 提供多种所有权模型（Box/Rc/Arc）
- ✅ 每种实现都有便利的组合方法
- ✅ 支持所有权转换
- ✅ 兼容闭包（通过 trait）
- ✅ 类型安全的并发控制（ArcTester 要求 Send + Sync）
- ✅ API 流畅，链式调用

**缺点**：
- ⚠️ 实现代码较多
- ⚠️ 三个类型可能让用户选择困难

**评估**：最完整的方案，适合生产环境。

---

### 方案对比总结

| 特性 | 方案一：类型别名 | 方案二：纯 Trait | 方案三：Trait + 实现 ⭐ |
|------|----------------|----------------|---------------------|
| **易用性** | 中 | 低 | 高 |
| **所有权模型** | Box 只 | 无 | Box/Rc/Arc |
| **组合方法** | 静态方法 | 无 | 实例方法 |
| **链式调用** | ❌ | ❌ | ✅ |
| **闭包兼容** | ✅ | ✅ | ✅ |
| **类型安全** | ✅ | ✅ | ✅ |
| **并发控制** | 无 | 无 | ✅ (ArcTester) |
| **实现复杂度** | 低 | 低 | 高 |
| **推荐度** | ⭐⭐ | ⭐ | ⭐⭐⭐ |

**最终推荐**：**方案三（Trait 抽象 + 多种实现）**

**理由**：
1. 提供了完整的所有权模型支持
2. 每种实现都有便利的 API
3. 类型安全的并发控制
4. 与其他函数式抽象（Predicate, Consumer, Supplier）保持一致

## 设计原则总结

### 核心原则

1. **单一职责**：Tester 只负责"判断条件"，不负责"管理状态"
2. **不可变性**：使用 `&self`，不修改自身状态
3. **可重复性**：同一个 Tester 可以多次调用
4. **与 Predicate 一致**：都是判断类抽象，都使用 `&self`

### 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| **self 可变性** | `&self` | 职责清晰，与 Predicate 一致 |
| **TesterOnce** | 不实现 | 使用场景极少，直接用闭包更好 |
| **实现方案** | Trait + 多种实现 | 完整的所有权模型，便利的 API |
| **所有权模型** | Box/Rc/Arc | 覆盖所有使用场景 |
| **闭包签名** | `Fn() -> bool` | 不需要修改状态 |

### Tester 与其他函数式抽象的关系

```
判断类抽象（使用 &self）：
├── Predicate<T>: Fn(&T) -> bool   // 判断一个值
└── Tester:       Fn() -> bool     // 测试一个状态

生成/消费类抽象（使用 &mut self）：
├── Supplier<T>:     FnMut() -> T     // 生成值
├── Consumer<T>:     FnMut(T)         // 消费值
└── Transformer<T,R>: FnMut(T) -> R   // 转换值
```

### 当前实现状态

| 组件 | 状态 | 说明 |
|------|------|------|
| **Tester trait** | ✅ 已实现 | 核心 trait，使用 `&self` |
| **BoxTester** | ✅ 已实现 | 单一所有权实现 |
| **RcTester** | ✅ 已实现 | 单线程共享所有权 |
| **ArcTester** | ✅ 已实现 | 多线程共享所有权 |
| **闭包实现** | ✅ 已实现 | `impl<F: Fn() -> bool> Tester for F` |
| **组合方法** | ✅ 已实现 | and, or, not |
| **转换方法** | ✅ 已实现 | into_box, into_rc, into_arc |

## 使用场景示例

### 1. 健康检查

```rust
use prism3_rust_function::tester::{BoxTester, Tester};

struct HealthChecker {
    database: Arc<Database>,
    cache: Arc<Cache>,
}

impl HealthChecker {
    fn create_health_tester(&self) -> BoxTester {
        let db = Arc::clone(&self.database);
        let cache = Arc::clone(&self.cache);

        BoxTester::new(move || {
            db.is_alive() && cache.is_connected()
        })
    }
}

// 使用
let checker = HealthChecker::new();
let health_test = checker.create_health_tester();

if health_test.test() {
    println!("System is healthy");
} else {
    println!("System is unhealthy");
}
```

### 2. 条件等待

```rust
use prism3_rust_function::tester::{ArcTester, Tester};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};

fn wait_until(tester: &dyn Tester, timeout: Duration) -> bool {
    let start = Instant::now();
    while !tester.test() {
        if start.elapsed() > timeout {
            return false;
        }
        thread::sleep(Duration::from_millis(100));
    }
    true
}

// 使用
let ready = Arc::new(AtomicBool::new(false));
let ready_clone = Arc::clone(&ready);
let tester = ArcTester::new(move || {
    ready_clone.load(Ordering::Acquire)
});

// 另一个线程设置标志
let ready_clone2 = Arc::clone(&ready);
thread::spawn(move || {
    thread::sleep(Duration::from_secs(2));
    ready_clone2.store(true, Ordering::Release);
});

// 等待条件成立
if wait_until(&tester, Duration::from_secs(5)) {
    println!("Condition met!");
} else {
    println!("Timeout!");
}
```

### 3. 重试限制

```rust
use prism3_rust_function::tester::{BoxTester, Tester};

fn retry_with_limit<F>(task: F, max_attempts: usize) -> Result<(), Error>
where
    F: Fn() -> Result<(), Error>,
{
    let mut attempts = 0;
    let should_retry = BoxTester::new(move || attempts < max_attempts);

    loop {
        match task() {
            Ok(_) => return Ok(()),
            Err(e) if should_retry.test() => {
                attempts += 1;
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}

// 使用
retry_with_limit(|| {
    send_request()
}, 3)?;
```

### 4. 缓存测试结果

```rust
use prism3_rust_function::tester::{BoxTester, Tester};
use std::time::{Duration, Instant};

struct CachedChecker {
    cached_result: Option<bool>,
    last_check_time: Instant,
    cache_duration: Duration,
}

impl CachedChecker {
    fn new(cache_duration: Duration) -> Self {
        Self {
            cached_result: None,
            last_check_time: Instant::now() - cache_duration,
            cache_duration,
        }
    }

    fn check(&mut self) -> bool {
        let now = Instant::now();
        if self.cached_result.is_none()
            || now.duration_since(self.last_check_time) > self.cache_duration {
            // 缓存失效，重新检查
            self.cached_result = Some(expensive_health_check());
            self.last_check_time = now;
        }
        self.cached_result.unwrap()
    }
}

// 使用 Tester 封装
let checker = Arc::new(Mutex::new(CachedChecker::new(Duration::from_secs(60))));
let checker_clone = Arc::clone(&checker);
let tester = BoxTester::new(move || {
    checker_clone.lock().unwrap().check()
});

// 多次调用 tester，缓存策略由 CachedChecker 管理
for _ in 0..10 {
    if tester.test() {
        println!("Health check passed");
    }
    thread::sleep(Duration::from_secs(5));
}
```

### 5. 逻辑组合

```rust
use prism3_rust_function::tester::{BoxTester, Tester};

let db_alive = BoxTester::new(|| check_database());
let cache_ready = BoxTester::new(|| check_cache());
let disk_ok = BoxTester::new(|| check_disk_space());

// AND 组合
let all_healthy = db_alive
    .and(cache_ready)
    .and(disk_ok);

if all_healthy.test() {
    println!("All systems operational");
}

// OR 组合
let db_check = BoxTester::new(|| check_primary_db());
let backup_check = BoxTester::new(|| check_backup_db());
let any_db_alive = db_check.or(backup_check);

// NOT 组合
let maintenance_mode = BoxTester::new(|| is_maintenance());
let not_maintenance = maintenance_mode.not();

// 复杂组合
let can_serve = any_db_alive.and(not_maintenance);
```

### 6. 多线程共享检查

```rust
use prism3_rust_function::tester::{ArcTester, Tester};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

let shutdown = Arc::new(AtomicBool::new(false));
let shutdown_clone = Arc::clone(&shutdown);
let should_continue = ArcTester::new(move || {
    !shutdown_clone.load(Ordering::Acquire)
});

// 多个工作线程共享同一个 tester
for i in 0..4 {
    let tester = should_continue.clone();
    thread::spawn(move || {
        while tester.test() {
            // 执行工作
            println!("Worker {} running", i);
            thread::sleep(Duration::from_millis(500));
        }
        println!("Worker {} shutdown", i);
    });
}

// 主线程控制关闭
thread::sleep(Duration::from_secs(3));
shutdown.store(true, Ordering::Release);
```

### 7. 前置条件检查

```rust
use prism3_rust_function::tester::{BoxTester, Tester};

struct Executor {
    precondition: BoxTester,
}

impl Executor {
    fn new(precondition: BoxTester) -> Self {
        Self { precondition }
    }

    fn execute<F>(&self, task: F) -> Result<(), Error>
    where
        F: FnOnce() -> Result<(), Error>,
    {
        if !self.precondition.test() {
            return Err(Error::PreconditionFailed);
        }
        task()
    }
}

// 使用
let authenticated = Arc::new(AtomicBool::new(false));
let auth_clone = Arc::clone(&authenticated);
let precondition = BoxTester::new(move || {
    auth_clone.load(Ordering::Acquire)
});

let executor = Executor::new(precondition);

// 尝试执行（失败，因为未认证）
executor.execute(|| {
    println!("Executing sensitive operation");
    Ok(())
})?;  // 返回 PreconditionFailed

// 认证后执行
authenticated.store(true, Ordering::Release);
executor.execute(|| {
    println!("Executing sensitive operation");
    Ok(())
})?;  // 成功
```

### 8. 状态变化检测

```rust
use prism3_rust_function::tester::BoxTester;

struct ChangeDetector {
    last_value: i32,
    source: Arc<AtomicI32>,
}

impl ChangeDetector {
    fn new(source: Arc<AtomicI32>) -> Self {
        let initial = source.load(Ordering::Relaxed);
        Self {
            last_value: initial,
            source,
        }
    }

    fn has_changed(&mut self) -> bool {
        let current = self.source.load(Ordering::Relaxed);
        if current != self.last_value {
            self.last_value = current;
            true
        } else {
            false
        }
    }
}

// 使用
let value = Arc::new(AtomicI32::new(0));
let detector = Arc::new(Mutex::new(ChangeDetector::new(Arc::clone(&value))));

let detector_clone = Arc::clone(&detector);
let change_tester = BoxTester::new(move || {
    detector_clone.lock().unwrap().has_changed()
});

// 检测变化
loop {
    if change_tester.test() {
        println!("Value changed!");
        handle_change();
    }
    thread::sleep(Duration::from_millis(100));
}
```

## 实现注意事项

### 1. 线程安全

```rust
// BoxTester: 不能跨线程
let test = BoxTester::new(|| check());
// thread::spawn(move || test.test());  // 编译错误：BoxTester 不是 Send

// ArcTester: 可以跨线程
let test = ArcTester::new(|| check());
thread::spawn(move || test.test());  // ✅ 编译通过
```

### 2. 所有权转换

```rust
// Box -> Rc
let box_test = BoxTester::new(|| check());
let rc_test = box_test.into_rc();

// Rc -> Arc（需要闭包满足 Send + Sync）
let rc_test = RcTester::new(|| check());
// let arc_test = rc_test.into_arc();  // 可能 panic，取决于闭包

// 安全的做法：直接创建 Arc
let arc_test = ArcTester::new(|| check());
```

### 3. 错误处理

```rust
// Tester 返回 bool，不返回 Result
// 错误处理应该在闭包内部完成

let tester = BoxTester::new(|| {
    match risky_check() {
        Ok(result) => result,
        Err(e) => {
            log::error!("Check failed: {}", e);
            false  // 将错误转换为 false
        }
    }
});
```

### 4. 性能考虑

```rust
// ✅ 好：轻量级检查
let tester = BoxTester::new(|| {
    flag.load(Ordering::Relaxed)
});

// ⚠️ 注意：昂贵的检查应该有缓存策略
let tester = BoxTester::new(|| {
    expensive_network_check()  // 每次调用都会执行
});

// ✅ 更好：使用缓存
let checker = Arc::new(Mutex::new(CachedChecker::new(Duration::from_secs(60))));
let checker_clone = Arc::clone(&checker);
let tester = BoxTester::new(move || {
    checker_clone.lock().unwrap().check()
});
```

## 与 Java 实现对比

### Java 版本

```java
// Java 中的 BooleanSupplier
@FunctionalInterface
public interface BooleanSupplier {
    boolean getAsBoolean();
}

// 使用示例
BooleanSupplier tester = () -> database.isAlive();
if (tester.getAsBoolean()) {
    // ...
}
```

### Rust 版本

```rust
// Rust 中的 Tester
pub trait Tester {
    fn test(&self) -> bool;
}

// 使用示例
let tester = BoxTester::new(|| database.is_alive());
if tester.test() {
    // ...
}
```

### 主要差异

| 特性 | Java BooleanSupplier | Rust Tester |
|------|---------------------|-------------|
| **方法名** | `getAsBoolean()` | `test()` |
| **所有权模型** | GC 管理 | Box/Rc/Arc 显式管理 |
| **线程安全** | 依赖对象 | ArcTester 明确要求 Send + Sync |
| **组合方法** | 无（需要工具类）| 有（and, or, not）|
| **类型安全** | 运行时检查 | 编译时检查 |

**Rust 的优势**：
- 更强的类型安全（编译时保证线程安全）
- 显式的所有权管理
- 零成本抽象
- 内置组合方法

## 参考资料

- [Rust Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [Fn, FnMut, FnOnce](https://doc.rust-lang.org/std/ops/trait.Fn.html)
- [Java BooleanSupplier](https://docs.oracle.com/javase/8/docs/api/java/util/function/BooleanSupplier.html)
- [Predicate 设计文档](./predicate_design.zh_CN.md)
- [Supplier 设计文档](./supplier_design.zh_CN.md)

- [Supplier 设计文档](./supplier_design.zh_CN.md)
