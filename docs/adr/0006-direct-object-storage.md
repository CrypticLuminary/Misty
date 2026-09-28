# ADR-0006: Direct private object-storage transfer

Status: Accepted

## Decision
Large media normally transfers directly between authorized clients and private S3-compatible object storage using short-lived scoped authorization.

## Why
The API should be the control plane, not an expensive media-byte proxy.

## Security
A signed URL/capability is a bearer secret: short-lived, narrowly scoped, never logged, and issued only after server-side authorization.
