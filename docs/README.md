## Introduction
As stated before Banish is an excellent DSL to write state-machines or have clean iterative logic without a nested mess, and the best part is that you can still interact with Rust in and out of it. 
Given Banish's small size, this guide will be realatively short, but feel free to post in Discussions if you have any input or questions.

## Syntax
- **@phase**: Defines a state. Phases run from top to bottom evaluating rules in declaration order, and repeat until no rules trigger or a phase jump occurs.
- **rule ? condition {}**: Runs logic if the condition is true.
- **rule? {}**: Without a condition, runs exactly once per phase entry.
- **=> @next;**: Transitions immediately to another phase, but is a top-level statement within a rule block only.

## Examples
### Hello World
Naturally, have to show the classics.
```rust
use banish::banish;

fn main() {
  banish! {
        @hello
            print? { println!("Hello, world!"); }
    }
}
```

### Traffic Lights
This demostration is a basic example to show off the transitions of phases and how to think about control flow in Banish.
```rust
use banish::banish;

fn main() {
   let mut ticks: i32 = 0;
   let mut loop_count: i32 = 0;
   banish! {
       @red
           announce ? {
               ticks = 0;
               println!("Red light");
               loop_count += 1;
            }

            timer ? ticks < 3 {
                ticks += 1;
           }

       @green
           announce ? {
               println!("Green light");
           }

           timer ? ticks < 6 {
               ticks += 1;
           }

       @yellow
           announce ? {
               println!("Yellow light");
           }

           timer ? ticks < 10 {
               ticks += 1;
           }

           reset ? ticks == 10 && loop_count < 2 {
               => @red;
           }
          
       @stop
           announce ? {
               println!("Stopping traffic lights...");
           }
    }
}
```
