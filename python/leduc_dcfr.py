from __future__ import annotations

from dataclasses import dataclass, field


CHECK_PASS = 0
BET_CALL = 1
RAISE = 2
FOLD = 3
DECK = (1, 1, 2, 2, 3, 3)
CARD_NAMES = {1: "J", 2: "Q", 3: "K"}
ALPHA = 1.5
BETA = 0.0
GAMMA = 2.0


@dataclass
class LeducState:
    private_cards: tuple[int, int]
    community: int | None = None
    history: list[int] = field(default_factory=list)
    pot: int = 2
    stacks: list[int] = field(default_factory=lambda: [-1, -1])
    round: int = 0
    round_start: int = 0

    def clone(self) -> "LeducState":
        return LeducState(
            self.private_cards,
            self.community,
            self.history.copy(),
            self.pot,
            self.stacks.copy(),
            self.round,
            self.round_start,
        )

    def current_player(self) -> int:
        return (len(self.history) - self.round_start) % 2

    def round_actions(self) -> list[int]:
        return self.history[self.round_start :]

    def bets_this_round(self) -> int:
        return sum(1 for action in self.round_actions() if action in (BET_CALL, RAISE))

    def facing_bet(self) -> bool:
        outstanding = False
        for action in self.round_actions():
            if action == BET_CALL:
                outstanding = not outstanding
            elif action == RAISE:
                outstanding = True
        return outstanding

    def amount_to_call(self) -> int:
        if not self.facing_bet():
            return 0
        if RAISE in self.round_actions():
            return 4 if self.round == 0 else 8
        return 2 if self.round == 0 else 4

    def round_complete(self) -> bool:
        actions = self.round_actions()
        if len(actions) >= 2 and all(action == CHECK_PASS for action in actions):
            return True
        return bool(actions) and not self.facing_bet() and self.bets_this_round() > 0

    def is_terminal(self) -> bool:
        return (self.history and self.history[-1] == FOLD) or (
            self.round == 1 and self.round_complete()
        )

    def legal_actions(self) -> list[int]:
        if self.is_terminal() or (self.round == 1 and self.community is None):
            return []
        if self.facing_bet():
            return [BET_CALL, RAISE, FOLD] if self.bets_this_round() < 2 else [BET_CALL, FOLD]
        return [CHECK_PASS, BET_CALL]

    def apply_action(self, action: int) -> None:
        player = self.current_player()
        bet_size = 2 if self.round == 0 else 4
        if action == BET_CALL:
            amount = self.amount_to_call() if self.facing_bet() else bet_size
            self.stacks[player] -= amount
            self.pot += amount
        elif action == RAISE:
            amount = self.amount_to_call() + bet_size
            self.stacks[player] -= amount
            self.pot += amount
        self.history.append(action)
        if action != FOLD and self.round_complete() and self.round == 0:
            self.round = 1
            self.round_start = len(self.history)

    def remaining_community_cards(self) -> list[int]:
        remaining = list(DECK)
        remaining.remove(self.private_cards[0])
        remaining.remove(self.private_cards[1])
        if self.community is not None:
            remaining.remove(self.community)
        return remaining

    def payoff(self, player: int) -> float:
        if self.history and self.history[-1] == FOLD:
            folder = (len(self.history) - self.round_start - 1) % 2
            return self.payoff_for_winner(player, 1 - folder)
        score0 = self.hand_score(0)
        score1 = self.hand_score(1)
        if score0 == score1:
            return 0.0
        return self.payoff_for_winner(player, 0 if score0 > score1 else 1)

    def payoff_for_winner(self, player: int, winner: int) -> float:
        return float(self.pot + self.stacks[player] if player == winner else self.stacks[player])

    def hand_score(self, player: int) -> int:
        private = self.private_cards[player]
        if self.community == private:
            return 100 + private
        return private


@dataclass
class DCFRNode:
    regret_sum: list[float] = field(default_factory=lambda: [0.0, 0.0, 0.0])
    strategy_sum: list[float] = field(default_factory=lambda: [0.0, 0.0, 0.0])

    def strategy(self, action_count: int) -> list[float]:
        positives = [max(value, 0.0) for value in self.regret_sum[:action_count]]
        total = sum(positives)
        if total > 0.0:
            return [value / total for value in positives]
        return [1.0 / action_count] * action_count

    def average_strategy(self, action_count: int) -> list[float]:
        total = sum(self.strategy_sum[:action_count])
        if total > 0.0:
            return [value / total for value in self.strategy_sum[:action_count]]
        return [1.0 / action_count] * action_count

    def discount(self, iteration: int) -> None:
        t = float(iteration)
        pos_discount = t**ALPHA / (t**ALPHA + 1.0)
        neg_discount = t**BETA / (t**BETA + 1.0)
        strategy_discount = (t**GAMMA / (t + 1.0)) ** GAMMA
        self.regret_sum = [
            value * (pos_discount if value >= 0.0 else neg_discount)
            for value in self.regret_sum
        ]
        self.strategy_sum = [value * strategy_discount for value in self.strategy_sum]


def infoset_key(state: LeducState, player: int) -> str:
    private = CARD_NAMES[state.private_cards[player]]
    community = CARD_NAMES[state.community] if state.community is not None else "-"
    history = "".join(str(action) for action in state.history)
    return f"{private}|{community}|{history}"


def cfr(nodes: dict[str, DCFRNode], state: LeducState, p0: float, p1: float) -> float:
    if state.is_terminal():
        return state.payoff(0)
    if state.round == 1 and state.community is None:
        cards = state.remaining_community_cards()
        utility = 0.0
        for card in cards:
            next_state = state.clone()
            next_state.community = card
            utility += cfr(nodes, next_state, p0, p1) / len(cards)
        return utility

    player = state.current_player()
    legal_actions = state.legal_actions()
    key = infoset_key(state, player)
    node = nodes.setdefault(key, DCFRNode())
    strategy = node.strategy(len(legal_actions))
    reach = p0 if player == 0 else p1
    for i, probability in enumerate(strategy):
        node.strategy_sum[i] += reach * probability

    action_utils = [0.0, 0.0, 0.0]
    node_util = 0.0
    for i, action in enumerate(legal_actions):
        next_state = state.clone()
        next_state.apply_action(action)
        if player == 0:
            action_utils[i] = cfr(nodes, next_state, p0 * strategy[i], p1)
        else:
            action_utils[i] = cfr(nodes, next_state, p0, p1 * strategy[i])
        node_util += strategy[i] * action_utils[i]

    perspective_node_util = node_util if player == 0 else -node_util
    opponent_reach = p1 if player == 0 else p0
    for i in range(len(legal_actions)):
        perspective_action_util = action_utils[i] if player == 0 else -action_utils[i]
        node.regret_sum[i] += opponent_reach * (
            perspective_action_util - perspective_node_util
        )
    return node_util


def private_card_deals() -> list[tuple[int, int]]:
    return [(DECK[i], DECK[j]) for i in range(len(DECK)) for j in range(len(DECK)) if i != j]


def dcfr_train(iterations: int) -> dict[str, DCFRNode]:
    nodes: dict[str, DCFRNode] = {}
    deals = private_card_deals()
    for iteration in range(1, iterations + 1):
        for node in nodes.values():
            node.discount(iteration)
        for deal in deals:
            cfr(nodes, LeducState(deal), 1.0, 1.0)
    return nodes


if __name__ == "__main__":
    trained_nodes = dcfr_train(10_000)
    print(f"DCFR nodes: {len(trained_nodes)}")
    for key in sorted(trained_nodes)[:12]:
        node = trained_nodes[key]
        strategy = node.average_strategy(3)
        print(
            f"{key}: a0={strategy[0]:.3f}, a1={strategy[1]:.3f}, a2={strategy[2]:.3f}"
        )
