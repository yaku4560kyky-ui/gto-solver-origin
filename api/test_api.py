from fastapi.testclient import TestClient
from main import app
from rate_limit import rate_limiter


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
