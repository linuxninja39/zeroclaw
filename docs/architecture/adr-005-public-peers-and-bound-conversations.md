# ADR-005: Public Peers and Bound Conversations

**Status:** Proposed

**Date:** 2026-04-14

## Context

ZeroClaw currently operates as a single top-level runtime driven by the existing
root configuration (`identity`, top-level runtime defaults, channel ingress, and
`agents` for private delegate workflows). This model works well for single-root
setups but does not provide a first-class way to expose multiple top-level
personas on the same host while keeping private delegates internal.

The fork needs a generalized single-host multi-peer model that:

- preserves existing installs without requiring config changes,
- allows multiple public top-level peers on one host,
- binds external conversations directly to public peers,
- keeps private delegates non-addressable from channels and peer tools, and
- supports peer-to-peer main-to-main communication through an internal runtime
  mechanism instead of a public gateway/API surface.

Backward compatibility is a hard requirement. Upgrading to the fork must not
require rewriting or migrating an existing `config.toml`.

## Decision

### 1. Implicit default/root peer

ZeroClaw SHALL synthesize an implicit public peer with reserved id `default`
from the existing top-level configuration.

This implicit peer preserves current single-root behavior:

- existing top-level identity remains valid,
- existing runtime defaults remain valid,
- existing delegates under `agents` remain valid,
- existing channel ingress remains valid.

When no explicit peer configuration exists, the runtime behaves exactly as it
does today by routing traffic to the implicit `default` peer.

### 2. Explicit peers are additive

ZeroClaw SHALL support optional explicit `peers` configuration for additional
public peers.

Explicit peers are additive:

- they do not replace the implicit `default` peer,
- they do not require existing top-level config to be rewritten,
- they may be introduced incrementally after upgrade.

### 3. Explicit bindings are optional and additive

ZeroClaw SHALL support optional explicit `bindings` configuration mapping an
external conversation surface to a public peer.

When bindings are absent, inbound traffic continues to route to the implicit
`default` peer using legacy behavior.

When bindings are present, only explicitly bound conversations are redirected to
explicit peers. Unbound traffic still falls back to the implicit `default` peer.

### 4. Public peers vs private delegates

Public peers are first-class top-level runtime entrypoints.

Private delegates remain internal implementation details of a peer/runtime.
They are not externally bindable and are not valid peer-to-peer targets.

### 5. Peer-to-peer communication

Peer-to-peer communication SHALL be implemented as an internal single-host
runtime dispatch mechanism.

It is:

- main-to-main only,
- in-process for V1,
- not exposed through REST,
- not exposed through webhook loopback,
- not a cross-host/federated transport.

### 6. Session isolation model

New peer-aware routing SHALL scope conversation state by peer identity plus the
canonical external conversation target.

Legacy traffic routed through the implicit `default` peer without explicit peer
configuration should preserve existing session continuity behavior.

## Consequences

### Positive

- Existing installs upgrade with zero config changes.
- Additional public peers can be added incrementally.
- Bound conversations get clear ownership by peer.
- Private delegates remain private.
- The architecture stays single-host and avoids premature federation work.

### Negative

- Runtime complexity increases because the system must support both the implicit
  compatibility path and the explicit multi-peer path.
- Session behavior must distinguish legacy routing from explicit peer-aware
  routing to preserve backward compatibility.

### Neutral

- The existing top-level config remains the source for the implicit `default`
  peer.
- Explicit peer config is optional and omitted by default.

## Compatibility / Migration

- Existing configs remain valid unchanged.
- No migration command is required.
- No automatic config rewrite is required.
- Users may later add explicit peers and bindings incrementally.

## Rollback

Rollback is configuration-safe:

- remove explicit `peers` and `bindings`,
- runtime falls back to the implicit `default` peer only,
- legacy routing behavior remains intact.

## Non-goals

This ADR does not introduce:

- multi-host orchestration or federation,
- dynamic peer discovery,
- public peer REST/WebSocket APIs,
- delegate exposure through bindings or peer tools,
- remote persona mutation,
- remote config mutation.
