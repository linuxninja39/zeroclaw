# Public Peers

Operator guide for exposing multiple top-level ZeroClaw personas on one host without adding a public peer API.

## What a public peer is

A public peer is a top-level runtime entrypoint that humans can reach through an existing channel conversation.

Think of the model like this:

- the legacy root runtime is still there as the implicit `default` peer,
- optional `[peers.<id>]` entries add more top-level peers on the same host,
- `[[bindings]]` decide which external conversation is owned by which public peer,
- private delegates under `[agents.<name>]` stay internal.

Public peers are channel-facing only. They do **not** create a separate REST, WebSocket, or webhook API surface.

## The implicit `default` peer

ZeroClaw always synthesizes a reserved peer id, `default`, from the existing top-level config.

That means an existing single-peer install keeps working without migration:

- top-level `identity` still applies,
- top-level provider/model defaults still apply,
- existing channel ingress still applies,
- existing delegates under `[agents]` still apply.

If you do not add `[peers]` or `[[bindings]]`, inbound traffic continues to behave like the legacy single-root runtime.

Use `default` when you want to explicitly target that legacy root peer from config or cron.

## Configure explicit public peers with `[peers.<id>]`

Use `[peers.<id>]` to define additional public top-level peers.

```toml
[peers.ops]
description = "Operations-facing assistant for the ops Telegram chat"
identity_ref = "peers/ops.md"
runtime_ref = "hint:ops-runtime"

[peers.support]
description = "Customer support peer used for escalations"
identity_ref = "peers/support.md"
```

Common keys:

| Key | Purpose |
|---|---|
| `public` | Whether the peer is externally routable/listable. Defaults to `true`. |
| `description` | Operator-facing label or summary. |
| `identity_ref` | Peer-specific identity overlay. |
| `runtime_ref` | Peer-specific runtime/profile override. |

Notes:

- Explicit peers are additive. They do not replace `default`.
- Peer ids should be lowercase and may contain digits, `_`, or `-`.
- Only top-level peers with `public = true` can be selected by bindings or explicit cron targets.
- `[agents.<name>]` delegates are not public peers and cannot be bound from channels.

## Bind external conversations with `[[bindings]]`

Use `[[bindings]]` when you want a specific external conversation to route to a specific public peer.

```toml
[[bindings]]
channel = "telegram"
conversation = "-1001234567890"
peer = "ops"

[[bindings]]
channel = "discord"
conversation = "123456789012345678"
peer = "support"

[[bindings]]
channel = "telegram"
conversation = "-1005550001112"
peer = "default"
```

How it works:

- If a binding matches, ZeroClaw routes that conversation to the selected peer.
- If no binding matches, ZeroClaw falls back to the implicit `default` peer.
- Bindings can target only `default` or a configured public peer under `[peers]`.
- Bindings cannot target private delegates or `public = false` peers.

`conversation` is the canonical conversation id for that channel. The exact value is channel-specific, for example a Telegram chat id vs. a Discord channel or thread id. Replace the example ids above with your real chat, channel, or thread ids.

## Target a public peer from cron

You can explicitly run agent cron jobs through a public peer with `target_public_peer`.

CLI examples:

```bash
zeroclaw cron add '0 14 * * *' --agent --target-public-peer ops \
  'Summarize overnight incidents and post a concise handoff'

zeroclaw cron add-every 3600000 --agent --target-public-peer default \
  'Check root-level housekeeping tasks'
```

Declarative example:

```toml
[cron]
enabled = true

[[cron.jobs]]
id = "ops-daily-digest"
name = "Ops daily digest"
job_type = "agent"
prompt = "Summarize overnight incidents and post a concise handoff."
target_public_peer = "ops"
allowed_tools = ["memory_search", "memory_get"]
session_target = "isolated"

[cron.jobs.schedule]
kind = "cron"
expr = "0 14 * * *"
tz = "America/Denver"

[cron.jobs.delivery]
mode = "announce"
channel = "telegram"
to = "-1001234567890"
best_effort = true
```

Rules:

- `target_public_peer` is supported only for `job_type = "agent"` or CLI `--agent` jobs.
- Shell jobs cannot target public peers.
- `target_public_peer = "default"` explicitly keeps legacy root behavior.
- Explicit cron targeting is same-host dispatch to a top-level public peer.

## Full example

This is a complete minimal configuration slice that keeps the implicit root peer, adds two explicit public peers, binds human conversations, and schedules one peer-specific job.

```toml
[identity]
name = "ZeroClaw"
role = "Root assistant"

[peers.ops]
description = "Operations-facing assistant for the ops Telegram chat"
identity_ref = "peers/ops.md"
runtime_ref = "hint:ops-runtime"

[peers.support]
description = "Customer support peer used for escalations"
identity_ref = "peers/support.md"

[[bindings]]
channel = "telegram"
conversation = "-1001234567890"
peer = "ops"

[[bindings]]
channel = "discord"
conversation = "123456789012345678"
peer = "support"

[cron]
enabled = true

[[cron.jobs]]
id = "ops-daily-digest"
name = "Ops daily digest"
job_type = "agent"
prompt = "Summarize overnight incidents and post a concise handoff."
target_public_peer = "ops"
allowed_tools = ["memory_search", "memory_get"]
session_target = "isolated"

[cron.jobs.schedule]
kind = "cron"
expr = "0 14 * * *"
tz = "America/Denver"

[cron.jobs.delivery]
mode = "announce"
channel = "telegram"
to = "-1001234567890"
best_effort = true
```

## Guardrails and limitations

- Same-host only: public-peer dispatch is an internal runtime/orchestrator path, not cross-host federation.
- Top-level/public only: explicit targets must be `default` or a `[peers.<id>]` entry with `public = true`.
- No delegate exposure: `[agents.<name>]` delegates are not bindable and are not valid public-peer targets.
- No public peer API: there is no REST or WebSocket API for public peers.
- Channel-facing access only: humans reach peers through configured channels and bindings.
- Shell jobs cannot set `target_public_peer`.

## See also

- [Config Reference](api/config-reference.md)
- [Commands Reference](cli/commands-reference.md)
- [ADR-005: Public Peers and Bound Conversations](../architecture/adr-005-public-peers-and-bound-conversations.md)
