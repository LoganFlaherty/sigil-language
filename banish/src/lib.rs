//! # Banish
//! An easy to use declarative DSL for creating state machines and rules-base logic. 
//! It allows you to define "States" and "Rules" that execute until they reach a fixed point or transition.
//!
//! ## Syntax
//! - **@state** : Defines a state that loops until no rules trigger or a state transition. States execute from top to bottom.
//! - **rule ? condition {}** : Defines a rule. Executes if its condition is true. Rules execute from top to bottom.
//! - **!? {}** : Defines an else clause after the closing brace of a rule with a condition.
//! - **rule ? {}** : A rule without a condition. Executes exactly once per state entry. Cannot have an else clause.
//! - **=> @state;** : Transitions immediately to another state, but is a rule top-level statement only.
//! - **return value;** : Immediately exit banish and return a value if passed.
//!
//! ## Example
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
//!             }
//! 
//!             stop ? loop_count == 2 { return; }
//!     }
//! }
//! ```

pub use banish_derive::banish;