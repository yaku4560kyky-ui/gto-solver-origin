use rand::Rng;
use rand::seq::SliceRandom;

use crate::action_abstraction::{BetSize, STANDARD_BET_SIZES, bet_amount};
use crate::hand_evaluator::evaluate_7card;

const SMALL_BLIND: i64 = 50;
const BIG_BLIND: i64 = 100;
const STARTING_STACK: i64 = 10_000;

#[derive(Debug, Clone)]
pub struct NLHEState {
    pub hole_cards: [[u8; 2]; 2],
    pub board: [u8; 5],
    pub street: u8,
    pub pot: i64,
    pub stacks: [i64; 2],
    pub player_bets: [i64; 2],
    pub current_bet: i64,
    pub to_act: usize,
    pub folded: Option<usize>,
    pub terminal: bool,
    actions_this_round: u8,
}

impl NLHEState {
    pub fn new_random(rng: &mut impl Rng) -> Self {
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(rng);

        let hole_cards = [[deck[0], deck[2]], [deck[1], deck[3]]];
        let board = [deck[4], deck[5], deck[6], deck[7], deck[8]];

        NLHEState {
            hole_cards,
            board,
            street: 0,
            pot: SMALL_BLIND + BIG_BLIND,
            stacks: [STARTING_STACK - SMALL_BLIND, STARTING_STACK - BIG_BLIND],
            player_bets: [SMALL_BLIND, BIG_BLIND],
            current_bet: BIG_BLIND,
            to_act: 0,
            folded: None,
            terminal: false,
            actions_this_round: 0,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.terminal || self.folded.is_some()
    }

    pub fn current_player(&self) -> usize {
        self.to_act
    }

    pub fn legal_actions(&self) -> Vec<BetSize> {
        if self.is_terminal() {
            return Vec::new();
        }

        let call_amount = self.call_amount(self.to_act);
        let mut actions = Vec::new();

        if call_amount > 0 {
            actions.push(BetSize::Fold);
            actions.push(BetSize::Call);
        } else {
            actions.push(BetSize::Check);
        }

        if self.stacks[self.to_act] > call_amount {
            for &size in &STANDARD_BET_SIZES {
                actions.push(BetSize::BetFraction(size));
            }
        }

        actions
    }

    pub fn apply_action(&mut self, action: BetSize) {
        if self.is_terminal() {
            return;
        }

        let player = self.to_act;
        match action {
            BetSize::Fold => {
                self.folded = Some(player);
                self.terminal = true;
                return;
            }
            BetSize::Check => {
                if self.call_amount(player) > 0 {
                    self.folded = Some(player);
                    self.terminal = true;
                    return;
                }
                self.actions_this_round += 1;
            }
            BetSize::Call => {
                let amount = self.call_amount(player).min(self.stacks[player]);
                self.commit(player, amount);
                self.actions_this_round += 1;
            }
            BetSize::BetFraction(fraction) => {
                let call_amount = self.call_amount(player);
                let raise_amount = bet_amount(fraction, self.pot).max(BIG_BLIND);
                let total = (call_amount + raise_amount).min(self.stacks[player]);
                self.commit(player, total);
                self.current_bet = self.current_bet.max(self.player_bets[player]);
                self.actions_this_round = 1;
            }
        }

        if self.round_complete() {
            self.advance_street();
        } else {
            self.to_act = 1 - self.to_act;
        }
    }

    pub fn payoff(&self) -> [i64; 2] {
        if let Some(folder) = self.folded {
            let winner = 1 - folder;
            let mut payoff = [-self.player_total_contribution(0), -self.player_total_contribution(1)];
            payoff[winner] += self.pot;
            return payoff;
        }

        let hand_0 = self.showdown_hand(0);
        let hand_1 = self.showdown_hand(1);
        let contribution = [
            STARTING_STACK - self.stacks[0],
            STARTING_STACK - self.stacks[1],
        ];

        match hand_0.cmp(&hand_1) {
            std::cmp::Ordering::Greater => [self.pot - contribution[0], -contribution[1]],
            std::cmp::Ordering::Less => [-contribution[0], self.pot - contribution[1]],
            std::cmp::Ordering::Equal => [
                self.pot / 2 - contribution[0],
                self.pot - self.pot / 2 - contribution[1],
            ],
        }
    }

    fn call_amount(&self, player: usize) -> i64 {
        (self.current_bet - self.player_bets[player]).max(0)
    }

    fn commit(&mut self, player: usize, amount: i64) {
        let chips = amount.min(self.stacks[player]).max(0);
        self.stacks[player] -= chips;
        self.player_bets[player] += chips;
        self.pot += chips;
    }

    fn round_complete(&self) -> bool {
        self.actions_this_round >= 2 && self.player_bets[0] == self.player_bets[1]
    }

    fn advance_street(&mut self) {
        if self.street >= 3 {
            self.terminal = true;
            return;
        }

        self.street += 1;
        self.player_bets = [0, 0];
        self.current_bet = 0;
        self.actions_this_round = 0;
        self.to_act = 0;
    }

    fn showdown_hand(&self, player: usize) -> crate::hand_evaluator::HandRank {
        evaluate_7card(&[
            self.hole_cards[player][0],
            self.hole_cards[player][1],
            self.board[0],
            self.board[1],
            self.board[2],
            self.board[3],
            self.board[4],
        ])
    }

    fn player_total_contribution(&self, player: usize) -> i64 {
        STARTING_STACK - self.stacks[player]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn new_random_posts_blinds() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let state = NLHEState::new_random(&mut rng);

        assert_eq!(state.street, 0);
        assert_eq!(state.pot, 150);
        assert_eq!(state.stacks, [9_950, 9_900]);
        assert_eq!(state.current_player(), 0);
    }

    #[test]
    fn fold_ends_hand() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);
        let mut state = NLHEState::new_random(&mut rng);
        state.apply_action(BetSize::Fold);

        assert!(state.is_terminal());
        assert_eq!(state.payoff(), [-50, 50]);
    }
}
