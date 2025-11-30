# Chain: rb_a -> rb_b -> rb_c
def rb_a
  rb_b()
end

def rb_b
  rb_c()
end

def rb_c
  puts "leaf"
end

# Cycle: rb_cycle_a -> rb_cycle_b -> rb_cycle_a
def rb_cycle_a
  rb_cycle_b()
end

def rb_cycle_b
  rb_cycle_a()
end

# Multiple callers
def rb_target
  puts "target"
end

def rb_caller1
  rb_target()
end

def rb_caller2
  rb_target()
end
