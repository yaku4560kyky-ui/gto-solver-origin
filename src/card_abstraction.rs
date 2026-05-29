#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct HandGroup {
    pub group_id: u8,
    pub hand: [u8; 2],
}

pub const PREFLOP_GROUPS: [[u8; 2]; 169] = build_preflop_groups();

pub fn classify_preflop(cards: &[u8; 2]) -> u8 {
    let rank_a = cards[0] / 4;
    let rank_b = cards[1] / 4;
    let suit_a = cards[0] % 4;
    let suit_b = cards[1] % 4;

    if rank_a == rank_b {
        return 12 - rank_a;
    }

    let high = rank_a.max(rank_b);
    let low = rank_a.min(rank_b);
    let combo_index = non_pair_index(high, low);

    if suit_a == suit_b {
        13 + combo_index
    } else {
        91 + combo_index
    }
}

const fn build_preflop_groups() -> [[u8; 2]; 169] {
    let mut groups = [[0u8; 2]; 169];
    let mut idx = 0usize;

    let mut rank = 12i32;
    while rank >= 0 {
        groups[idx] = [(rank as u8) * 4, (rank as u8) * 4 + 1];
        idx += 1;
        rank -= 1;
    }

    let mut high = 12i32;
    while high >= 1 {
        let mut low = high - 1;
        while low >= 0 {
            groups[idx] = [(high as u8) * 4, (low as u8) * 4];
            idx += 1;
            low -= 1;
        }
        high -= 1;
    }

    high = 12;
    while high >= 1 {
        let mut low = high - 1;
        while low >= 0 {
            groups[idx] = [(high as u8) * 4, (low as u8) * 4 + 1];
            idx += 1;
            low -= 1;
        }
        high -= 1;
    }

    groups
}

fn non_pair_index(high: u8, low: u8) -> u8 {
    debug_assert!(high > low);

    let mut idx = 0u8;
    for rank in ((high + 1)..=12).rev() {
        idx += rank;
    }
    idx + (high - 1 - low)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aa_is_pair_group_id() {
        assert_eq!(classify_preflop(&[12 * 4, 12 * 4 + 1]), 0);
        assert_eq!(PREFLOP_GROUPS[0], [12 * 4, 12 * 4 + 1]);
    }

    #[test]
    fn aks_is_suited_group_id() {
        assert_eq!(classify_preflop(&[12 * 4, 11 * 4]), 13);
        assert_eq!(PREFLOP_GROUPS[13], [12 * 4, 11 * 4]);
    }

    #[test]
    fn ako_is_offsuit_group_id() {
        assert_eq!(classify_preflop(&[12 * 4, 11 * 4 + 1]), 91);
        assert_eq!(PREFLOP_GROUPS[91], [12 * 4, 11 * 4 + 1]);
    }

    #[test]
    fn all_cards_map_into_169_groups() {
        let mut seen = [false; 169];
        for a in 0u8..52 {
            for b in (a + 1)..52 {
                seen[classify_preflop(&[a, b]) as usize] = true;
            }
        }

        assert!(seen.iter().all(|&present| present));
    }
}
