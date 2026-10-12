# GROOT migration table

This document maps the real grit architecture to the migration target in GROOT and identifies the safest order of backend substitution.

## Goal

Use gitoxide as the low-level backend for the critical Git subsystems of grit, while preserving grit's user-facing semantics, compatibility behavior, and edge-case handling.

## Core rule

Do not rewrite the whole project in one step.

Instead, migrate by subsystem, in the following order:

1. repository discovery
2. object database access
3. index and worktree logic
4. refs and HEAD semantics
5. transport / fetch / push
6. diff / merge / revision logic
7. CLI simplification

## Mapping: grit modules -> gitoxide analogues -> migration action

| grit module / area | real grit structure | gitoxide analogue | target phase | risk | effort | notes |
| --- | --- | --- | --- | --- | --- | --- |
| repository discovery | `grit-lib/src/repo.rs` and discovery helpers | `gix-discover`, `gix-sec`, `gix-config` | Phase 1 | Medium | 1-3d | Start here. Preserve env and edge-case semantics. |
| object storage / ODB | `grit-lib/src/odb.rs` and object store internals | `gix-odb`, `gix-pack` | Phase 2 | Medium | 2-5d | Good early win. |
| index / staging area | `grit-lib/src/index.rs` and worktree/index helpers | `gix-index`, `gix-worktree` | Phase 3 | High | 5-10d | Most subtle behavior; requires compatibility tests. |
| worktree semantics | worktree logic in grit-lib / repo | `gix-worktree`, `gix-worktree-state` | Phase 3 | High | 3-5d | Include linked worktree and .git-file cases. |
| refs / HEAD | `grit-lib/src/refs*` and reference resolution | `gix-ref`, `gix-refspec` | Phase 4 | Medium | 2-4d | Validate HEAD semantics and replace refs. |
| transport / network | protocol and transport support in grit | `gix-transport`, `gix-protocol`, `gix-credentials` | Phase 5 | Medium | 3-8d | Important for clone/fetch/push. |
| diff / merge / revision | revision logic and merge helpers | `gix-diff`, `gix-merge`, `gix-revision`, `gix-traverse` | Phase 6 | Medium | 3-7d | Later stage. |
| CLI UX | `grit-simple`, `grit` command layer | `gix` CLI / wrapper layer | Phase 7 | Low | 1-3d | Keep UX stable; backend hidden behind adapter. |

## Practical migration plan by repository area

### 1) Repository discovery

Goals:
- move root repo resolution to gitoxide discovery primitives
- preserve grit semantics around env variables and worktree edge cases

Concrete targets:
- `grit-lib/src/repo.rs`
- repo discovery helpers in grit-lib
- GIT_DIR / GIT_WORK_TREE overrides
- `.git` file vs `.git` directory resolution
- linked worktree / commondir semantics
- bare repo detection

Recommended backing code:
- `gix-discover`
- `gix-sec`
- `gix-config`

### 2) Object database access

Goals:
- replace custom object read/write with gix-odb
- preserve replacement refs and alternates

Concrete targets:
- `grit-lib/src/odb.rs`
- object database codepaths
- loose object handling
- pack / delta access

Recommended backing code:
- `gix-odb`
- `gix-pack`

### 3) Index and worktree logic

Goals:
- use gix-index for index reading/writing
- preserve sparse index and worktree-specific edge cases

Concrete targets:
- `grit-lib/src/index.rs`
- split/sparse/index handling
- worktree logic in grit-lib

Recommended backing code:
- `gix-index`
- `gix-worktree`
- `gix-worktree-state`

### 4) Refs and HEAD

Goals:
- replace reference handling with gix-ref logic
- keep HEAD semantics stable

Concrete targets:
- `grit-lib/src/refs*`
- refspec logic
- branch and HEAD handling

Recommended backing code:
- `gix-ref`
- `gix-refspec`

### 5) Transport and remote logic

Goals:
- move network stack to gix transport layer
- preserve fetch/push semantics and auth behavior

Concrete targets:
- protocol and transport in grit
- remote auth / credential helpers

Recommended backing code:
- `gix-transport`
- `gix-protocol`
- `gix-credentials`

### 6) Diff / merge / revision

Goals:
- after core migration is stable, move higher-level operations to gitoxide equivalents

Concrete targets:
- revision/diff/merge modules in grit-lib

Recommended backing code:
- `gix-diff`
- `gix-merge`
- `gix-revision`
- `gix-traverse`

### 7) CLI UX

Goals:
- keep `grit` or `grit-simple` user experience stable
- move implementation internals onto backend layer

Concrete targets:
- command layer in `grit`, `grit-simple`

Recommended backend:
- the gitoxide-backed engine behind a compatibility facade

## Why the order matters

This order is chosen to minimize risk and maximize win:

- discovery is a root capability; incorrect repo resolution breaks everything
- object DB access is a direct and high-value migration target
- index/worktree semantics are surprisingly subtle and must be exercised carefully
- refs and transport are important, but depend on core repo correctness

## Risks to watch

### 1) behavior mismatch

Git behavior is full of edge cases. The adapter must preserve grit compatibility even when low-level primitives come from gitoxide.

Mitigation:
- keep a legacy adapter as fallback
- add compatibility tests for exact edge cases

### 2) type and API mismatch

gix and grit do not necessarily expose identical low-level types.

Mitigation:
- create thin conversion layers
- keep adapters narrow and explicit

### 3) accidental overreach

Do not rewrite the whole engine in one PR.

Mitigation:
- keep PRs small and focused
- migrate one subsystem at a time

## Concrete compatibility test categories (must-have)

- repo discovery from nested directory
- repo discovery with `.git` file instead of `.git` dir
- linked worktree discovery
- bare repo detection
- GIT_DIR override semantics
- GIT_WORK_TREE override semantics
- index roundtrip
- object read from loose object store
- object read from packed object store
- replacement refs
- basic fetch/push smoke tests

## Engineering guidance

Important principle:

GROOT is not meant to replace grit. It is meant to make migration from grit to gitoxide safe, observable, and reviewable.

The legacy adapter stays alive long enough to validate parity. The gitoxide adapter becomes the future target, while the CLI and higher-level semantics remain stable.

## Summary

The migration is not a rewrite. It is an adapter-driven transition.

- grit remains the compatibility reference
- gitoxide becomes the backend engine
- GROOT is the migration harness where this process is tested before it is proposed to production code in grit
