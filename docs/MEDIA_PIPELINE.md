# Media Pipeline

## Invariant
**Original = immutable. Derivative = disposable/reproducible.**

A source object may produce thumbnails, gallery previews, larger previews, video playback renditions, poster frames, metadata, checksums, perceptual hashes, quality signals and AI indexes. None of these may overwrite the accepted original.

## Planned flow
1. Client asks API for upload authorization.
2. API checks Space capability/quota and creates upload state.
3. Client transfers bytes directly to private object storage.
4. Client/API completes upload idempotently.
5. Server verifies storage metadata and media validity.
6. Accepted Asset records immutable source identity/checksum.
7. Durable event/outbox schedules asynchronous derivatives.
8. Gallery becomes useful as early derivatives finish; AI can lag independently.

## Processing priority
Thumbnail → metadata/integrity → core preview → face/basic organization → semantic AI → expensive optional ranking.

User actions may reprioritize work without changing correctness.

## Integrity
The first vertical slice must prove uploaded and downloaded originals have matching cryptographic checksums.
