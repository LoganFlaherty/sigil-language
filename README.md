# Sigil
## Overview
A repo for the Sigil language, a signal oriented programming language designed around the idea of signal propagation rather than traditional function calls and control flow. The core premise is that execution is all top-level and driven by the reactive relationships between sources (signal variables) and sigils (a combined idea of a signal, function, and conditional statement).

## Recent Updates
- Heavily optimized the interpreter to only be ~2x slower than python equivalent code.
- Supports the basic data types (int, float, string literal, and bools).

## Design
Syntax:
- Sources "src {name}" are dynamic state holders and signal emitters. They are declared outside of sigils, typically near the top of your file.
    - Changing a source doesn’t implicitly trigger reactions (to avoid chaos). Reactions only occur through explicit invokes "invoke {name}" of either a source or a sigil.
- Sigils "sigil {name}" define when something should happen through a conditional statement started with "?", and if it evaluates true then it moves on to the body (after ":", newlined, and indented).
    ```
    sigil Print ? x != "" and y != "":
        invoke Whisper
    ```
    - Optionally you can define a sigil with no conditional, but it makes it only directly invokable and not through source invokes.
- Assignments use a colon ":".
    ```
    src x : "7"
    ```
- Comparisons allow the use of a single equals sign "=" or double. Note that "=" use require space around it.
- Built-in sigils (like Whisper) are defined inside the interpreter. However, unlike regular sigils, they can only be invoked inside a sigil due to arg passing restrictions. All built-in sigils can be found at the bottom of this README.
- Any invokes wrote outside of a sigil, is considered your run code and how you kickstart a program.
    ```
    invoke x
    ```
    - Additionally, you can invoke mutiple times on a single line. Both inside and out of a sigil.
        ```
        invoke Pulse, Whisper
        ```

Execution order:
- A source or sigil is invoked.
  - When invoking a source, all sigils dependent on it from it's conditional are then executed in the order they were defined in the file.
  - Invoked sigils are executed next in queue, allowing for direct linear flow.
- Program ends once the invoke queue has reached zero.

Interpretation:
- Wrote in Python 3.14.
    - Cmd: python run.py {file path} {optional args}
        - Option 'c' prints the runtime chain after execution.
        - Option 't' prints the execution time.
- Since sigil does not support nested logic so an AST is not needed for interpretation since it executes from a queue.
- During parsing, the invoke queue is filled with all explicit invokes.
- Sigils that will be invoked through a source invoke, are not put in the queue but interpreted at runtime.

Restrictions:
- Cannot declare new sources inside a sigil. Declare it right over it if it's meant to only be used there. It isn't supported directly, but more for readability.
- Cannot declare a sigil within a sigil.
- Cannot assign a sigil to a source.
- Cannot directly pass args to a built-in sigil.

## Goals
- Happy with this language prototype and will be working on a Rust version of the interpreter. From there new features will then be developed.

## Built-in Sigils
- Whisper: a print to standard output that implicitly takes in the args within conditional statement of the sigil Whisper was invoked.
    ```
    # Prints x and y joined
    sigil Print ? x and y:
        invoke Whisper
    ```
- Pulse: Requeues the sigil invoked in up to Pulse, until the conditional statement fails.
    ```
    # Will loop until x equals 5, then Whisper will print x
    sigil Loop ? x < 5:
        x : x + 1
        invoke Pulse
        invoke Whisper
    ```
