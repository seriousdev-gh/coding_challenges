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

    Marble = Struct.new(:symbol, :q, :r, :removed)

    def call(garden)
        solution = []
        metals = []
        marbles = []

        garden.each_with_index do |row, q|
            row.each_with_index do |element, r|
                next unless element

                metals << element if METALS.include?(element)
                marbles << Marble.new(element, q, r, false)
            end
        end

        metals = METALS & metals # sort metals
        t = Time.now
        solved = solve(garden, metals, marbles, solution)

        puts "Completed in #{Time.now - t} sec"

        [solved, solution.reverse]
    end

    private

    def solve(garden, metals, marbles, solution, depth = 0)
        return true if marbles.all?(&:removed)

        available = []
        marbles.each_with_index do |marble, i|
            next if marble.removed

            available << [marble, i] if enabled?(garden, marble.q, marble.r)
        end

        return false if available.empty?

        available.each do |marble1, marble_index1|
            q1 = marble1.q
            r1 = marble1.r
            el1 = marble1.symbol
            available.each do |marble2, marble_index2|
                q2 = marble2.q
                r2 = marble2.r
                el2 = marble2.symbol
                if marble_index1 == marble_index2
                    if el1 == :gold && metals.size == 1 && metals.last == :gold
                        garden[q1][r1] = nil
                        removed_metal = metals.pop
                        marble1.removed = true

                        if solve(garden, metals, marbles, solution, depth + 1)
                            solution << [el1, [q1-5, r1-5]]
                            return true 
                        end
                        
                        marble1.removed = false
                        metals.push(removed_metal)
                        garden[q1][r1] = :gold
                    end
                    next
                end

                if can_remove_pair?(garden, metals, el1, el2)
                    garden[q1][r1] = nil
                    garden[q2][r2] = nil
                    marble1.removed = true
                    marble2.removed = true
                    
                    if el1 == :mercury || el2 == :mercury
                        el1_metal = el1 == :lead || el1 == :tin || el1 == :iron || el1 == :copper || el1 == :silver
                        el2_metal = el2 == :lead || el2 == :tin || el2 == :iron || el2 == :copper || el2 == :silver
                        removed_metal = metals.pop if el1_metal || el2_metal
                    end

                    if solve(garden, metals, marbles, solution, depth + 1)
                        solution << [el1, [q1-5, r1-5], el2, [q2-5, r2-5]]
                        return true
                    end
                    
                    metals.push(removed_metal) if removed_metal
                    marble1.removed = false
                    marble2.removed = false
                    garden[q1][r1] = el1
                    garden[q2][r2] = el2
                end
            end
        end

        false
    end
    
    # we can remove marble if it has three consequent empty neighbour cells
    def enabled?(garden, q, r)
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

    def can_remove_pair?(garden, metals, el1, el2)
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
