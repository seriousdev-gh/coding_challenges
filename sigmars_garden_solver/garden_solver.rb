require 'json'

# TODO: improve performance. For now it is very slow.

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

    def call(garden)
        solution = []
        metals = []

        garden.each_value do |rs_hash| 
            rs_hash.each_value do |element| 
                metals << element if METALS.include?(element)
            end
        end

        metals = METALS & metals # sort metals
        t = Time.now
        solved = solve(garden, metals, solution)
        puts "Completed in #{Time.now - t} sec"

        [solved, solution.reverse]
    end

    private

    def solve(garden, metals, solution)
        # p [:solve, garden, metals, solution] 
        return true if solved?(garden, metals)

        available = []
        garden.each do |q, rs|
            rs.each_key do |r|
                if enabled?(garden, q, r)
                    available << [q, r]
                end
            end
        end

        return false if available.empty?

        # p [:available, available]

        available.each do |q1, r1|
            el1 = garden[q1][r1]
            available.each do |q2, r2|
                el2 = garden[q2][r2]

                if q1 == q2 && r1 == r2
                    if el1 == :gold && metals.size == 1 && metals.last == :gold
                        garden[q1].delete(r1)
                        removed_metal = metals.pop

                        if solve(garden, metals, solution)
                            solution << [el1, [q1, r1]]
                            return true 
                        end
                        
                        metals.push(removed_metal)
                        garden[q1][r1] = :gold
                    end
                    next
                end

                # p [:can_remove_pair, can_remove_pair?(garden, metals, q1, r1, q2, r2)]
                if can_remove_pair?(garden, metals, el1, el2)
                    garden[q1].delete(r1)
                    garden[q2].delete(r2)
                    
                    if el1 == :mercury || el2 == :mercury
                        el1_metal = el1 == :lead || el1 == :tin || el1 == :iron || el1 == :copper || el1 == :silver
                        el2_metal = el2 == :lead || el2 == :tin || el2 == :iron || el2 == :copper || el2 == :silver
                        removed_metal = metals.pop if el1_metal || el2_metal
                    end

                    if solve(garden, metals, solution)
                        solution << [el1, [q1, r1], el2, [q2, r2]]
                        return true
                    end
                    
                    metals.push(removed_metal) if removed_metal
                    garden[q1][r1] = el1
                    garden[q2][r2] = el2
                end
            end
        end

        false
    end

    def solved?(garden, metals)
        return false unless metals.empty?

        garden.empty? || garden.values.all? do |rs_hash|   
            rs_hash.empty? || rs_hash.values.all?(&:nil?)
        end
    end
    
    # we can remove marble if it has three consequent empty neighbour cells
    def enabled?(garden, q, r)
        return false unless garden[q][r]
        if t = garden[q+1]
            p1 = t[r]
            p6 = t[r-1]
        end

        if t = garden[q]
            p2 = t[r+1]
            p5 = t[r-1]
        end

        if t = garden[q-1]
            p3 = t[r+1]
            p4 = t[r]
        end

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
