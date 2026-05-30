from fastapi.testclient import TestClient
import hmac
import os
from hashlib import sha256

from fastapi.security import HTTPAuthorizationCredentials

from api import solver_cache
from main import app
from rate_limit import rate_limiter
from stripe_webhook import DEFAULT_WEBHOOK_SECRET


client = TestClient(app)


def _clear_cache_entry(solver_type: str, iterations: int) -> None:
    cached_result = solver_cache.get(solver_type, iterations)
    if cached_result is not None:
        solver_cache.delete(cached_result["id"])


def test_health():
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json()["status"] == "ok"


def test_solve_kuhn_cfr_mock():
    response = client.post("/api/v1/solve/kuhn-cfr", json={"iterations": 1000})
    data = response.json()
    assert response.status_code == 200
    assert "strategies" in data
    assert data["iterations"] == 1000


def test_evaluate_hand_wrong_count():
    response = client.post("/api/v1/evaluate/hand", json={"cards": [0, 1, 2]})
    assert response.status_code == 400


def test_evaluate_hand_ok():
    response = client.post(
        "/api/v1/evaluate/hand",
        json={"cards": [0, 4, 8, 12, 16, 20, 24]},
    )
    assert response.status_code == 200
    assert "rank" in response.json()


def test_classify_preflop():
    response = client.post(
        "/api/v1/classify/preflop",
        json={"card1": 51, "card2": 47},
    )
    assert response.status_code == 200
    assert "group_id" in response.json()


def test_rate_limit_headers():
    rate_limiter.reset()
    response = client.post("/api/v1/solve/kuhn-cfr", json={"iterations": 1000})
    assert response.status_code == 200
    assert response.headers["x-ratelimit-limit"] == "60"
    assert "x-ratelimit-remaining" in response.headers


def test_security_headers():
    response = client.get("/health")
    assert response.headers["x-frame-options"] == "DENY"
    assert response.headers["x-content-type-options"] == "nosniff"


def test_input_validator_bad_card():
    response = client.post(
        "/api/v1/classify/preflop",
        json={"card1": 99, "card2": 47},
    )
    assert response.status_code == 422


def test_input_validator_duplicate_cards():
    response = client.post(
        "/api/v1/evaluate/hand",
        json={"cards": [0, 0, 8, 12, 16, 20, 24]},
    )
    assert response.status_code == 422


def test_legal_terms():
    response = client.get("/api/v1/legal/terms")
    data = response.json()
    assert response.status_code == 200
    assert data["title"] == "利用規約"
    assert data["jurisdiction"] == "Japan"


def test_tokushouhou():
    response = client.get("/api/v1/legal/tokushouhou")
    data = response.json()
    assert response.status_code == 200
    assert data["title"] == "特定商取引法に基づく表記"
    assert data["language"] == "ja"


def test_auth_token_invalid():
    response = client.post(
        "/api/v1/auth/token",
        data={"username": "demo", "password": "wrong"},
    )
    assert response.status_code == 401


def test_health_services():
    response = client.get("/health")
    data = response.json()
    assert response.status_code == 200
    assert data["services"]["beta"] == "ok"
    assert data["services"]["billing"] == "configured"


def test_beta_waitlist():
    response = client.post(
        "/api/v1/beta/waitlist",
        json={
            "name": "Ada",
            "email": "ada@example.com",
            "company": "Solver Lab",
            "intended_use": "Testing poker strategy analysis workflows",
        },
    )
    data = response.json()
    assert response.status_code == 201
    assert data["status"] == "queued"
    assert data["email"] == "ada@example.com"


def test_beta_feedback():
    response = client.post(
        "/api/v1/beta/feedback",
        json={
            "email": "tester@example.com",
            "category": "feature",
            "message": "Please add deeper preflop reports.",
        },
    )
    data = response.json()
    assert response.status_code == 201
    assert data["status"] == "received"
    assert data["category"] == "feature"


def test_stripe_plans():
    response = client.get("/api/v1/stripe/plans")
    data = response.json()
    assert response.status_code == 200
    assert len(data["plans"]) == 3
    assert data["plans"][1]["id"] == "pro"


def test_stripe_webhook_rejects_bad_signature():
    response = client.post(
        "/api/v1/stripe/webhook",
        json={"type": "checkout.session.completed"},
        headers={"Stripe-Signature": "bad-signature"},
    )
    assert response.status_code == 400


def test_stripe_webhook_accepts_valid_signature():
    payload = b'{"type":"checkout.session.completed"}'
    signature = hmac.new(
        DEFAULT_WEBHOOK_SECRET.encode("utf-8"), payload, sha256
    ).hexdigest()
    response = client.post(
        "/api/v1/stripe/webhook",
        content=payload,
        headers={
            "Content-Type": "application/json",
            "Stripe-Signature": signature,
        },
    )
    data = response.json()
    assert response.status_code == 200
    assert data["received"] is True
    assert data["event_type"] == "checkout.session.completed"


def test_env_example_exists():
    assert os.path.exists(".env.example")


def test_docker_compose_exists():
    assert os.path.exists("docker-compose.yml")


def test_supabase_auth_dev_mode():
    from api.supabase_auth import verify_supabase_token

    credentials = HTTPAuthorizationCredentials(
        scheme="Bearer",
        credentials="any-token",
    )
    result = verify_supabase_token(credentials)
    assert result["sub"] == "dev-user"


def test_stripe_plans_structure():
    response = client.get("/api/v1/stripe/plans")
    plans = response.json()["plans"]
    assert plans[0]["id"] == "free"
    assert plans[1]["price_jpy"] == 2980
    assert plans[2]["price_jpy"] == 5980


def test_cache_miss_then_save():
    iterations = 2001
    _clear_cache_entry("kuhn-cfr", iterations)

    response = client.post("/api/v1/solve/kuhn-cfr", json={"iterations": iterations})
    data = response.json()

    assert response.status_code == 200
    assert data["cached"] is False
    assert data["iterations"] == iterations
    assert data["elapsed_ms"] >= 0
    assert isinstance(data["result_id"], int)

    cached_result = solver_cache.get("kuhn-cfr", iterations)
    assert cached_result is not None
    assert cached_result["id"] == data["result_id"]


def test_cache_hit_second_request():
    iterations = 2002
    _clear_cache_entry("kuhn-cfr", iterations)

    first_response = client.post(
        "/api/v1/solve/kuhn-cfr",
        json={"iterations": iterations},
    )
    second_response = client.post(
        "/api/v1/solve/kuhn-cfr",
        json={"iterations": iterations},
    )
    first_data = first_response.json()
    second_data = second_response.json()

    assert first_response.status_code == 200
    assert second_response.status_code == 200
    assert first_data["cached"] is False
    assert second_data["cached"] is True
    assert second_data["result_id"] == first_data["result_id"]
    assert second_data["strategies"] == first_data["strategies"]


def test_cache_list():
    iterations = 2003
    _clear_cache_entry("kuhn-cfr-plus", iterations)
    solve_response = client.post(
        "/api/v1/solve/kuhn-cfr-plus",
        json={"iterations": iterations},
    )

    response = client.get("/api/v1/cache/")
    data = response.json()

    assert solve_response.status_code == 200
    assert response.status_code == 200
    assert "results" in data
    assert any(
        result["id"] == solve_response.json()["result_id"]
        for result in data["results"]
    )


def test_cache_get_by_id():
    iterations = 2004
    _clear_cache_entry("kuhn-cfr", iterations)
    solve_response = client.post(
        "/api/v1/solve/kuhn-cfr",
        json={"iterations": iterations},
    )
    result_id = solve_response.json()["result_id"]

    response = client.get(f"/api/v1/cache/{result_id}")
    data = response.json()

    assert solve_response.status_code == 200
    assert response.status_code == 200
    assert data["id"] == result_id
    assert data["solver_type"] == "kuhn-cfr"
    assert data["iterations"] == iterations
    assert "strategies" in data


def test_cache_delete():
    iterations = 2005
    _clear_cache_entry("kuhn-cfr-plus", iterations)
    solve_response = client.post(
        "/api/v1/solve/kuhn-cfr-plus",
        json={"iterations": iterations},
    )
    result_id = solve_response.json()["result_id"]

    response = client.delete(f"/api/v1/cache/{result_id}")
    get_response = client.get(f"/api/v1/cache/{result_id}")

    assert solve_response.status_code == 200
    assert response.status_code == 200
    assert response.json()["deleted"] is True
    assert get_response.status_code == 404
