# ADR-0003: Python AI workers

Status: Accepted

## Decision
Keep experimental/model-heavy AI processing in Python workers behind versioned language-neutral job contracts.

## Why
Python's ML/CV ecosystem materially reduces model integration/evaluation cost. Forcing model research into Rust would optimize the wrong constraint.

## Consequence
Core product behavior cannot depend synchronously on Python AI availability.
