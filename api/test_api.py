from fastapi.testclient import TestClient
import hmac
import os
from hashlib import sha256

from fastapi.security import HTTPAuthorizationCredentials

from main import app
from rate_limit import rate_limiter
from stripe_webhook import DEFAULT_WEBHOOK_SECRET


client = TestClient(app)


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
