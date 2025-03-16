require_relative 'garden_parser'
require_relative 'garden_solver'

def small_test
    garden = Array.new(11) { Array.new(11) }

    garden[5][5] = :gold
    garden[5][6] = :fire
    garden[6][5] = :salt
    garden[4][5] = :fire
    garden[5][4] = :fire
    garden[6][4] = :fire
    garden[7][7] = :lead
    garden[8][3] = :earth
    garden[3][3] = :mercury

    result, solution = GardenSolver.new.call(garden)

    raise 'expected to find solution' unless result
    raise 'expected solution' if solution.empty?
end

def medium_test
    garden_string = '{"symbols": [{"x": 1181.0, "y": 818.0, "name": "silver_a", "confidence": 1.0}, {"x": 852.0, "y": 703.0, "name": "gold", "confidence": 1.0}, {"x": 1050.0, "y": 363.0, "name": "water", "confidence": 0.99}, {"x": 784.0, "y": 360.0, "name": "fire", "confidence": 0.99}, {"x": 1050.0, "y": 133.0, "name": "death", "confidence": 0.97}, {"x": 916.0, "y": 360.0, "name": "fire", "confidence": 0.97}, {"x": 1248.0, "y": 1162.0, "name": "earth", "confidence": 0.96}, {"x": 786.0, "y": 135.0, "name": "water", "confidence": 0.96}, {"x": 1116.0, "y": 933.0, "name": "water", "confidence": 0.95}, {"x": 1050.0, "y": 1047.0, "name": "water", "confidence": 0.95}, {"x": 918.0, "y": 1048.0, "name": "earth", "confidence": 0.94}, {"x": 656.0, "y": 364.0, "name": "earth", "confidence": 0.92}, {"x": 391.0, "y": 359.0, "name": "air", "confidence": 0.9}, {"x": 918.0, "y": 134.0, "name": "salt", "confidence": 0.9}, {"x": 720.0, "y": 245.0, "name": "air", "confidence": 0.89}, {"x": 788.0, "y": 1276.0, "name": "earth", "confidence": 0.88}, {"x": 1248.0, "y": 250.0, "name": "earth", "confidence": 0.87}, {"x": 1182.0, "y": 362.0, "name": "salt", "confidence": 0.87}, {"x": 522.0, "y": 820.0, "name": "earth", "confidence": 0.83}, {"x": 984.0, "y": 1159.0, "name": "mercury", "confidence": 0.82}, {"x": 324.0, "y": 929.0, "name": "life", "confidence": 0.76}], "width": 1702, "height": 1388}'
    parser = GardenParser.new(garden_string).call
    result, solution = GardenSolver.new.call(parser.garden)

    raise 'expected to find solution' unless result
    raise 'expected solution' if solution.empty?
end


def big_test
    garden_string = '{"symbols": [{"x": 458.0, "y": 257.0, "name": "earth_a", "confidence": 1.0}, {"x": 854.0, "y": 704.0, "name": "gold", "confidence": 1.0}, {"x": 788.0, "y": 1280.0, "name": "water_a", "confidence": 1.0}, {"x": 1052.0, "y": 364.0, "name": "water", "confidence": 0.99}, {"x": 1184.0, "y": 818.0, "name": "silver", "confidence": 0.99}, {"x": 1117.0, "y": 477.0, "name": "copper", "confidence": 0.99}, {"x": 786.0, "y": 361.0, "name": "fire", "confidence": 0.99}, {"x": 788.0, "y": 136.0, "name": "water", "confidence": 0.99}, {"x": 920.0, "y": 1274.0, "name": "death", "confidence": 0.98}, {"x": 524.0, "y": 591.0, "name": "salt", "confidence": 0.98}, {"x": 788.0, "y": 1055.0, "name": "earth_a", "confidence": 0.98}, {"x": 1052.0, "y": 134.0, "name": "death", "confidence": 0.97}, {"x": 918.0, "y": 361.0, "name": "fire", "confidence": 0.97}, {"x": 1050.0, "y": 1273.0, "name": "fire", "confidence": 0.96}, {"x": 1250.0, "y": 1163.0, "name": "earth", "confidence": 0.96}, {"x": 1184.0, "y": 592.0, "name": "water", "confidence": 0.96}, {"x": 1184.0, "y": 1277.0, "name": "earth", "confidence": 0.96}, {"x": 920.0, "y": 1055.0, "name": "earth_a", "confidence": 0.95}, {"x": 1118.0, "y": 934.0, "name": "water", "confidence": 0.95}, {"x": 656.0, "y": 368.0, "name": "water_a", "confidence": 0.95}, {"x": 1052.0, "y": 1048.0, "name": "water", "confidence": 0.95}, {"x": 524.0, "y": 827.0, "name": "earth_a", "confidence": 0.95}, {"x": 458.0, "y": 704.0, "name": "mercury", "confidence": 0.95}, {"x": 392.0, "y": 599.0, "name": "earth_a", "confidence": 0.94}, {"x": 986.0, "y": 1160.0, "name": "mercury", "confidence": 0.94}, {"x": 589.0, "y": 475.0, "name": "fire", "confidence": 0.93}, {"x": 1184.0, "y": 363.0, "name": "salt", "confidence": 0.92}, {"x": 722.0, "y": 246.0, "name": "air", "confidence": 0.89}, {"x": 920.0, "y": 135.0, "name": "salt", "confidence": 0.89}, {"x": 1250.0, "y": 251.0, "name": "earth", "confidence": 0.87}, {"x": 261.0, "y": 816.0, "name": "air", "confidence": 0.85}, {"x": 260.0, "y": 596.0, "name": "water_a", "confidence": 0.85}, {"x": 656.0, "y": 135.0, "name": "salt", "confidence": 0.85}, {"x": 393.0, "y": 360.0, "name": "air", "confidence": 0.83}, {"x": 327.0, "y": 474.0, "name": "air", "confidence": 0.82}, {"x": 590.0, "y": 930.0, "name": "life", "confidence": 0.78}, {"x": 326.0, "y": 931.0, "name": "life", "confidence": 0.76}], "width": 1711, "height": 1397}'
    parser = GardenParser.new(garden_string).call
    result, solution = GardenSolver.new.call(parser.garden)

    raise 'expected to find solution' unless result
    raise 'expected solution' if solution.empty?
end

def slow_test
    garden_string = '{"symbols": [{"x": 1547.0, "y": 506.0, "name": "air_a", "confidence": 1.0}, {"x": 1250.0, "y": 681.0, "name": "earth_a", "confidence": 0.99}, {"x": 1283.0, "y": 738.0, "name": "earth_a", "confidence": 0.98}, {"x": 1382.0, "y": 795.0, "name": "earth_a", "confidence": 0.98}, {"x": 1217.0, "y": 505.0, "name": "gold", "confidence": 0.98}, {"x": 986.0, "y": 453.0, "name": "earth_a", "confidence": 0.98}, {"x": 1250.0, "y": 790.0, "name": "lead", "confidence": 0.97}, {"x": 1382.0, "y": 334.0, "name": "silver", "confidence": 0.97}, {"x": 1118.0, "y": 220.0, "name": "life", "confidence": 0.97}, {"x": 1019.0, "y": 505.0, "name": "death", "confidence": 0.96}, {"x": 1184.0, "y": 339.0, "name": "earth_a", "confidence": 0.96}, {"x": 1481.0, "y": 392.0, "name": "salt", "confidence": 0.96}, {"x": 1382.0, "y": 220.0, "name": "iron", "confidence": 0.96}, {"x": 1315.0, "y": 335.0, "name": "copper", "confidence": 0.96}, {"x": 920.0, "y": 448.0, "name": "life", "confidence": 0.95}, {"x": 1052.0, "y": 562.0, "name": "mercury", "confidence": 0.95}, {"x": 1349.0, "y": 391.0, "name": "death", "confidence": 0.95}, {"x": 1382.0, "y": 453.0, "name": "earth_a", "confidence": 0.95}, {"x": 1118.0, "y": 791.0, "name": "salt", "confidence": 0.94}, {"x": 1184.0, "y": 679.0, "name": "water_a", "confidence": 0.94}, {"x": 1084.0, "y": 391.0, "name": "tin", "confidence": 0.94}, {"x": 1118.0, "y": 337.0, "name": "water_a", "confidence": 0.94}, {"x": 1448.0, "y": 679.0, "name": "water_a", "confidence": 0.94}, {"x": 1481.0, "y": 624.0, "name": "earth_a", "confidence": 0.93}, {"x": 1184.0, "y": 221.0, "name": "death", "confidence": 0.93}, {"x": 1151.0, "y": 277.0, "name": "death", "confidence": 0.93}, {"x": 953.0, "y": 392.0, "name": "mercury", "confidence": 0.93}, {"x": 1448.0, "y": 563.0, "name": "air_a", "confidence": 0.93}, {"x": 1447.0, "y": 334.0, "name": "fire_a", "confidence": 0.93}, {"x": 1085.0, "y": 619.0, "name": "life", "confidence": 0.92}, {"x": 1316.0, "y": 677.0, "name": "air_a", "confidence": 0.92}, {"x": 1382.0, "y": 565.0, "name": "water_a", "confidence": 0.92}, {"x": 1316.0, "y": 225.0, "name": "earth_a", "confidence": 0.92}, {"x": 1019.0, "y": 280.0, "name": "water_a", "confidence": 0.91}, {"x": 1051.0, "y": 790.0, "name": "fire_a", "confidence": 0.91}, {"x": 1514.0, "y": 565.0, "name": "water_a", "confidence": 0.91}, {"x": 920.0, "y": 562.0, "name": "life", "confidence": 0.91}, {"x": 1315.0, "y": 790.0, "name": "fire_a", "confidence": 0.9}, {"x": 1348.0, "y": 619.0, "name": "fire_a", "confidence": 0.9}, {"x": 886.0, "y": 505.0, "name": "fire_a", "confidence": 0.9}, {"x": 1415.0, "y": 508.0, "name": "water_a", "confidence": 0.9}, {"x": 1183.0, "y": 790.0, "name": "fire_a", "confidence": 0.9}, {"x": 1117.0, "y": 676.0, "name": "fire_a", "confidence": 0.89}, {"x": 1250.0, "y": 334.0, "name": "mercury", "confidence": 0.89}, {"x": 1018.0, "y": 733.0, "name": "fire_a", "confidence": 0.89}, {"x": 1052.0, "y": 449.0, "name": "air_a", "confidence": 0.88}, {"x": 986.0, "y": 335.0, "name": "salt", "confidence": 0.88}, {"x": 1052.0, "y": 677.0, "name": "air_a", "confidence": 0.88}, {"x": 1415.0, "y": 734.0, "name": "air_a", "confidence": 0.86}, {"x": 1514.0, "y": 449.0, "name": "air_a", "confidence": 0.85}, {"x": 953.0, "y": 619.0, "name": "mercury", "confidence": 0.85}, {"x": 1415.0, "y": 280.0, "name": "water_a", "confidence": 0.85}, {"x": 1250.0, "y": 220.0, "name": "mercury", "confidence": 0.82}, {"x": 986.0, "y": 677.0, "name": "air_a", "confidence": 0.81}, {"x": 1052.0, "y": 221.0, "name": "salt", "confidence": 0.78}, {"x": 1383.0, "y": 881.0, "name": "death", "confidence": 0.73}, {"x": 1344.0, "y": 880.0, "name": "lead", "confidence": 0.71}], "width": 1920, "height": 1080}'
    parser = GardenParser.new(garden_string).call
    result, solution = GardenSolver.new.call(parser.garden)

    raise 'expected to find solution' unless result
    raise 'expected solution' if solution.empty?
end

small_test
medium_test
big_test
slow_test