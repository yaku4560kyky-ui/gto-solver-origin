import hmac
import os
from hashlib import sha256
from typing import Literal

from fastapi import APIRouter, Header, HTTPException, Request, status
from pydantic import BaseModel


router = APIRouter(prefix="/api/v1/stripe", tags=["stripe"])

DEFAULT_WEBHOOK_SECRET = "phase6-demo-webhook-secret"


class Plan(BaseModel):
    id: Literal["free", "pro", "team"]
    name: str
    price_jpy: int
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
        id="free",
        name="Free",
        price_jpy=0,
        price_cents=0,
        currency="usd",
        interval="month",
        features=["Kuhn CFR demo", "Hand evaluator", "Community support"],
        cta="Join waitlist",
    ),
    Plan(
        id="pro",
        name="Pro",
        price_jpy=2980,
        price_cents=4900,
        currency="usd",
        interval="month",
        features=["CFR+ experiments", "Preflop abstraction", "Priority beta access"],
        cta="Reserve Pro",
    ),
    Plan(
        id="team",
        name="Team",
        price_jpy=5980,
        price_cents=14900,
        currency="usd",
        interval="month",
        features=["Shared workspaces", "Webhook billing", "Launch support"],
        cta="Contact sales",
    ),
]


def _expected_signature(payload: bytes, secret: str) -> str:
    return hmac.new(secret.encode("utf-8"), payload, sha256).hexdigest()


def verify_stripe_signature(payload: bytes, sig_header: str, secret: str) -> bool:
    if not secret:
        return True

    parts = {}
    for item in sig_header.split(","):
        key, separator, value = item.partition("=")
        if separator:
            parts[key] = value

    timestamp = parts.get("t")
    signature = parts.get("v1")
    if timestamp and signature:
        signed_payload = timestamp.encode("utf-8") + b"." + payload
        expected = hmac.new(secret.encode("utf-8"), signed_payload, sha256).hexdigest()
        return hmac.compare_digest(signature, expected)

    expected = _expected_signature(payload, secret)
    return hmac.compare_digest(sig_header, expected)


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

    if not stripe_signature or not verify_stripe_signature(
        payload, stripe_signature, secret
    ):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Invalid Stripe webhook signature",
        )

    event = await request.json()
    return WebhookResponse(received=True, event_type=event.get("type"))
