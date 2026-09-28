# Security Policy

Misty is pre-alpha. Please do not use the repository or current builds to store real private media.

## Reporting a vulnerability
Do not open a public issue containing exploit details, credentials, private data or a working proof of concept against a deployed system. Use GitHub's private vulnerability reporting for this repository when available, or contact the repository owner privately.

Include the affected component, impact, reproduction conditions and a minimal safe proof when possible.

## Security expectations
- Never commit secrets, access tokens, signed object URLs or real private media.
- Never make the originals bucket public.
- Treat invitation tokens and signed URLs as bearer credentials.
- Keep authorization server-side and Space-scoped.
- Preserve accepted original bytes unchanged.
- Treat uploaded media, metadata, webhooks and AI output as untrusted.
- Background work must be idempotent and retry-safe.
- Security checks may be fixed; they must not be disabled merely to make a build pass.

There is no production security SLA while Misty remains pre-alpha. This file will be updated before a public production launch.
