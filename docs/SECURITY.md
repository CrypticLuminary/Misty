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

## Sessions
Use secure cookie/session practices appropriate to deployment, including HttpOnly/Secure/SameSite where applicable. Sensitive account operations may require reauthentication. Revocation must have server-side meaning.

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
