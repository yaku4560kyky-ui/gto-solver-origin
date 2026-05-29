from datetime import datetime, timezone
from typing import Literal

from fastapi import APIRouter
from pydantic import BaseModel, Field


router = APIRouter(prefix="/api/v1/beta", tags=["beta"])

WAITLIST_SIGNUPS: list["WaitlistSignup"] = []
FEEDBACK_ITEMS: list["Feedback"] = []


class WaitlistRequest(BaseModel):
    email: str = Field(pattern=r"^[^@\s]+@[^@\s]+\.[^@\s]+$", max_length=254)
    name: str = Field(min_length=1, max_length=80)
    company: str | None = Field(default=None, max_length=120)
    intended_use: str = Field(min_length=3, max_length=500)


class WaitlistSignup(WaitlistRequest):
    id: int
    status: Literal["queued"]
    created_at: str


class FeedbackRequest(BaseModel):
    email: str | None = Field(
        default=None, pattern=r"^[^@\s]+@[^@\s]+\.[^@\s]+$", max_length=254
    )
    category: Literal["bug", "feature", "pricing", "other"] = "other"
    message: str = Field(min_length=5, max_length=1000)


class Feedback(FeedbackRequest):
    id: int
    status: Literal["received"]
    created_at: str


@router.post("/waitlist", response_model=WaitlistSignup, status_code=201)
def join_waitlist(request: WaitlistRequest) -> WaitlistSignup:
    signup = WaitlistSignup(
        **request.model_dump(),
        id=len(WAITLIST_SIGNUPS) + 1,
        status="queued",
        created_at=datetime.now(timezone.utc).isoformat(),
    )
    WAITLIST_SIGNUPS.append(signup)
    return signup


@router.post("/feedback", response_model=Feedback, status_code=201)
def submit_feedback(request: FeedbackRequest) -> Feedback:
    feedback = Feedback(
        **request.model_dump(),
        id=len(FEEDBACK_ITEMS) + 1,
        status="received",
        created_at=datetime.now(timezone.utc).isoformat(),
    )
    FEEDBACK_ITEMS.append(feedback)
    return feedback
