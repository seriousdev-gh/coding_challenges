require_relative 'garden_parser'
require_relative 'garden_solver'

# TODO: refactor this file
# TODO: refactor method hex_to_screen in garden_parser.rb
# TODO: implement solver for Sigmars Garden 2 (new element - essence)

if ARGV.first == 'wait_and_solve'
    require 'auto_click'
    ac = AutoClick.new()
    initial_files = Dir['C:/Users/boris/Pictures/Screenshots/*']

    loop do
        sleep 0.5
        current_files = Dir['C:/Users/boris/Pictures/Screenshots/*']
        new_files = current_files - initial_files
        initial_files = current_files
        if new_files.empty?
            puts 'New screenshots not found'
            next
        end

        screenshot = new_files.first
        puts "Found new screenshot: #{screenshot}. Detecting symbols..."
        sleep 0.5 # wait for screenshot to fully write on disk
        
        detected_symbols = `python symbol_detector.py "#{screenshot}" debug`

        garden = GardenParser.new.call(detected_symbols)
        hex_to_screen = GardenParser.new.hex_to_screen(detected_symbols)
        solved, solution = GardenSolver.new.call(garden)

        raise 'Solution not found' unless solved

        screen_positions = solution.map { [hex_to_screen[_1[1]], hex_to_screen[_1[3]]] }.flatten.compact
        screen_positions.each do |sp|
            ac.mouse_move(sp[:x], sp[:y])
            sleep 0.03
            ac.mouse_down(:left)
            sleep 0.03
            ac.mouse_up(:left)
            sleep 0.03
        end
    end
else
    screenshot = ARGV.first
    raise 'Screenshot file is not provided' if screenshot.nil? || screenshot.empty?

    detected_symbols = `python symbol_detector.py "#{screenshot}" debug`
    puts detected_symbols

    garden = GardenParser.new.call(detected_symbols)
    solved, solution = GardenSolver.new.call(garden)

    if solved
        puts "Solved"
        solution.each { p _1 }
    else
        puts "Solution not found"
    end
end