use super::card::{LeducCard, LeducDeck};

pub const CHECK_PASS: u8 = 0;
pub const BET_CALL: u8 = 1;
pub const RAISE: u8 = 2;
pub const FOLD: u8 = 3;

#[derive(Debug, Clone)]
pub struct LeducState {
    pub private_cards: [LeducCard; 2],
    pub community: Option<LeducCard>,
    pub history: Vec<u8>,
    pub pot: i64,
    pub stacks: [i64; 2],
    pub round: u8,
    round_start: usize,
}

impl LeducState {
    pub fn new(private_cards: [LeducCard; 2]) -> Self {
        Self {
            private_cards,
            community: None,
            history: Vec::new(),
            pot: 2,
            stacks: [-1, -1],
            round: 0,
            round_start: 0,
        }
    }

    pub fn remaining_community_cards(&self) -> Vec<LeducCard> {
        let mut used = [false; 6];
        let deck = LeducDeck::cards();
        for card in self.private_cards {
            if let Some(index) = deck
                .iter()
                .enumerate()
                .find(|(i, c)| !used[*i] && **c == card)
                .map(|(i, _)| i)
            {
                used[index] = true;
            }
        }
        if let Some(card) = self.community {
            if let Some(index) = deck
                .iter()
                .enumerate()
                .find(|(i, c)| !used[*i] && **c == card)
                .map(|(i, _)| i)
            {
                used[index] = true;
            }
        }
        deck.iter()
            .enumerate()
            .filter_map(|(i, card)| (!used[i]).then_some(*card))
            .collect()
    }

    pub fn apply_action(&mut self, action: u8) {
        if self.is_terminal() || !self.legal_actions().contains(&action) {
            return;
        }

        let player = self.current_player();
        let facing_bet = self.facing_bet();
        let bet_size = if self.round == 0 { 2 } else { 4 };
        match action {
            BET_CALL => {
                let amount = if facing_bet {
                    self.amount_to_call()
                } else {
                    bet_size
                };
                self.stacks[player] -= amount;
                self.pot += amount;
            }
            RAISE => {
                let amount = self.amount_to_call() + bet_size;
                self.stacks[player] -= amount;
                self.pot += amount;
            }
            _ => {}
        }

        self.history.push(action);
        if action != FOLD && self.round_complete() && self.round == 0 {
            self.round = 1;
            self.round_start = self.history.len();
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.history.last() == Some(&FOLD) || (self.round == 1 && self.round_complete())
    }

    pub fn payoff(&self, player: usize) -> f64 {
        if self.history.last() == Some(&FOLD) {
            let folder = (self.history.len() - self.round_start - 1) % 2;
            let winner = 1 - folder;
            return self.payoff_for_winner(player, winner);
        }

        let score0 = self.hand_score(0);
        let score1 = self.hand_score(1);
        if score0 == score1 {
            0.0
        } else if score0 > score1 {
            self.payoff_for_winner(player, 0)
        } else {
            self.payoff_for_winner(player, 1)
        }
    }

    pub fn current_player(&self) -> usize {
        (self.history.len() - self.round_start) % 2
    }

    pub fn legal_actions(&self) -> Vec<u8> {
        if self.is_terminal() || (self.round == 1 && self.community.is_none()) {
            return Vec::new();
        }

        if self.facing_bet() {
            if self.bets_this_round() < 2 {
                vec![BET_CALL, RAISE, FOLD]
            } else {
                vec![BET_CALL, FOLD]
            }
        } else {
            vec![CHECK_PASS, BET_CALL]
        }
    }

    fn payoff_for_winner(&self, player: usize, winner: usize) -> f64 {
        if player == winner {
            (self.pot + self.stacks[player]) as f64
        } else {
            self.stacks[player] as f64
        }
    }

    fn hand_score(&self, player: usize) -> i64 {
        let private = self.private_cards[player] as i64;
        match self.community {
            Some(community) if community == self.private_cards[player] => 100 + private,
            _ => private,
        }
    }

    fn round_actions(&self) -> &[u8] {
        &self.history[self.round_start..]
    }

    fn bets_this_round(&self) -> usize {
        self.round_actions()
            .iter()
            .filter(|&&action| action == BET_CALL || action == RAISE)
            .count()
    }

    fn facing_bet(&self) -> bool {
        let mut outstanding = false;
        for &action in self.round_actions() {
            match action {
                BET_CALL => outstanding = !outstanding,
                RAISE => outstanding = true,
                _ => {}
            }
        }
        outstanding
    }

    fn amount_to_call(&self) -> i64 {
        if self.facing_bet() {
            if self.round_actions().contains(&RAISE) {
                if self.round == 0 { 4 } else { 8 }
            } else if self.round == 0 {
                2
            } else {
                4
            }
        } else {
            0
        }
    }

    fn round_complete(&self) -> bool {
        let actions = self.round_actions();
        if actions.len() >= 2 && actions.iter().all(|&action| action == CHECK_PASS) {
            return true;
        }
        !actions.is_empty() && !self.facing_bet() && self.bets_this_round() > 0
    }
}
