# Performance Strategy

Optimize perceived and measured performance without changing original bytes.

- Direct client ↔ object-storage transfer for large media.
- Multipart/resumable upload when file/network characteristics justify it.
- Immediate client-local previews while uploads run.
- Asynchronous derivative/AI processing.
- Cursor pagination and virtualized galleries.
- Lazy responsive derivatives through a CDN.
- Prefetch adjacent gallery items, never entire original collections.
- Prioritize thumbnail/metadata/core preview before expensive AI.
- Use checksums for exact duplicates and perceptual/embedding signals for near-duplicates.
- Prepare very large exports asynchronously.

Measure API p50/p95/p99, DB queries/pool pressure, upload success/throughput, queue depth, worker throughput, derivative latency and storage/CDN behavior before invasive optimization.
