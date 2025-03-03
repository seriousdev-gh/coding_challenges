require 'ruby-prof'
require_relative 'garden_parser'
require_relative 'garden_solver'

garden_string = '{"symbols": [{"x": 1181.0, "y": 818.0, "name": "silver_a", "confidence": 1.0}, {"x": 852.0, "y": 703.0, "name": "gold", "confidence": 1.0}, {"x": 1050.0, "y": 363.0, "name": "water", "confidence": 0.99}, {"x": 784.0, "y": 360.0, "name": "fire", "confidence": 0.99}, {"x": 1050.0, "y": 133.0, "name": "death", "confidence": 0.97}, {"x": 916.0, "y": 360.0, "name": "fire", "confidence": 0.97}, {"x": 1248.0, "y": 1162.0, "name": "earth", "confidence": 0.96}, {"x": 786.0, "y": 135.0, "name": "water", "confidence": 0.96}, {"x": 1116.0, "y": 933.0, "name": "water", "confidence": 0.95}, {"x": 1050.0, "y": 1047.0, "name": "water", "confidence": 0.95}, {"x": 918.0, "y": 1048.0, "name": "earth", "confidence": 0.94}, {"x": 656.0, "y": 364.0, "name": "earth", "confidence": 0.92}, {"x": 391.0, "y": 359.0, "name": "air", "confidence": 0.9}, {"x": 918.0, "y": 134.0, "name": "salt", "confidence": 0.9}, {"x": 720.0, "y": 245.0, "name": "air", "confidence": 0.89}, {"x": 788.0, "y": 1276.0, "name": "earth", "confidence": 0.88}, {"x": 1248.0, "y": 250.0, "name": "earth", "confidence": 0.87}, {"x": 1182.0, "y": 362.0, "name": "salt", "confidence": 0.87}, {"x": 522.0, "y": 820.0, "name": "earth", "confidence": 0.83}, {"x": 984.0, "y": 1159.0, "name": "mercury", "confidence": 0.82}, {"x": 324.0, "y": 929.0, "name": "life", "confidence": 0.76}], "width": 1702, "height": 1388}'
garden = GardenParser.new.call(garden_string)

prof = RubyProf::Profile.new()
res = prof.profile do
    result, _ = GardenSolver.new.call(garden)
    p result
end


require 'ruby-prof-speedscope'
File.open("trace.rubyprof", 'w') do |f|
  RubyProf::SpeedscopePrinter.new(res).print(f)
end