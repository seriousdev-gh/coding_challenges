require 'open3'

files = Dir['1brc/src/test/resources/samples/*.txt']
files_max_name_size = files.map(&:size).max
files.each do |input_file|
    print input_file.ljust(files_max_name_size + 3, '.')
    expected = File.read(input_file.sub('.txt', '.out'))
    stdout_str, stderr_str, status = Open3.capture3("ruby main.rb #{input_file} 1")
    status = stdout_str.strip == expected.strip ? 'ok' : 'not ok'
    puts status
end