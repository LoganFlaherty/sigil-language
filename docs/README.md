## Introduction
As stated before Banish is an excellent DSL to write state-machines or have easy to read conditional logic. 
Given Banish's small size, this guide will be realatively short, but feel free to post in Discussions if you have any input or questions.

## Syntax
- **@state** : Defines a state that loops until no rules trigger or a state transition. States execute from top to bottom.
- **rule ? condition {}** : Defines a rule. Executes if its condition is true. Rules execute from top to bottom.
- **!? {}** : Defines an else clause after the closing brace of a rule with a condition.
- **rule ? {}** : A rule without a condition. Executes exactly once per state entry. Cannot have an else clause.
- **=> @state;** : Transitions immediately to another state, but is a rule top-level statement only.
- **return value;** : Immediately exit banish and return a value if passed.

## Examples
### Hello World
Naturally, have to show the classics.
```rust
use banish::banish;

fn main() {
    banish! {
        @hello
            print? {
              println!("Hello, world!");
              return;
            }
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

           stop ? loop_count == 2 { return; }
    }
}
```

### Dragon Fight
This example demostrates a little bit more complex logic such as early returning with a value to be used later, using an external library within Banish, and else clauses.
```rust
use banish::banish;
use rand::prelude::*;

fn main() {
    let mut rng = rand::rng();
    let mut player_hp = 20;
    let mut dragon_hp = 50;
    
    println!("BATTLE START");

    let result: &str = banish! {
        @player_turn
            // Conditionless Rule: Player attacks dragon
            attack ? {
                let damage = rng.random_range(5..15); // Using external lib!
                dragon_hp -= damage;
                println!("You hit the dragon for {} dmg! (Dragon HP: {})", damage, dragon_hp);
            }

            check_win ? dragon_hp <= 0 {
                return "Victory!"; // Early exit with value
            }

            end_turn ? {
                => @dragon_turn; // Explicit transition else player just keeps attacking forever
            }

        @dragon_turn
            attack ? {
                let damage = rng.random_range(2..20);
                player_hp -= damage;
                println!("Dragon breathes fire for {} dmg! (Player HP: {})", damage, player_hp);
            }

            halfway ? player_hp <= 10 && dragon_hp <= 25 {
                println!("\nThe battle is getting intense!\n");
            } !? { println!("\nThe dragon is getting weak!\n"); } // Else clause

            check_loss ? player_hp <= 0 {
                return "Defeat...";
            }

            end_turn ? {
                => @player_turn;
            }
    };

    // Handle the returned result
    println!("GAME OVER: {}", result)
}
```
