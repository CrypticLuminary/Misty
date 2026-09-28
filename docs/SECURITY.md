# Security Model

Misty stores private personal media and potentially biometric-like face embeddings/location metadata. Security is a product requirement.

## Trust boundaries
Treat browsers, invitation links, filenames, MIME declarations, EXIF, uploaded bytes, webhooks and AI outputs as untrusted input.

## Authorization
Authorization is server-side and capability-based. Roles are convenience presets. Sensitive operations evaluate explicit capabilities such as `can_view`, `can_upload`, `can_download_original`, `can_delete_any`, `can_invite`, and `can_manage_space`.

Cross-Space access is a critical invariant: possession of an asset ID must never authorize access.

## Invitations
Invitation tokens must be unpredictable, revocable and expirable. A link grants at most the intended scoped join capability; it is not a permanent public object URL.

## Storage
Buckets/containers holding originals are private. Downloads require server authorization followed by a short-lived scoped signed URL/capability. Signed URLs are bearer credentials and must not be logged.

## Uploads
Do not trust extensions or client MIME. Validate file signatures/decodability, supported type, configured limits and authorization. New uploads may enter quarantine before acceptance. Client validation is UX only.

## Identity and sessions
An identity is a neutral principal, not an authorization role or global public persona. Whether that identity is an owner, member or guest—and the display name shown for it—is defined by its Space membership. This avoids role or guest-alias state leaking across Spaces.

Session authorization is server-recognized: the browser holds only a 256-bit opaque random secret while PostgreSQL stores only its 32-byte SHA-256 verifier plus expiry/revocation state. The raw secret is generated from operating-system cryptographic randomness, encoded as unpadded Base64URL for transport, returned only through the issuance boundary and never persisted. A session ID, identity ID, membership ID, Space ID or stored verifier is never sufficient to authenticate.

Owner sessions are identity-scoped. Guest sessions are additionally bound to exactly one Space membership, so a guest credential cannot become ambient cross-Space authority.

For the intended HTTPS web topology, the browser credential is a host-only `__Host-` cookie with `HttpOnly; Secure; SameSite=Lax; Path=/` and no `Domain` attribute. This prevents sibling subdomains from setting the production session cookie. Mutating browser requests additionally require same-origin validation and a CSRF token bound to the server-recognized session. CORS is not an authorization mechanism. Local development may relax `Secure` only on loopback through explicit development configuration.

Logout/revocation sets server-side revocation state; expiry is checked server-side on every authenticated use. Scoped sessions also require their bound membership to remain active and to belong to the same identity; removing or leaving a Space therefore invalidates scoped authority immediately. Rotating a session creates a new verifier and invalidates the old credential. Raw session secrets, CSRF secrets and invitation secrets must never enter logs, analytics or error payloads.

## Privacy-sensitive metadata
Generated previews should omit unnecessary EXIF. GPS is hidden by default in product surfaces. Face embeddings are scoped to a Space by default; do not silently build a permanent global identity graph.

## Abuse resistance
Rate-limit sensitive endpoints and invitations. Enforce storage quotas. Design for invite revocation, joining pause, suspicious mass-join detection and mass-download controls.

## Deletion
Permanent deletion is a workflow, not a single SQL delete. It must eventually remove originals, derivatives, transcodes, embeddings/indexes and temporary exports according to the declared policy.

## Secrets
No secrets in source control. CI must scan for accidental credentials. Production should use managed secrets/short-lived identity where practical.

## Logging
Use structured logs with correlation IDs. Never log credentials, auth tokens, signed URLs, raw private media, or unnecessary sensitive EXIF.

## CI security gates
Dependency review/audit, static analysis, secret scanning, type/lint/test gates and container scanning as containers appear. Dynamic security testing is introduced when a runnable HTTP surface exists.
