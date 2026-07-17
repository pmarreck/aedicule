# Deferred GPUI display-refresh and presentation API proposal

Status: design note only, 2026-07-17. No upstream issue, discussion, branch, or
pull request has been opened from this repository.

## Why GPUI needs this

Aedicule can already deliver an exact rational `AE_EVENT_DISPLAY_REFRESH` to a
WAT application and renegotiate its fixed simulation rate. GPUI publicly
exposes display identity and bounds, but not the display's nominal refresh rate
or a notification when a window crosses to a new display. The native backends
already possess much of this information:

- macOS binds a `CVDisplayLink` to the window's `CGDirectDisplayID`;
- Windows owns the window `HMONITOR` and uses a DXGI swap chain;
- X11 queries RandR CRTC mode timing for its refresh loop; and
- Wayland receives each `wl_output::Mode`, including its refresh value, and
  knows which output currently contains the surface.

The public abstraction should expose this once instead of downstream programs
creating a second display-server connection, guessing a monitor from window
bounds, or shelling out to `xrandr`.

## Proposed display contract

GPUI should define a platform-neutral exact nominal rate rather than returning
`f32` or integer hertz:

```rust
pub struct DisplayRefreshRate {
	pub numerator_hz: u32,
	pub denominator: u32,
}

pub trait PlatformDisplay {
	// Existing identity and bounds methods ...
	fn nominal_refresh_rate(&self) -> Option<DisplayRefreshRate>;
}
```

`numerator_hz / denominator` must be normalized. A backend may return `None`
when its platform does not reveal a nominal rate. The numerator/denominator
representation preserves X11 mode timing and gives Wayland's millihertz mode
value an honest representation (`millihertz / 1000`) without pretending it is
more precise than the protocol supplied.

GPUI also needs a window-level callback or observable state change when the
current display changes. It should fire once for a genuine display identity or
nominal-rate change, not once per rendered frame. A public `Window` method may
be preferable to exposing `PlatformWindow` directly; its exact shape should be
designed with the maintainers.

## Presentation policy: separate from simulation

Display-refresh observation informs a fixed simulation policy. It does *not*
by itself prevent tearing. Presentation needs an independent, host-only policy:

```rust
pub enum PresentationPolicy {
	/// Vertical-blank/FIFO presentation; the default and tear-free target.
	Synchronized,
	/// Use VRR when the platform, driver, and monitor support it; otherwise Sync.
	Adaptive,
	/// Lowest-latency best effort; visible tearing is permitted.
	Immediate,
}
```

No WAT import or event should expose per-frame present timestamps. Variable
refresh rate deliberately changes scanout intervals; making that ambient guest
input would undermine deterministic simulation. Aedicule should continue to
accumulate simulation from its monotonic clock and render on native frame
requests, optionally interpolating presentation state.

On Windows, `Adaptive` should use DXGI's documented capability query and swap
chain/present flags, with `Synchronized` fallback. The API's
`ALLOW_TEARING` name is required for DXGI VRR support, but it must never be
treated as a promise that an unsupported display cannot visibly tear. See
[Microsoft's VRR guidance](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/variable-refresh-rate-displays).

Wayland compositors ultimately control presentation and VRR policy. Frame
callbacks remain useful pacing signals; accurate displayed-frame timestamps
would require the optional presentation-time protocol. X11 needs native RandR
output/CRTC association and, separately, a present-feedback investigation.
macOS can derive nominal/predicted timing from its existing display link.

## Adoption and testing plan

1. Discuss the narrow API with Zed maintainers before writing an upstream PR.
   Zed's contribution policy welcomes LLM assistance but requires a human who
   understands and owns the work; it does not accept autonomous-agent PRs.
2. Land display rate plus changed-display notification before adding
   presentation-policy controls.
3. Test exact normalization and change de-duplication in GPUI's platform test
   backend; test backend conversion where an OS-independent fake is possible.
4. Manually verify macOS, Windows, X11, and Wayland with windows moved between
   distinct-refresh displays. Verify VRR separately on supported hardware.
5. Keep Aedicule's `AE_DISPLAY_REFRESH_RATE` environment override as an
   explicit developer/test fallback until native GPUI delivery is available.

This document is intentionally a future discussion brief, not upstream prose.
Peter must decide whether to open the discussion and must author the public
conversation in his own voice.
