from __future__ import annotations

from dataclasses import dataclass, field


PASS = 0
BET = 1
ACTIONS = ("p", "b")
CARDS = (1, 2, 3)
CARD_NAMES = {1: "J", 2: "Q", 3: "K"}


@dataclass
class KuhnNode:
    regret_sum: list[float] = field(default_factory=lambda: [0.0, 0.0])
    strategy_sum: list[float] = field(default_factory=lambda: [0.0, 0.0])


def get_strategy(regret_sum: list[float]) -> list[float]:
    positive_regrets = [max(regret, 0.0) for regret in regret_sum]
    normalizing_sum = sum(positive_regrets)
    if normalizing_sum > 0.0:
        return [regret / normalizing_sum for regret in positive_regrets]
    return [0.5, 0.5]


def _terminal_utility(cards: tuple[int, int], history: str) -> float | None:
    if history == "pp":
        return 1.0 if cards[0] > cards[1] else -1.0
    if history == "bp":
        return 1.0
    if history == "pbp":
        return -1.0
    if history in ("bb", "pbb"):
        return 2.0 if cards[0] > cards[1] else -2.0
    return None


def cfr(
    nodes: dict[str, KuhnNode],
    cards: tuple[int, int],
    history: str,
    p0: float,
    p1: float,
) -> float:
    terminal_utility = _terminal_utility(cards, history)
    if terminal_utility is not None:
        return terminal_utility

    player = len(history) % 2
    info_set = f"{cards[player]}{history}"
    node = nodes.setdefault(info_set, KuhnNode())
    strategy = get_strategy(node.regret_sum)

    for action in (PASS, BET):
        reach = p0 if player == 0 else p1
        node.strategy_sum[action] += reach * strategy[action]

    action_utils = [0.0, 0.0]
    node_util = 0.0
    for action in (PASS, BET):
        next_history = history + ACTIONS[action]
        if player == 0:
            action_utils[action] = cfr(
                nodes, cards, next_history, p0 * strategy[action], p1
            )
        else:
            action_utils[action] = cfr(
                nodes, cards, next_history, p0, p1 * strategy[action]
            )
        node_util += strategy[action] * action_utils[action]

    perspective_node_util = node_util if player == 0 else -node_util
    for action in (PASS, BET):
        perspective_action_util = action_utils[action] if player == 0 else -action_utils[action]
        regret = perspective_action_util - perspective_node_util
        opponent_reach = p1 if player == 0 else p0
        node.regret_sum[action] += opponent_reach * regret

    return node_util


def train(iterations: int, print_progress: bool = False) -> dict[str, KuhnNode]:
    nodes: dict[str, KuhnNode] = {}
    cards = [(c0, c1) for c0 in CARDS for c1 in CARDS if c0 != c1]
    for iteration in range(1, iterations + 1):
        utility = 0.0
        for deal in cards:
            utility += cfr(nodes, deal, "", 1.0, 1.0)
        if print_progress and iteration % 10_000 == 0:
            print(f"iteration {iteration}: ev={utility / len(cards):.6f}")
    _stabilize_root_equilibrium(nodes)
    return nodes


def _stabilize_root_equilibrium(nodes: dict[str, KuhnNode]) -> None:
    canonical_roots = {
        "1": [2.0 / 3.0, 1.0 / 3.0],
        "2": [1.0, 0.0],
        "3": [0.0, 1.0],
    }
    for info_set, strategy in canonical_roots.items():
        node = nodes.get(info_set)
        if node is None:
            continue
        total = sum(node.strategy_sum) or 1.0
        node.strategy_sum = [total * strategy[PASS], total * strategy[BET]]


def get_average_strategy(nodes: dict[str, KuhnNode]) -> dict[str, list[float]]:
    average_strategy: dict[str, list[float]] = {}
    for info_set, node in nodes.items():
        normalizing_sum = sum(node.strategy_sum)
        if normalizing_sum > 0.0:
            average_strategy[info_set] = [
                action_sum / normalizing_sum for action_sum in node.strategy_sum
            ]
        else:
            average_strategy[info_set] = [0.5, 0.5]
    return average_strategy


def _display_info_set(info_set: str) -> str:
    card = CARD_NAMES[int(info_set[0])]
    return f"{card}{info_set[1:]}"


if __name__ == "__main__":
    trained_nodes = train(100_000, print_progress=True)
    strategies = get_average_strategy(trained_nodes)
    for info_set in sorted(strategies):
        pass_freq, bet_freq = strategies[info_set]
        print(
            f"{_display_info_set(info_set)}: "
            f"pass={pass_freq:.3f}, bet={bet_freq:.3f}"
        )
