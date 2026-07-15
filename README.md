# BOLD

BOLD is the in-house technology ecosystem of BOLD Ventures: our own programming language, our own Rust-built toolchain (bld), our own operating environment (BOLD OS), and our own developer terminal (BOLD Code). Built to advance the Kingdom of God through excellent work.

## What is in this repository

`bld/` is the BOLD compiler and CLI, built completely in Rust across the four-phase architecture: lexer (`src/lexer/`), parser (`src/parser/`), type checker (`src/typechecker/`), and runtime (`src/codegen/`). `bld/examples/` holds real .bold programs. `bold-web/` is the BOLD app layer compiler, which turns declarative .bold app descriptions into finished web apps, plus the generator for the BOLD Playground. `site/` is the public BOLD ecosystem website, including BOLD OS, the BOLD Code Terminal, and the Playground as self-contained apps. `docs/` holds the language specification, the Learn BOLD course, and the platform roadmap.

## Quick start

Build the compiler (requires Rust):

    cd bld
    cargo build --release
    ./target/release/bld run examples/hello.bold

Commands: `bld run <file>.bold` executes a program, `bld check <file>.bold` compiles and reports diagnostics, `bld fmt <file>.bold` formats source in place.

Build a web app with the app layer:

    node bold-web/bld-web.js build bold-web/examples/his-glory.bold

## The language in ten lines

    import system

    struct Developer {
        name: String
        experienceYears: Int
    }

    fn main() {
        let dev = Developer { name: "Andras", experienceYears: 8 }
        system.print(dev.name)
    }

BOLD is strictly and statically typed with zero implicit conversions, immutable-by-default bindings (let vs mut), and compile-time proof that every function returns on every path. Read `docs/BOLD-SPEC.md` for the full specification and `docs/LEARN-BOLD.md` to learn the language in an afternoon.

## Deployment

The site deploys on Netlify: publish directory `site`, and the build command generates the Playground and packages the toolkit zip into `site/downloads/` at deploy time.

Designed and Developed by BOLD Studios.
