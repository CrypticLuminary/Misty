# Storage

## Classes
- Originals: private, immutable, irreplaceable user source bytes.
- Derivatives: private/restricted generated thumbnails, previews and transcodes; reproducible.
- Temporary exports: short-lived and aggressively cleaned.
- Indexes/embeddings: versioned derived data; rebuildable according to source/retention policy.

## Provider boundary
Use an S3-compatible abstraction for authorization, stat and deletion operations. Provider selection is intentionally deferred until throughput, geography, durability, lifecycle and egress economics are measured.

## Lifecycle
Active/hot storage may later transition to archive/cooler storage before policy-driven purge. Product language must accurately describe actual retention.

Never solve authorization by making an originals bucket public.
