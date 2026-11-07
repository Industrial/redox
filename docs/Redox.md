Redox
Language Specification & Implementation Blueprint

Version: 0.1.0
Status: Initial specification
Source extension: .sdx
Generated extension: .xrs
Implementation language: Rust
Target language: Rust
Project category: Source-to-source compiler

1. Project Overview

Redox is a lightweight, MoonScript-inspired programming language that compiles directly into Rust source code.

Its purpose is to make Rust more expressive and enjoyable to write without changing Rust's underlying semantics.

Redox introduces:

Indentation-based syntax.
Concise function declarations.
Implicit function calls.
List and object comprehensions.
Destructuring.
String interpolation.
Pattern matching.
Convenient control flow.
MoonScript-inspired syntactic sugar.

Redox does not introduce a runtime, virtual machine, garbage collector, standard library, or independent execution environment.

The entire purpose of Redox is to transform .sdx source files into .xrs files containing ordinary Rust code.

1.1 Compilation pipeline
text
       main.sdx
           |
           v
     Redox compiler
           |
           v
        main.xrs
           |
           v
       Rust compiler
           |
           v
      Native binary


.xrs files contain standard Rust syntax. The extension is a Redox naming convention, not a new Rust language dialect.

The compiler must account for Rust compiler input-file extension handling. If necessary, it should use standard .rs staging files or compiler input through stdin rather than requiring changes to Rust itself.

1.2 Project principles
Rust is the target language.
Rust remains the semantic authority.
No Redox runtime.
No Redox standard library.
No independent type system.
No independent ownership or borrowing system.
No custom package manager.
Maximum Rust ecosystem interoperability.
Minimal implementation and dependencies.
Eventual self-hosting.
1.3 What Redox does not provide
Runtime.
Virtual machine.
Garbage collector.
Standard library.
Package manager.
Build system replacement.
Type checker.
Borrow checker.
Optimizer.
Linker.
Async executor.
Dependency manager.

Cargo and Rust remain responsible for their respective functionality.

2. Project Structure

Recommended initial structure:

text
redox/
├── Cargo.toml
├── README.md
├── SPEC.md
├── src/
│   ├── main.rs
│   ├── lexer.rs
│   ├── parser.rs
│   ├── ast.rs
│   ├── transpiler.rs
│   ├── diagnostics.rs
│   └── watcher.rs
├── examples/
│   ├── hello.sdx
│   ├── functions.sdx
│   ├── comprehensions.sdx
│   └── structs.sdx
├── tests/
│   ├── lexer.rs
│   ├── parser.rs
│   ├── transpiler.rs
│   └── integration.rs
└── bootstrap/
    └── README.md


This structure is illustrative. Modules should be combined whenever doing so reduces unnecessary code.

Use Rust's standard library wherever possible.

A filesystem watching dependency such as notify is acceptable.

Avoid parser generators and unnecessary compiler frameworks.

3. Compilation Architecture

The compiler consists of four stages:

Lexer.
Parser.
Abstract syntax tree.
Rust source emitter.
text
Source
  |
  v
Lexer
  |
  v
Tokens
  |
  v
Parser
  |
  v
AST
  |
  v
Transpiler
  |
  v
Rust source emitter
  |
  v
.xrs

3.1 Lexer

The lexer recognizes:

Identifiers.
Keywords.
Numeric literals.
Strings.
Interpolated strings.
Comments.
Operators.
Delimiters.
Newlines.
Indentation.
Dedentation.

Indentation is syntactically significant.

Use four spaces per indentation level. Tabs should be rejected or handled through a single documented normalization rule.

Preserve source positions for diagnostics.

3.2 Parser

The parser converts tokens into a minimal AST.

It should represent Redox-specific constructs without recreating the entire Rust grammar.

Rust expressions, types, generics, macros, and other native syntax should be preserved wherever possible.

3.3 Transpiler

The transpiler translates Redox syntax into Rust.

Examples:

Redox	Rust
name = value	let name = value;
mut name = value	let mut name = value;
@field	self.field
@@method	Self::method
unless	Negated if
until	Negated while
switch	match
! invocation	()
do	Block expression
String interpolation	format!
List comprehension	Iterator pipeline
3.4 Generated output

Default mapping:

text
main.sdx -> main.xrs


Custom output:

bash
redox build main.sdx -o output.xrs


Output must be deterministic and readable.

4. Language Fundamentals
4.1 Comments

Single-line:

coffeescript
# Comment

name = "Redox" # Inline comment


Multiline:

coffeescript
###
Multiline
comment
###


Comments are removed during compilation.

4.2 Indentation
coffeescript
main = ->
  print "Hello"

  if true
    print "World"


Equivalent Rust:

rust
fn main() {
    println!("Hello");

    if true {
        println!("World");
    }
}


Semicolons are optional in Redox.

Newlines terminate statements unless an expression is continued.

4.3 Variables

Immutable by default:

coffeescript
name = "Alice"
age = 30


Mutable:

coffeescript
mut counter = 0
counter += 1


Constants:

coffeescript
PI = 3.14159


Generate Rust let, let mut, and const declarations as appropriate.

4.4 Type annotations
coffeescript
age: u32 = 25

name: String = String::from "Alice"

users: Vec<User> = Vec::new!


References:

coffeescript
value: &String
mutable_value: &mut String


Generics and lifetimes retain native Rust syntax.

5. Functions
5.1 Canonical syntax

The standard function syntax is:

coffeescript
name = (parameters): ReturnType ->
  body


Return types are optional.

Examples:

coffeescript
add = (a: i32, b: i32): i32 ->
  a + b

square = (n: i32): i32 -> n * n

double = (n: i32) -> n * 2

greet = (name: &str): String ->
  "Hello, #{name}!"

5.2 Generated Rust

Input:

coffeescript
add = (a: i32, b: i32): i32 ->
  a + b


Output:

rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

5.3 Generic functions
coffeescript
identity = <T>(value: T): T ->
  value


Trait bounds:

coffeescript
print_value = <T: Display>(value: T) ->
  print value

5.4 Closures
coffeescript
double = (n: i32) -> n * 2

callback = (value) =>
  print value


-> represents an ordinary closure.

=> represents a receiver-capturing closure when used in a receiver context.

Closure ownership and capture behavior remain Rust-native.

5.5 Default arguments

Proposed syntax:

coffeescript
greet = (name: &str = "World") ->
  print "Hello, #{name}!"


Since Rust does not support default parameters, the compiler generates an appropriate convenience wrapper.

This feature can be postponed if it significantly complicates the implementation.

6. Function Calls
6.1 Implicit calls
coffeescript
print "Hello"

add 10, 20

user.save!


Equivalent Rust:

rust
println!("Hello");
add(10, 20);
user.save();

6.2 Parenthesized calls

Parentheses remain available:

coffeescript
result = add(10, 20)


They should be encouraged for ambiguous nested expressions.

6.3 Zero-argument calls

The ! suffix indicates a zero-argument invocation:

coffeescript
user.save!
config.load!
Vec.new!


This is distinct from Rust macro syntax.

Rust macros retain their native syntax:

coffeescript
println! "Hello"
vec! [1, 2, 3]

6.4 Nested calls
coffeescript
result = add (multiply 2, 3), 4


Equivalent:

rust
let result = add(multiply(2, 3), 4);


The parser must define unambiguous precedence rules.

7. Strings
7.1 Interpolation
coffeescript
name = "Alice"
age = 25

message = "Hello #{name}, age #{age}"


Equivalent:

rust
let message = format!("Hello {}, age {}", name, age);


Expressions are allowed:

coffeescript
message = "Result: #{add 10, 20}"


Interpolation follows Rust formatting semantics.

7.2 Multiline strings
coffeescript
query = """
  SELECT *
  FROM users
  WHERE active = true
"""


Remove common indentation while preserving relative indentation.

8. Control Flow
8.1 If / else
coffeescript
if age >= 18
  print "Adult"
else if age >= 13
  print "Teenager"
else
  print "Child"

8.2 Unless
coffeescript
unless finished
  process!


Equivalent:

rust
if !finished {
    process();
}


Postfix:

coffeescript
print "Running" unless finished

8.3 Postfix conditionals
coffeescript
print "Welcome" if logged_in

return unless valid

8.4 While
coffeescript
while running
  update!

8.5 Until
coffeescript
until finished
  process_next!


Equivalent:

rust
while !finished {
    process_next();
}

8.6 Postfix loops
coffeescript
counter += 1 while counter < 10

wait! until ready

8.7 For loops
coffeescript
for number in numbers
  print number


References:

coffeescript
for user in &users
  print user.name


Mutable references:

coffeescript
for user in &mut users
  user.active = true

9. List Comprehensions

List comprehensions are a core Redox feature.

9.1 Basic comprehension
coffeescript
squares = [n * n for n in 1..10]


Equivalent Rust:

rust
let squares: Vec<_> = (1..=10)
    .map(|n| n * n)
    .collect();

9.2 Filtering
coffeescript
evens = [n for n in numbers when n % 2 == 0]


Equivalent:

rust
let evens: Vec<_> = numbers
    .into_iter()
    .filter(|n| n % 2 == 0)
    .collect();

9.3 Mapping and filtering
coffeescript
positive = [
  n * 2
  for n in numbers
  when n > 0
]

9.4 Multiple conditions
coffeescript
adults = [
  user.name.clone!
  for user in users
  when user.active
  when user.age >= 18
]


Multiple conditions are combined using logical AND.

9.5 Nested comprehensions
coffeescript
pairs = [
  (x, y)
  for x in 1..3
  for y in 1..3
]

9.6 Index access
coffeescript
indexed = [
  "#{i}: #{value}"
  for value, i in values
]


Generate enumerate().

9.7 Ownership

Redox must never silently clone values.

coffeescript
# Consume
values = [x for x in items]

# Borrow
values = [x for x in &items]

# Mutably borrow
values = [x for x in &mut items]


Rust determines the resulting types and borrowing validity.

10. Object Comprehensions

Object comprehensions create maps.

Syntax:

coffeescript
mapping = {
  key_expression, value_expression
  for item in collection
}


Example:

coffeescript
squares = {
  n, n * n
  for n in 1..10
}


The generated code should use std::collections::HashMap unless a type context specifies another map.

Filtering:

coffeescript
active_users = {
  user.id, user.name.clone!
  for user in &users
  when user.active
}


No external dependencies may be introduced.

11. Destructuring

Tuples:

coffeescript
(a, b) = get_coordinates!


Arrays:

coffeescript
[first, second] = values


Struct fields:

coffeescript
{name, age} = user


Renaming:

coffeescript
{id: user_id, name: username} = user


Rest patterns:

coffeescript
[first, ...rest] = values


All destructuring must translate into ordinary Rust patterns.

Ownership behavior must remain unchanged.

12. Splat Operators

The ... operator supports expansion.

Collection expansion:

coffeescript
numbers = [1, 2, 3]

all_numbers = [0, ...numbers, 4]


Argument expansion:

coffeescript
args = (10, 20)

result = add args...


Rest patterns:

coffeescript
[first, ...remaining] = values


Expansion must be statically resolvable.

Redox does not introduce dynamically variadic functions.

The parser must distinguish expansion from exclusive range syntax.

13. Optional Syntax

Redox uses Rust's Option type.

13.1 Optional chaining
coffeescript
name = user?.profile?.name


Translate to equivalent Option::and_then and Option::map operations.

Do not silently flatten nested options.

13.2 Null-coalescing
coffeescript
username = user.name ?? "Anonymous"


Translate to unwrap_or or unwrap_or_else, as appropriate.

13.3 Error propagation

Rust's ? remains available:

coffeescript
load_config = (path: &str): Result<Config, Error> ->
  content = fs.read_to_string path?
  parse_config content?


Standalone existential checks such as value? are excluded from the initial specification due to ambiguity.

14. Comparisons

All ordinary Rust comparison operators are supported.

coffeescript
a == b
a != b
a < b
a > b
a <= b
a >= b


Chained comparisons:

coffeescript
if 0 < age < 120
  print "Valid age"


Equivalent:

rust
if 0 < age && age < 120 {
    println!("Valid age");
}


Intermediate expressions must be evaluated only once.

15. Pipeline Operator

Redox supports |>.

coffeescript
result = numbers
  |> filter (n) -> n > 0
  |> map (n) -> n * n
  |> sum


The preceding expression is passed as the first argument to the following function.

Pipeline precedence must be explicitly defined and must not conflict with Rust bitwise operators.

16. Receiver Shorthand
16.1 @
coffeescript
@name
@save!


Equivalent:

rust
self.name
self.save();

16.2 @@
coffeescript
@@create!


Equivalent:

rust
Self::create();


These transformations are valid only in receiver contexts.

17. Structs, Enums, Traits, and Classes
17.1 Structs
coffeescript
struct Person
  name: String
  age: u32
  active: bool


Generates:

rust
struct Person {
    name: String,
    age: u32,
    active: bool,
}

17.2 Struct initialization
coffeescript
person = Person
  name: "Alice"
  age: 30
  active: true

17.3 Enums
coffeescript
enum Animal
  Dog(String)
  Cat(String)
  Bird


Generates a native Rust enum.

17.4 Traits
coffeescript
trait Greetable
  greet: (self) -> String

17.5 Trait implementations
coffeescript
impl Greetable for Person
  greet = (self) ->
    "Hello, #{self.name}!"

17.6 Inherent implementations
coffeescript
impl Person
  new = (name: String, age: u32): Self ->
    Person
      name: name
      age: age
      active: true

17.7 Classes

class is syntactic sugar for a Rust struct and inherent implementation.

coffeescript
class Counter
  value: i32

  new = (value: i32): Self ->
    Counter value: value

  increment = (self: &mut Self) ->
    self.value += 1


No classical inheritance is introduced.

extends is excluded from the initial specification.

Rust traits and composition are the intended alternatives.

18. Pattern Matching

switch is an alternative syntax for Rust match.

coffeescript
message = switch status
  Active  => "Online"
  Pending => "Waiting"
  Failed  => "Error"
  _       => "Unknown"


Guards:

coffeescript
result = switch value
  Some(n) when n > 0 => "Positive"
  Some(n)            => "Non-positive"
  None               => "Missing"


Rust exhaustiveness checking is preserved.

19. Do Expressions
coffeescript
result = do
  x = 10
  y = 20
  x + y


Equivalent:

rust
let result = {
    let x = 10;
    let y = 20;
    x + y
};


The final expression is the block's result.

20. With Expressions

Proposed syntax:

coffeescript
with config
  host = "localhost"
  port = 8080


This feature is optional for the initial implementation.

It must not introduce hidden copies, implicit mutable references, or changes to borrowing semantics.

If implementation requires complex semantic analysis, postpone it.

21. Ranges and Slices

Inclusive ranges:

coffeescript
1..10


Equivalent:

rust
1..=10


Exclusive ranges:

coffeescript
1...10


Equivalent:

rust
1..10


Slices:

coffeescript
first_three = values[0..2]
everything_after = values[3..]
everything_before = values[..4]


Preserve Rust bounds-checking behavior.

22. Modules and Imports
coffeescript
import std.collections.HashMap
import std.io.{File, Read}
import my_app.models.User


Aliases:

coffeescript
import std.collections.HashMap as Map


Exports:

coffeescript
export struct Config
  host: String
  port: u16


Generate native Rust use declarations and pub visibility.

Do not implement a separate module loader.

23. Macros and Inline Rust

Native Rust macros are supported.

coffeescript
println! "Hello"

vec! [1, 2, 3]

format! "Hello #{name}"


Macro semantics remain unchanged.

23.1 Inline Rust escape hatch

Proposed syntax:

coffeescript
rust """
  unsafe {
      // Native Rust code
  }
"""


The compiler inserts the contents into the generated Rust.

This feature allows direct access to Rust syntax not yet expressible in Redox.

24. Async and Await
coffeescript
fetch_user = async (id: u64): Result<User, Error> ->
  response = await http.get "/users/#{id}"
  user = await response.json!
  Ok user


Translate to native Rust async syntax.

Redox provides no async executor or runtime.

25. Ownership and Borrowing

This is a mandatory compatibility requirement.

Redox must preserve Rust semantics exactly.

25.1 No implicit cloning
coffeescript
name = String::from "Alice"
other = name


This is an ordinary Rust move.

25.2 Explicit borrowing
coffeescript
inspect = (value: &String) ->
  print value

modify = (value: &mut String) ->
  value.push_str "!"

25.3 Lifetimes
coffeescript
get = <'a>(value: &'a str): &'a str ->
  value

25.4 Unsafe

Unsafe Rust is supported through native Rust syntax or the inline Rust escape hatch.

The compiler must never insert unsafe to bypass Rust restrictions.

26. Rust Interoperability

The following must remain possible:

External crates.
Rust modules.
Rust functions.
Rust traits.
Rust macros.
Generic types.
Lifetimes.
Unsafe Rust.
Async Rust.
Procedural macro integrations.
FFI.

Generated .xrs contents must be ordinary Rust.

No Redox-specific runtime dependencies may be introduced.

27. Command-Line Interface

The initial CLI should remain small.

Build
bash
redox build main.sdx


Produces:

text
main.xrs

Custom output
bash
redox build main.sdx -o generated.xrs

Check
bash
redox check main.sdx


Checks parsing and transpilation, not Rust type correctness.

Watch
bash
redox watch .

Version
bash
redox --version

Help
bash
redox --help


Do not add unnecessary commands.

28. File Watcher

The watcher is a simple development utility.

It must:

Watch .sdx files.
Detect changes.
Recompile modified files.
Write .xrs output.
Avoid unnecessary recompilation.
Display errors.
Continue watching after errors.
Support recursive directories.

It must not become a build system.

It should not automatically execute programs, manage dependencies, or implement hot reloading.

29. Diagnostics

Report errors with file names, line numbers, and column numbers.

Example:

text
error: unexpected token

  --> main.sdx:4:12

4 | add = (a: i32, b: i32): i32
  |            ^

expected function body '->'


Distinguish:

Lexer errors.
Parser errors.
Unsupported syntax.
Rust compilation errors.

Pass Rust errors through wherever possible.

30. Minimal Implementation Strategy

The project's most important engineering constraint is minimizing lines of code.

Do not build an elaborate compiler.

Avoid:

Bytecode.
Custom intermediate languages.
Type inference engines.
Borrow checkers.
Custom optimizers.
Runtime systems.
Plugin architectures.
Unnecessary compiler passes.

Prefer direct transformations.

For example:

coffeescript
unless ready
  wait!


Should simply generate:

rust
if !ready {
    wait();
}


The AST should contain only the structures necessary to parse and translate Redox syntax.

Rust-compatible expressions and types should be preserved whenever practical.

31. Implementation Roadmap
Phase 1: Minimal compiler

Implement:

File reading.
Lexer.
Indentation.
Basic parser.
Variables.
Functions.
Return types.
Blocks.
Rust emitter.
CLI.

Acceptance test:

coffeescript
add = (a: i32, b: i32): i32 ->
  a + b

main = ->
  print add 10, 20


Must generate valid Rust.

Phase 2: Core syntax

Implement:

Implicit calls.
Zero-argument calls.
String interpolation.
If/else.
Unless.
While.
Until.
For loops.
Postfix conditions.
Postfix loops.
Switch.
Phase 3: Collections

Implement:

List comprehensions.
Filtering.
Nested comprehensions.
Object comprehensions.
Destructuring.
Splat.
Ranges.
Slices.
Phase 4: Rust integration

Implement:

Structs.
Enums.
Traits.
Implementations.
Generics.
Lifetimes.
Imports.
Exports.
Macros.
Async/await.
Inline Rust.
Phase 5: Advanced syntax

Implement:

Fat arrows.
@ and @@.
Optional chaining.
??.
Pipeline.
Do expressions.
With expressions.
Default arguments.
Classes.
Phase 6: Watcher and usability

Implement:

Recursive file watching.
Change detection.
Recompilation.
Diagnostics.
Output management.
Integration tests.
Phase 7: Self-hosting

Rewrite the compiler in Redox.

Verify that the self-hosted compiler produces semantically equivalent output to the Rust bootstrap implementation.

32. Testing Requirements

Every implemented syntax feature requires tests.

Test both:

Generated Rust output.
Successful Rust compilation.

Example:

Input:

coffeescript
squares = [n * n for n in 1..5]


Equivalent Rust:

rust
let squares: Vec<_> = (1..=5)
    .map(|n| n * n)
    .collect();


Test:

Nested indentation.
Multiline expressions.
Generic types.
Nested function calls.
Borrowed collections.
Mutable collections.
Interpolation.
Operator precedence.
Diagnostics.
Macros.
Inline Rust.
Optional chaining.
33. Compatibility Requirements

Mandatory rules:

Never change Rust ownership semantics.
Never silently clone values.
Never introduce garbage collection.
Never introduce runtime dependencies.
Never require a Redox standard library.
Never restrict Rust library access.
Never change Rust type semantics.
Preserve Rust error propagation.
Generate valid Rust source.
Remain interoperable with ordinary Rust projects.

Any feature that violates these requirements must be redesigned or excluded.

34. Bootstrap and Self-Hosting
Stage 1: Rust implementation

Build the initial compiler in Rust.

It reads .sdx and generates .xrs.

Stage 2: Redox implementation

Once the language is sufficiently capable, rewrite the compiler in Redox.

The self-hosted compiler must generate the same Rust semantics as the bootstrap compiler.

Stage 3: Verification

Compare outputs from both compilers across a shared test suite.

Formatting differences may be normalized, but semantic equivalence is mandatory.

Keep the Rust bootstrap compiler available as a reliable fallback.

35. Definition of Done
Redox source files compile to .xrs.
All specified core syntax works.
Generated Rust compiles successfully.
Rust ownership and borrowing are preserved.
External Rust crates work.
Rust macros work.
Generics and lifetimes work.
Async/await works.
CLI works.
File watcher works.
Automated tests pass.
Compiler can be rewritten in Redox.
Self-hosted compiler can compile itself.
36. Instructions to the Implementing AI

You are implementing Redox.

Your objective is to create the smallest practical source-to-source compiler that translates Redox into Rust.

Follow these requirements strictly.

Architecture
Implement the first compiler in Rust.
Generate Rust source, not machine code.
Do not create a runtime.
Do not create a standard library.
Do not create an independent type system.
Do not implement Rust semantics.
Implementation
Prefer minimal code.
Prefer direct transformations.
Avoid unnecessary dependencies.
Avoid excessive abstractions.
Avoid premature optimization.
Implement incrementally.
Test each feature independently.
Language
Follow this specification.
Do not invent undocumented syntax.
Resolve ambiguities explicitly.
Preserve native Rust expressiveness.
Output
Generate readable Rust.
Keep .xrs as the generated extension.
Ensure generated source can be passed to the Rust compiler.
Do not introduce Redox runtime scaffolding.
Self-hosting
Maintain the Rust bootstrap compiler.
Eventually rewrite the compiler in Redox.
Verify the self-hosted implementation against the bootstrap compiler.

Start with the lexer, parser, and minimal Rust emitter.

Do not attempt to implement the entire language at once.

Correctness, simplicity, and Rust compatibility take priority over feature quantity.

37. Final Project Definition

Redox is a lightweight, MoonScript-inspired syntax for Rust.

It is implemented in Rust and eventually in itself.

It compiles .sdx files into .xrs files containing ordinary Rust source code.

The .xrs extension represents the project's inspiration from redox chemistry: reduction and oxidation.

Redox does not provide a runtime, standard library, package manager, or independent execution environment.

It relies entirely on Rust for type checking, memory safety, concurrency, libraries, compilation, and execution.

Its purpose is simple:

Make writing Rust more enjoyable without changing what Rust is.

Redox should be as small as possible, as expressive as necessary, and entirely compatible with the Rust ecosystem.
