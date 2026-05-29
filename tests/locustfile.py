from locust import HttpUser, between, task


class GTOSolverUser(HttpUser):
    wait_time = between(1, 3)

    @task(3)
    def health_check(self) -> None:
        self.client.get("/health")

    @task(2)
    def get_plans(self) -> None:
        self.client.get("/api/v1/stripe/plans")

    @task(1)
    def solve_kuhn_cfr(self) -> None:
        self.client.post("/api/v1/solve/kuhn-cfr", json={"iterations": 1000})

    @task(1)
    def evaluate_hand(self) -> None:
        self.client.post(
            "/api/v1/evaluate/hand",
            json={"cards": [0, 4, 8, 12, 16, 20, 24]},
        )

    @task(2)
    def classify_preflop(self) -> None:
        self.client.post(
            "/api/v1/classify/preflop",
            json={"card1": 51, "card2": 47},
        )
