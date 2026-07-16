//! Identity attribute used when GPUI's optional tracing is disabled.
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn instrument(_attribute: TokenStream, item: TokenStream) -> TokenStream {
	item
}
