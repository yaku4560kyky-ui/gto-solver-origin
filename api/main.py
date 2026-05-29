from typing import Dict, List

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel


app = FastAPI(title="GTO Solver API", version="0.1.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class TrainRequest(BaseModel):
    iterations: int = 100000


class StrategyResponse(BaseModel):
    strategies: Dict[str, List[float]]
    iterations: int


class HandEvalRequest(BaseModel):
    cards: List[int]


class HandEvalResponse(BaseModel):
    rank: int
    rank_name: str


class PreflopRequest(BaseModel):
    card1: int
    card2: int


class PreflopResponse(BaseModel):
    group_id: int


HAND_RANK_NAMES = [
    "High Card",
    "One Pair",
    "Two Pair",
    "Three of a Kind",
    "Straight",
    "Flush",
    "Full House",
    "Four of a Kind",
    "Straight Flush",
]

MOCK = {"J": [0.67, 0.33], "Q": [1.0, 0.0], "K": [0.0, 1.0]}


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok", "version": "0.1.0"}


@app.post("/api/v1/solve/kuhn-cfr", response_model=StrategyResponse)
def solve_kuhn_cfr(request: TrainRequest) -> StrategyResponse:
    try:
        import gto_solver

        strategies = gto_solver.train_kuhn_cfr(request.iterations)
    except ImportError:
        strategies = MOCK

    return StrategyResponse(strategies=strategies, iterations=request.iterations)


@app.post("/api/v1/solve/kuhn-cfr-plus", response_model=StrategyResponse)
def solve_kuhn_cfr_plus(request: TrainRequest) -> StrategyResponse:
    try:
        import gto_solver

        strategies = gto_solver.train_kuhn_cfr_plus(request.iterations)
    except ImportError:
        strategies = MOCK

    return StrategyResponse(strategies=strategies, iterations=request.iterations)


@app.post("/api/v1/evaluate/hand", response_model=HandEvalResponse)
def evaluate_hand(request: HandEvalRequest) -> HandEvalResponse:
    if len(request.cards) != 7:
        raise HTTPException(status_code=400, detail="Need 7 cards")

    try:
        import gto_solver

        rank = gto_solver.evaluate_hand(request.cards)
    except ImportError:
        rank = 1

    return HandEvalResponse(rank=rank, rank_name=HAND_RANK_NAMES[rank])


@app.post("/api/v1/classify/preflop", response_model=PreflopResponse)
def classify_preflop(request: PreflopRequest) -> PreflopResponse:
    try:
        import gto_solver

        group_id = gto_solver.classify_preflop(request.card1, request.card2)
    except ImportError:
        group_id = 0

    return PreflopResponse(group_id=group_id)
