require_relative 'garden_parser'
require_relative 'garden_solver'

screenshot = ARGV.first
raise 'Screenshot file is not provided' if screenshot.nil? || screenshot.empty?

detected_symbols = `python symbol_detector.py "#{screenshot}"`

garden = GardenParser.new.call(detected_symbols)
solved, solution = GardenSolver.new.call(garden)

if solved
    puts "Solved"
    solution.each { p _1 }
else
    puts "Solution not found"
end
