## Banish
Banish is a declarative DSL (Domain Specific Language) for Rust that simplifies complex state machines and rule-based logic.

It allows you to define Phases and Rules that automatically re-evaluate until your logic "settles" (reaches a fixed point) or transitions to a new state. It compiles down to zero-overhead, standard Rust loops and match statements.

## Why Banish?
- Writing complex state machines in raw Rust often leads to "spaghetti code" full of nested if/else, loop, and match blocks. Banish provides a clean, readable syntax to organize this logic.
- Fixed-Point Solving: Unlike a standard function that runs top-to-bottom once, a Banish phase loops internally until no more interactions occur. This makes it perfect for layout engines, constraint solvers, or complex game logic.
- Zero Runtime Overhead: Banish is a procedural macro. It generates standard, optimized Rust code at compile time. There is no interpreter or virtual machine.
- Safe State Transitions: The => @phase syntax makes flow control explicit and impossible to miss, preventing "fall-through" bugs common in manual state machines.
- Mix Standard Rust: The body of every rule is just standard Rust code. You don't have to learn a whole new language, just a new structure.

## Features
- @Phases: Group logic into distinct states (e.g., @init, @process, @report).
- ? Guards: Rules that only execute when a condition is met (e.g., increment ? tick < 120).
- Convergence Loops: If a rule modifies state, the phase automatically re-evaluates to ensure consistency.
- Direct Transitions: Instant state switching using the => @phase syntax. This is a jump, so you do no return to where you were.
- Scope Isolation: Variables declared in your outer scope are available inside the DSL, making it easy to integrate into existing projects.

## Examples
docs/README.md

## Install
### With Cargo
```
cargo add banish
```

### With TOML
```
[dependencies]
banish = "1.0.0"
```

### With Github in TOML
```
[dependencies]
banish = { git = "https://github.com/LoganFlaherty/banish" }
```

## License
This project is dual-licensed under **Apache 2.0** OR **MIT**.
