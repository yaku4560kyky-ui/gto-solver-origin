from collections import defaultdict, deque
from time import monotonic

from fastapi import HTTPException, Request, Response, status


class RateLimiter:
    def __init__(self, max_requests: int = 60, window_seconds: int = 60) -> None:
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self._requests: dict[str, deque[float]] = defaultdict(deque)

    def check(self, ip_address: str) -> tuple[bool, int, int]:
        now = monotonic()
        window_start = now - self.window_seconds
        timestamps = self._requests[ip_address]

        while timestamps and timestamps[0] <= window_start:
            timestamps.popleft()

        if len(timestamps) >= self.max_requests:
            retry_after = max(1, int(self.window_seconds - (now - timestamps[0])))
            return False, 0, retry_after

        timestamps.append(now)
        remaining = self.max_requests - len(timestamps)
        return True, remaining, 0

    def reset(self) -> None:
        self._requests.clear()


rate_limiter = RateLimiter()


async def rate_limit_dependency(request: Request, response: Response) -> None:
    client_ip = request.client.host if request.client else "unknown"
    allowed, remaining, retry_after = rate_limiter.check(client_ip)

    response.headers["X-RateLimit-Limit"] = str(rate_limiter.max_requests)
    response.headers["X-RateLimit-Remaining"] = str(remaining)

    if not allowed:
        raise HTTPException(
            status_code=status.HTTP_429_TOO_MANY_REQUESTS,
            detail="Rate limit exceeded",
            headers={
                "Retry-After": str(retry_after),
                "X-RateLimit-Limit": str(rate_limiter.max_requests),
                "X-RateLimit-Remaining": "0",
            },
        )
