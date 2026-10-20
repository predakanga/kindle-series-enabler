# 0005: One KPM index for all the developer's packages (kindle-apps)

- Status: Proposed
- Date: 2026-10-20

## Context
Until now each package published its own KPM index on its own GitHub
Pages, built by its release workflow (Kindle Wireless Sync's ADR 0014,
copied here). A user adds one repository per package.

The developer wants one repository for all their packages (2026-10-20):
this one, Kindle Wireless Sync, and any later ones. There are no other
users yet, so nobody has an old URL to move from.

From KPM 0.2.2's source (`src/index.c`): an index is one object of
packages, each with its artifacts, so one index can list any number of
packages. Packages are stored per repository and ID.

## Decision
**A separate repository, `predakanga/kindle-apps`, publishes the index**
at `https://predakanga.github.io/kindle-apps/repo.json`, with a page
listing the packages.
- `packages.txt` lists the app repositories.
- Its workflow builds the index from each repository's releases: every
  published `vX.Y.Z` release with a `.kpkg` attached is an artifact,
  downloaded from the release. The package's name, author, description,
  dependencies and platforms come from the `manifest.json` inside each
  `.kpkg`, so app repositories need nothing beyond their releases. The tag
  must match the manifest's version.
- **Triggers:** a `repository_dispatch` (`release`) sent by each app's
  release workflow, a push to `main` (a changed `packages.txt`), and
  `workflow_dispatch` by hand. No schedule.

**This repository:** the release workflow drops its `pages` job and,
after publishing the release, sends the dispatch with the
`KINDLE_APPS_TOKEN` secret: a fine-grained token for `kindle-apps` only,
with Contents read and write (what the dispatch API needs). Without the
secret it warns and the release still succeeds; the index can then be
rebuilt by hand. `tools/kpm-index.sh` and `package/pages/` go, and the
README points at the shared index.

Kindle Wireless Sync moves over in its own session (the developer,
2026-10-20).

## Alternatives considered
- **An index per package** (as before): no tokens, but users add a
  repository for each package.
- **A daily schedule** instead of, or as well as, the dispatch: no token,
  but GitHub disables scheduled workflows in quiet repositories and asks
  for them to be turned back on (the developer, 2026-10-20).
- **Committing `repo.json` or the packages to `kindle-apps`:** binaries in
  git, and a second copy of each release.
- **Reading the metadata from each app repository's `manifest.json.in`:**
  ties the index to each repository's layout, and to its current text
  rather than what each release shipped.
- **The Kindle Modding Club's official repository:** not ours to run;
  packages could still be submitted there later.

## Consequences
- A token to create, store in each app repository, and renew when it
  expires.
- The dispatch is fire-and-forget: the release workflow can't tell
  whether the index was rebuilt. The index workflow's own run shows it.
- Each rebuild downloads every release's `.kpkg` (small, and there are
  few).
- This repository's own Pages site stops being updated and can be
  turned off.
