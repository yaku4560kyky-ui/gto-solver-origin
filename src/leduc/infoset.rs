use super::card::LeducCard;
use super::game::LeducState;

pub fn key(state: &LeducState, player: usize) -> String {
    let private = card_name(state.private_cards[player]);
    let community = state.community.map(card_name).unwrap_or("-");
    let history = state
        .history
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join("");
    format!("{}|{}|{}", private, community, history)
}

fn card_name(card: LeducCard) -> &'static str {
    match card {
        LeducCard::Jack => "J",
        LeducCard::Queen => "Q",
        LeducCard::King => "K",
    }
}
