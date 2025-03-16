require_relative 'garden_parser'
require_relative 'garden_solver'

# TODO: implement solver for Sigmars Garden 2 (new element - essence)

def print_usage_end_exit
    puts "Usage: ruby main.rb autosolve <screenshot_folder>"
    puts "       ruby main.rb solve <screenshot_file>"
    exit 1
end

if ARGV.size == 0
    print_usage_end_exit
end

if ARGV[0] == 'autosolve'
    unless ARGV[1]
        puts "ERROR: Screenshot folder is not provided\n"
        print_usage_end_exit
    end
    require_relative 'autosolver'
    Autosolver.new(ARGV[1]).call
    exit 0
end

if ARGV[0] == 'solve'
    unless ARGV[1]
        puts "ERROR: Screenshot file is not provided\n"
        print_usage_end_exit
    end

    detected_symbols = `python symbol_detector.py "#{ARGV[1]}" debug`
    parser = GardenParser.new(detected_symbols).call
    solved, solution = GardenSolver.new.call(parser.garden)

    if solved
        puts "Solved"
        solution.each { p _1 }
    else
        puts "Solution not found"
    end
end
