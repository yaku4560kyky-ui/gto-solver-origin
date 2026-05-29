from fastapi.testclient import TestClient
from main import app


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
