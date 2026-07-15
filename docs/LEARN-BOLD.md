# Learn BOLD

The official beginner's course for BOLD, the in-house programming language of BOLD Ventures. No prior coding experience required. By the end you will have written real programs in the core language and shipped a complete web app with the app layer.

## Why BOLD is easy to learn

BOLD is built on one conviction: code should read like intent. The core language uses a handful of plain keywords (fn for function, let for a value, mut for a value that changes) and the compiler catches your mistakes before the program ever runs, with error messages that tell you exactly what to fix and how.

## Part One: The Core Language

### Lesson 1: Hello, World (2 minutes)

Create a file called hello.bold:

    import system

    fn main() {
        system.print("Hello, World!")
    }

Run it: `bld run hello.bold`

Three ideas in five lines. import brings in a module. fn main() is where every program starts. system.print puts words on the screen.

### Lesson 2: Values that hold still, values that move (4 minutes)

    fn main() {
        let name = "Brett"          // let means this never changes
        mut score: Int = 0          // mut means this can change
        score = score + 10
        system.print(name)
        system.print(score)
    }

Try reassigning name and run it. The compiler stops you:

    type error: name is immutable (declared with let). Declare it with mut to allow reassignment

That is BOLD protecting you. Most bugs in software are things changing that were not supposed to change. In BOLD, nothing changes unless you said mut out loud.

### Lesson 3: Decisions (4 minutes)

    fn main() {
        let systemLoad = 85
        if systemLoad > 90 {
            system.print("Critical Overhead")
        } else if systemLoad > 70 {
            system.print("Warning Balance")
        } else {
            system.print("System Nominal")
        }
    }

No parentheses needed around the condition. Braces always required. Read it like English: if the load is over 90, say this, else if over 70, say that.

### Lesson 4: Repetition (3 minutes)

    fn main() {
        mut total = 0
        for i in 0..5 {
            total = total + i
        }
        system.print(total)    // prints 10
    }

0..5 means 0, 1, 2, 3, 4, up to but not including 5. There is also while for loops that run until a condition turns false.

### Lesson 5: Functions, the verbs of your program (5 minutes)

    import system

    fn double(n: Int) -> Int {
        return n * 2
    }

    fn main() {
        system.print(double(21))    // prints 42
    }

Parameters declare their types. The arrow declares what comes back. If your function promises to return an Int, the compiler proves it returns one on every path, or it refuses to build. No surprises at runtime.

### Lesson 6: Structs, the nouns of your program (5 minutes)

    import system

    struct Developer {
        name: String
        experienceYears: Int
        isActive: Bool
    }

    fn calculateBonus(dev: Developer, rating: Float) -> Float {
        if dev.experienceYears > 5 {
            return rating * 1500.00
        }
        return rating * 500.00
    }

    fn main() {
        let senior = Developer { name: "Andras", experienceYears: 8, isActive: true }
        system.print(calculateBonus(senior, 4.5))
    }

A struct groups related facts into one shape. Every field is typed, every literal must fill every field, and dot notation reads the values back.

### Lesson 7: Strict typing is a feature (2 minutes)

Try this: `let x = 1 + 2.5`

    type error: cannot add Int and Float. BOLD is strictly typed and does not convert implicitly

Other languages guess what you meant and guess wrong at 2am in production. BOLD asks you to say what you mean. It is the Proverbs 22:29 of language design: skilled work, done deliberately.

## Part Two: The App Layer

The app layer is the part of BOLD that describes user interfaces. Where the core language uses braces and functions, the app layer reads like an outline:

    app "My First App"
      theme gold

    page Home at "/"
      hero
        title "Welcome"
        subtitle "Built with BOLD."
        button "About" goto About

    page About at "/about"
      heading "About"
      text "I build things with BOLD."

Run `bld build app.bold` and you get a finished, deployable, beautifully designed web app: routing, responsive layout, and the BOLD design system included. The playground teaches every element interactively, from grids and cards to stats, data blocks, and the verse element that sets Scripture apart with honor.

## Ship something

The fastest way to learn a language is to say something true in it. Rebuild a tool you use daily as a .bold program, then `bld check` until it compiles clean, then `bld run`. Read BOLD-SPEC.md when you want the whole map.

Designed and Developed by BOLD Studios.
