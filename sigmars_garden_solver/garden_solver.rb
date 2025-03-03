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
                    if el1 == :gold && metals.size == 1 && metals.last == :gold && enabled?(garden, q1, r1)
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
                if can_remove_pair?(garden, metals, q1, r1, q2, r2)
                    garden[q1].delete(r1)
                    garden[q2].delete(r2)
                    removed_metal = metals.pop if METALS.include?(el1) || METALS.include?(el2)

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
        return false if garden[q][r].nil?
        if garden[q+1]
            p1 = garden[q+1][r]
            p6 = garden[q+1][r-1]
        end

        if garden[q]
            p2 = garden[q][r+1]
            p5 = garden[q][r-1]
        end

        if garden[q-1]
            p3 = garden[q-1][r+1]
            p4 = garden[q-1][r]
        end

        p1.nil? && p2.nil? && p3.nil? ||
        p2.nil? && p3.nil? && p4.nil? ||
        p3.nil? && p4.nil? && p5.nil? ||
        p4.nil? && p5.nil? && p6.nil? ||
        p5.nil? && p6.nil? && p1.nil? ||
        p6.nil? && p1.nil? && p2.nil?
    end

    def can_remove_pair?(garden, metals, q1, r1, q2, r2)
        # return false if !enabled?(garden, q1, r1) || !enabled?(garden, q2, r2)

        el1 = garden[q1][r1]
        el2 = garden[q2][r2]

        return true if el1 == el2 && (BASIC_ELEMENTS.include?(el1) || el1 == :salt)
        return true if el1 == :life && el2 == :death
        return true if el1 == :death && el2 == :life
        return true if el1 == :salt && BASIC_ELEMENTS.include?(el2)
        return true if el2 == :salt && BASIC_ELEMENTS.include?(el1)

        if el1 == :mercury
            return true if el2 == metals.last
        end
        
        if el2 == :mercury
            return true if el1 == metals.last
        end

        false
    end
end
