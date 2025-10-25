# Sigil
The Sigil language is a Rust interpreted experimental event driven scripting language that abandons traditional methods of control flow. This is achieved through the core concepts of invokes (event triggers), sources (variables), sigils (a combination of a function and conditional statement), relationships, and a queue.

## Philosophy
Sigil aims to simplify control flow and encourage readable logical code.

## Execution
- During parsing, all invokes not within a sigil are put into the queue in order they appear and popped first in first out.
- Sigil bodies are not evaluated till they are invoked.
- Any invokes within a sigil body are injected next into the queue.
- Relationships work like this, sigils containing a source in their condition are then tied to it. When a source is invoked, every sigil in a relationship with it is then evaluated in the order they appear.
- When a sigil is invoked directly, they alone are evaluated.
- Program ends once the queue is empty.

## Syntax
- Invokes of either a source or a sigil are the main control flow mechanism.
    ```
    invoke x
    ```
    - Additionally, you can invoke mutiple times on a single line. Both inside and out of a sigil.
        ```
        invoke Pulse, Whisper
        ```
- Sources are dynamic variables. They are declared outside of sigils, typically near the top of the file.
    ```
    src x
    ```
    - Changing a source value doesn’t implicitly trigger a reaction. This is to provide more control and predictability.
- Sigils "sigil {name}" are rule like structures that define when something should happen through a conditional statement started with "?", and if it evaluates true then it moves on to the body (after ":", newlined, and indented).
    ```
    sigil Print ? x != "" and y != "":
        invoke Whisper
    ```
    - Optionally you can define a sigil with no conditional, but it makes it only directly invokable.
- Assignments use a colon ":".
    ```
    src x : 7
    ```
- Comparisons allow the use of a single equals sign "=" or double. Note that "=" use require space around it. Also the operators "and" and "or" are supported. 
- Built-in sigils (like Whisper) are defined inside the interpreter. However, unlike regular sigils, they can only be invoked inside a sigil due to arg passing restrictions. All built-in sigils can be found at the bottom of this README.

## Interpreter
- Wrote in the latest version of Rust. Meaning you must have the latest rust compiler installed https://rust-lang.org/tools/install/.
- Run cmd within a cargo project: 'cargo run {file path} {optional args}'.
    - Option '-c' prints the runtime chain after execution.
    - Option '-t' prints the execution time.
- Sigil's execution is all top-level.

## Restrictions
- Cannot declare new sources inside a sigil. Declare it right over it if it's meant to only be used there. It isn't supported directly, but more for readability.
- Cannot declare a sigil within a sigil.
- Cannot assign a sigil to a source.
- Cannot directly pass args to a built-in sigil.

## Built-in Sigils
- Whisper: a print to standard output that implicitly takes in the args within conditional statement.
    ```
    # Prints x and y
    sigil Print ? x and y:
        invoke Whisper
    ```
- Pulse: Requeues the sigil up to the Pulse invoke, until the conditional statement fails, then the rest of the body is executed.
    ```
    # Will loop until x equals 5, then Whisper will print x
    sigil Loop ? x < 5:
        x : x + 1
        invoke Pulse
        invoke Whisper
    ```
