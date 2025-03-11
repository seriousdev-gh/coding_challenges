# represents single filled cell on hex grid
class Marble
    AVAILABLE_LOOKUP = (0..63).map { |a| b = a.to_s(2).rjust(6, '0'); (0..6).any? { |i| b[i]=='0' && b[(i+1) % 6]=='0' && b[(i+2)%6] == '0' } }

    attr_accessor :symbol, :q, :r, :removed, :type, :neighbours, :available

    def initialize(symbol, q, r)
        @symbol = symbol
        @q = q
        @r = r
        @removed = false
        @type = calculate_type
        @available = false
    end

    def remove(grid)
        @removed = true
        grid[q][r] = nil

        neighbours.each do |dir, marble|
            next if marble.removed

            marble.on_neighbour_removed((dir + 3) % 6)
        end
    end

    def on_neighbour_removed(dir)
        @neighbours_state &= ~(1<<dir)
        @available = AVAILABLE_LOOKUP[@neighbours_state]
    end

    def add(grid)
        @removed = false
        grid[q][r] = self

        neighbours.each do |dir, marble|
            next if marble.removed

            marble.on_neighbour_added((dir + 3) % 6)
        end
    end

    def on_neighbour_added(dir)
        @neighbours_state |= (1<<dir)
        @available = AVAILABLE_LOOKUP[@neighbours_state]
    end
    
    #    4   5
    #   3  *  0
    #    2   1
    def init_update(grid)
        @neighbours = []
        @neighbours_state = 0
        if q < 10 && grid[q+1][r]
            @neighbours << [0, grid[q+1][r]]
            @neighbours_state |= (1<<0)
        end
        if r < 10 && grid[q][r+1]
            @neighbours << [1, grid[q][r+1]]
            @neighbours_state |= (1<<1)
        end
        if q > 0 && r < 10 && grid[q-1][r+1]
            @neighbours << [2, grid[q-1][r+1]]
            @neighbours_state |= (1<<2)
        end
        if q > 0 && grid[q-1][r]
            @neighbours << [3, grid[q-1][r]]
            @neighbours_state |= (1<<3)
        end
        if r > 0 && grid[q][r-1]
            @neighbours << [4, grid[q][r-1]]
            @neighbours_state |= (1<<4)
        end
        if q < 10 && r > 0 && grid[q+1][r-1]
            @neighbours << [5, grid[q+1][r-1]]
            @neighbours_state |= (1<<5)
        end
        @available = AVAILABLE_LOOKUP[@neighbours_state]
    end

    def inspect
        "#{symbol} [#{q-5},#{r-5}] avail: #{available} removed: #{removed}]"
    end

    private
    
    def calculate_type
        return :basic if symbol == :fire || symbol == :water || symbol == :earth || symbol == :air
        return :metal if symbol == :lead || symbol == :tin || symbol == :iron || symbol == :copper || symbol == :silver || symbol == :gold
        return :mercury if symbol == :mercury
        return :vitality if symbol == :life || symbol == :death
        return :salt if symbol == :salt

        raise 'unreachable'
    end

end