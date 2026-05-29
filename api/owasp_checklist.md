# OWASP Top 10 Mitigation Checklist

| OWASP Top 10 2021 Category | Status | API Mitigation |
| --- | --- | --- |
| A01 Broken Access Control | Partial | JWT bearer-token helpers and `get_current_user` are implemented for protected endpoints. Public demo endpoints remain intentionally unauthenticated. |
| A02 Cryptographic Failures | Partial | Password hashing uses `passlib[bcrypt]`; JWTs are signed with HS256. Production deployments must move `SECRET_KEY` to secret storage and enforce TLS at the edge. |
| A03 Injection | Mitigated | Solver inputs are strongly typed by Pydantic and range-checked before native solver calls. No SQL or shell execution is used in the API layer. |
| A04 Insecure Design | Partial | Rate limiting, security headers, and input validation are in place. Threat modeling and abuse-case review should be repeated before paid launch. |
| A05 Security Misconfiguration | Partial | Security headers include `nosniff`, `DENY`, XSS protection, and HSTS. CORS remains restricted to the local frontend origin. |
| A06 Vulnerable and Outdated Components | Partial | Dependencies are pinned in `api/requirements.txt`. Add automated dependency scanning in CI before production. |
| A07 Identification and Authentication Failures | Partial | OAuth2 password token endpoint rejects invalid credentials and issues expiring JWTs. Demo credentials must not be used in production. |
| A08 Software and Data Integrity Failures | Partial | Native solver import is isolated behind typed API handlers. Add artifact signing and deployment provenance for production builds. |
| A09 Security Logging and Monitoring Failures | Not Started | Add structured audit logs for auth failures, rate-limit events, validation failures, and solver errors. |
| A10 Server-Side Request Forgery | Mitigated | The API does not make user-controlled outbound network requests. |
