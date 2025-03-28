require 'json'

class GardenParser
    attr_reader :garden, :conversion 

    def initialize(data_string)
        @data_string = data_string
        @garden = Array.new(11) { Array.new(11) }
        @conversion = {}
    end

    def call
        data = JSON.parse(@data_string)
        
        hex_width = calculate_grid_size(data['symbols'])
        grid_size = hex_width / Math.sqrt(3.0)

        center = data['symbols'].find { _1['name'] == 'gold' }
        raise 'center not found' if center.nil?

        data['symbols'].each do |symbol|
            q, r = pixel_to_grid(symbol['x'] - center['x'], symbol['y'] - center['y'], grid_size)
            next if q > 5 || r > 5
            next if q < -5 || r < -5

            name = symbol['name'].sub('_a', '').to_sym

            @garden[q+5][r+5] = name
            @conversion[[q, r]] = { x: symbol['x'], y: symbol['y'] }
        end

        self
    end

    private

    def euclidean_distance(x1, y1, x2, y2)
        Math.sqrt((x2 - x1)**2 + (y2 - y1)**2)
    end

    def calculate_grid_size(symbols)
        neighbours_distances = []

        symbols.each do |s1|
            minimum_distance = 999999
            symbols.each do |s2|
                next if s1 == s2
                distance = euclidean_distance(s1['x'], s1['y'], s2['x'], s2['y'])
                if distance < minimum_distance
                    minimum_distance = distance
                end
            end

            neighbours_distances << minimum_distance
        end

        neighbours_distances.sort[neighbours_distances.length/2]
    end

    def pixel_to_grid(x, y, grid_size)
        q = (x * Math.sqrt(3) / 3 - y / 3) / grid_size
        r = (2 * y / 3) / grid_size
        [q.round, r.round]
    end
end
