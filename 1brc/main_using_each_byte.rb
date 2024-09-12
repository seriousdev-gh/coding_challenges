$entrypoint = __FILE__
require_relative 'common'

def calculate_data(file, chunk_size, index, nworkers)
  offset = index * chunk_size
  skip_line = index != 0
  buffer = IO::Buffer.map(File.open(file), nil, 0, IO::Buffer::READONLY)
  data = {}

  name = nil
  line_start_offset = 0
  number_start_offset = 0

  if skip_line
    buffer.each(:U8, offset) do |byte_offset, byte|
      if byte == 10
        chunk_size -= byte_offset - offset + 1
        offset = byte_offset + 1
        line_start_offset = offset
        break
      end
    end
  end

  pw_count = 0

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

