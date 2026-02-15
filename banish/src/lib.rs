//! # Banish
//! Banish is a declarative DSL for building rule-driven state machines in Rust. 
//! It allows you to define states and rules that execute until they reach a stable 
//! fixed point or trigger transitions, making complex control flow easier to express and reason about.
//!
//! ## Syntax
//! - **@state** : Defines a state that loops until no rules trigger or a state transition. States execute from top to bottom.
//! - **rule ? condition {}** : Defines a rule. Executes if its condition is true. Rules execute from top to bottom.
//! - **!? {}** : Defines an else clause after the closing brace of a rule with a condition.
//! - **rule ? {}** : A rule without a condition. Executes exactly once per state entry. Cannot have an else clause.
//! - **=> @state;** : Transitions immediately to another state, but is a rule top-level statement only.
//! - **return value;** : Immediately exit banish and return a value if passed.
//!
//! ## Examples
//! https://github.com/LoganFlaherty/banish/blob/main/docs/README.md
//!
//! ```rust
//! use banish::banish;
//!
//! fn main() {
//!     let mut ticks: i32 = 0;
//!     let mut loop_count: i32 = 0;
//!     banish! {
//!         @red
//!             announce ? {
//!                 ticks = 0;
//!                 println!("Red light");
//!                 loop_count += 1;
//!              }
//!
//!             timer ? ticks < 3 {
//!                 ticks += 1;
//!             }
//!
//!         @green
//!             announce ? {
//!                 println!("Green light");
//!             }
//!
//!             timer ? ticks < 6 {
//!                 ticks += 1;
//!             }
//!
//!         @yellow
//!             announce ? {
//!                 println!("Yellow light");
//!             }
//!
//!             timer ? ticks < 10 {
//!                 ticks += 1;
//!             }
//!
//!             reset ? ticks == 10 && loop_count < 2 {
//!                 => @red;
//!             } !? { return; }
//!     }
//! }
//! ```

pub use banish_derive::banish;