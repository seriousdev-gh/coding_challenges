require 'auto_click'

class Autosolver
    def initialize(folder)
        @folder = folder.gsub('\\', '/')
        @folder += '/' unless @folder.end_with?('/')
        @folder += '*' unless @folder.end_with?('*')
    end

    def call
        @ac = AutoClick.new()
        @initial_files = Dir[@folder]

        loop do
            sleep 1
            detect_new_screenshots
        end
    end


    def detect_new_screenshots
        new_files = Dir[@folder] - @initial_files
        @initial_files = Dir[@folder]

        if new_files.empty?
            putc '.'
            return
        end
        puts

        screenshot = new_files.first
        puts "Found new screenshot: #{screenshot}."
        sleep 0.5 # wait for screenshot to be fully written on disk
        
        try_to_solve(screenshot)
    end

    def try_to_solve(screenshot)
        puts "Processing image..."
        detected_symbols = `python symbol_detector.py "#{screenshot}" debug`

        puts "Finding solution..."
        parser = GardenParser.new(detected_symbols).call
        solved, solution = GardenSolver.new.call(parser.garden)

        raise 'Solution not found' unless solved

        puts "Solution found. Go!"
        clicks = []
        solution.each do |step|
            clicks << parser.conversion.fetch(step[1])
            clicks << parser.conversion.fetch(step[3]) if step[3]
        end
        clicks.each do |sp|
            @ac.mouse_move(sp[:x], sp[:y])
            sleep 0.03 # wait few frames for game to update mouse position 
            @ac.mouse_down(:left)
            sleep 0.03 # wait for not to press and release in between single frame
            @ac.mouse_up(:left)
            sleep 0.03
        end
        puts "Done."
    end
end