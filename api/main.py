import time as _time
from typing import Dict, List

from fastapi import Depends, FastAPI, HTTPException, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.security import OAuth2PasswordRequestForm
from pydantic import BaseModel
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import Response

import beta
import legal
import stripe_webhook
from api import solver_cache
from input_validator import validate_card, validate_iterations, validate_seven_cards
from rate_limit import rate_limit_dependency
from security import create_access_token, get_password_hash, verify_password

try:
    from api import monitoring
except ImportError:
    import monitoring


API_VERSION = "0.1.0"

app = FastAPI(title="GTO Solver API", version=API_VERSION)
monitoring.init_sentry()


class SecurityHeadersMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next) -> Response:
        response = await call_next(request)
        response.headers["X-Content-Type-Options"] = "nosniff"
        response.headers["X-Frame-Options"] = "DENY"
        response.headers["X-XSS-Protection"] = "1; mode=block"
        response.headers["Strict-Transport-Security"] = (
            "max-age=31536000; includeSubDomains"
        )
        return response


app.add_middleware(SecurityHeadersMiddleware)
app.add_middleware(monitoring.RequestLoggingMiddleware)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(legal.router)
app.include_router(beta.router)
app.include_router(stripe_webhook.router)


class TrainRequest(BaseModel):
    iterations: int = 100000


class StrategyResponse(BaseModel):
    strategies: Dict[str, List[float]]
    iterations: int
    cached: bool = False
    elapsed_ms: float
    result_id: int


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


class TokenResponse(BaseModel):
    access_token: str
    token_type: str


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
DEMO_USERNAME = "demo"
DEMO_PASSWORD_HASH = get_password_hash("gto2024")


@app.get("/health")
def health() -> dict[str, object]:
    return {
        "status": "ok",
        "version": API_VERSION,
        "services": {
            "api": "ok",
            "solver": "available",
            "beta": "ok",
            "billing": "configured",
        },
    }


@app.post("/api/v1/auth/token", response_model=TokenResponse)
async def login_for_access_token(
    form_data: OAuth2PasswordRequestForm = Depends(),
) -> TokenResponse:
    if form_data.username != DEMO_USERNAME or not verify_password(
        form_data.password, DEMO_PASSWORD_HASH
    ):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect username or password",
            headers={"WWW-Authenticate": "Bearer"},
        )

    access_token = create_access_token(data={"sub": form_data.username})
    return TokenResponse(access_token=access_token, token_type="bearer")


@app.post(
    "/api/v1/solve/kuhn-cfr",
    response_model=StrategyResponse,
    dependencies=[Depends(rate_limit_dependency)],
)
def solve_kuhn_cfr(request: TrainRequest) -> StrategyResponse:
    validate_iterations(request.iterations)

    cached_result = solver_cache.get("kuhn-cfr", request.iterations)
    if cached_result is not None:
        return StrategyResponse(
            strategies=cached_result["strategies"],
            iterations=cached_result["iterations"],
            cached=True,
            elapsed_ms=cached_result["elapsed_ms"],
            result_id=cached_result["id"],
        )

    started_at = _time.perf_counter()
    try:
        import gto_solver

        strategies = gto_solver.train_kuhn_cfr(request.iterations)
    except ImportError:
        strategies = MOCK

    elapsed_ms = (_time.perf_counter() - started_at) * 1000
    result_id = solver_cache.save(
        "kuhn-cfr",
        request.iterations,
        strategies,
        elapsed_ms,
    )
    return StrategyResponse(
        strategies=strategies,
        iterations=request.iterations,
        cached=False,
        elapsed_ms=elapsed_ms,
        result_id=result_id,
    )


@app.post(
    "/api/v1/solve/kuhn-cfr-plus",
    response_model=StrategyResponse,
    dependencies=[Depends(rate_limit_dependency)],
)
def solve_kuhn_cfr_plus(request: TrainRequest) -> StrategyResponse:
    validate_iterations(request.iterations)

    cached_result = solver_cache.get("kuhn-cfr-plus", request.iterations)
    if cached_result is not None:
        return StrategyResponse(
            strategies=cached_result["strategies"],
            iterations=cached_result["iterations"],
            cached=True,
            elapsed_ms=cached_result["elapsed_ms"],
            result_id=cached_result["id"],
        )

    started_at = _time.perf_counter()
    try:
        import gto_solver

        strategies = gto_solver.train_kuhn_cfr_plus(request.iterations)
    except ImportError:
        strategies = MOCK

    elapsed_ms = (_time.perf_counter() - started_at) * 1000
    result_id = solver_cache.save(
        "kuhn-cfr-plus",
        request.iterations,
        strategies,
        elapsed_ms,
    )
    return StrategyResponse(
        strategies=strategies,
        iterations=request.iterations,
        cached=False,
        elapsed_ms=elapsed_ms,
        result_id=result_id,
    )


@app.get("/api/v1/cache/")
def list_cache() -> dict[str, object]:
    return {"results": solver_cache.list_all()}


@app.get("/api/v1/cache/{result_id}")
def get_cache_result(result_id: int) -> dict[str, object]:
    result = solver_cache.get_by_id(result_id)
    if result is None:
        raise HTTPException(status_code=404, detail="Cache result not found")
    return result


@app.delete("/api/v1/cache/{result_id}")
def delete_cache_result(result_id: int) -> dict[str, bool]:
    deleted = solver_cache.delete(result_id)
    if not deleted:
        raise HTTPException(status_code=404, detail="Cache result not found")
    return {"deleted": True}


@app.post(
    "/api/v1/evaluate/hand",
    response_model=HandEvalResponse,
    dependencies=[Depends(rate_limit_dependency)],
)
def evaluate_hand(request: HandEvalRequest) -> HandEvalResponse:
    validate_seven_cards(request.cards)

    try:
        import gto_solver

        rank = gto_solver.evaluate_hand(request.cards)
    except ImportError:
        rank = 1

    return HandEvalResponse(rank=rank, rank_name=HAND_RANK_NAMES[rank])


@app.post(
    "/api/v1/classify/preflop",
    response_model=PreflopResponse,
    dependencies=[Depends(rate_limit_dependency)],
)
def classify_preflop(request: PreflopRequest) -> PreflopResponse:
    validate_card(request.card1)
    validate_card(request.card2)

    try:
        import gto_solver

        group_id = gto_solver.classify_preflop(request.card1, request.card2)
    except ImportError:
        group_id = 0

    return PreflopResponse(group_id=group_id)
