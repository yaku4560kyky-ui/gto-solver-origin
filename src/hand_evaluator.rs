use std::cmp::Ordering;

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Clone, Copy)]
pub enum HandRank {
    HighCard(u32),
    OnePair(u32),
    TwoPair(u32),
    ThreeOfAKind(u32),
    Straight(u32),
    Flush(u32),
    FullHouse(u32),
    FourOfAKind(u32),
    StraightFlush(u32),
}

pub fn evaluate_5card(cards: &[u8; 5]) -> HandRank {
    let mut rank_counts = [0u8; 13];
    let mut suit_counts = [0u8; 4];
    let mut ranks = [0u8; 5];

    for (idx, &card) in cards.iter().enumerate() {
        let rank = card / 4;
        let suit = card % 4;
        rank_counts[rank as usize] += 1;
        suit_counts[suit as usize] += 1;
        ranks[idx] = rank;
    }

    let is_flush = suit_counts.iter().any(|&count| count == 5);
    let straight_high = straight_high(&rank_counts);

    if let Some(high) = straight_high {
        if is_flush {
            return HandRank::StraightFlush(high as u32);
        }
    }

    let mut quads = [0u8; 1];
    let mut quad_len = 0usize;
    let mut trips = [0u8; 2];
    let mut trips_len = 0usize;
    let mut pairs = [0u8; 2];
    let mut pairs_len = 0usize;
    let mut singles = [0u8; 5];
    let mut singles_len = 0usize;

    for rank in (0..13).rev() {
        match rank_counts[rank] {
            4 => {
                quads[quad_len] = rank as u8;
                quad_len += 1;
            }
            3 => {
                trips[trips_len] = rank as u8;
                trips_len += 1;
            }
            2 => {
                pairs[pairs_len] = rank as u8;
                pairs_len += 1;
            }
            1 => {
                singles[singles_len] = rank as u8;
                singles_len += 1;
            }
            _ => {}
        }
    }

    if quad_len > 0 {
        return HandRank::FourOfAKind(pack_ranks(&[quads[0], singles[0]]));
    }

    if trips_len > 0 && pairs_len > 0 {
        return HandRank::FullHouse(pack_ranks(&[trips[0], pairs[0]]));
    }

    if is_flush {
        ranks.sort_unstable_by(|a, b| b.cmp(a));
        return HandRank::Flush(pack_ranks(&ranks));
    }

    if let Some(high) = straight_high {
        return HandRank::Straight(high as u32);
    }

    if trips_len > 0 {
        return HandRank::ThreeOfAKind(pack_ranks(&[trips[0], singles[0], singles[1]]));
    }

    if pairs_len >= 2 {
        return HandRank::TwoPair(pack_ranks(&[pairs[0], pairs[1], singles[0]]));
    }

    if pairs_len == 1 {
        return HandRank::OnePair(pack_ranks(&[pairs[0], singles[0], singles[1], singles[2]]));
    }

    ranks.sort_unstable_by(|a, b| b.cmp(a));
    HandRank::HighCard(pack_ranks(&ranks))
}

pub fn evaluate_7card(cards: &[u8; 7]) -> HandRank {
    let mut rank_counts = [0u8; 13];
    let mut suit_counts = [0u8; 4];
    let mut suited_rank_counts = [[0u8; 13]; 4];

    for &card in cards {
        let rank = (card / 4) as usize;
        let suit = (card % 4) as usize;
        rank_counts[rank] += 1;
        suit_counts[suit] += 1;
        suited_rank_counts[suit][rank] += 1;
    }

    for suit in 0..4 {
        if suit_counts[suit] >= 5 {
            if let Some(high) = straight_high(&suited_rank_counts[suit]) {
                return HandRank::StraightFlush(high as u32);
            }
        }
    }

    let mut quads = [0u8; 1];
    let mut quad_len = 0usize;
    let mut trips = [0u8; 2];
    let mut trips_len = 0usize;
    let mut pairs = [0u8; 3];
    let mut pairs_len = 0usize;
    let mut singles = [0u8; 7];
    let mut singles_len = 0usize;

    for rank in (0..13).rev() {
        match rank_counts[rank] {
            4 => {
                quads[quad_len] = rank as u8;
                quad_len += 1;
            }
            3 => {
                trips[trips_len] = rank as u8;
                trips_len += 1;
            }
            2 => {
                pairs[pairs_len] = rank as u8;
                pairs_len += 1;
            }
            1 => {
                singles[singles_len] = rank as u8;
                singles_len += 1;
            }
            _ => {}
        }
    }

    if quad_len > 0 {
        let kicker = highest_excluding(&rank_counts, quads[0]);
        return HandRank::FourOfAKind(pack_ranks(&[quads[0], kicker]));
    }

    if trips_len > 0 && (pairs_len > 0 || trips_len > 1) {
        let pair_rank = if pairs_len > 0 { pairs[0] } else { trips[1] };
        return HandRank::FullHouse(pack_ranks(&[trips[0], pair_rank]));
    }

    for suit in 0..4 {
        if suit_counts[suit] >= 5 {
            let mut flush_ranks = [0u8; 5];
            let mut len = 0usize;
            for rank in (0..13).rev() {
                if suited_rank_counts[suit][rank] > 0 {
                    flush_ranks[len] = rank as u8;
                    len += 1;
                    if len == 5 {
                        break;
                    }
                }
            }
            return HandRank::Flush(pack_ranks(&flush_ranks));
        }
    }

    if let Some(high) = straight_high(&rank_counts) {
        return HandRank::Straight(high as u32);
    }

    if trips_len > 0 {
        let kickers = top_excluding(&rank_counts, &[trips[0]], 2);
        return HandRank::ThreeOfAKind(pack_ranks(&[trips[0], kickers[0], kickers[1]]));
    }

    if pairs_len >= 2 {
        let kicker = top_excluding(&rank_counts, &[pairs[0], pairs[1]], 1);
        return HandRank::TwoPair(pack_ranks(&[pairs[0], pairs[1], kicker[0]]));
    }

    if pairs_len == 1 {
        let kickers = top_excluding(&rank_counts, &[pairs[0]], 3);
        return HandRank::OnePair(pack_ranks(&[pairs[0], kickers[0], kickers[1], kickers[2]]));
    }

    let high_cards = top_excluding(&rank_counts, &[], 5);
    HandRank::HighCard(pack_ranks(&high_cards))
}

pub fn compare_hands(a: HandRank, b: HandRank) -> Ordering {
    a.cmp(&b)
}

fn straight_high(rank_counts: &[u8; 13]) -> Option<u8> {
    for high in (4..13).rev() {
        let mut found = true;
        for rank in (high - 4)..=high {
            if rank_counts[rank] == 0 {
                found = false;
                break;
            }
        }
        if found {
            return Some(high as u8);
        }
    }

    if rank_counts[12] > 0
        && rank_counts[0] > 0
        && rank_counts[1] > 0
        && rank_counts[2] > 0
        && rank_counts[3] > 0
    {
        Some(3)
    } else {
        None
    }
}

fn pack_ranks(ranks: &[u8]) -> u32 {
    ranks
        .iter()
        .fold(0u32, |acc, &rank| acc * 13 + rank as u32)
}

fn highest_excluding(rank_counts: &[u8; 13], excluded: u8) -> u8 {
    top_excluding(rank_counts, &[excluded], 1)[0]
}

fn top_excluding(rank_counts: &[u8; 13], excluded: &[u8], count: usize) -> [u8; 5] {
    let mut ranks = [0u8; 5];
    let mut len = 0usize;

    for rank in (0..13).rev() {
        if rank_counts[rank] == 0 || excluded.contains(&(rank as u8)) {
            continue;
        }

        ranks[len] = rank as u8;
        len += 1;
        if len == count {
            break;
        }
    }

    ranks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn royal_flush_beats_straight_flush() {
        let royal = [8 * 4, 9 * 4, 10 * 4, 11 * 4, 12 * 4];
        let straight_flush = [7 * 4 + 1, 8 * 4 + 1, 9 * 4 + 1, 10 * 4 + 1, 11 * 4 + 1];

        assert!(evaluate_5card(&royal) > evaluate_5card(&straight_flush));
    }

    #[test]
    fn full_house_beats_flush() {
        let full_house = [12 * 4, 12 * 4 + 1, 12 * 4 + 2, 6 * 4, 6 * 4 + 1];
        let flush = [12 * 4 + 3, 10 * 4 + 3, 8 * 4 + 3, 5 * 4 + 3, 2 * 4 + 3];

        assert!(evaluate_5card(&full_house) > evaluate_5card(&flush));
    }

    #[test]
    fn equal_pairs_tie() {
        let a = [12 * 4, 12 * 4 + 1, 10 * 4, 8 * 4 + 2, 3 * 4 + 1];
        let b = [12 * 4 + 2, 12 * 4 + 3, 10 * 4 + 1, 8 * 4 + 3, 3 * 4 + 2];

        assert_eq!(compare_hands(evaluate_5card(&a), evaluate_5card(&b)), Ordering::Equal);
    }

    #[test]
    fn evaluate_7card_one_million_under_two_seconds() {
        let cards = [12 * 4, 12 * 4 + 1, 11 * 4, 10 * 4, 9 * 4, 8 * 4, 2 * 4 + 3];
        let start = Instant::now();
        let mut result = HandRank::HighCard(0);

        for _ in 0..1_000_000 {
            result = evaluate_7card(&cards);
        }

        assert_eq!(result, HandRank::StraightFlush(12));
        assert!(
            start.elapsed().as_secs_f64() < 2.0,
            "1M evaluate_7card calls took {:?}",
            start.elapsed()
        );
    }
}
