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
        self.built_in_sigils = {"Whisper": self.whisper}
        self.invoke_queue = []
        self.runtime_chain = []

    # Defined built-in sigils:

    def whisper(self, *args):
        print("".join(str(a) for a in args))

    # Rest of interpreter:

    def parse(self, program):
        lines = [l.rstrip() for l in program.strip().splitlines() if l.strip() and not l.strip().startswith("#")]
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            if line.startswith("src "):
                name, val = line[4:].split(":", 1)
                self.src_table[name.strip()] = val.strip().strip('"')
            elif line.startswith("sigil "):
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
                i -= 1
                self.sigil_table[name] = SigilDecl(name, condition_expr, body)
            elif line.startswith("invoke "):
                targets = [t.strip() for t in line[7:].split(",")]
                for t in targets:
                    self.invoke_queue.append(t)
            i += 1

    def eval_expr(self, expr):
        expr_eval = expr

        # Convert comparison '=' to '==' for Python eval
        # Only replace '=' that are not part of !=, >=, <=, or ==
        expr_eval = re.sub(r'(?<![!<>=])=(?!=)', '==', expr_eval)

        # Replace sources with their values
        for k, v in self.src_table.items():
            expr_eval = re.sub(rf"\b{k}\b", f"'{v}'", expr_eval)
        
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
                for s in self.sigil_table.values():
                    if s.condition_expr and target in s.deps:
                        cond = self.eval_expr(s.condition_expr)
                        if cond:
                            self.runtime_chain.append(s.name)
                            self.execute_sigil(s)
            elif target in self.sigil_table:
                self.execute_sigil(self.sigil_table[target])
            elif target in self.built_in_sigils:
                self.built_in_sigils[target]()
            else:
                print(f"[Err] Unknown target {target}")

    def execute_sigil(self, sigil):
        for stmt in sigil.body:
            if stmt.startswith("invoke "):
                name = stmt.split(" ", 1)[1].strip()
                if name in self.built_in_sigils:
                    args = [self.src_table[dep] for dep in sigil.deps if dep in self.src_table]
                    self.built_in_sigils[name](*args)
                else:
                    self.invoke_queue.append(name)
            elif ":" in stmt:
                name, expr = stmt.split(":", 1)
                name = name.strip()
                expr = expr.strip()

                # Evaluate expression before assignment
                value = self.eval_expr(expr)

                # Store in sources table
                self.src_table[name] = value


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




