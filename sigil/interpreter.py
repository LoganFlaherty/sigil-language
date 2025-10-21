import os
import sys
import time


## Classes

class Source:
    def __init__(self, name, value):
        self.name = name
        self.value = value
        self.sig_deps = [] # All sigils dependent on this source

class Sigil:
    def __init__(self, name, condition_expr, body, src_deps):
        self.name = name
        self.condition_expr = condition_expr
        self.body = body
        self.src_deps = src_deps # All sources it dependents on in it's conditional

class BuiltInSigil:
    def __init__(self, name, func, interp):
        self.name = name
        self.func = getattr(self, func)
        self.interp = interp
        self.in_sigil = None

    # Prints to stdout
    def whisper(self, *args):
        print("".join(str(arg) for arg in args))

    # Loops the body of the sigil up to Pulse
    def pulse(self, *_):
        if not self.in_sigil:
            raise Exception("Pulse has nothing to queue.")

        if self.in_sigil.condition_expr and self.interp.eval_expr(self.in_sigil.condition_expr):
            # Requeue sigil
            self.interp.invoke_queue[0] = self.in_sigil.name

class Interpreter:
    def __init__(self):
        self.key_type_table = {
            "src" : self.construct_src,
            "sigil" : self.construct_sigil,
            "invoke" : self.queue_invokes
        }

        self.global_table = {
            "Whisper" : BuiltInSigil("Whisper", "whisper", self),
            "Pulse" : BuiltInSigil("Pulse", "pulse", self)
        }

        self.src_cache = {}
        self.invoke_queue = []
        self.runtime_chain = []
        self.line_i = 0

    def parse(self, program):
        lines = program.split("\n")
        lines_len = len(lines)
        while self.line_i < lines_len:
            line = lines[self.line_i]

            # Skip empty lines and comments
            is_line = line.lstrip()
            if not is_line or is_line.startswith("#"):
                self.line_i += 1
                continue
            
            # Remove inline comment
            line = line.split("#", 1)[0].rstrip()
                
            # Seperate type
            line =  line.split(" ", 1)

            # Try to map type
            keyword = line[0].strip()
            type = self.key_type_table.get(keyword)
            if type:
                type(lines, line[1].strip())
            else:
                raise Exception(f"{keyword} is an unknown keyword.")
            
            self.line_i += 1

    def construct_sigil(self, lines, line):
        # Extract name
        sigil_header = line.split(':')[0].split("?")
        name = sigil_header[0].strip()

        # Catch built-in sigil overrides
        if name in ("Whisper", "Pulse"):
            raise Exception(f"Cannot override built-in {name}.")

        # Extract optional conditional statement
        condition_expr = None
        src_deps = []
        if len(sigil_header) > 1:
            condition_expr = sigil_header[1].strip()
            condition_expr = condition_expr.replace(" = ", " == ")

            # Extract dependencies in order of appearance from condition_expr
            for dep in condition_expr.split():
                dep = self.global_table.get(dep)
                if isinstance(dep, Source) and dep.name not in src_deps:
                    src_deps.append(dep.name)
                    dep.sig_deps.append(name)

        # Collect body
        body = []
        i = self.line_i + 1
        lines_len = len(lines)
        while i < lines_len and lines[i].startswith((" ", "\t")):
            body.append(lines[i].split("#", 1)[0].strip())
            i += 1
        
        # Finish construct
        self.global_table[name] = Sigil(name, condition_expr, body, src_deps)
        i -= 1
        self.line_i = i

    def construct_src(self, _, line):
        line = line.split(":", 1)
        name = line[0].strip()
        
        if len(line) == 2:
            val = line[1].strip()
            val = self.parse_value(val)
        else:
            val = None
        
        self.global_table[name] = Source(name, val)
        self.src_cache[name] = val

    def queue_invokes(self, _, line, sigil = None):
        targets = line.split(",")
        for target in targets:
            target = target.strip()
            # Catch trailing comma split
            if not target:
                continue

            node = self.global_table.get(target)
            if isinstance(node, (Sigil, Source)):
                self.invoke_queue.append(node.name)
            elif isinstance(node, BuiltInSigil):
                if sigil:
                    node.in_sigil = sigil
                    self.invoke_queue.append(node.name)
                else:
                    raise Exception(f"'{node.name}' can only be invoked inside a sigil.")
            else:
                raise Exception(f"{node.name} is not a valid invoke.")
    
    def invoke(self):
        while self.invoke_queue:
            target = self.invoke_queue.pop(0)
            self.runtime_chain.append(target)
            node = self.global_table.get(target)
            if isinstance(node, Source):
                # Invoke all dependent sigils
                for sig_dep in node.sig_deps:
                    sigil = self.global_table[sig_dep]
                    if sigil.condition_expr and self.eval_expr(sigil.condition_expr):
                        self.runtime_chain.append(sigil.name)
                        self.execute_sigil(sigil)
            elif isinstance(node, Sigil):
                self.execute_sigil(node)
            elif isinstance(node, BuiltInSigil):
                args = []
                for dep in sigil.src_deps:
                    args.append(self.global_table[dep].value)
                node.func(*args)
            else:
                raise Exception(f"Unknown target {target.name}.")

    def execute_sigil(self, sigil):
        for line in sigil.body:
            line_split = line.split(" ", 1)
            type = line_split[0].strip()
            if type == "invoke":
                self.key_type_table.get(type)(None, line_split[1].strip(), sigil)
            elif ":" in line:
                src, expr = line.split(":", 1)
                src = src.strip()
                expr = expr.strip()

                # Check if src exists before assigning
                node = self.global_table.get(src)
                if isinstance(node, Source):
                    value = self.eval_expr(expr)
                    node.value = value
                    self.src_cache[node.name] = value
                else:
                    raise Exception(f"Attempted to assign to invalid object '{node.name}'.")
            else:
                raise Exception(f"Invalid statement in sigil '{sigil.name}': {line}")

    def eval_expr(self, expr):
        try:
            return eval(expr, {"__builtins__": None}, self.src_cache)
        except Exception as e:
            raise Exception(f"Error evaluating expression '{expr}': {e}")

    def parse_value(self, val):
        # String literals
        if val.startswith('"') or val.startswith("'"):
            val = val[1:-1]
            return val
        
        # Bools
        if val == "true":
            return True
        if val == "false":
            return False
        
        # Nums
        try:
            return int(val)
        except:
            pass
        try:
            return float(val)
        except:
            pass

        raise Exception(f"{val} is not a valid value.")


## Run Interpreter

args = len(sys.argv)
if args < 2:
    raise Exception("File path not passed.")

path = sys.argv[1]
if os.path.exists(path):
    with open(path, 'r') as file:
        file = file.read()

    start_time = time.perf_counter()
    intr = Interpreter()
    intr.parse(file)
    intr.invoke()
    end_time = time.perf_counter()
    
else:
    raise Exception(f"'{path}' does not exist.")

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