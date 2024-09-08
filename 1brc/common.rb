# frozen_string_literal: true

require 'open3'

DEFAULT_NWORKERS = 6

Measurement = Struct.new(:min, :max, :sum, :count)

# usage: `ruby main_readtostring.rb <file> <threads>`
# single worker: `ruby main_readtostring.rb <file> <threads> worker <index>`

def main
  mode = ARGV[2]

  if mode == 'worker'
    run_worker
  else
    t = Time.now
    results = calculate_in_processes
    Dir['worker_data_*.bin'].each { File.delete(_1) }
    merge(results)
    puts "#{Time.now - t} s"
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
      stdout, stderr, status = Open3.capture3("ruby --yjit #{$entrypoint} #{file} #{nworkers} worker #{i}")
      unless status.success?
        puts stderr
        # TODO: kill running processes when one is not successful
        exit
      end
      result = 
        File.open(result_file_path(i), 'rb') do |inp|
          Marshal.load(inp)
        end
      File.delete(result_file_path(i))
      [result, stdout]
    end
  end
  threads.map do |t|
    result, stdout = t.value
    puts stdout
    result
  end
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

  # print '{'
  # print sorted.join(', ')
  # puts '}'
  File.open("#{File.basename($0)}.output.txt", 'wb') do |out|
    out.print '{'
    out.print sorted.join("\n")
    out.puts '}'
  end
end

def run_worker
  file = ARGV[0]
  nworkers = ARGV[1].to_i
  index = ARGV[3].to_i

  chunk_size = File.size(file) / nworkers

  result = calculate_data(file, chunk_size, index, nworkers)
  File.open(result_file_path(index), 'wb') do |out|
    Marshal.dump(result, out)
  end
end