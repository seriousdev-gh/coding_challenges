require_relative 'ru_numbers'
include RuNumbers

# examples
puts один равно два # => ложь
puts ⦅ сто двадцать три вычесть сто ⦆ умножить на десять # => двести тридцать
puts((один прибавить два умножить на три).to_i) # => 7
puts((один миллион два).to_i)
puts(тысяча два)
