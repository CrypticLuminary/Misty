# ADR-0004: PostgreSQL + pgvector first

Status: Accepted

## Decision
Use PostgreSQL for durable relational state and pgvector for initial vector similarity needs.

## Why
Misty's core data is relational and transactional. A separate vector database adds operational complexity before scale demonstrates need.

## Consequence
Dedicated vector infrastructure may be introduced later using measured index size/latency/throughput evidence.
