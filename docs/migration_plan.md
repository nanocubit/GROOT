# GROOT migration plan

## Summary

GROOT is intended to act as an integration workspace between grit and gitoxide.

The goal is not to duplicate the entire Git implementation. The goal is to create a compatibility and adapter layer that allows grit to migrate gradually to gitoxide-backed primitives while preserving Git-compatible behavior and user experience.

## Target architecture

The intended architecture is intentionally incremental and adapter-oriented:

- grit remains the compatibility and UX reference
- gitoxide remains the mature backend engine
- GROOT remains the migration harness / adapter layer

ASCII overview:

            +--------------------+      +---------------------------+
            | grit CLI / shell   | <-> | grit-lib compatibility    |
            | (grit / grit-simple)|     | core / repo semantics     |
            +--------------------+     +---------------------------+
                                              /          |          \
                                             /           |           \
                                            /            |            \
                  +---------------------------+    +---------------------------+
                  | Legacy Adapter (fallback) |    | Gitoxide Adapter (gix)    |
                  +---------------------------+    +---------------------------+
                                    \               /
                                     \             /
                                      \           /
                                       \         /
                                        \       /
                                         V     V
                              +-------------------------------+
                              |   Backend interface / trait    |
                              |   (discovery / index / object) |
                              +-------------------------------+
                                                       |
                                                       V
                                     +-------------------------------+
                                     | gitoxide crates               |
                                     | gix-discover / gix-odb /      |
                                     | gix-index / gix-worktree      |
                                     +-------------------------------+

## Migration order

### Phase 1 — backend abstraction

Objectives:
- define a backend trait
- allow strict selection between legacy grit and gitoxide
- preserve fallback behavior during migration

Scope:
- adapter definitions
- backend selection
- compatibility smoke tests

### Phase 2 — repository discovery

Objectives:
- migrate repository discovery to gix-discover
- preserve grit edge cases such as:
  - `.git` file vs `.git` directory
  - GIT_DIR / GIT_WORK_TREE overrides
  - linked worktree resolution
  - bare repository detection
  - ownership / safe.directory logic

### Phase 3 — object database access

Objectives:
- use gix-odb as the backend for object reads and writes
- preserve replace-ref semantics
- validate loose object and pack behavior

### Phase 4 — index and worktree logic

Objectives:
- migrate index access to gix-index
- handle sparse and split-index semantics
- validate worktree path resolution and index roundtrip

### Phase 5 — refs and transport

Objectives:
- move ref logic to gix-ref
- later move network transport to gix-transport / gix-protocol / gix-credentials

### Phase 6 — CLI and UX

Objectives:
- keep CLI behavior stable
- keep the human-facing commands simple and consistent
- move command internals to backend calls without changing UX

## Migration mapping: grit -> gitoxide

| grit subsystem | gitoxide analogue | priority | risk | effort |
| --- | --- | --- | --- | --- |
| repository discovery | gix-discover | P0 | Medium | 1-3d |
| object database | gix-odb + gix-pack | P0 | Medium | 2-5d |
| index access | gix-index + gix-worktree | P0 | High | 5-10d |
| refs / HEAD | gix-ref | P1 | Medium | 2-4d |
| transport / fetch / push | gix-transport + gix-protocol + gix-credentials | P1 | Medium | 3-8d |
| diff / merge / revision | gix-diff + gix-merge + gix-revision | P2 | Medium | 3-7d |
| CLI UX | grit-simple / gix CLI surface | P2 | Low | 1-3d |

## Compatibility inputs

The first compatibility scenarios should come from the real grit repo structure and behavior:

- nested repository discovery
- `.git` file inside working tree
- linked worktree / common dir
- bare repositories
- index roundtrip
- replace-refs logic
- object read/write from loose and packed stores
- environment overrides for GIT_DIR / GIT_WORK_TREE

## Why this is the right architecture

GROOT is not a second implementation of an entire Git stack. It is the adapter-layer and migration scaffold.

The correct model is:

- grit = source of Git semantics and UX
- gitoxide = source of mature Rust backend core
- GROOT = compatibility and migration layer

This keeps the project focused, reviewable, and safe to evolve.

## Acceptance criteria

A given phase is complete when:

- the relevant adapter is implemented and passes tests
- legacy behavior remains available as fallback
- all new compatibility tests pass
- CI remains green
- no patch tries to replace the entire grit subsystem in one jump

## Immediate next steps

1. finalize backend trait contract
2. keep legacy backend as default fallback
3. implement gitoxide adapter stub with TODO markers
4. expand compatibility tests around discovery and index flow
5. then migrate object storage and ref logic

## Summary

GROOT provides the migration topology needed to connect the strong semantics of grit with the mature Rust backend ecosystem of gitoxide. It is intentionally not a competing implementation, but a compatibility workspace designed to make the migration safe, incremental, and testable.
