import os
from typing import Any

from fastapi import Depends, HTTPException, status
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from jose import JWTError, jwt


security = HTTPBearer()


def verify_supabase_token(
    credentials: HTTPAuthorizationCredentials = Depends(security),
) -> dict[str, Any]:
    jwt_secret = os.getenv("SUPABASE_JWT_SECRET", "")
    if not jwt_secret:
        return {
            "sub": "dev-user",
            "email": "dev@example.com",
            "role": "authenticated",
        }

    try:
        return jwt.decode(
            credentials.credentials,
            jwt_secret,
            algorithms=["HS256"],
            options={"verify_aud": False},
        )
    except JWTError as exc:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid Supabase token",
            headers={"WWW-Authenticate": "Bearer"},
        ) from exc


def get_user_plan(user: dict[str, Any]) -> str:
    user_metadata = user.get("user_metadata") or {}
    app_metadata = user.get("app_metadata") or {}
    return user_metadata.get("plan") or app_metadata.get("plan") or "free"
