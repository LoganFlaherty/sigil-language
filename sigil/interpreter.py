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
        self.key_type_table = {
            "src" : self.construct_src,
            "sigil" : self.construct_sigil,
            "invoke" : self.queue_invokes
        }
        
        self.built_in_sigils = {
            "Whisper" : self.whisper,
            "Pulse" : self.pulse
        }

        self.global_table = {}
        self.invoke_queue = []
        self.runtime_chain = []
        self.parse_flag = True
        self.last_sigil_popped = None
        self.pulse_flag = False
        self.line_i = 0

    def parse(self, program):
        lines = program.splitlines()
        while self.line_i < len(lines):
            line = lines[self.line_i]
            
            # Skip empty and comment only line
            if not line.strip() or line.strip().startswith("#"):
                self.line_i += 1
                continue
            
            # Remove inline comment
            line = line.split("#", 1)[0].rstrip()
                
            # Seperate type
            line =  line.split(" ", 1)

            # Try to map type
            type = line[0].strip()
            type_mapped = self.key_type_table.get(type)
            if type_mapped:
                type_mapped(lines, line[1].strip())
            else:
                raise Exception(f"{type} is unknown.")
            
            self.line_i += 1
        self.parse_flag = False

    def construct_sigil(self, lines, line):
        # Catch sigil inside a sigil
        if not self.parse_flag:
            raise Exception("Cannot declare a sigil inside a sigil.")

        # Extract name and optional conditional
        sigil_header = line.split(':')[0].split("?")
        name = sigil_header[0].strip()
        condition_expr = None
        src_deps = []
        if len(sigil_header) > 1:
            condition_expr = sigil_header[1].strip()

            # Change = into ==
            if "=" in condition_expr:
                condition_expr = re.sub(r'(?<=\s)=(?=\s)', '==', condition_expr)

            # Extract dependencies in order of appearance from condition_expr
            for e in condition_expr.split():
                e = e.strip()
                i_glob = self.global_table.get(e)
                if i_glob and i_glob.name not in src_deps:
                    src_deps.append(i_glob.name)
                    i_glob.sig_deps.append(name)


        # Collect body
        body = []
        i = self.line_i + 1
        while lines[i].startswith((" ", "\t")):
            body.append(lines[i].split("#", 1)[0].strip())
            i += 1
        
        self.global_table[name] = SigilDecl(name, condition_expr, body, src_deps)
        i -= 1
        self.line_i = i

    def construct_src(self, _, line):
        line = line.split(":")
        line_size = len(line)
        if line_size > 2:
            raise Exception("Cannot use : as source names or values.")
        
        name = line[0].strip()
        
        if line_size == 2:
            val = line[1].strip()
            if val.startswith('"') or val.startswith("'"):
                val = val[1:-1]
        else:
            val = None
        
        self.global_table[name] = SrcDecl(name, val)

    def queue_invokes(self, _, line, sigil = None):
        targets = line.split(",")
        for target in targets:
            target = target.strip()
            if target not in self.built_in_sigils:
                self.invoke_queue.append(target)
            # Already know target is a built-in from previous if
            elif not self.parse_flag and sigil:
                args = []
                for dep in sigil.src_deps:
                    args.append(self.global_table[dep].value)
                self.built_in_sigils[target](*args)

                # When Pulse is active, cut off the rest of the body
                if self.pulse_flag:
                    return
            else:
                raise Exception(f"Cannot invoke {target} outside of a sigil.")
    
    def run(self):
        while self.invoke_queue:
            target_name = self.invoke_queue.pop(0)
            self.runtime_chain.append(target_name)
            target = self.global_table.get(target_name)
            if isinstance(target, SrcDecl):
                # Evaluate all dependent sigils
                for sig_dep in target.sig_deps:
                    sigil = self.global_table[sig_dep]
                    if sigil.condition_expr and self.eval_expr(sigil.condition_expr, sigil.src_deps):
                        self.runtime_chain.append(sigil.name)
                        self.execute_sigil(sigil) 
            elif isinstance(target, SigilDecl):
                self.last_sigil_popped = target
                self.execute_sigil(target)
            elif target_name in self.built_in_sigils:
                self.built_in_sigils[target]()
            else:
                raise Exception(f"Unknown target {target}.")

    def eval_expr(self, expr, src_deps = None):
        # Build local cache
        expr_cache = {}
        if src_deps:
            for dep in src_deps:
                expr_cache[dep] = self.global_table.get(dep).value
        else:
            for name, obj in self.global_table.items():
                if isinstance(obj, SrcDecl):
                    expr_cache[name] = obj.value
        
        # Evaluate with locals
        try:
            return eval(expr, {"__builtins__": None}, expr_cache)
        except Exception:
            return None

    def execute_sigil(self, sigil):
        for line in sigil.body:
            line_split = line.split(" ", 1)
            target = self.key_type_table.get(line_split[0])
            if target:
                target(None, line_split[1].strip(), sigil)
            elif ":" in line:
                src, expr = line.split(":", 1)
                src = src.strip()
                expr = expr.strip()
                value = self.eval_expr(expr)
                self.global_table[src].value = value


    ## Built-in Sigils

    # Prints to stdout
    def whisper(self, *args):
        print("".join(str(arg) for arg in args))

    # Loops the body of the sigil up to Pulse
    def pulse(self, *_):
        if not self.pulse_flag:
            self.pulse_flag = True
        
        sigil = self.last_sigil_popped
        if not sigil:
            raise Exception("Pulse has nothing to queue.")

        if sigil.condition_expr and self.eval_expr(sigil.condition_expr, sigil.src_deps):
            # Requeue sigil
            self.invoke_queue.append(sigil.name)
        else:
            self.pulse_flag = False


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
    intr.run()
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