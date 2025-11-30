# Chain: py_a -> py_b -> py_c
def py_a():
    py_b()

def py_b():
    py_c()

def py_c():
    print("leaf")

# Cycle: py_cycle_a -> py_cycle_b -> py_cycle_a
def py_cycle_a():
    py_cycle_b()

def py_cycle_b():
    py_cycle_a()

# Multiple callers
def py_target():
    print("target")

def py_caller1():
    py_target()

def py_caller2():
    py_target()
