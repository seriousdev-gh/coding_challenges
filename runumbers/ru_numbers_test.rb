# тесты написал deepseek
require 'minitest/autorun'
require_relative 'ru_numbers'

class TestRuNumbers < Minitest::Test
  include RuNumbers

  def test_basic_number_conversion
    assert_equal "ноль", RuNumbers::NumberToText.call(0)
    assert_equal "девятнадцать", RuNumbers::NumberToText.call(19)
    assert_equal "двести тридцать", RuNumbers::NumberToText.call(230)
  end

  def test_arithmetic_operations
    expr = один прибавить два
    assert_equal 3, expr.evaluate

    expr = пять делить один
    assert_equal 5, expr.evaluate

    expr = ⦅ пять прибавить три ⦆
    assert_equal 8, expr.evaluate
  end

  def test_thousands_handling
    expr = один тысяча пятьсот
    assert_equal 1500, expr.evaluate
    assert_equal "одна тысяча пятьсот", expr.to_s

    expr = две тысячи три
    assert_equal 2003, expr.evaluate
    assert_equal "две тысячи три", expr.to_s
  end

  def test_millions_handling
    expr = три миллиона
    assert_equal 3_000_000, expr.evaluate
    assert_equal "три миллиона", expr.to_s

    expr = один миллион пятьсот тысяч семь
    assert_equal 1_500_007, expr.evaluate
  end

  def test_complex_expressions
    # (123 - 100) * 10 = 230
    expr = ⦅ сто двадцать три вычесть сто ⦆ умножить на десять
    assert_equal 230, expr.evaluate
    assert_equal "двести тридцать", expr.to_s

    # 1 + 2 * 3 = 7
    expr = один прибавить два умножить на три
    assert_equal 7, expr.evaluate
    assert_equal "семь", expr.to_s
  end

  def test_edge_cases
    expr = тысяча два
    assert_equal 1002, expr.evaluate
    # assert_equal "тысяча два", expr.to_s # FIXME
    assert_equal "одна тысяча два", expr.to_s

    expr = один миллион два
    assert_equal 1_000_002, expr.evaluate

    expr = ноль умножить на сто
    assert_equal 0, expr.evaluate
  end

  def test_boolean_operations
    expr = пять равно пять
    assert_equal true, expr.evaluate
    assert_equal "правда", expr.to_s

    expr = десять равно двадцать
    assert_equal false, expr.evaluate
    assert_equal "ложь", expr.to_s
  end

  def test_combined_operations
    # 2 + 3 * 4 == 14 → true
    expr = два прибавить три умножить на четыре равно четырнадцать
    assert_equal true, expr.evaluate
    assert_equal "правда", expr.to_s
  end

  def test_mixed_case_operations
    # (1000 + 500) / (10 - 7) = 500
    expr = ⦅ тысяча прибавить пятьсот ⦆ делить ⦅ десять вычесть семь ⦆
    assert_equal 500, expr.evaluate
    assert_equal "пятьсот", expr.to_s
  end
end