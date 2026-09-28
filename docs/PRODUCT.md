# Product Definition

## Problem
Groups returning from trips, weddings, college events and similar gatherings often exchange media through messaging apps or generic cloud folders. Messaging may be poor for original-quality archival workflows, while generic folders force each participant to manually find their own media.

## Product
Misty is a temporary shared memory space where a group can deposit original-quality photos and videos. Misty preserves originals and builds disposable derivatives and indexes that organize the collection into useful views such as people, moments and places.

## Core journey
CREATE → INVITE → DUMP EVERYTHING → ORGANIZE → FIND → VIEW → SELECT → DOWNLOAD ORIGINALS → ARCHIVE/EXPIRE.

## Product principles
- Original quality is a contractual product invariant, not a UI setting.
- Joining should be low-friction; guests should not need full accounts merely to participate.
- AI is assistive and confidence-aware, never an authoritative identity oracle.
- Privacy defaults should be conservative, especially for faces and GPS.
- Retention must be transparent. Misty must not claim data is deleted while secretly retaining it indefinitely.
- Users must be able to retrieve their original media.
- Core operation remains useful when AI is unavailable.

## V1 target
A group can securely create/join a Space, upload many original media files, browse fast derivatives, find useful subsets, download untouched originals, and understand the Space's retention state.

## Non-goals for early V1
- Global social network
- Public photo discovery
- Permanent global face identity database
- Full photo editor
- End-to-end encrypted server-side AI (this is a separate architecture with major tradeoffs)
- Microservice platform for its own sake
