module RuNumbers
  Numbers = [
    [0, 'ноль'],
    [1, 'один', 'одна'],
    [2, 'два', 'две'],
    [3, 'три'],
    [4, 'четыре'],
    [5, 'пять'],
    [6, 'шесть'],
    [7, 'семь'],
    [8, 'восемь'],
    [9, 'девять'],
    [10, 'десять'],
    [11, 'одиннадцать'],
    [12, 'двенадцать'],
    [13, 'тринадцать'],
    [14, 'четырнадцать'],
    [15, 'пятнадцать'],
    [16, 'шестнадцать'],
    [17, 'семнадцать'],
    [18, 'восемнадцать'],
    [19, 'девятнадцать'],
    [20, 'двадцать'],
    [30, 'тридцать'],
    [40, 'сорок'],
    [50, 'пятьдесят'],
    [60, 'шестьдесят'],
    [70, 'семьдесят'],
    [80, 'восемьдесят'],
    [90, 'девяносто'],
    [100, 'сто'],
    [200, 'двести'],
    [300, 'триста'],
    [400, 'четыреста'],
    [500, 'пятьсот'],
    [600, 'шестьсот'],
    [700, 'семьсот'],
    [800, 'восемьсот'],
    [900, 'девятьсот'],
    [1_000, 'тысяча', 'тысячи', 'тысяч'],
    [1_000_000, 'миллион', 'миллиона', 'миллионов'],
  ]

  Numbers.each do |info|
    n, *words = info
    words.each do |w|
      define_method(w) do |next_token = nil|
        Token.new(:number, n, next_token)
      end
    end
  end
  
  Binop = Data.define(:type, :left, :right) do
    def evaluate
      lhs = left.respond_to?(:evaluate) ? left.evaluate : left
      rhs = right.respond_to?(:evaluate) ? right.evaluate : right
      case type
      when :eq then lhs == rhs
      when :plus then lhs + rhs
      when :sub then lhs - rhs
      when :mult then lhs * rhs
      when :div then lhs / rhs
      end
    end
  end

  class NumberToText
    def self.call(number, gender = nil)
      return Numbers[0][1] if number == 0
      return 'правда' if number == true
      return 'ложь' if number == false

      res = []
      mil = number / 1_000_000
      if mil > 0
        res << call(mil)
        info = Numbers.find { _1.first == 1_000_000 }
        rem = mil % 100
        
        res <<
          if (11..14).cover?(rem)
            info[3]
          else
            rem = mil % 10
            if rem == 2 || rem == 3 || rem == 4
              info[2]
            elsif rem == 1
              info[1]
            else
              info[3]
            end
          end

        number %= 1_000_000
      end

      thousands = number / 1_000
      if thousands > 0
        res << call(thousands, :f)
        info = Numbers.find { _1.first == 1_000 }
        rem = mil % 100
        res <<
          if (11..14).cover?(rem)
            info[3]
          else
            rem = thousands % 10
          
            if rem == 2 || rem == 3 || rem == 4
              info[2]
            elsif rem == 1
              info[1]
            else
              info[3]
            end
          end

        number %= 1_000
      end

      hundreds = number / 100
      if hundreds > 0
        info = Numbers.find { _1.first == hundreds * 100 }
        res << info[1]
        number %= 100
      end

      tenths = number / 10
      if tenths > 0
        if number <= 20
          info = Numbers.find { _1.first == number }
          number = 0
        else
          info = Numbers.find { _1.first == tenths * 10 }
        end
        
        res << info[1]
        number %= 10
      end

      info = Numbers.find { _1.first == number }
      if info && number != 0
        if gender == :f && (number == 1 || number == 2)
          res << info[2]
        else
          res << info[1]
        end
      end

      res.join(' ')
    end

  end

  class Parser
    def initialize(tokens)
      @pos = 0
      @tokens = tokens
    end

    def parse
      parse_equality
    end


    def parse_equality
      left = parse_addition_subtraction
      if current_type == :eq
        consume(:eq)
        right = parse
        return Binop.new(:eq, left, right)
      end
      left
    end

    def parse_addition_subtraction
      left = parse_multiplication_division
      type = current_type
      if type == :plus || type == :sub
        consume(type)
        right = parse_addition_subtraction
        return Binop.new(type, left, right)
      end
      left
    end

    def parse_multiplication_division
      left = parse_parenthesis
      type = current_type
      if type == :mult || type == :div
        consume(type)
        right = parse_multiplication_division
        return Binop.new(type, left, right)
      end
      left
    end

    def current_type
      @tokens[@pos]&.type
    end

    def parse_parenthesis
      if current_type == :paren_left
        consume(:paren_left) 
        expr = parse
        consume(:paren_right)
        expr
      else
        parse_number
      end
    end

    def parse_number
      numbers = [consume(:number).value]
      while current_type == :number
        numbers << consume(:number).value
      end

      sum = 0
      moving_sum = 0
      numbers.each_with_index do |n|
        if n % 1000 == 0
          moving_sum = 1 if moving_sum == 0
          sum += moving_sum * n
          moving_sum = 0
        else
          moving_sum += n
        end
      end
      sum += moving_sum

      sum
    end

    def consume(type)
      if current_type == type
        ret = @tokens[@pos]
        @pos += 1
        ret
      else
        raise "Expected: #{type}, got: #{@tokens[@pos].inspect}"
      end
    end
  end

  class Token
    attr_reader :type, :value, :next_token
    def initialize(type, value, next_token)
      @type = type
      @value = value
      @next_token = next_token
    end

    def evaluate
      expr = Parser.new(linearize).parse
      expr.is_a?(Binop) ? expr.evaluate : expr
    end

    def linearize
      res = []
      res << self if type != :skip
      res += next_token.linearize if next_token
      res
    end

    def to_s
      NumberToText.call(evaluate)
    end
    
    def to_i
      evaluate
    end

    def inspect
      "(#{type}: #{value})"
    end
  end

  def равно(next_token)
    Token.new(:eq, nil, next_token)
  end

  def прибавить(next_token)
    Token.new(:plus, nil, next_token)
  end

  def вычесть(next_token)
    Token.new(:sub, nil, next_token)
  end

  def умножить(next_token)
    Token.new(:mult, nil, next_token)
  end

  def делить(next_token)
    Token.new(:div, nil, next_token)
  end

  def на(next_token)
    Token.new(:skip, nil, next_token)
  end

  def ⦅(next_token)
    Token.new(:paren_left, nil, next_token)
  end

  def ⦆(next_token = nil)
    Token.new(:paren_right, nil, next_token)
  end
end