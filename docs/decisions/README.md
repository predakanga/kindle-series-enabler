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

- [0001: What the package changes: the marketplace override, nothing else](0001-what-the-package-changes.md)
- [0002: When it runs, and how it reports](0002-when-it-runs.md)
- [0003: Installing and uninstalling](0003-installing-and-uninstalling.md)
- [0004: A Rust binary, with SQLite and a JSON parser built in](0004-a-rust-binary.md)
