# ADR-0001: Modular monolith plus workers

Status: Accepted

## Decision
Begin with a modular core API and independently runnable asynchronous media/AI workers in one monorepo.

## Why
Misty needs strong domain boundaries and asynchronous heavy processing, but does not yet have measured scale requiring operationally expensive microservices.

## Consequence
Modules must avoid accidental coupling. A subsystem may be extracted later only when reliability, scaling, deployment or team evidence justifies it.
