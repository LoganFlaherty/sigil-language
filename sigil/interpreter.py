import re
import os
import sys


## Classes

class SrcDecl:
    def __init__(self, name, value):
        self.name = name
        self.value = value

class SigilDecl:
    def __init__(self, name, condition_expr, body):
        self.name = name
        self.condition_expr = condition_expr
        self.body = body

        # Extract dependencies in order of appearance from condition_expr
        self.deps = re.findall(r'\b[a-zA-Z_][a-zA-Z0-9_]*\b', condition_expr) if condition_expr else []

class Interpreter:
    def __init__(self):
        self.src_table = {}
        self.sigil_table = {}
        self.built_in_sigils = {
            "Whisper": self.whisper,
            "Pulse" : self.pulse
        }
        self.invoke_queue = []
        self.runtime_chain = []
        self.last_sigil_popped = None
        self.pulse_flag = False

    def parse(self, program):
        lines = deconstruct_program(program)
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            if line.startswith("src "):
                construct_src(self, line)
            elif line.startswith("sigil "):
                i = construct_sigil(self, lines, line, i)
            elif line.startswith("invoke "):
                targets = get_invoke_targets(line)
                for target in targets:
                    if target in self.built_in_sigils:
                        print(f"[Warn] cannot invoke {target} outside of a sigil.")
                    else:
                        self.invoke_queue.append(target)
            i += 1

    def eval_expr(self, expr):
        # Convert comparison '=' to '==' for Python eval
        # Only replace '=' that are not part of !=, >=, <=, or ==
        expr_eval = re.sub(r'(?<![!<>=])=(?!=)', '==', expr)

        # Replace sources with their values
        for key, val in self.src_table.items():
            expr_eval = re.sub(rf"\b{key}\b", f"'{val}'", expr_eval)
        
        try:
            return eval(expr_eval)
        except Exception:
            return None
    
    def run(self):
        while self.invoke_queue:
            target = self.invoke_queue.pop(0)
            self.runtime_chain.append(target)
            if target in self.src_table:
                # Evaluate all conditional sigils
                for sigil in self.sigil_table.values():
                    if sigil.condition_expr and target in sigil.deps and self.eval_expr(sigil.condition_expr):
                        self.runtime_chain.append(sigil.name)
                        self.execute_sigil(sigil)
            elif target in self.sigil_table:
                self.last_sigil_popped = target
                self.execute_sigil(self.sigil_table[target])
            elif target in self.built_in_sigils:
                self.built_in_sigils[target]()
            else:
                print(f"[Err] Unknown target {target}.")

    def execute_sigil(self, sigil):
        for stmt in sigil.body:
            if stmt.startswith("invoke "):
                if "," in stmt:
                    targets = get_invoke_targets(stmt)
                    for target in targets:
                        route_invokes(self, target, sigil)
                        
                        # When Pulse is active, cut off the rest of the body
                        if self.pulse_flag:
                            return
                else:
                    target = stmt.split(" ", 1)[1].strip()
                    route_invokes(self, target, sigil)

                    # When Pulse is active, cut off the rest of the body
                    if self.pulse_flag:
                        return
                    
            elif ":" in stmt:
                src, expr = stmt.split(":", 1)
                src = src.strip()
                expr = expr.strip()
                value = self.eval_expr(expr)
                self.src_table[src] = value


    ## Built-in Sigils

    # Prints to stdout
    def whisper(self, *args):
        print("".join(str(arg) for arg in args))

    # Loops the body of the sigil invoked in until the the condition fails
    def pulse(self):
        sigil = self.sigil_table.get(self.last_sigil_popped)
        if not sigil:
            print(f"[Err] Pulse has nothing to queue.")
            return

        if sigil.condition_expr and self.eval_expr(sigil.condition_expr):
            # Requeue sigil
            self.invoke_queue.append(sigil.name)
        else:
            self.pulse_flag = False


## Helpers

def construct_src(self, line):
    try:
        name, val = line[4:].split(":", 1)
        val = val.strip().strip('"')
    except:
        name = line[4:]
        val = None
    self.src_table[name.strip()] = val

def construct_sigil(self, lines, line, i):
    # Extract name and optional conditional
    rest = line[6:]
    name = rest.split("?")[0].split(":")[0].strip()
    condition_expr = None
    if "?" in line:
        condition_expr = line.split("?", 1)[1].split(":", 1)[0].strip()
    
    # Collect body
    body = []
    i += 1
    while i < len(lines) and lines[i].startswith((" ", "\t")):
        body.append(lines[i].strip())
        i += 1
    self.sigil_table[name] = SigilDecl(name, condition_expr, body)
    i -= 1
    return i

def deconstruct_program(program):
    lines = []
    for line in program.strip().splitlines():
        if line.strip() and not line.strip().startswith("#"):
            lines.append(line.rstrip())
    return lines

def get_sigil_dependencies(self, sigil):
    dependencies = []
    for dep in sigil.deps:
        if dep in self.src_table:
            dependencies.append(self.src_table[dep])
    return dependencies

def get_invoke_targets(line):
    targets = []
    for target in line[7:].split(","):
        targets.append(target.strip())
    return targets

def route_invokes(self, target, sigil):
    if target == "Pulse":
        self.pulse_flag = True
        self.built_in_sigils[target]()
    elif target in self.built_in_sigils:
        args = get_sigil_dependencies(self, sigil)
        self.built_in_sigils[target](*args)
    else:
        self.invoke_queue.append(target)


## Run Interpreter

args = len(sys.argv)
if args < 2:
    print("[Err] File path not passed.")
    sys.exit(1)
elif args > 3:
    print("[Err] Too many args. Only pass a file path and optionally 'y' to print runtime chain option.")

path = sys.argv[1]
if os.path.exists(path):
    with open(path, 'r') as file:
        file = file.read()

    intr = Interpreter()
    intr.parse(file)
    intr.run()

    if args == 3 and sys.argv[2] == "y":
        print(f"[Runtime chain] {intr.runtime_chain}")
else:
    print(f"[Err] '{sys.argv}' does not exist.")
    sys.exit(1)