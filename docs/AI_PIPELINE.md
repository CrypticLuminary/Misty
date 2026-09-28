# AI Pipeline

AI is optional enhancement, never a dependency for core storage/sharing.

## Likely tasks
- Face detection and Space-scoped face embeddings.
- Face clustering.
- Semantic image embeddings.
- Similarity/moment signals.
- Quality/best-shot assistance.
- Search/ranking.

Use deterministic metadata/CV when it solves the problem more cheaply and reliably than a generative model.

## Model governance
Every persisted AI artifact records model identity/version and relevant pipeline version. Model upgrades require evaluation and a reprocessing strategy; incompatible embeddings must not be compared blindly.

## Find Me
A user-provided reference face is embedded and compared only within the relevant Space by default. Results are presented as likely matches, with user correction. Do not create a global real-world identity database by accident.

## Evaluation
Before calling an AI feature production-ready, measure task-appropriate precision/recall/error rates, latency and cost on representative data. "The model runs" is not acceptance evidence.
