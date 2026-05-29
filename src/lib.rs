use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::HashMap;

mod action_abstraction;
mod card_abstraction;
mod cfr;
mod cfr_plus;
mod hand_evaluator;
mod nlhe;

#[pyfunction]
fn train_kuhn_cfr(iterations: u32) -> PyResult<HashMap<String, Vec<f64>>> {
    Ok(cfr::train(iterations)
        .into_iter()
        .map(|(info_set, node)| {
            let strategy = node.get_average_strategy();
            (info_set, vec![strategy[0], strategy[1]])
        })
        .collect())
}

#[pyfunction]
fn train_kuhn_cfr_plus(iterations: u32) -> PyResult<HashMap<String, Vec<f64>>> {
    Ok(cfr_plus::train(iterations)
        .into_iter()
        .map(|(info_set, node)| {
            let strategy = node.get_average_strategy();
            (info_set, vec![strategy[0], strategy[1]])
        })
        .collect())
}

#[pyfunction]
fn evaluate_hand(cards: Vec<u8>) -> PyResult<u32> {
    if cards.len() != 7 {
        return Err(PyValueError::new_err("Need 7 cards"));
    }

    let mut array = [0u8; 7];
    array.copy_from_slice(&cards);

    Ok(match hand_evaluator::evaluate_7card(&array) {
        hand_evaluator::HandRank::HighCard(_) => 0,
        hand_evaluator::HandRank::OnePair(_) => 1,
        hand_evaluator::HandRank::TwoPair(_) => 2,
        hand_evaluator::HandRank::ThreeOfAKind(_) => 3,
        hand_evaluator::HandRank::Straight(_) => 4,
        hand_evaluator::HandRank::Flush(_) => 5,
        hand_evaluator::HandRank::FullHouse(_) => 6,
        hand_evaluator::HandRank::FourOfAKind(_) => 7,
        hand_evaluator::HandRank::StraightFlush(_) => 8,
    })
}

#[pyfunction]
fn classify_preflop(card1: u8, card2: u8) -> PyResult<u8> {
    Ok(card_abstraction::classify_preflop(&[card1, card2]))
}

#[pymodule]
fn gto_solver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(train_kuhn_cfr, m)?)?;
    m.add_function(wrap_pyfunction!(train_kuhn_cfr_plus, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_hand, m)?)?;
    m.add_function(wrap_pyfunction!(classify_preflop, m)?)?;
    Ok(())
}
