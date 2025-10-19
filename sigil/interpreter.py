import re
import os
import sys
import time


## Classes

class SrcDecl:
    def __init__(self, name, value):
        self.name = name
        self.value = value
        self.sig_deps = [] # All sigils dependent on this source

class SigilDecl:
    def __init__(self, name, condition_expr, body, src_deps):
        self.name = name
        self.condition_expr = condition_expr
        self.body = body
        self.src_deps = src_deps # All sources it dependents on in it's conditional

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
                        raise Exception(f"Cannot invoke {target} outside of a sigil.")
                    else:
                        self.invoke_queue.append(target)
            i += 1

    def eval_expr(self, expr, src_deps = None):
        # Replace = that are not part of !=, >=, <=, or == with ==
        expr = re.sub(r'(?<![!<>=])=(?!=)', '==', expr)
        
        # Replace sources with their values
        if src_deps:
            for dep in src_deps:
                src = self.src_table[dep]
                expr = re.sub(rf"\b{src.name}\b", f"'{src.value}'", expr)
        else:
            for src in self.src_table.values():
                expr = re.sub(rf"\b{src.name}\b", f"'{src.value}'", expr)
        
        return eval(expr)
    
    def run(self):
        while self.invoke_queue:
            target = self.invoke_queue.pop(0)
            self.runtime_chain.append(target)
            if target in self.src_table:
                # Evaluate all conditional sigils
                for sig_dep in self.src_table[target].sig_deps:
                    sigil = self.sigil_table[sig_dep]
                    if sigil.condition_expr and self.eval_expr(sigil.condition_expr, sigil.src_deps):
                        self.runtime_chain.append(sigil.name)
                        self.execute_sigil(sigil) 
            elif target in self.sigil_table:
                self.last_sigil_popped = target
                self.execute_sigil(self.sigil_table[target])
            elif target in self.built_in_sigils:
                self.built_in_sigils[target]()
            else:
                raise Exception(f"Unknown target {target}.")

    def execute_sigil(self, sigil):
        for stmt in sigil.body:
            if stmt.startswith("invoke "):
                targets = get_invoke_targets(stmt)
                for target in targets:
                    route_invokes(self, target, sigil)
                    
                    # When Pulse is active, cut off the rest of the body
                    if self.pulse_flag:
                        return
            elif ":" in stmt:
                src, expr = stmt.split(":", 1)
                src = src.strip()
                expr = expr.strip()
                value = self.eval_expr(expr)
                self.src_table[src].value = value


    ## Built-in Sigils

    # Prints to stdout
    def whisper(self, *args):
        print("".join(str(arg) for arg in args))

    # Loops the body of the sigil invoked in until the the condition fails
    def pulse(self):
        sigil = self.sigil_table.get(self.last_sigil_popped)
        if not sigil:
            raise Exception("Pulse has nothing to queue.")

        if sigil.condition_expr and self.eval_expr(sigil.condition_expr, sigil.src_deps):
            # Requeue sigil
            self.invoke_queue.append(sigil.name)
        else:
            self.pulse_flag = False


## Helpers

def construct_sigil(self, lines, header, i):
    # Extract name and optional conditional
    sigil_header = header[6:].split(':')[0].split("?")
    name = sigil_header[0].strip()
    condition_expr = None
    if len(sigil_header) > 1:
        condition_expr = sigil_header[1].strip()
    
    # Collect body
    body = []
    i += 1
    while lines[i].startswith((" ", "\t")):
        body.append(lines[i].strip())
        i += 1
    
    # Extract dependencies in order of appearance from condition_expr
    src_deps = []
    if condition_expr:
        possible = re.findall(r"\b[A-Za-z_]\w*\b", condition_expr)
        for find in possible:
            if find not in src_deps and find in self.src_table:
                src_deps.append(find)
                self.src_table[find].sig_deps.append(name)
    
    self.sigil_table[name] = SigilDecl(name, condition_expr, body, src_deps)
    i -= 1
    return i

def construct_src(self, line):
    line = line[4:].strip()
    if ":" in line:
        name, val = line.split(":", 1)
        name = name.strip()
        val = val.strip()
        if val.startswith('"') or val.startswith("'"):
            val = val[1:-1]
    else:
        name = line[4:].strip()
        val = None
    src = SrcDecl(name, val)
    self.src_table[name] = src

def deconstruct_program(program):
    lines = []
    for line in program.strip().splitlines():
        line = line.split("#", 1)[0].rstrip()
        if line:
            lines.append(line.rstrip())
    return lines

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
        args = []
        for dep in sigil.src_deps:
            args.append(self.src_table[dep].value)
        self.built_in_sigils[target](*args)
    else:
        self.invoke_queue.append(target)


## Run Interpreter

start_time = time.perf_counter()

args = len(sys.argv)
if args < 2:
    raise Exception("File path not passed.")

path = sys.argv[1]
if os.path.exists(path):
    with open(path, 'r') as file:
        file = file.read()

    intr = Interpreter()
    intr.parse(file)
    intr.run()
    
else:
    raise Exception(f"'{path}' does not exist.")

end_time = time.perf_counter()

i = 0
while i < args:
    if i < 2:
        pass
    elif sys.argv[i] == "c":
        print(f"Runtime chain: {intr.runtime_chain}")
    elif sys.argv[i] == "t":
        elapsed_time = end_time - start_time
        print(f"Execution time: {elapsed_time:.4f} seconds")
    else:
        raise Warning(f"{sys.argv[i]} is not a valid arg.")

    i += 1
