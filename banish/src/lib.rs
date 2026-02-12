//! # Banish
//!
//! An easy to use DSL for creating state machines and fixed-point loops in Rust.
//! It allows you to define "Phases" and "Rules" that interact until they settle or transition.
//!
//! ## Syntax
//!
//! - **@phase**: Defines a state. States run from top to bottom, and repeat until no rules trigger or a phase jump occurs.
//! - **rule ? condition**: Runs logic if the condition is true.
//! - **rule?**: Without a condition, runs exactly once per phase entry.
//! - **=> @next**: Transitions immediately to another phase, but is a top-level statement within a rule block only.
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
//!         @stop
//!             announce ? {
//!                 println!("Stopping traffic light simulation...");
//!             }
//!     }
//! }
//! ```

pub use banish_derive::banish;