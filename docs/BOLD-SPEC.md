# BOLD Language Specification

Version 1.0. BOLD is the in-house programming language of BOLD Ventures. Always capitalized. Never suffixed: it is not BOLD Lang, BOLD Code, or BOLD Tech. It is BOLD.

## Identity

Official name: BOLD. File extension: .bold, used universally for all source files. CLI and compiler binary: bld, built completely in Rust, distributed as a single native executable with no runtime dependencies. Primary commands: `bld run <file>.bold` executes a program, `bld check <file>.bold` compiles and reports diagnostics without running, `bld fmt <file>.bold` formats source in place.

## Philosophy

BOLD stands on three convictions. Code should read like intent: explicit declarations, no ambiguity, no cleverness at the reader's expense. Excellence is the floor: strict static typing catches mistakes at compile time, and the app layer ships the full BOLD design system by default. The work is worship: built to Colossians 3:23, whatever you do, work at it with all your heart.

## The two layers

BOLD is one language with two layers, the same relationship Swift has to SwiftUI.

The core layer is a strictly typed, general-purpose language for logic, data, and systems work. It uses fn, let, mut, struct, and braces, and it runs through the Rust-built bld runtime.

The app layer is a declarative dialect for describing user interfaces (app, page, hero, grid, card). The app layer compiler turns a .bold app description into a finished, deployable web app with the BOLD design system baked in. Its reference implementation currently ships in the BOLD web toolkit (bld-web.js) and merges into the bld binary in v1.1.

## Core layer

### Program structure

Every executable program has a main function. Modules are brought in with import.

    import system

    fn main() {
        system.print("Hello, World!")
    }

### Types

BOLD is strictly and statically typed. Primitive types: Int (64-bit signed integer), Float (64-bit floating point), String, Bool. Unit is the type of functions that return nothing. Struct types are user-defined. There are no implicit conversions anywhere in the language: Int plus Float is a compile-time error, by design.

### Variables

let declares an immutable binding. mut declares a mutable one. Type annotations are optional where the type can be inferred.

    let message: String = "Engine Active"
    let inferred = "Implicit Type"
    mut counter: Int = 0
    counter = counter + 1

Reassigning a let binding is a compile-time error with a diagnostic that names the fix.

### Control flow

Conditions take no parentheses. Braces are mandatory.

    if systemLoad > 90 {
        system.print("Critical Overhead")
    } else if systemLoad > 70 {
        system.print("Warning Balance")
    } else {
        system.print("System Nominal")
    }

    for i in 0..5 {
        system.print(i)
    }

    while count < 10 {
        count = count + 1
    }

Ranges are half-open: 0..5 yields 0, 1, 2, 3, 4. Range bounds must be Int. Conditions must be Bool.

### Functions

Parameters are explicitly typed. Return types follow an arrow. A function with a declared return type must return on every path; the type checker proves it.

    fn calculateBonus(dev: Developer, rating: Float) -> Float {
        if dev.experienceYears > 5 {
            return rating * 1500.00
        }
        return rating * 500.00
    }

### Structs

    struct Developer {
        name: String
        experienceYears: Int
        isActive: Bool
    }

    let senior = Developer { name: "Andras", experienceYears: 8, isActive: true }
    system.print(senior.name)

Struct literals must provide every field with the correct type. Field access uses dot notation.

### Operators

Arithmetic: + - * / % (two Ints or two Floats; + also concatenates two Strings). Comparison: < > <= >= (numeric). Equality: == != (same type on both sides). Logic: && || ! (Bool only, with short-circuit evaluation). Unary minus negates numbers.

### The system module

import system exposes the standard output interface. v1 surface: system.print(value) prints any value followed by a newline. The module grows in 1.x (system.read, system.time, system.env) without breaking existing programs.

### Diagnostics

bld reports errors with the kind, file, line, message, and the offending source line. Categories: syntax errors (lexing), parse errors (structure), type errors (checking), and runtime errors (execution, such as division by zero). Sample real diagnostics:

    type error [main.bold:3] x is declared as Int but assigned a String
    type error [main.bold:4] locked is immutable (declared with let). Declare it with mut to allow reassignment
    type error [main.bold:3] cannot add Int and Float. BOLD is strictly typed and does not convert implicitly
    type error [main.bold:2] function f declares return type Int but does not return on every path

## App layer

The app layer describes interfaces declaratively with indentation as structure. Full element reference lives in the playground's Learn tab. Summary: app names the application and selects a theme (gold, midnight, ember, light); page declares routed pages; hero, title, subtitle, heading, text, button, section, grid, card, row, stat, list, cta, verse, image, and spacer compose the interface; data blocks declare collections once and grids or lists render them with from. Compilation produces a single self-contained index.html with routing, SEO metadata, responsive layout, and the BOLD Studios footer credit.

## Compiler architecture

The bld compiler is implemented in Rust across four phases, one module per phase: the lexer (src/lexer/) tokenizes .bold streams; the parser (src/parser/) builds the abstract syntax tree and validates brace-enclosed block structure; the type checker (src/typechecker/) enforces every rule above at compile time; the runtime (src/codegen/) executes validated programs, with native code generation on the roadmap. Development commands: `cargo build --release` builds the compiler, `cargo test` runs the suite, `./target/release/bld run examples/hello.bold` runs a program.

## Reserved words

Core layer: fn, let, mut, struct, import, if, else, for, in, while, return, true, false. App layer: app, theme, page, at, data, item, from, goto, hero, title, subtitle, heading, text, button, link, section, grid, card, row, stat, list, cta, verse, image, spacer.

## Versioning

Semantic versioning. Programs that compile clean under 1.x compile clean under every later 1.x. Planned for 1.x: arrays and maps, string interpolation, methods on structs, modules and multi-file programs, the app layer merged into the bld binary, and native binary output via bld build.

Designed and Developed by BOLD Studios.
