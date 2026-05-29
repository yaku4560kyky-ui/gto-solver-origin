use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LeducCard {
    Jack = 1,
    Queen = 2,
    King = 3,
}

impl fmt::Display for LeducCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            LeducCard::Jack => "J",
            LeducCard::Queen => "Q",
            LeducCard::King => "K",
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeducDeck {
    cards: Vec<LeducCard>,
}

impl LeducDeck {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            cards: vec![
                LeducCard::Jack,
                LeducCard::Jack,
                LeducCard::Queen,
                LeducCard::Queen,
                LeducCard::King,
                LeducCard::King,
            ],
        }
    }

    pub fn cards() -> [LeducCard; 6] {
        [
            LeducCard::Jack,
            LeducCard::Jack,
            LeducCard::Queen,
            LeducCard::Queen,
            LeducCard::King,
            LeducCard::King,
        ]
    }

    #[allow(dead_code)]
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    #[allow(dead_code)]
    pub fn deal(&mut self) -> Option<LeducCard> {
        self.cards.pop()
    }
}
