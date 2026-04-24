# Fork Maintenance Principles

This is a personal fork that tracks `upstream/master` (zeroclaw-labs/zeroclaw).
The goal of these principles is to keep merges from upstream cheap and
mechanical. Every conflict resolved during a merge is a chance to introduce
bugs and slows down the next merge.

## Core principle: be additive, never modify

When extending core systems (database schema, config, shared structs, SQL
queries, hot call sites), prefer adding new things alongside upstream code
rather than modifying upstream code in place. New files and new tables never
conflict; modified columns and renamed fields almost always do.

## Database changes

**Always add a new table for new concepts.** Do not add columns to existing
upstream tables.

- Upstream owns the column list of every table it created. Adding columns
  inline forces every `SELECT`/`INSERT`/`UPDATE` touching that table to be
  modified, and upstream is doing the same — every column add becomes N
  conflicts (one per query).
- A new side table with a foreign key to the upstream row gives you the
  same logical data model with zero conflicts on upstream queries.
- Hydrate/persist the side-table data with separate helper functions
  called from the same call sites that already use the upstream API.

**Example:** `target_public_peer` started as a column on `cron_jobs` and
caused 9 conflicts in one merge (every SQL query referenced it). It now
lives in `cron_job_public_peers` (job_id PK + FK to `cron_jobs`), loaded
and saved by dedicated helpers. Upstream queries are untouched.

## Config struct access

**Wrap upstream config field access behind thin accessors in your own
modules.** Upstream actively restructures `Config` (e.g. `default_temperature`
→ `providers.fallback_provider().temperature`, `model_routes` →
`providers.model_routes`, `channels_config` → `channels`). Each rename
breaks every call site that reached into the field directly.

A single helper method on `Config` (or in a fork-owned compat module)
absorbs the rename in one place instead of dozens.

## Hot call sites

**Don't modify shared call sites — wrap them.** If you need to change how
a function like `agent::run(...)` is invoked from a cron scheduler, wrap
the entire call in your own function rather than threading new arguments
through the existing call site. Upstream tweaks the argument list of the
inner call regularly; if you own the wrapper, your wrapper absorbs the
change.

The `prepare_agent_job_run` helper in `cron/scheduler.rs` was a step in
the right direction but stopped short — it returned data that the shared
call site then passed inline, so the call site still conflicted. The
better shape is for the wrapper to own the entire `agent::run` call.

## Don't reinvent upstream abstractions

**Check `upstream/master` before adding a new enum variant, trait impl,
or `PropKind`-style discriminator.** Upstream may be building the same
concept in parallel. The `StringArray` vs `StringList` collision happened
because both branches independently added a `Vec<String>` property kind.
Two impls of the same trait for the same type is a hard compilation error
that's annoying to resolve and forces renames everywhere.

## Rebase / merge cadence

**Merge upstream weekly.** Going 130 commits behind compounds every
conflict — semantic changes pile up, struct shapes drift, and a clean
3-way merge becomes a manual reconciliation. A weekly merge keeps each
resolution to a handful of trivial line-level decisions.

## File-level patterns

- **New files never conflict.** If a feature can live entirely in new
  files (a new module, a new table's helpers, a new trait extension),
  put it there. The `public_peer.rs` module is a good example.
- **New code at the end of files conflicts less than new code in the
  middle.** When you must edit an upstream file, prefer appending to
  inserting.
- **Avoid touching the call-site lines upstream recently changed.**
  `git log -p --since=1.month` on a file before editing it shows where
  upstream is currently active.
