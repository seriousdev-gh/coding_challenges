# frozen_string_literal: true

require 'open3'

DEFAULT_NWORKERS = 6
PROGRAM_NAME = 'main.rb'

Measurement = Struct.new(:min, :max, :sum, :count)

# usage: `ruby main.rb <file> <threads>`
# single worker: `ruby main.rb <file> <threads> worker <index>`

def main
  mode = ARGV[2]

  if mode == 'worker'
    run_worker
  else
    results = calculate_in_processes
    Dir['worker_data_*.bin'].each { File.delete(_1) }
    merge(results)
  end
end

def result_file_path(process_index)
  "worker_data_#{process_index}.bin"
end

def calculate_in_processes
  file = ARGV[0]
  nworkers = ARGV[1]&.to_i || DEFAULT_NWORKERS
  threads = (0...nworkers).map do |i|
    Thread.new do
      _, stderr_str, status = Open3.capture3("ruby --yjit #{PROGRAM_NAME} #{file} #{nworkers} worker #{i}")
      unless status.success?
        puts stderr_str
        # TODO: kill running processes when one is not successful
        exit
      end
      result = 
        File.open(result_file_path(i), 'rb') do |inp|
          Marshal.load(inp)
        end
      File.delete(result_file_path(i))
      result
    end
  end
  threads.map(&:value)
end

def merge(results)
  result, *other = results


  result.merge!(*other) do |_, old_value, new_value|
    Measurement.new(
      [old_value.min, new_value.min].min,
      [old_value.max, new_value.max].max,
      old_value.sum + new_value.sum,
      old_value.count + new_value.count
    )
  end

  sorted = result.sort.map do |name, stats|
    min = stats.min.round(1)
    mean = (stats.sum / stats.count + 0.0000001).round(1)
    max = stats.max.round(1)
    "#{name}=#{min}/#{mean}/#{max}"
  end

  print '{'
  print sorted.join(', ')
  puts '}'
end

def run_worker
  file = ARGV[0]
  nworkers = ARGV[1].to_i
  index = ARGV[3].to_i

  chunk_size = File.size(file) / nworkers
  offset = index * chunk_size

  result = calculate_data(file, offset, chunk_size, index)
  File.open(result_file_path(index), 'wb') do |out|
    Marshal.dump(result, out)
  end
end

def calculate_data(file, offset, chunk_size, index)
  skip_line = index != 0
  buffer = IO::Buffer.map(File.open(file), nil, 0, IO::Buffer::READONLY)
  data = {}

  name = nil
  line_start_offset = 0
  number_start_offset = 0

  if skip_line
    buffer.each(:U8, offset) do |byte_offset, byte|
      next unless byte == 10

      offset = offset + (byte_offset - offset) + 1
      line_start_offset = offset
      break
    end
  end

  buffer.each(:U8, offset) do |byte_offset, byte|
    if byte == 59 # ';'
      name = buffer.get_string(line_start_offset, byte_offset - line_start_offset)
      number_start_offset = byte_offset + 1
    elsif byte == 10 # '\n'
      value = buffer.get_string(number_start_offset, byte_offset - number_start_offset).to_f
      measurement = data[name]
      measurement ||= data[name] = Measurement.new(value, value, 0, 0)
      measurement.min = value if value < measurement.min
      measurement.max = value if value > measurement.max
      measurement.count += 1
      measurement.sum += value

      line_start_offset = byte_offset + 1

      break if byte_offset >= offset + chunk_size
    end
  end

  data
end

main
