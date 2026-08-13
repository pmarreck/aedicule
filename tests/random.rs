//! Independent integration controls for Aedicule's pinned RandomZ engine.
//!
//! Expected values come from the pre-existing LuaJIT/Zig/`randomr`
//! conformance contract at RandomZ revision 346889f1762d, rather than from an
//! Aedicule wrapper or values generated during this test.

use randomr::{Drbg, Fixed, exponential, normal, poisson};

fn seed_42() -> [u8; 32] {
	let mut seed = [0_u8; 32];
	seed[31] = 42;
	seed
}

#[test]
fn pinned_engine_matches_the_frozen_seed_42_stream() {
	let mut engine = Drbg::new(&seed_42());
	let mut actual = [0_u8; 64];
	engine.fill(&mut actual).expect("seed-42 stream");
	assert_eq!(
		actual,
		[
			0x69, 0xdf, 0xe2, 0xe9, 0xb5, 0x79, 0xcf, 0x6d, 0xfe, 0x3d, 0x71, 0xb1,
			0x10, 0x24, 0xdb, 0x6e, 0xb4, 0x9d, 0x5b, 0x98, 0x61, 0x50, 0x5b, 0x3e,
			0xcf, 0xc3, 0xd3, 0x79, 0xa6, 0xdc, 0x8f, 0x04, 0xb6, 0x90, 0x0d, 0xb3,
			0x33, 0xd2, 0x07, 0x60, 0x66, 0x12, 0x26, 0xda, 0x01, 0x0d, 0xb5, 0x89,
			0xc5, 0x08, 0x0a, 0xaf, 0x5f, 0x60, 0x68, 0xfc, 0x08, 0x74, 0xce, 0x62,
			0xac, 0xa3, 0x6f, 0x60,
		]
	);
}

#[test]
fn pinned_engine_matches_frozen_nonlinear_samples_and_consumption() {
	let one = Fixed::from_i64(1);
	let mut source = Drbg::new(&seed_42());
	assert_eq!(
		normal(&mut source, Fixed::ZERO, one)
			.expect("normal sample")
			.parts(),
		(-6_261_580_692_471_259_166, -2)
	);
	assert_eq!(source.position(), 8);

	let mut source = Drbg::new(&seed_42());
	assert_eq!(
		exponential(&mut source, one)
			.expect("exponential sample")
			.parts(),
		(8_143_522_549_336_293_876, -1)
	);
	assert_eq!(source.position(), 4);

	let mut source = Drbg::new(&seed_42());
	assert_eq!(poisson(&mut source, Fixed::from_i64(5)), Ok(5));
	assert_eq!(source.position(), 24);
}
