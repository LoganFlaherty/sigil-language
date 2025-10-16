# sigil-language
## Overview
A repo for the sigil, a signal oriented programming language designed around the idea of signal propagation rather than traditional function calls or control flow. The core premise is that execution is driven by reactive relationships between sources (signal variables) and sigils (a combined idea of a signal, function, and conditional statement) and how they are invoked. The execution flow is akin to a reactive graph. 

## Design
Syntax:
- Sources "src" are state holders and signal emitters.
    - Changing a source doesn’t implicitly trigger reactions (to avoid chaos). Reactions only occur through explicit invokes "invoke" of either a source or a sigil.
- Sigils "sigil" define when something should happen through a conditional statement started with "?", and if it evaluates true then it moves on to the body (after ":", newlined, and indented).
    - For example: sigil Print ? x != "" and y != "":
                     invoke Whisper
    - Optionally you can define a sigil with no conditional, but it makes it only invokable directly and not through source invokes.
- Assignments use a colon ":".
    - For example: src x : "7"
- Comparisons use a single equals sign "=".
- Built-in sigils (like Whisper) are defined inside the interpreter, so no need to define in a file. All built-in sigils can be found at the bottom of this README.

Execution order:
- Either a source or sigil is invoked.
  - When invoking a source, all sigils with that source in its conditional are then executed in the order they were defined in the file.
  - Invoked sigils are executed next in queue, allowing recursion and looping.
- Program ends once the invoke queue has reached zero.

Interpretation:
- Wrote in Python, so Python is needed to run the interpreter.
    - Cmd: python interpreter.py {file path} {Optional: y (determines if the runtime chain is printed to stdout after the program ends.)}
- Since sigil, at this time, does not support nested logic an AST is not required to have it interpreted i.e. it is all top-level.
- During parsing, the invoke queue is filled with all explicit invokes.
- Sigils that will be invoked through a source invoke, are not put in the queue but interpreted at runtime.

Limitations:
- Can only handle strings.
- Cannot declare new sources inside a sigil. Declare it right over it if it's meant to only be use there. It isn't supported directly, but more for readability.

## Goals
I would like to continue to develop sigil further by including all standard data types and built in functions. A milestone project goal with sigil is be to be able to built a fully functional calculator minus graphs.

Once sigil is about 1.0 ready, then a c interpreter will be developed.

## Built-in Sigils
- Whisper: a print to standard output that implicitly takes in the args with in conditional statement of the sigil Whisper was invoked. Does not support explitic arg passing yet.
    - For example: sigil Print ? x and y:
                       invoke Whisper
