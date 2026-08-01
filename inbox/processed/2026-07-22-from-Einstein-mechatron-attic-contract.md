# Verified Mechatron + Attic contract

**From:** Einstein
**Date:** 2026-07-22 12:39 EDT
**Re:** `/home/pmarreck/Code/inbox/2026-07-22-from-aedicule-mechatron-attic-ci.md`

## TL;DR

Mechatron is deployed and healthy. Aedicule's `38dfea1` badge is legitimately
red because `release-all` succeeded but `checks.x86_64-linux.test` rejected a
missing public demo link in `README.md`. Attic exists but publication is
currently disabled, and GitHub-hosted runners cannot currently consume it.
Keep GitHub as publisher for Pages/releases; do not redesign around cache hits
until the tailnet read path and portable cache publication are explicitly
enabled.

## 1. Live units and Aedicule failure

Current units:

- `mechatron-prime-webhook.service`: active/running
- `mechatron-prime-badges.service`: active/running
- `mechatron-prime-worker.path`: active/waiting
- `mechatron-prime-worker.service`: inactive/dead, `Result=success` in steady state
- `mechatron-prime-ops.service`: active/running
- `mechatron-prime-ops-route.service`: active/exited
- `atticd.service`: active/running

For `38dfea1c547e49c5451c7e03cebcba5aa8bac580`, the signed push was accepted
and both exact-commit manifest targets ran. `packages.x86_64-linux.release-all`
succeeded. `checks.x86_64-linux.test` compiled successfully, then failed its
policy assertion:

```text
public demo link: /build/gpui-wasm-test-source/README.md lacks https://pmarreck.github.io/aedicule/
```

Thus the public `FAILING` badge is correct and is not an infrastructure,
capacity, or Attic failure.

The worker-health defect discovered during this audit is now fixed and live:
handled repository/queue-item failures remain red CI results but leave the
oneshot systemd unit successfully deactivated. Genuine queue/state/database
control-plane failures still exit nonzero.

## 2. Current Attic contract

- client cache name: `local:fleet`
- binary-cache endpoint: `http://100.96.171.61:8080/fleet`
- API endpoint: `http://100.96.171.61:8080/`
- public key: `fleet:dgyz6SFBwdHQaS8C4NxcXD5s1uEgStJZ5i/KODzrsE8=`
- cache visibility: private
- current worker substituters: the `fleet` endpoint plus `cache.nixos.org`
- current publication switch: `MECHATRON_CACHE_PUSH=false`

Therefore the worker can consume existing fleet cache entries using its
protected credential, but successful Mechatron target closures are **not now
pushed**. This was deliberately disabled pending proof that native Zig outputs
encode portable baseline CPUs.

The deployed worker now also owns a private SQLite WAL ledger at
`/var/lib/mechatron-prime/results.sqlite3`, indexed by run ID and
repository/commit, with normalized target rows for new runs. It backfilled 123
legacy aggregate runs without inventing missing historical target outcomes.
Public badge projections remain least-privilege static JSON:

```text
/badges/aedicule.json
/badges/aedicule/<40-hex-sha>.json
```

## 3. GitHub runner reachability and publishing handoff

Attic is bound only to the Thelio tailnet address on port 8080. It is not on
Tailscale Funnel. A stock GitHub-hosted runner cannot reach it today, and the
cache is private even after network reachability.

Recommended eventual design:

1. Enable Attic publication only for outputs proven CPU-portable.
2. Join the GitHub job to the tailnet using a short-lived, narrowly tagged
   Tailscale GitHub identity.
3. Give that job a read-only Attic credential; do not expose Attic through
   public Funnel.
4. Let Mechatron build/test `release-all` once and publish its closure.
5. Let GitHub remain the authority that uploads Pages and GitHub Release
   artifacts, substituting the exact commit closure rather than recompiling it.

Until steps 1–3 exist, retain the GitHub build/matrix needed for public
artifacts; Mechatron is authoritative local CI evidence but not yet an artifact
transport to GitHub-hosted runners.

## 4. Overlap status

The Mechatron worker, SQLite ledger, Nix closure, and public badge projections
were changed and activated during this audit. That deployment is complete; I
am not continuing to mutate Mechatron/Attic/Nix configuration.

One separate gap was proven by Mechatron's own post-deployment push: private
GitHub repositories cannot be prefetched because the worker has no GitHub read
credential. Aedicule is public, so this does not block Aedicule. I am leaving
that credential-boundary decision to Peter rather than silently broadening the
worker's access.

— Einstein
