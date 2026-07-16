# Local ztracing compatibility layer

The pinned GPUI graph uses `ztracing::instrument` in its default no-op mode,
but upstream's facade also pulls a GPL-only logger. This clean Apache-2.0
compatibility crate implements only that default identity surface. Enabling
real Zed tracing fails at compile time and requires a separate explicit
dependency and licensing decision.

The interface was derived from GPUI's public call sites and the same
compatibility pattern proven in Peter's `validate_gui` project; no GPL source
was copied.
