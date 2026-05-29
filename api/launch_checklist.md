# Phase 6 Launch Checklist

## Product
- Landing page includes demo, pricing, legal links, and waitlist form.
- Beta feedback endpoint is available for bug reports and feature requests.
- Pricing plans are exposed through the API for frontend rendering.

## Payments
- Stripe webhook endpoint validates request signatures before processing events.
- Production `STRIPE_WEBHOOK_SECRET` is configured outside the repository.
- Webhook event logs are monitored during beta onboarding.

## Security
- Health endpoint exposes service readiness without leaking secrets.
- CORS origins are reviewed before public launch.
- OWASP checklist remains current for authentication, rate limits, and headers.

## Operations
- All API tests pass before deployment.
- Support inbox and feedback triage owner are assigned.
- Rollback plan and beta invite list are prepared.
