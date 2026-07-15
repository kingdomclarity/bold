# BOLD Developer Platform Roadmap

The plan for the BOLD technical product line. Updated July 2026, after the v1 builds.

## Naming, locked

The language is BOLD. Always capitalized, never suffixed. Not BOLD Lang, not BOLD Code the language, not BOLD Tech. The technical sub-brand developers touch every day is the bld CLI, and the file signature is .bold. BOLD Code remains the name of the developer terminal application only. In developer-facing docs, lead with the tooling: the bld toolchain for the BOLD programming language.

## What exists today (v1, shipped)

The bld compiler, built completely in Rust: a real lexer, parser, strict type checker, and runtime across the four-phase architecture, compiled to a single native binary. It runs real .bold programs (structs, typed functions, control flow, recursion) in microseconds and refuses bad programs with world-class diagnostics: immutability violations, type mismatches, missing return paths, and missing imports, each with the file, line, and the fix. Commands: bld run, bld check, bld fmt.

The BOLD app layer: the declarative dialect that compiles an outline-style .bold description into a finished web app with the BOLD design system, routing, and SEO baked in. Reference implementation in the BOLD web toolkit, merging into the bld binary in v1.1.

BOLD Code Terminal: the developer command center with the project sidebar (core BOLD projects, divider, client projects), editor, live terminal, and the BOLD Intelligence build bar. The BOLD Playground: write BOLD, watch it build live, with the Learn BOLD course built in. BOLD OS: the operating environment shell with the App Store and BOLD One tiers (Believer $9, Builder $29, Kingdom $79).

## The honest strategic frame

The UVP is true today: BOLD Ventures builds with its own programming language and its own Rust-built toolchain. That goes on every proposal. What stays under the hood, deliberately: the v1 runtime interprets rather than emitting native binaries, the app layer compiles to the open web platform, and GitHub plus Netlify keep doing the plumbing until revenue justifies replacing them. Apple built macOS on Unix; React sits on JavaScript. Owning the experience while standing on proven foundations is the playbook, not a shortcut.

## Phase 2: Toolchain maturity (30 to 60 days)

Merge the app layer into the bld binary so one tool does everything: bld run for programs, bld build for apps. Add arrays, maps, and string interpolation to the core language. Ship the VS Code extension with .bold syntax highlighting, because a language is judged by its editor experience on day one. Stand up the one-line installer (curl to sh) and a developer-centric docs site kept separate from the corporate pages, modeled on rust-lang.org rather than an agency site.

## Phase 3: BOLD Intelligence for real (60 to 120 days)

Wire the terminal's AI bar to the Claude API so "ask BOLD to build anything" generates .bold source, compiles it through bld, and previews it live. A client watching their site spoken into existence in BOLD on a sales call is the demo that closes high-level clients by itself.

## Phase 4: Teach the world (parallel, start anytime)

Publish Learn BOLD free, built from LEARN-BOLD.md and the playground. Free education is the moat: everyone who learns BOLD becomes an evangelist, a hire, or a client, and the language can never be called obscure. Dogfood in public: rebuild one real internal service in BOLD and publish the case study with honest metrics. When the toolchain is solid, a Show HN launch and r/programming post, engineering-first, no marketing voice.

## Phase 5: Own more of the stack (12 months plus, revenue-gated)

Native binary output from bld build. A BOLD-branded git experience starting as a layer over GitHub's API. A native desktop shell for BOLD OS via Tauri once the web version has daily users. Each piece replaces commodity infrastructure only when owning it compounds, never for pride.

## The one rule

The platform serves the businesses. Clipping clients, His Glory, and the sales pipeline fund every phase, and each phase ships only when the previous one is producing revenue or closing clients. Unless the Lord builds the house, the builders labor in vain. Psalm 127:1.

Designed and Developed by BOLD Studios.
