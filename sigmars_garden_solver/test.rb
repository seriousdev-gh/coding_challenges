require_relative 'garden_parser'
require_relative 'garden_solver'

def small_test
    garden = Hash.new { |hash, key| hash[key] = Hash.new }

    garden[0][0] = :gold
    garden[0][1] = :fire
    garden[1][0] = :salt
    garden[-1][0] = :fire
    garden[0][-1] = :fire
    garden[1][-1] = :fire
    garden[2][2] = :lead
    garden[3][3] = :earth
    garden[-2][-2] = :mercury

    garden.default_proc = nil

    result, solution = GardenSolver.new.call(garden)

    raise 'expected to find solution' unless result
    raise 'expected solution' if solution.empty?
end

small_test

def big_test
    garden_string = '{"symbols": [{"x": 458.0, "y": 257.0, "name": "earth_a", "confidence": 1.0}, {"x": 854.0, "y": 704.0, "name": "gold", "confidence": 1.0}, {"x": 788.0, "y": 1280.0, "name": "water_a", "confidence": 1.0}, {"x": 1052.0, "y": 364.0, "name": "water", "confidence": 0.99}, {"x": 1184.0, "y": 818.0, "name": "silver", "confidence": 0.99}, {"x": 1117.0, "y": 477.0, "name": "copper", "confidence": 0.99}, {"x": 786.0, "y": 361.0, "name": "fire", "confidence": 0.99}, {"x": 788.0, "y": 136.0, "name": "water", "confidence": 0.99}, {"x": 920.0, "y": 1274.0, "name": "death", "confidence": 0.98}, {"x": 524.0, "y": 591.0, "name": "salt", "confidence": 0.98}, {"x": 788.0, "y": 1055.0, "name": "earth_a", "confidence": 0.98}, {"x": 1052.0, "y": 134.0, "name": "death", "confidence": 0.97}, {"x": 918.0, "y": 361.0, "name": "fire", "confidence": 0.97}, {"x": 1050.0, "y": 1273.0, "name": "fire", "confidence": 0.96}, {"x": 1250.0, "y": 1163.0, "name": "earth", "confidence": 0.96}, {"x": 1184.0, "y": 592.0, "name": "water", "confidence": 0.96}, {"x": 1184.0, "y": 1277.0, "name": "earth", "confidence": 0.96}, {"x": 920.0, "y": 1055.0, "name": "earth_a", "confidence": 0.95}, {"x": 1118.0, "y": 934.0, "name": "water", "confidence": 0.95}, {"x": 656.0, "y": 368.0, "name": "water_a", "confidence": 0.95}, {"x": 1052.0, "y": 1048.0, "name": "water", "confidence": 0.95}, {"x": 524.0, "y": 827.0, "name": "earth_a", "confidence": 0.95}, {"x": 458.0, "y": 704.0, "name": "mercury", "confidence": 0.95}, {"x": 392.0, "y": 599.0, "name": "earth_a", "confidence": 0.94}, {"x": 986.0, "y": 1160.0, "name": "mercury", "confidence": 0.94}, {"x": 589.0, "y": 475.0, "name": "fire", "confidence": 0.93}, {"x": 1184.0, "y": 363.0, "name": "salt", "confidence": 0.92}, {"x": 722.0, "y": 246.0, "name": "air", "confidence": 0.89}, {"x": 920.0, "y": 135.0, "name": "salt", "confidence": 0.89}, {"x": 1250.0, "y": 251.0, "name": "earth", "confidence": 0.87}, {"x": 261.0, "y": 816.0, "name": "air", "confidence": 0.85}, {"x": 260.0, "y": 596.0, "name": "water_a", "confidence": 0.85}, {"x": 656.0, "y": 135.0, "name": "salt", "confidence": 0.85}, {"x": 393.0, "y": 360.0, "name": "air", "confidence": 0.83}, {"x": 327.0, "y": 474.0, "name": "air", "confidence": 0.82}, {"x": 590.0, "y": 930.0, "name": "life", "confidence": 0.78}, {"x": 326.0, "y": 931.0, "name": "life", "confidence": 0.76}], "width": 1711, "height": 1397}'
    garden = GardenParser.new.call(garden_string)
    result, solution = GardenSolver.new.call(garden)

    raise 'expected to find solution' unless result
    raise 'expected solution' if solution.empty?
end

big_test