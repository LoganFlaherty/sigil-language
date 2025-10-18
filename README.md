# sigil-language
## Overview
A repo for the sigil language, a signal oriented programming language designed around the idea of signal propagation rather than traditional function calls and control flow. The core premise is that execution is driven by reactive relationships between sources (signal variables) and sigils (a combined idea of a signal, function, and conditional statement) and how they are invoked. The execution flow is akin to a reactive graph. 

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
- Comparisons use a single equals sign "=".
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
  - When invoking a source, all sigils dependent on it in it's conditional are then executed in the order they were defined in the file.
  - Invoked sigils are executed next in queue, allowing for direct linear flow.
- Program ends once the invoke queue has reached zero.

Interpretation:
- Wrote in Python, so Python is needed to run the interpreter.
    - Cmd: python interpreter.py {file path} {optional args}
        - Option 'c' prints the runtime chain after execution.
        - Option 't' prints the execution time.
- Since sigil, at this time, does not support nested logic an AST is not required to have it interpreted i.e. it is all top-level.
- During parsing, the invoke queue is filled with all explicit invokes.
- Sigils that will be invoked through a source invoke, are not put in the queue but interpreted at runtime.

Limitations:
- Can only handle strings.
- Cannot declare new sources inside a sigil. Declare it right over it if it's meant to only be used there. It isn't supported directly, but more for readability.

## Goals
I would like to continue to develop sigil further by including all standard data types and built in functions. A milestone project goal with sigil is be to be able to built a fully functional calculator minus graphs.

Once sigil is about 1.0 ready, then a c interpreter will be developed.

## Built-in Sigils
- Whisper: a print to standard output that implicitly takes in the args with in conditional statement of the sigil Whisper was invoked. Does not support explitic arg passing yet.
    ```
    sigil Print ? x and y:
        invoke Whisper
    ```
- Pulse: a loop handler that will requeue the sigil it is invoked in, up to till it is invoked, until the conditional statement fails.
    ```
    # Loop will be pulsed until x equals 5, then Whisper will print x.
    sigil Loop ? x < 5:
        x : x + 1
        invoke Pulse
        invoke Whisper
    ```
