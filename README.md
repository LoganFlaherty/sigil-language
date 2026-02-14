## Banish
Banish is a declarative DSL that simplifies complex state machines and rule-based logic.

It allows you to define States and Rules that automatically re-evaluate until your logic reaches a fixed point or transitions to a new state.

## Why Banish?
- Writing complex state machines in raw Rust often leads to "spaghetti code" full of nested if/else, loop, and match blocks. Banish provides a clean, readable syntax to organize this logic.
- Fixed-Point Solving: Unlike a standard function that runs top-to-bottom once, a Banish state loops internally until no more rules trigger. This makes it perfect for layout engines, constraint solvers, or complex game logic.
- Zero Runtime Overhead: Banish is a procedural macro. It generates standard, optimized Rust code at compile time. There is no interpreter or virtual machine.
- Mix Standard Rust: The body of every rule is just standard Rust code. You don't have to learn a whole new language, just a new structure.

## Features
- @States: Group logic into distinct states (e.g., @init, @process, @report).
- ? Guards: Rules that only execute when a condition is met (e.g., increment ? tick < 120).
- Convergence Loops: If a rule is triggered, the state automatically re-evaluates to ensure consistency.
- Automatic State Transitions: Once a state reaches a fixed point it transitions to the next state. However, the => @state syntax offers explicit transitions to any state.
- Scope Aware: Variables declared in your outer scope are available inside the DSL, making it easy to integrate into existing projects.
- Returns: Inside a rule, return can be used to exit out of Banish early or return a value.

## Examples
https://github.com/LoganFlaherty/banish/blob/main/docs/README.md

## Install
### With Cargo
```
cargo add banish
```

### With TOML
```
[dependencies]
banish = "1.1.1"
```

### With Github in TOML
```
[dependencies]
banish = { git = "https://github.com/LoganFlaherty/banish" }
```

## License
This project is dual-licensed under **Apache 2.0** OR **MIT**.
