import hmac
import os
from hashlib import sha256
from typing import Literal

from fastapi import APIRouter, Header, HTTPException, Request, status
from pydantic import BaseModel


router = APIRouter(prefix="/api/v1/stripe", tags=["stripe"])

DEFAULT_WEBHOOK_SECRET = "phase6-demo-webhook-secret"


class Plan(BaseModel):
    id: Literal["starter", "pro", "team"]
    name: str
    price_cents: int
    currency: Literal["usd"]
    interval: Literal["month"]
    features: list[str]
    cta: str


class PlansResponse(BaseModel):
    plans: list[Plan]


class WebhookResponse(BaseModel):
    received: bool
    event_type: str | None = None


PLANS = [
    Plan(
        id="starter",
        name="Starter",
        price_cents=1900,
        currency="usd",
        interval="month",
        features=["Kuhn CFR demo", "Hand evaluator", "Community support"],
        cta="Join waitlist",
    ),
    Plan(
        id="pro",
        name="Pro",
        price_cents=4900,
        currency="usd",
        interval="month",
        features=["CFR+ experiments", "Preflop abstraction", "Priority beta access"],
        cta="Reserve Pro",
    ),
    Plan(
        id="team",
        name="Team",
        price_cents=14900,
        currency="usd",
        interval="month",
        features=["Shared workspaces", "Webhook billing", "Launch support"],
        cta="Contact sales",
    ),
]


def _expected_signature(payload: bytes, secret: str) -> str:
    return hmac.new(secret.encode("utf-8"), payload, sha256).hexdigest()


@router.get("/plans", response_model=PlansResponse)
def list_plans() -> PlansResponse:
    return PlansResponse(plans=PLANS)


@router.post("/webhook", response_model=WebhookResponse)
async def stripe_webhook(
    request: Request,
    stripe_signature: str | None = Header(default=None, alias="Stripe-Signature"),
) -> WebhookResponse:
    payload = await request.body()
    secret = os.getenv("STRIPE_WEBHOOK_SECRET", DEFAULT_WEBHOOK_SECRET)
    expected = _expected_signature(payload, secret)

    if stripe_signature != expected:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid Stripe webhook signature",
        )

    event = await request.json()
    return WebhookResponse(received=True, event_type=event.get("type"))
