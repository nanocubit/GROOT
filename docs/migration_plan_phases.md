# Migration plan — phases for GROOT

## Goal

Move grit's critical Git subsystems onto gitoxide-backed implementations while preserving grit compatibility and user-facing behavior.

## Phase order

### Phase 1 — Backend abstraction
- define backend trait
- keep legacy adapter as default fallback
- add smoke tests for adapter selection

### Phase 2 — Discovery
- use gix-discover
- preserve `.git` file and `commondir` logic
- handle `GIT_DIR` / `GIT_WORK_TREE`
- cover linked worktree detection

### Phase 3 — Object database
- use `gix-odb` and pack access
- support replacement refs and alternates
- validate loose and packed object reads

### Phase 4 — Index and worktree
- use `gix-index`
- validate index roundtrip behavior
- preserve sparse/split-index semantics where applicable

### Phase 5 — Refs and HEAD
- use `gix-ref`
- validate HEAD resolution and branch semantics

### Phase 6 — Transport
- use `gix-transport` / `gix-protocol`
- cover fetch / push / remote negotiation

### Phase 7 — Diff, merge, revision, CLI cleanup
- migrate higher-level operations once core plumbing is stable
- keep command UX consistent with grit

## Migration order by subsystem

| Subsystem | grit area | gitoxide analogue | Priority | Risk |
| --- | --- | --- | --- | --- |
| repo discovery | `grit-lib` repo handling | `gix-discover` | P0 | Medium |
| ODB | object storage | `gix-odb` / `gix-pack` | P0 | Medium |
| index / worktree | index logic | `gix-index` / `gix-worktree` | P0 | High |
| refs / HEAD | ref resolution | `gix-ref` | P1 | Medium |
| transport | fetch / push | `gix-transport` | P1 | Medium |
| diff / merge | higher-level Git operations | `gix-diff` / `gix-merge` | P2 | Medium |
| CLI | command layer | thin backend facade | P2 | Low |

## Guardrails

- Keep legacy fallback until parity is proven.
- Prefer small PRs with single subsystem scope.
- Validate each phase with compatibility tests.
- Do not rewrite the whole engine in one patch.

## Exit criteria for each phase

- relevant adapter works and passes tests
- legacy fallback still works
- compatibility checks are green
- no broad regressions in CLI behavior
