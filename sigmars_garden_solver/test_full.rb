require_relative 'garden_parser'
require_relative 'garden_solver'

def full_test
    output = `ruby main.rb "test_assets\\Screenshot 2025-03-05 001425.png"`

    # puts output
    raise 'expected to find solution' unless output.include?('Solved') && output.include?('gold')
end

full_test
