require_relative 'marble'

class GardenSolver
    METALS = [:lead, :tin, :iron, :copper, :silver, :gold].reverse.freeze

    # hexagonal coordinate system:
    #
    #   *   *
    # *   *   * 
    #   *   *
    #
    # (q, r)
    #
    #       (0, -1)  (1, -1)
    #   (-1, 0)  (0, 0)  (1, 0)
    #       (-1, 1)  (0, 1)

    attr_reader :solution, :metals, :grid

    def call(garden)
        @solution = []
        marbles = []

        @metals = []
        @grid = Array.new(11) { Array.new(11) }
        garden.each_with_index do |row, q|
            row.each_with_index do |symbol, r|
                next unless symbol

                @metals << symbol if METALS.include?(symbol)
                marble = Marble.new(symbol, q, r)
                @grid[q][r] = marble
                marbles << marble
            end
        end

        marbles.each { _1.init_update(@grid) }

        @metals = METALS & @metals # sort metals

        @steps = {}

        t = Time.now
        solved = solve(marbles)
        puts "Completed in #{Time.now - t} sec"
        [solved, solution.reverse]
    end

    private

    def solve(marbles, depth = 0)
        marbles = marbles.reject(&:removed)
        return true if marbles.empty?

        available = []
        available_metals_and_mercury = []
        available_vitalities = []
        available_basic = []
        available_salts = []
        available_essence = []
        total_fire = 0
        total_earth = 0
        total_air = 0
        total_water = 0
        total_salt = 0
        total_essence = 0
        marbles.each do |marble|
            total_fire += 1 if marble.symbol == :fire 
            total_earth += 1 if marble.symbol == :earth 
            total_air += 1 if marble.symbol == :air 
            total_water += 1 if marble.symbol == :water 
            total_salt += 1 if marble.symbol == :salt 
            total_essence += 1 if marble.symbol == :essence 

            next unless marble.available

            if marble.symbol == :gold && metals.last == :gold
                return true if check_gold(marble, marbles, depth)
                next
            end

            next if marble.type == :metal && metals.last != marble.symbol

            available_metals_and_mercury << marble if marble.type == :metal || marble.type == :mercury
            available_vitalities << marble if marble.type == :vitality
            available_basic << marble if marble.type == :basic
            available_salts << marble if marble.type == :salt
            available_essence << marble if marble.type == :essence
        end

        number_of_odds = 0
        number_of_odds += 1 if total_fire.odd?
        number_of_odds += 1 if total_earth.odd?
        number_of_odds += 1 if total_air.odd?
        number_of_odds += 1 if total_water.odd?

        # run with heuristics:
        # immediately return in case when parity is broken
        #
        # first try remove all basic elements 
        #
        # then remove only salt pairs it significantly reduces number of combinations to check because
        # removing basic salt pair may introduces impossible to solve branches later
        return false if total_salt == 0 && total_essence == 0 && number_of_odds > 0
        return false if total_essence > 0 && (total_fire == 0 || total_earth == 0 || total_air == 0 || total_water == 0)
        # TODO: check parity total_salt and total_essence
        # return false if total_salt < number_of_odds

        return true if process_essence(available_essence, available_basic, marbles, depth)

        return true if process(available_basic, marbles, depth) do |a, b|
            a.symbol == b.symbol
        end

        return true if process(available_salts, marbles, depth) # skip block check, pairs of salts always removable

        return true if process(available_vitalities, marbles, depth) do |a, b|
            a.symbol != b.symbol
        end

        return true if process(available_metals_and_mercury, marbles, depth) do |a, b|
            a.symbol == :mercury && b.symbol == metals.last || b.symbol == :mercury && a.symbol == metals.last
        end

        return true if process(available_basic + available_salts, marbles, depth) do |a, b|
            a.type != b.type
        end
        false
    end

    def dump(selected = [])
        # system 'cls'
        (0...11).each do |r|
            shift = (r - 5).abs
            print('  ' * shift) if r < 5
            print('  ' * shift) if r > 5
            start = r < 5 ? shift : 0
            ending = r > 5 ? 11-shift : 11
            (start...ending).each do |q|
                if grid[q][r]
                    if selected.include?([q,r])
                        print "[#{grid[q][r].symbol.to_s[0..1]}]"
                    else
                        print " #{grid[q][r].symbol.to_s[0..1]} "
                    end
                else
                    print("    ")
                end
            end
            puts
        end
        puts
    end

    def check_gold(marble, marbles, depth)
        marble.remove(grid)

        if solve(marbles, depth + 1)
            solution.push [[marble.symbol, marble.q-5, marble.r-5]]
            return true 
        end
        
        marble.add(grid)
        false
    end

    def process(list, marbles, depth)
        list.each_with_index do |marble1, i|
            list.each_with_index do |marble2, j|
                next if i == j || j < i # order of pairs doesnt matter, so we dont need to run pair that already checked
                next unless yield(marble1, marble2) if block_given?

                marble1.remove(grid)
                marble2.remove(grid)

                if (marble1.symbol == :mercury || marble2.symbol == :mercury) && (marble1.type == :metal || marble2.type == :metal)
                    removed_metal = metals.pop
                end

                if solve(marbles, depth + 1)
                    solution.push [[marble1.symbol, marble1.q-5, marble1.r-5], [marble2.symbol, marble2.q-5, marble2.r-5]]
                    return true
                end
                
                metals.push(removed_metal) if removed_metal
                marble2.add(grid)
                marble1.add(grid)
            end
        end

        false
    end

    def process_essence(essence_list, basic_list, marbles, depth)
        return false if essence_list.empty?
        return false if basic_list.empty?

        list_air = basic_list.select { _1.symbol == :air }
        list_fire = basic_list.select { _1.symbol == :fire }
        list_water = basic_list.select { _1.symbol == :water }
        list_earth = basic_list.select { _1.symbol == :earth }

        return false if list_air.empty? || list_fire.empty? || list_water.empty? || list_earth.empty?

        essence_list.each do |marble_essence|
            list_air.each do |marble_air|
                list_fire.each do |marble_fire|
                    list_water.each do |marble_water|
                        list_earth.each do |marble_earth|
                            marble_essence.remove(grid)
                            marble_air.remove(grid)
                            marble_fire.remove(grid)
                            marble_water.remove(grid)
                            marble_earth.remove(grid)

                            if solve(marbles, depth + 1)
                                solution.push([
                                    [marble_essence.symbol, marble_essence.q-5, marble_essence.r-5], 
                                    [marble_air.symbol, marble_air.q-5, marble_air.r-5],
                                    [marble_fire.symbol, marble_fire.q-5, marble_fire.r-5],
                                    [marble_water.symbol, marble_water.q-5, marble_water.r-5],
                                    [marble_earth.symbol, marble_earth.q-5, marble_earth.r-5]])
                                return true
                            end

                            marble_earth.add(grid)
                            marble_water.add(grid)
                            marble_fire.add(grid)
                            marble_air.add(grid)
                            marble_essence.add(grid)
                        end
                    end
                end
            end
        end

        false
    end
    
    def step_debug(depth, marble1, marble2 = nil)
        @steps[depth] ||= 0
        @steps[depth] += 1

        if depth < 7
            p ["Removing: #{marble1.inspect} #{marble2&.inspect} at depth: #{depth}"]
            p @steps
            selected = [[marble1.q, marble1.r]]
            selected << [marble2.q, marble2.r] if marble2
            dump(selected)
            puts
        end
    end
end
