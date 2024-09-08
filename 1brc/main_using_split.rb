$entrypoint = __FILE__
require_relative 'common'

def calculate_data(file, chunk_size, index, nworkers)
  offset = index * chunk_size
  buffer = IO::Buffer.map(File.open(file), nil, 0, IO::Buffer::READONLY)

  # skip to next line if worker starts in the middle of line (first worker always start at beginning)
  if index != 0
    buffer.each(:U8, offset) do |byte_offset, byte|
      if byte == 10
        chunk_size -= byte_offset - offset + 1
        offset = byte_offset + 1
        break
      end
    end
  end

  # increase chunk size in case when last line not fit in the chunk (last worker always have last full line)
  if index != nworkers - 1
    last_byte_offset = offset + chunk_size
    buffer.each(:U8, last_byte_offset) do |byte_offset, byte|
      if byte == 10
        chunk_size += byte_offset - last_byte_offset
        break
      end
    end
  end

  chunk = buffer.get_string(offset, chunk_size)

  data = {}

  chunk.each_line(chomp: true) do |line|
    name, value_str = line.split(";")

    value = value_str.to_f

    measurement = data[name]
    measurement ||= data[name] = Measurement.new(value, value, 0, 0)
    measurement.min = value if value < measurement.min
    measurement.max = value if value > measurement.max
    measurement.count += 1
    measurement.sum += value
  end

  data
end

main

# 100M: 26sec on windows 6 threads