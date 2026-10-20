# Decision Log

One file per decision, named `NNNN-short-title.md`, numbered in sequence.
Don't edit a decision after the fact to change it. Write a new one that
supersedes it, and set the old one's status to `Superseded by NNNN`.

## Template

```markdown
# NNNN: Title

- Status: Proposed | Accepted | Superseded by NNNN
- Date: YYYY-MM-DD

## Context
What problem or question prompted this, and what research/evidence applies
(link sources: firmware binaries and strings, KPM source, device observations).

## Decision
What we chose.

## Alternatives considered
What else was on the table and why it lost.

## Consequences
What this commits us to, and any follow-up work or risks.
```

## Index

- [0001: What installing changes](0001-what-installing-changes.md)
- [0002: What uninstalling restores](0002-what-uninstalling-restores.md)
- [0003: After a server config push](0003-after-a-server-push.md)
- [0004: A Rust binary, with SQLite built in](0004-a-rust-binary.md)
