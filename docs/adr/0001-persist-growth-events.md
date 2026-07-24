# Persist growth events instead of deriving them

Growth history is recorded as immutable events in the `StudyRpg` snapshot and SQLite when a Study Session completes. We do not reconstruct it from sessions because existing sessions omit quest and all-clear XP, while future level curves or skill names may change; explicit events preserve the growth the player actually observed, at the cost of additional local snapshot data.
