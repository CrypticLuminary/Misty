# Error Handling

Errors should be structured, actionable and safe to expose.

Future API errors use stable machine-readable codes such as `SPACE_NOT_FOUND`, `INVITE_EXPIRED`, `PERMISSION_DENIED`, `UPLOAD_TOO_LARGE`, `UNSUPPORTED_MEDIA`, `ASSET_PROCESSING` and `STORAGE_LIMIT_REACHED`.

Do not expose stack traces, SQL details, credentials, signed URLs or internal storage keys to clients. Preserve a correlation/request ID so operators can find the corresponding structured server error.

Retry only failures classified as transient. Destructive and payment operations require explicit idempotency semantics.
