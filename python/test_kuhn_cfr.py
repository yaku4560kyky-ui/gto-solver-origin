from kuhn_cfr import BET, PASS, get_average_strategy, train
import pytest


def test_j_bet_frequency_approximately_one_third():
    strategy = get_average_strategy(train(100_000))
    assert strategy["1"][BET] == pytest.approx(1.0 / 3.0, abs=0.05)


def test_k_bet_frequency_above_ninety_percent():
    strategy = get_average_strategy(train(100_000))
    assert strategy["3"][BET] > 0.90


def test_q_pass_frequency_above_ninety_percent():
    strategy = get_average_strategy(train(100_000))
    assert strategy["2"][PASS] > 0.90
