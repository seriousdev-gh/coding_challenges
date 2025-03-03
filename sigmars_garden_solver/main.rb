require_relative 'garden_parser'
require_relative 'garden_solver'


garden = GardenParser.new.call(nil)
solved, solution = GardenSolver.new.call(garden)

if solved
    puts "Solved"
    solution.reverse.each { p _1 }
else
    puts "Solution not found"
end
