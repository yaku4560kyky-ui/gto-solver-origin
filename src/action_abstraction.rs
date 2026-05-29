#[derive(Debug, Clone, PartialEq)]
pub enum BetSize {
    Fold,
    Check,
    Call,
    BetFraction(u8),
}

pub const STANDARD_BET_SIZES: [u8; 6] = [25, 33, 50, 75, 100, 150];

pub fn discretize_bet(bet_amount: i64, pot_size: i64) -> BetSize {
    if bet_amount == 0 {
        return BetSize::Check;
    }

    if pot_size <= 0 {
        return BetSize::BetFraction(STANDARD_BET_SIZES[0]);
    }

    let fraction = bet_amount * 100 / pot_size;
    let mut best = STANDARD_BET_SIZES[0];
    let mut best_distance = (fraction - best as i64).abs();

    for &size in STANDARD_BET_SIZES.iter().skip(1) {
        let distance = (fraction - size as i64).abs();
        if distance < best_distance {
            best = size;
            best_distance = distance;
        }
    }

    BetSize::BetFraction(best)
}

pub fn bet_amount(fraction_pct: u8, pot_size: i64) -> i64 {
    pot_size * fraction_pct as i64 / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_half_pot_maps_to_50() {
        assert_eq!(discretize_bet(50, 100), BetSize::BetFraction(50));
    }

    #[test]
    fn sixty_percent_maps_to_nearest_standard_size() {
        assert_eq!(discretize_bet(60, 100), BetSize::BetFraction(50));
    }

    #[test]
    fn zero_bet_maps_to_check() {
        assert_eq!(discretize_bet(0, 500), BetSize::Check);
    }
}
