import os
import sys
import time
import interpreter


### Run sigil interpreter

args = len(sys.argv)
if args < 2:
    raise Exception("File path not passed.")

path = sys.argv[1]
if os.path.exists(path):
    with open(path, 'r') as file:
        file = file.read()

    start_time = time.perf_counter()
    inter = interpreter.Interpreter()
    inter.parse(file)
    inter.invoke()
    end_time = time.perf_counter()
    
else:
    raise Exception(f"'{path}' does not exist.")

i = 0
while i < args:
    if i < 2:
        pass
    elif sys.argv[i] == "c":
        print(f"Runtime chain: {inter.runtime_chain}")
    elif sys.argv[i] == "t":
        elapsed_time = end_time - start_time
        print(f"Execution time: {elapsed_time:.4f} seconds")
    else:
        raise Warning(f"{sys.argv[i]} is not a valid arg.")
    i += 1