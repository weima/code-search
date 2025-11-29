# Sample Ruby file for testing call graph extraction

def process_order(order)
  validate_order(order)
  total = calculate_total(order)
  send_confirmation(order, total)
  total
end

def validate_order(order)
  raise 'Invalid order' if order.nil?
  true
end

def calculate_total(order)
  order.items.sum(&:price)
end

def send_confirmation(order, total)
  puts "Order confirmed: #{total}"
end

def main
  order = create_order
  process_order(order)
end
