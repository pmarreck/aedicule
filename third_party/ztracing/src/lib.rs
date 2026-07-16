//! Default GPUI instrumentation compatibility without a tracing runtime.
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

#[cfg(ztracing)]
compile_error!(
	"real Zed tracing is deliberately unsupported by gpui-wasm's Apache compatibility layer"
);

pub use ztracing_macro::instrument;

#[derive(Clone, Copy, Debug, Default)]
pub struct Span;

impl Span {
	pub fn current() -> Self { Self }
	pub fn enter(&self) {}
	pub fn record<T, S>(&self, _field: T, _value: S) {}
}

#[macro_export]
macro_rules! trace_span { ($($tokens:tt)*) => { $crate::Span }; }
#[macro_export]
macro_rules! info_span { ($($tokens:tt)*) => { $crate::Span }; }
#[macro_export]
macro_rules! debug_span { ($($tokens:tt)*) => { $crate::Span }; }
#[macro_export]
macro_rules! warn_span { ($($tokens:tt)*) => { $crate::Span }; }
#[macro_export]
macro_rules! error_span { ($($tokens:tt)*) => { $crate::Span }; }
#[macro_export]
macro_rules! event { ($($tokens:tt)*) => { $crate::Span }; }
#[macro_export]
macro_rules! span { ($($tokens:tt)*) => { $crate::Span }; }
