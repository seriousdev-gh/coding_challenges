class GardenSolver
    BASIC_ELEMENTS = [:fire, :water, :earth, :air].freeze
    METALS = [:lead, :tin, :iron, :copper, :silver, :gold].reverse.freeze

    # hexagonal coordinate system:
    #
    #  * *
    # * * *
    #  * *
    #
    # (q, r)
    #
    #       (0, -1)  (1, -1)
    #   (-1, 0)  (0, 0)  (1, 0)
    #       (-1, 1)  (0, 1)

    Marble = Struct.new(:symbol, :q, :r, :removed, :type)

    attr_reader :solution, :metals, :marbles, :garden

    def call(garden)
        # srand(2000)
        @solution = []
        @marbles = []
        @garden = garden

        @metals = []
        garden.each_with_index do |row, q|
            row.each_with_index do |symbol, r|
                next unless symbol

                @metals << symbol if METALS.include?(symbol)
                @marbles << Marble.new(symbol, q, r, false, type(symbol))
            end
        end

        @metals = METALS & @metals # sort metals

        # @marbles.shuffle!

        # puts @marbles.map { _1.symbol }.join(' ')

        t = Time.now

        solved = solve

        puts "Completed in #{Time.now - t} sec"

        [solved, solution.reverse]
    end

    private

    def solve(depth = 0)
        # p solution
        return true if marbles.all?(&:removed)

        available = []
        available_metals_and_mercury = []
        available_vitalities = []
        available_basic = []
        available_basic_and_salts = []
        marbles.each do |marble|
            next if marble.removed
            next unless enabled?(marble.q, marble.r)

            if marble.symbol == :gold && metals.last == :gold
                return true if check_gold(marble, depth)
                next
            end

            available_metals_and_mercury << marble if marble.type == :metal || marble.type == :mercury
            available_vitalities << marble if marble.type == :vitality
            available_basic << marble if marble.type == :basic
            available_basic_and_salts << marble if marble.type == :basic || marble.type == :salt
        end

        # run with heuristics
        return true if check_list(available_metals_and_mercury, depth)
        return true if check_list(available_vitalities, depth)
        return true if check_list(available_basic, depth) 
        return true if check_list(available_basic_and_salts, depth) # checking salts last seems to greatly improve performance

        false
    end

    def check_gold(marble, depth)
        garden[marble.q][marble.r] = nil
        removed_metal = metals.pop
        marble.removed = true

        # solution.push marble.symbol
        if solve(depth + 1)
            solution.push [marble.symbol, [marble.q-5, marble.r-5]]
            return true 
        end
        
        marble.removed = false
        metals.push(removed_metal)
        garden[marble.q][marble.r] = :gold

        false
    end

    def check_list(list, depth)
        list.each_with_index do |marble1, i|
            q1 = marble1.q
            r1 = marble1.r
            el1 = marble1.symbol
            list.each_with_index do |marble2, j|
                next if i == j || j < i # order of pairs doesnt matter, so we dont need to run pair that already checked

                q2 = marble2.q
                r2 = marble2.r
                el2 = marble2.symbol

                next unless can_remove_pair?(el1, el2)

                garden[q1][r1] = nil
                garden[q2][r2] = nil
                marble1.removed = true
                marble2.removed = true
                
                if el1 == :mercury || el2 == :mercury
                    el1_metal = el1 == :lead || el1 == :tin || el1 == :iron || el1 == :copper || el1 == :silver
                    el2_metal = el2 == :lead || el2 == :tin || el2 == :iron || el2 == :copper || el2 == :silver
                    removed_metal = metals.pop if el1_metal || el2_metal
                end

                # solution.push [el1, el2]
                if solve(depth + 1)
                    solution.push [el1, [q1-5, r1-5], el2, [q2-5, r2-5]]
                    return true
                end
                # solution.pop
                
                metals.push(removed_metal) if removed_metal
                marble1.removed = false
                marble2.removed = false
                garden[q1][r1] = el1
                garden[q2][r2] = el2
            end
        end

        false
    end
    
    def type(s)
        return :basic if s == :fire || s == :water || s == :earth || s == :air
        return :metal if s == :lead || s == :tin || s == :iron || s == :copper || s == :silver || s == :gold
        return :mercury if s == :mercury
        return :vitality if s == :life || s == :death
        return :salt if s == :salt

        raise 'unreachable'
    end

    # we can remove marble if it has three consequent empty neighbour cells
    def enabled?(q, r)
        p1 = q < 10 && garden[q+1][r]
        p6 = q < 10 && r > 0 && garden[q+1][r-1]
        p2 = r < 10 && garden[q][r+1]
        p5 = r > 0 && garden[q][r-1]
        p3 = q > 0 && r < 10 && garden[q-1][r+1]
        p4 = q > 0 && garden[q-1][r]

        !p1 && !p2 && !p3 ||
        !p2 && !p3 && !p4 ||
        !p3 && !p4 && !p5 ||
        !p4 && !p5 && !p6 ||
        !p5 && !p6 && !p1 ||
        !p6 && !p1 && !p2
    end

    def can_remove_pair?(el1, el2)
        return true if el1 == el2 && (el1 == :fire || el1 == :water || el1 == :earth || el1 == :air || el1 == :salt)
        return true if el1 == :life && el2 == :death
        return true if el1 == :death && el2 == :life
        return true if el1 == :salt && (el2 == :fire || el2 == :water || el2 == :earth || el2 == :air)
        return true if el2 == :salt && (el1 == :fire || el1 == :water || el1 == :earth || el1 == :air)

        if el1 == :mercury
            return true if el2 == metals.last
        end
        
        if el2 == :mercury
            return true if el1 == metals.last
        end

        false
    end
end
