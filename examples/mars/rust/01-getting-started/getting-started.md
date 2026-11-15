# Getting started with Project Mars

**Project Mars** is a Rust-first interactive notebook. A `.mars` notebook is a JSON
document made of **markdown**, **code**, and **raw** cells. Code cells are
written in Rust and share state as they run from top to bottom.

This notebook is a tour of the basics. It covers:

- notebook cells, Markdown, and outputs
- variables, types, operators, and strings
- mutability, shadowing, constants, and statics
- control flow, loops, functions, and closures
- tuples, arrays, slices, vectors, iterators, and hash maps
- structs, enums, traits, `Option`, and `Result`
- modules, macros, attributes, references, lifetimes, generics, and type aliases
- summary statistics and a least-squares regression

Run the cells in order. Each code cell can use every binding created before it.

## How a notebook works

| Cell type | Purpose |
| --------- | ------- |
| `markdown` | Explanations, headings, lists, tables, links, and code snippets |
| `code` | Rust source that is compiled and executed |
| `raw` | Text that is preserved without Markdown or Rust processing |

When a code cell runs:

1. Project Mars replays all earlier successful code cells, so their bindings are available.
2. The current cell is compiled and executed.
3. Its final expression is displayed automatically.
4. Standard output and errors are captured.
5. The execution count and outputs are stored in the `.mars` file.

Because earlier cells are replayed, side effects such as printing, file writes,
or random numbers can happen again. Keep side-effecting cells idempotent, or
run them once and store the result.

> Tip: the final expression of a code cell is shown without a `println!`.
> End a cell with a semicolon when you do not want output.

### Markdown essentials

Markdown cells support **bold**, *italic*, ~~strikethrough~~, `inline code`,
[links](https://github.com/hieudoanm/mars), lists, tables, block quotes, and
fenced code blocks.

- unordered list item
- another item
  - nested item

1. ordered item
2. another item

```bash
mars notebook run getting-started.mars --report
```

```text
A raw cell is stored and exported verbatim.
It is useful for generated text or content that should not be parsed.
```

## Your first code cell

The next cell sets up a tiny `Report` helper. It wraps formatted text and
implements `Debug`, so multi-line output prints exactly as written instead of
being wrapped in quotes.

The helper is defined once and stays available to every later cell.

```rust
use std::fmt;

struct Report(String);

impl fmt::Debug for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn report(text: impl Into<String>) -> Report {
    Report(text.into())
}
```

## Variables and types

Rust infers types, but you can annotate them. Names use `snake_case`, and
bindings are immutable unless declared with `mut`.

```rust
let language = "Rust";
let year: u32 = 2024;
let pi = 3.141_592_653_589_793;
let is_fun = true;
let initial = 'M';

report(format!(
    "language = {language}\nyear = {year}\npi = {pi}\nis_fun = {is_fun}\ninitial = {initial}"
))
```

```text
language = Rust
year = 2024
pi = 3.141592653589793
is_fun = true
initial = M
```

## Arithmetic and operators

Rust distinguishes integer division (`/`) from floating-point division and uses
`%` for the remainder.

```rust
let sum = 7 + 5;
let difference = 7 - 5;
let product = 7 * 5;
let quotient = 7 / 5;
let remainder = 7 % 5;
let float_quotient = 7.0 / 5.0;
let power = 2_i32.pow(10);

(sum, difference, product, quotient, remainder, float_quotient, power)
```

```text
(12, 2, 35, 1, 2, 1.4, 1024)
```

## Mutability and shadowing

Use `mut` when a binding changes. Shadowing creates a new binding with the same
name, which is handy for changing a value's type.

```rust
let mut counter = 0;
counter += 1;
counter *= 10;
counter -= 5;

let answer = 21;
let answer = answer * 2;
let answer: f64 = answer as f64 + 0.5;

report(format!("counter = {counter}\nanswer = {answer}"))
```

```text
counter = 5
answer = 42.5
```

## Strings

`&str` is a borrowed string slice, while `String` owns its contents. Both can
be formatted with `format!`, and many string methods return new values.

```rust
let name = String::from("Project Mars");
let greeting = format!("Hello, {name}!");
let shout = greeting.to_uppercase();
let words: Vec<&str> = greeting.split(' ').collect();
let character_count = greeting.chars().count();
let replaced = greeting.replace("Project Mars", "notebooks");

report(format!(
    "greeting = {greeting}\nshout = {shout}\nwords = {words:?}\ncharacters = {character_count}\nreplaced = {replaced}"
))
```

```text
greeting = Hello, Project Mars!
shout = HELLO, PROJECT MARS!
words = ["Hello,", "Project", "Mars!"]
characters = 20
replaced = Hello, notebooks!
```

## Control flow

`if` is an expression, so it can produce a value. `match` compares patterns and
can bind parts of the value.

```rust
let temperature = 21;

let advice = if temperature > 25 {
    "Wear something light"
} else if temperature >= 15 {
    "A jacket is enough"
} else {
    "Bring a coat"
};

let description = match temperature {
    value if value < 0 => "freezing",
    0..=14 => "cold",
    15..=24 => "mild",
    _ => "warm",
};

report(format!("{temperature} C is {description}: {advice}"))
```

```text
21 C is mild: A jacket is enough
```

## Loops

`for` walks an iterator, `while` repeats while a condition holds, `while let`
repeats while a pattern matches, and `loop` repeats until `break`.

```rust
let mut countdown = Vec::new();
for number in (1..=5).rev() {
    countdown.push(number);
}

let mut doubled = 0;
while doubled < 10 {
    doubled += 2;
}

let mut stack = vec![1, 2, 3];
let mut popped = Vec::new();
while let Some(value) = stack.pop() {
    popped.push(value);
}

let mut loop_count = 0;
loop {
    loop_count += 1;
    if loop_count == 3 {
        break;
    }
}

report(format!(
    "countdown = {countdown:?}\ndoubled = {doubled}\npopped = {popped:?}\nloop_count = {loop_count}"
))
```

```text
countdown = [5, 4, 3, 2, 1]
doubled = 10
popped = [3, 2, 1]
loop_count = 3
```

## Functions

Functions declare their parameter and return types. A function's final
expression is its return value when there is no semicolon.

```rust
fn square(value: i32) -> i32 {
    value * value
}

fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

report(format!(
    "square(9) = {}\naverage([2, 4, 6, 8]) = {}",
    square(9),
    average(&[2.0, 4.0, 6.0, 8.0])
))
```

```text
square(9) = 81
average([2, 4, 6, 8]) = 5
```

## Tuples, arrays, and slices

Tuples group values of different types. Arrays have a fixed length. Slices
borrow a contiguous range from an array or vector.

```rust
let point = (3, 4);
let (x, y) = point;
let array = [10, 20, 30, 40];
let slice = &array[1..3];
let array_total: i32 = array.iter().sum();

report(format!(
    "point = {point:?}\nx = {x}, y = {y}\narray = {array:?}\nslice = {slice:?}\ntotal = {array_total}"
))
```

```text
point = (3, 4)
x = 3, y = 4
array = [10, 20, 30, 40]
slice = [20, 30]
total = 100
```

## Vectors

`Vec<T>` is a growable list. You can push values, read them by index, borrow a
range, and inspect the first or last item safely with `Option`.

```rust
let mut primes = vec![2, 3, 5, 7, 11];
primes.push(13);

let first = primes.first().copied();
let last = primes.last().copied();
let middle = &primes[1..4];

report(format!(
    "primes = {primes:?}\nfirst = {first:?}\nlast = {last:?}\nmiddle = {middle:?}"
))
```

```text
primes = [2, 3, 5, 7, 11, 13]
first = Some(2)
last = Some(13)
middle = [3, 5, 7]
```

## Iterators

Iterator chains are the idiomatic way to transform collections. Methods such
as `map`, `filter`, `sum`, `product`, `max`, and `collect` consume or build
iterators without writing manual loops.

```rust
let numbers = vec![1, 2, 3, 4, 5, 6];

let squares: Vec<i32> = numbers.iter().map(|value| value * value).collect();
let evens: Vec<i32> = numbers
    .iter()
    .copied()
    .filter(|value| *value % 2 == 0)
    .collect();
let total: i32 = numbers.iter().sum();
let product: i32 = numbers.iter().product();
let maximum = numbers.iter().max();

report(format!(
    "numbers = {numbers:?}\nsquares = {squares:?}\nevens = {evens:?}\nsum = {total}\nproduct = {product}\nmax = {maximum:?}"
))
```

```text
numbers = [1, 2, 3, 4, 5, 6]
squares = [1, 4, 9, 16, 25, 36]
evens = [2, 4, 6]
sum = 21
product = 720
max = Some(6)
```

## Hash maps

`HashMap` stores key-value pairs and provides fast lookup. The `entry` API
inserts a default value and returns a mutable reference, which makes counting
simple.

```rust
use std::collections::HashMap;

let text = "to be or not to be";
let mut word_counts = HashMap::new();

for word in text.split_whitespace() {
    *word_counts.entry(word).or_insert(0) += 1;
}

let mut counts: Vec<(&str, i32)> = word_counts.into_iter().collect();
counts.sort();

report(format!("{counts:?}"))
```

```text
[("be", 2), ("not", 1), ("or", 1), ("to", 2)]
```

## Structs and methods

A `struct` groups related fields. An `impl` block adds methods, and `&self`
borrows the value without taking ownership.

```rust
#[derive(Debug)]
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

let rectangle = Rectangle {
    width: 3.0,
    height: 4.0,
};

report(format!(
    "rectangle = {rectangle:?}\narea = {}\nperimeter = {}",
    rectangle.area(),
    rectangle.perimeter()
))
```

```text
rectangle = Rectangle { width: 3.0, height: 4.0 }
area = 12
perimeter = 14
```

## Enums and pattern matching

An `enum` represents one of several variants. Variants can carry data, and
`match` handles every case explicitly.

```rust
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

impl Shape {
    fn name(&self) -> &str {
        match self {
            Shape::Circle { .. } => "circle",
            Shape::Rectangle { .. } => "rectangle",
        }
    }

    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
        }
    }
}

let shapes = vec![
    Shape::Circle { radius: 2.0 },
    Shape::Rectangle { width: 3.0, height: 4.0 },
];

let summary: Vec<String> = shapes
    .iter()
    .map(|shape| format!("{} = {:.2}", shape.name(), shape.area()))
    .collect();

report(summary.join("\n"))
```

```text
circle = 12.57
rectangle = 12.00
```

## Option and Result

`Option<T>` models a value that may be missing. `Result<T, E>` models an
operation that may fail. Both can be inspected with `match`, `if let`, `?`, or
combinators such as `map` and `unwrap_or`.

```rust
fn parse_score(text: &str) -> Result<i32, String> {
    text.parse::<i32>()
        .map_err(|_| format!("`{text}` is not a number"))
}

let good = parse_score("42");
let bad = parse_score("forty-two");
let first_even = (1..10).find(|value| value % 2 == 0);
let fallback = bad.clone().unwrap_or(-1);

let parsed = if let Ok(score) = &good {
    format!("parsed {score}")
} else {
    "invalid".to_owned()
};

let category = match &good {
    Ok(score) if *score >= 40 => "pass",
    Ok(_) => "retry",
    Err(_) => "invalid",
};

report(format!(
    "good = {good:?}\nbad = {bad:?}\nfirst_even = {first_even:?}\nfallback = {fallback}\nparsed = {parsed}\ncategory = {category}"
))
```

```text
good = Ok(42)
bad = Err("`forty-two` is not a number")
first_even = Some(2)
fallback = -1
parsed = parsed 42
category = pass
```

## Closures

Closures are anonymous functions that can capture values from their
environment. They are commonly passed to iterator methods.

```rust
let offset = 10;
let add_offset = |value: i32| value + offset;
let values = vec![1, 2, 3, 4];
let adjusted: Vec<i32> = values.iter().map(|value| add_offset(*value)).collect();

report(format!(
    "offset = {offset}\nvalues = {values:?}\nadjusted = {adjusted:?}"
))
```

```text
offset = 10
values = [1, 2, 3, 4]
adjusted = [11, 12, 13, 14]
```

## Constants and statics

`const` values are inlined at compile time, evaluated each time they are
used, and cannot be mutated. `static` items live for the entire program and
may be shared across threads. Both require an explicit type annotation.

```rust
const MAX_POINTS: u32 = 100;
static NEXT_ID: u32 = 1;
let constant = MAX_POINTS;
let static_ref: &'static u32 = &NEXT_ID;

report(format!("MAX_POINTS = {MAX_POINTS}\nNEXT_ID = {NEXT_ID}\nstatic_ref = {static_ref}"))
```

```text
MAX_POINTS = 100
NEXT_ID = 1
static_ref = 1
```

## References and borrowing

A reference (`&T`) borrows a value without taking ownership. `&mut T` grants
exclusive, mutable borrow access. Rust's borrow checker prevents a mutable
reference from coexisting with any other reference to the same data.

```rust
let total = 100;
let reference = &total;
let mut counter = 5;
let mut_ref = &mut counter;
*mut_ref += 1;

report(format!("total = {total}\nreference = {reference}\ncounter = {counter}"))
```

```text
total = 100
reference = 100
counter = 6
```

## Modules

A `mod` groups related items and can be nested. `pub` makes items visible
outside the module, and `use` brings names into the current scope.

```rust
mod math {
    pub fn double(value: i32) -> i32 {
        value * 2
    }
}

report(format!("math::double(21) = {}", math::double(21)))
```

```text
math::double(21) = 42
```

## Macros

Declarative macros (`macro_rules!`) expand at compile time using pattern
matching. They generate code from code, which is why their output can look
very different from the invocation.

```rust
macro_rules! greet {
    ($name:expr) => {
        format!("hello, {name}")
    };
}

report(greet!("Project Mars"))
```

```text
hello, Project Mars
```

## Attributes

Attributes like `#[derive(...)]` attach metadata to items. The compiler uses
them to generate code, enforce rules, or control warnings.

```rust
#[derive(Debug, PartialEq)]
struct Point2D {
    x: i32,
    y: i32,
}

let origin = Point2D { x: 0, y: 0 };
let other = Point2D { x: 0, y: 0 };

report(format!("origin = {origin:?}\norigin == other: {}", origin == other))
```

```text
origin = Point2D { x: 0, y: 0 }
origin == other: true
```

## Lifetimes, generics, and type aliases

Generics let you write functions and types that work for many types.
Lifetimes are a special kind of generic that describe how references relate
to each other. `type` aliases give a fresh name to an existing type.

```rust
type Meters = f64;
fn pick_first<'a>(items: &'a [i32]) -> &'a i32 {
    &items[0]
}
fn identity<T>(value: T) -> T {
    value
}

let distance: Meters = 12.5;
let numbers = vec![7, 14, 21];

report(format!(
    "distance = {distance} m\nidentity(3) = {}\nfirst = {}",
    identity(3),
    pick_first(&numbers)
))
```

```text
distance = 12.5 m
identity(3) = 3
first = 7
```

## Traits

A `trait` defines shared behavior that many types can implement. Trait
objects (`&dyn Trait`) let you work with several different types through one
common interface.

```rust
trait Describe {
    fn describe(&self) -> String;
}

impl Describe for i32 {
    fn describe(&self) -> String {
        format!("integer {self}")
    }
}

let number = 42;

report(number.describe())
```

```text
integer 42
```

## Summary statistics

The next cell computes count, total, mean, minimum, maximum, and median for a
small list of integers. This is the data-analysis example merged into the tour.

```rust
let values = vec![3_i32, 7, 4, 9, 12, 6];
let total: i32 = values.iter().sum();
let mean = total as f64 / values.len() as f64;
let minimum = values.iter().min().copied();
let maximum = values.iter().max().copied();

let mut sorted = values.clone();
sorted.sort();
let median = if sorted.len() % 2 == 0 {
    let middle = sorted.len() / 2;
    (sorted[middle - 1] + sorted[middle]) as f64 / 2.0
} else {
    sorted[sorted.len() / 2] as f64
};

report(format!(
    "values = {values:?}\ncount = {}\ntotal = {total}\nmean = {mean}\nmin = {minimum:?}\nmax = {maximum:?}\nmedian = {median}",
    values.len()
))
```

```text
values = [3, 7, 4, 9, 12, 6]
count = 6
total = 41
mean = 6.833333333333333
min = Some(3)
max = Some(12)
median = 6.5
```

## Least-squares regression

A simple linear regression finds the line `y = slope * x + intercept` that
minimizes squared error. The slope is the covariance of `x` and `y` divided by
the variance of `x`. This is the linear-regression example merged into the tour.

```rust
let x = vec![1.0_f64, 2.0, 3.0, 4.0];
let y = vec![2.0_f64, 4.1, 5.9, 8.2];

let x_mean = x.iter().sum::<f64>() / x.len() as f64;
let y_mean = y.iter().sum::<f64>() / y.len() as f64;

let covariance = x
    .iter()
    .zip(&y)
    .map(|(x, y)| (x - x_mean) * (y - y_mean))
    .sum::<f64>();
let variance = x.iter().map(|x| (x - x_mean).powi(2)).sum::<f64>();

let slope = covariance / variance;
let intercept = y_mean - slope * x_mean;

report(format!(
    "x mean = {x_mean}\ny mean = {y_mean}\nslope = {slope}\nintercept = {intercept}\nprediction at x = 5: {:.2}",
    slope * 5.0 + intercept
))
```

```text
x mean = 2.5
y mean = 5.05
slope = 2.04
intercept = -0.04999999999999982
prediction at x = 5: 10.15
```

## Recap

The final code cell collects the topics from this tour into an aligned summary.

```rust
let topics = vec![
    ("cells", "markdown, code, raw"),
    ("basics", "variables, types, operators"),
    ("flow", "if, match, loops, functions"),
    ("collections", "tuples, arrays, vectors, iterators, maps"),
    ("abstractions", "structs, enums, traits, closures"),
    ("modules", "mod, use, pub"),
    ("macros", "macro_rules!, attributes"),
    ("generics", "lifetimes, generics, type aliases"),
    ("results", "Option and Result"),
    ("analysis", "statistics and regression"),
];

let width = topics.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
let summary: Vec<String> = topics
    .iter()
    .map(|(name, detail)| format!("{name:width$}  {detail}"))
    .collect();

report(summary.join("\n"))
```

```text
cells         markdown, code, raw
basics        variables, types, operators
flow          if, match, loops, functions
collections   tuples, arrays, vectors, iterators, maps
abstractions  structs, enums, traits, closures
modules       mod, use, pub
macros        macro_rules!, attributes
generics      lifetimes, generics, type aliases
results       Option and Result
analysis      statistics and regression
```

## What comes next

You have now seen the core of a Project Mars notebook: Markdown and raw cells, Rust
code cells, persisted bindings, automatic output, and stored results.

This tour also maps onto a Rust syllabus. It covers basic syntax
(statements, expressions, variables, mutability, constants, modules,
macros, and attributes), control flow (`if`, `if let`, `match`, loops),
functions and closures, the type system (primitives, arrays, `Vec`,
`&str`/`String`, tuples, structs, enums, `Option`, `Result`), and traits and
generics (lifetimes, type aliasing, traits, generics). Two topics are
intentionally left out of this tour: procedural macros, which need a separate
`proc-macro` crate, and the Cargo toolchain (`cargo build`, `cargo test`,
`cargo doc`, crates.io), which you run from a terminal rather than a notebook.

Continue with:

| Example | What it adds |
| ------- | ------------ |
| `04-machine-learning/machine-learning.mars` | A longer workflow with data generation, evaluation, and a plotters chart |
| `05-plotting/plotting.mars` | Vector figures rendered as SVG output |

Useful commands:

```bash
mars notebook run getting-started.mars --report   # execute and save outputs
mars notebook run getting-started.mars --no-save  # check without writing
mars notebook export getting-started.mars md      # Markdown export
mars notebook export getting-started.mars html    # HTML export
mars notebook export getting-started.mars pdf     # PDF export
```