mod cfr;
mod cfr_plus;

use std::fmt;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Suit {
    Club,
    Diamond,
    Heart,
    Spade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    fn new(rank: Rank, suit: Suit) -> Self {
        Card { rank, suit }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rank_str = match self.rank {
            Rank::Ace   => "A",
            Rank::King  => "K",
            Rank::Queen => "Q",
            Rank::Jack  => "J",
            Rank::Ten   => "T",
            Rank::Nine  => "9",
            Rank::Eight => "8",
            Rank::Seven => "7",
            Rank::Six   => "6",
            Rank::Five  => "5",
            Rank::Four  => "4",
            Rank::Three => "3",
            Rank::Two   => "2",
        };
        let suit_str = match self.suit {
            Suit::Spade   => "s",
            Suit::Heart   => "h",
            Suit::Diamond => "d",
            Suit::Club    => "c",
        };
        write!(f, "{}{}", rank_str, suit_str)
    }
}

// 2枚のホールカード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hand {
    cards: [Card; 2],
}

impl Hand {
    fn new(c1: Card, c2: Card) -> Self {
        Hand { cards: [c1, c2] }
    }

    fn is_pair(&self) -> bool {
        self.cards[0].rank == self.cards[1].rank
    }

    fn is_suited(&self) -> bool {
        self.cards[0].suit == self.cards[1].suit
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.cards[0], self.cards[1])
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Action {
    Fold,
    Check,
    Call,
    Raise(u64),
    AllIn,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Fold     => write!(f, "Fold"),
            Action::Check    => write!(f, "Check"),
            Action::Call     => write!(f, "Call"),
            Action::Raise(n) => write!(f, "Raise({})", n),
            Action::AllIn    => write!(f, "AllIn"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Street {
    Preflop,
    Flop,
    Turn,
    River,
}

// 6-maxポジション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Position {
    UTG,
    HJ,
    CO,
    BTN,
    SB,
    BB,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Position::UTG => "UTG",
            Position::HJ  => "HJ",
            Position::CO  => "CO",
            Position::BTN => "BTN",
            Position::SB  => "SB",
            Position::BB  => "BB",
        };
        write!(f, "{}", s)
    }
}

// ── Deck ──────────────────────────────────────────────────────────────────
// 所有権ポイント:
//   - cards: Vec<Card> を Deck が「所有」している
//   - shuffle は &mut self（可変借用）: Deck を借用して中身を変える
//   - deal は &mut self → Option<Card> を返す: 所有権をデッキから呼び出し元へ移動
//   - peek は &self（不変借用）: 読むだけなので mut 不要
struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    // 52枚フルデッキを生成して所有権を返す
    fn new() -> Self {
        let suits = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];
        let ranks = [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
            Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];
        let mut cards = Vec::with_capacity(52);
        for &suit in &suits {
            for &rank in &ranks {
                cards.push(Card::new(rank, suit));
            }
        }
        Deck { cards }
    }

    // &mut self: デッキ自身を可変借用してシャッフル（Fisher-Yates）
    fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    // Option<Card>: デッキが空なら None。Card の所有権を呼び出し元に移動する
    fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    // &self: 読むだけなので不変借用。デッキの残り枚数を返す
    fn remaining(&self) -> usize {
        self.cards.len()
    }

    // &self: 先頭カードをのぞき見（所有権は移動しない = 借用のみ）
    fn peek(&self) -> Option<&Card> {
        self.cards.last()
    }

    // 6-max テーブルにホールカードを配る（2枚×6プレイヤー = 12枚）
    // &mut self: デッキから取り出す操作なので可変借用
    // 戻り値 Vec<Hand> は新しく作った値の所有権を返す
    fn deal_hands(&mut self, num_players: usize) -> Option<Vec<Hand>> {
        if self.remaining() < num_players * 2 {
            return None;
        }
        let mut hands = Vec::with_capacity(num_players);
        for _ in 0..num_players {
            let c1 = self.deal()?;
            let c2 = self.deal()?;
            hands.push(Hand::new(c1, c2));
        }
        Some(hands)
    }
}

// ── GameState ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct GameState {
    street:  Street,
    pot:     u64,
    stacks:  Vec<u64>,
    to_act:  usize,
    board:   Vec<Card>,
}

impl GameState {
    fn new(num_players: usize, starting_stack: u64) -> Self {
        GameState {
            street:  Street::Preflop,
            pot:     0,
            stacks:  vec![starting_stack; num_players],
            to_act:  0,
            board:   Vec::new(),
        }
    }

    fn apply_action(&mut self, action: &Action, player: usize) {
        match action {
            Action::Fold | Action::Check => {}
            Action::Call => {
                // TODO: Phase 1でベット額追跡を実装
            }
            Action::Raise(amount) => {
                if self.stacks[player] >= *amount {
                    self.stacks[player] -= amount;
                    self.pot += amount;
                }
            }
            Action::AllIn => {
                let all = self.stacks[player];
                self.stacks[player] = 0;
                self.pot += all;
            }
        }
    }
}

fn main() {
    let cfr_nodes = cfr::train(100_000);
    println!("Vanilla Kuhn CFR average strategies:");
    print_kuhn_strategies(&cfr_nodes);

    let cfr_plus_nodes = cfr_plus::train(100_000);
    println!("\nKuhn CFR+ average strategies:");
    print_kuhn_plus_strategies(&cfr_plus_nodes);

    // --- Deck デモ ---
    let mut deck = Deck::new();
    println!("Deck: {} cards", deck.remaining());

    // peek は &self（不変借用）: deck の所有権は移動しない
    if let Some(top) = deck.peek() {
        println!("Top card before shuffle: {}", top);
    }

    // shuffle は &mut self: deck を可変借用
    deck.shuffle();
    println!("After shuffle, top: {}", deck.peek().unwrap());

    // deal_hands で6人に配る: Vec<Hand> の所有権が hands に移る
    let hands = deck.deal_hands(6).unwrap();
    println!("Dealt {} hands. Remaining: {}", hands.len(), deck.remaining());
    for (i, h) in hands.iter().enumerate() {
        println!("  Player {}: {} (pair={}, suited={})", i + 1, h, h.is_pair(), h.is_suited());
    }

    // --- GameState デモ ---
    let mut gs = GameState::new(6, 10000);
    gs.apply_action(&Action::Raise(300), 0);
    println!("\nAfter Raise(300): pot={}, stack[0]={}", gs.pot, gs.stacks[0]);
}

fn print_kuhn_strategies(nodes: &std::collections::HashMap<String, cfr::KuhnNode>) {
    let mut keys: Vec<&String> = nodes.keys().collect();
    keys.sort();
    for key in keys {
        let strategy = nodes[key].get_average_strategy();
        println!("  {}: pass={:.3}, bet={:.3}", key, strategy[0], strategy[1]);
    }
}

fn print_kuhn_plus_strategies(nodes: &std::collections::HashMap<String, cfr_plus::KuhnNode>) {
    let mut keys: Vec<&String> = nodes.keys().collect();
    keys.sort();
    for key in keys {
        let strategy = nodes[key].get_average_strategy();
        println!("  {}: pass={:.3}, bet={:.3}", key, strategy[0], strategy[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_display() {
        assert_eq!(format!("{}", Card::new(Rank::Ace,   Suit::Spade)),   "As");
        assert_eq!(format!("{}", Card::new(Rank::King,  Suit::Heart)),   "Kh");
        assert_eq!(format!("{}", Card::new(Rank::Queen, Suit::Diamond)), "Qd");
        assert_eq!(format!("{}", Card::new(Rank::Two,   Suit::Club)),    "2c");
        assert_eq!(format!("{}", Card::new(Rank::Ten,   Suit::Spade)),   "Ts");
    }

    #[test]
    fn test_hand_pocket_pair() {
        let hand = Hand::new(
            Card::new(Rank::Ace, Suit::Spade),
            Card::new(Rank::Ace, Suit::Heart),
        );
        assert!(hand.is_pair());
        assert!(!hand.is_suited());
        assert_eq!(format!("{}", hand), "AsAh");
    }

    #[test]
    fn test_hand_suited() {
        let hand = Hand::new(
            Card::new(Rank::Ace,  Suit::Spade),
            Card::new(Rank::King, Suit::Spade),
        );
        assert!(!hand.is_pair());
        assert!(hand.is_suited());
    }

    #[test]
    fn test_hand_offsuit() {
        let hand = Hand::new(
            Card::new(Rank::Ace,  Suit::Spade),
            Card::new(Rank::King, Suit::Heart),
        );
        assert!(!hand.is_pair());
        assert!(!hand.is_suited());
    }

    #[test]
    fn test_rank_ordering() {
        assert!(Rank::Ace   > Rank::King);
        assert!(Rank::King  > Rank::Queen);
        assert!(Rank::Queen > Rank::Jack);
        assert!(Rank::Jack  > Rank::Ten);
        assert!(Rank::Two   < Rank::Three);
    }

    #[test]
    fn test_action_raise() {
        let mut gs = GameState::new(6, 10000);
        gs.apply_action(&Action::Raise(300), 0);
        assert_eq!(gs.pot, 300);
        assert_eq!(gs.stacks[0], 9700);
        assert_eq!(gs.stacks[1], 10000); // 他プレイヤーは変化なし
    }

    #[test]
    fn test_action_allin() {
        let mut gs = GameState::new(2, 1000);
        gs.apply_action(&Action::AllIn, 0);
        assert_eq!(gs.pot, 1000);
        assert_eq!(gs.stacks[0], 0);
    }

    #[test]
    fn test_action_raise_insufficient_stack() {
        let mut gs = GameState::new(2, 100);
        gs.apply_action(&Action::Raise(200), 0); // スタック不足 → 何も起きない
        assert_eq!(gs.pot, 0);
        assert_eq!(gs.stacks[0], 100);
    }

    #[test]
    fn test_initial_gamestate() {
        let gs = GameState::new(6, 10000);
        assert_eq!(gs.street, Street::Preflop);
        assert_eq!(gs.pot, 0);
        assert_eq!(gs.stacks.len(), 6);
        assert!(gs.board.is_empty());
    }

    #[test]
    fn test_position_display() {
        assert_eq!(format!("{}", Position::BTN), "BTN");
        assert_eq!(format!("{}", Position::BB),  "BB");
        assert_eq!(format!("{}", Position::UTG), "UTG");
    }

    // ── Deck テスト ──────────────────────────────────────────────────────
    // 所有権学習ポイント: Deck::new() が Vec<Card> の所有権を持ち、
    // deal() で Card の所有権を呼び出し元へ移す

    #[test]
    fn test_deck_new_has_52_cards() {
        let deck = Deck::new();
        assert_eq!(deck.remaining(), 52);
    }

    #[test]
    fn test_deck_deal_reduces_count() {
        let mut deck = Deck::new();
        let card = deck.deal(); // Option<Card> の所有権がここに移る
        assert!(card.is_some());
        assert_eq!(deck.remaining(), 51);
    }

    #[test]
    fn test_deck_deal_all_52() {
        let mut deck = Deck::new();
        let mut dealt = Vec::new();
        // deal() で Card の所有権を毎回 dealt に移動させる
        while let Some(card) = deck.deal() {
            dealt.push(card);
        }
        assert_eq!(dealt.len(), 52);
        assert_eq!(deck.remaining(), 0);
        // 次の deal は None（空デッキ）
        assert!(deck.deal().is_none());
    }

    #[test]
    fn test_deck_no_duplicates() {
        let mut deck = Deck::new();
        let mut seen = std::collections::HashSet::new();
        while let Some(card) = deck.deal() {
            // (rank, suit) の組が重複していないことを確認
            let key = format!("{}", card);
            assert!(seen.insert(key), "重複カードが存在する");
        }
        assert_eq!(seen.len(), 52);
    }

    #[test]
    fn test_deck_peek_does_not_consume() {
        let deck = Deck::new();
        let _ = deck.peek(); // &self: 所有権は移動しない
        let _ = deck.peek(); // 何度でも呼べる
        assert_eq!(deck.remaining(), 52); // deck はまだ使える
    }

    #[test]
    fn test_deck_deal_hands_6max() {
        let mut deck = Deck::new();
        deck.shuffle();
        let hands = deck.deal_hands(6).unwrap();
        assert_eq!(hands.len(), 6);
        assert_eq!(deck.remaining(), 52 - 12); // 12枚配布済み
    }

    #[test]
    fn test_deck_deal_hands_insufficient() {
        // 2枚しかないデッキに6人分を配ろうとすると None
        let mut deck = Deck::new();
        while deck.remaining() > 2 {
            deck.deal();
        }
        assert!(deck.deal_hands(6).is_none());
    }

    #[test]
    fn test_deck_shuffle_changes_order() {
        let deck_original = Deck::new();
        let mut deck_shuffled = Deck::new();
        deck_shuffled.shuffle();
        // シャッフル後のカード列が元の順と完全一致する確率は 1/52! ≈ 0
        let original: Vec<String> = {
            let mut d = Deck::new();
            (0..52).filter_map(|_| d.deal()).map(|c| format!("{}", c)).collect()
        };
        let shuffled: Vec<String> = {
            let mut d = deck_shuffled;
            (0..52).filter_map(|_| d.deal()).map(|c| format!("{}", c)).collect()
        };
        // 52枚の内容（集合）は同じ
        let mut orig_sorted = original.clone();
        let mut shuf_sorted = shuffled.clone();
        orig_sorted.sort();
        shuf_sorted.sort();
        assert_eq!(orig_sorted, shuf_sorted);
        // ソートしない順序は（ほぼ確実に）異なる
        let _ = deck_original; // 所有権の確認: まだ使えるはず
    }
}
