//! Independent integration controls for Aedicule's pinned RandomZ engine.
//!
//! Expected values come from the pre-existing LuaJIT/Zig/`randomr`
//! conformance contract at RandomZ revision 346889f1762d, rather than from an
//! Aedicule wrapper or values generated during this test.

use aedicule::{Frontplane, FrontplaneError, Limits};
use randomr::{
    Drbg, Fixed, beta, exponential, log_normal, normal, normal_int, poisson, range, uniform,
};

const RANDOM_ABI_WAT: &str = r#"(module
	(import "aedicule.v0" "AE_random_v1_seed_u64"
		(func $seed_u64 (param i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_seed_bytes"
		(func $seed_bytes (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_fill"
		(func $fill (param i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_range_i64"
		(func $range (param i32 i64 i64 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_uniform"
		(func $uniform (param i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_normal"
		(func $normal (param i32 i64 i32 i64 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_normal_i64"
		(func $normal_i64 (param i32 i64 i64 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_exponential"
		(func $exponential (param i32 i64 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_poisson"
		(func $poisson (param i32 i64 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_log_normal"
		(func $log_normal (param i32 i64 i32 i64 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_beta"
		(func $beta (param i32 i64 i32 i64 i32 i32 i32) (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 2400)
		"\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\2a")
	(func $seed (param $state i32)
		local.get $state i32.const 42 i32.const 0 call $seed_u64 drop)
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 9)
	(func (export "AE_configure") (result i32) i32.const 0)
	(func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32)
		i32.const 0 call $seed
		i32.const 0 i32.const 64 i32.const 64 call $fill drop

		i32.const 256 call $seed
		i32.const 256 i64.const 0 i64.const 99 i32.const 320 i32.const 3 call $range drop

		i32.const 512 call $seed
		i32.const 512 i32.const 576 i32.const 3 call $uniform drop

		i32.const 768 call $seed
		i32.const 768 i64.const 0 i32.const 0
		i64.const 4611686018427387904 i32.const 0
		i32.const 832 i32.const 2 call $normal drop

		i32.const 1024 call $seed
		i32.const 1024 i64.const 0 i64.const 99 i32.const 1088 i32.const 3
		call $normal_i64 drop

		i32.const 1280 call $seed
		i32.const 1280 i64.const 4611686018427387904 i32.const 0
		i32.const 1344 i32.const 2 call $exponential drop

		i32.const 1536 call $seed
		i32.const 1536 i64.const 5764607523034234880 i32.const 2
		i32.const 1600 i32.const 3 call $poisson drop

		i32.const 1792 call $seed
		i32.const 1792 i64.const 0 i32.const 0
		i64.const 4611686018427387904 i32.const 0
		i32.const 1856 i32.const 2 call $log_normal drop

		i32.const 2048 call $seed
		i32.const 2048 i64.const 4611686018427387904 i32.const 1
		i64.const 4611686018427387904 i32.const 1
		i32.const 2112 i32.const 2 call $beta drop

		i32.const 2304 i32.const 2400 call $seed_bytes drop
		i32.const 2304 i32.const 2352 i32.const 16 call $fill drop
		i32.const 0)
	(func (export "AE_event_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32) i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 0)
	(func (export "AE_state_len") (result i32) i32.const 2432)
	(func (export "AE_state_schema") (result i32) i32.const 1))"#;

const RANDOM_FAILURE_WAT: &str = r#"(module
	(import "aedicule.v0" "AE_random_v1_seed_u64"
		(func $seed (param i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_fill"
		(func $fill (param i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_random_v1_normal"
		(func $normal (param i32 i64 i32 i64 i32 i32 i32) (result i32)))
	(memory (export "memory") 2)
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 9)
	(func (export "AE_configure") (result i32) i32.const 0)
	(func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32)
		i32.const 0 i32.const 42 i32.const 0 call $seed)
	(func (export "AE_event_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
	(func (export "AE_tick") (param $which i32) (result i32)
		local.get $which i32.const 1 i32.eq
		if (result i32)
			i32.const 0 i32.const 131064 i32.const 16 call $fill
		else
			local.get $which i32.const 2 i32.eq
			if (result i32)
				i32.const 0 i32.const 64 i32.const 65537 call $fill
			else
				local.get $which i32.const 3 i32.eq
				if (result i32)
					i32.const 0 i32.const 0 i32.const 1 call $fill
				else
					local.get $which i32.const 4 i32.eq
					if (result i32)
						i32.const 0 i64.const 0 i32.const 0
						i64.const 4611686018427387904 i32.const 0
						i32.const 64 i32.const 1 call $normal
					else
						i32.const 0 i32.const 64 i32.const 8 call $fill
					end
				end
			end
		end)
	(func (export "AE_render") (result i32) i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 0)
	(func (export "AE_state_len") (result i32) i32.const 48)
	(func (export "AE_state_schema") (result i32) i32.const 1))"#;

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
            0x69, 0xdf, 0xe2, 0xe9, 0xb5, 0x79, 0xcf, 0x6d, 0xfe, 0x3d, 0x71, 0xb1, 0x10, 0x24,
            0xdb, 0x6e, 0xb4, 0x9d, 0x5b, 0x98, 0x61, 0x50, 0x5b, 0x3e, 0xcf, 0xc3, 0xd3, 0x79,
            0xa6, 0xdc, 0x8f, 0x04, 0xb6, 0x90, 0x0d, 0xb3, 0x33, 0xd2, 0x07, 0x60, 0x66, 0x12,
            0x26, 0xda, 0x01, 0x0d, 0xb5, 0x89, 0xc5, 0x08, 0x0a, 0xaf, 0x5f, 0x60, 0x68, 0xfc,
            0x08, 0x74, 0xce, 0x62, 0xac, 0xa3, 0x6f, 0x60,
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

fn read_i64s(bytes: &[u8], offset: usize, count: usize) -> Vec<i64> {
    bytes[offset..offset + count * 8]
        .chunks_exact(8)
        .map(|value| i64::from_le_bytes(value.try_into().unwrap()))
        .collect()
}

fn read_fixed(bytes: &[u8], offset: usize, count: usize) -> Vec<Fixed> {
    bytes[offset..offset + count * 12]
        .chunks_exact(12)
        .map(|value| {
            Fixed::from_parts(
                i64::from_le_bytes(value[..8].try_into().unwrap()),
                i32::from_le_bytes(value[8..].try_into().unwrap()),
            )
            .unwrap()
        })
        .collect()
}

fn samples<T>(count: usize, mut draw: impl FnMut(&mut Drbg) -> T) -> Vec<T> {
    let mut source = Drbg::new(&seed_42());
    (0..count).map(|_| draw(&mut source)).collect()
}

#[test]
fn guest_random_v1_batches_match_the_pinned_engine_and_wire_format() {
    let mut frontplane = Frontplane::from_wat(RANDOM_ABI_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(42, 640.0, 480.0).unwrap();
    let bytes = frontplane.snapshot().unwrap().bytes;

    let mut raw = [0_u8; 64];
    Drbg::new(&seed_42()).fill(&mut raw).unwrap();
    assert_eq!(&bytes[64..128], &raw);
    assert_eq!(&bytes[2352..2368], &raw[..16]);

    assert_eq!(
        read_i64s(&bytes, 320, 3),
        samples(3, |source| range(source, 0, 99).unwrap())
    );
    assert_eq!(
        read_fixed(&bytes, 576, 3),
        samples(3, |source| uniform(source).unwrap())
    );
    assert_eq!(
        read_fixed(&bytes, 832, 2),
        samples(2, |source| normal(source, Fixed::ZERO, Fixed::from_i64(1))
            .unwrap())
    );
    assert_eq!(
        read_i64s(&bytes, 1088, 3),
        samples(3, |source| normal_int(source, 0, 99).unwrap())
    );
    assert_eq!(
        read_fixed(&bytes, 1344, 2),
        samples(2, |source| exponential(source, Fixed::from_i64(1)).unwrap())
    );
    assert_eq!(
        read_i64s(&bytes, 1600, 3),
        samples(3, |source| poisson(source, Fixed::from_i64(5)).unwrap())
    );
    assert_eq!(
        read_fixed(&bytes, 1856, 2),
        samples(2, |source| {
            log_normal(source, Fixed::ZERO, Fixed::from_i64(1)).unwrap()
        })
    );
    assert_eq!(
        read_fixed(&bytes, 2112, 2),
        samples(2, |source| {
            beta(source, Fixed::from_i64(2), Fixed::from_i64(2)).unwrap()
        })
    );

    for state_offset in [0, 256, 512, 768, 1024, 1280, 1536, 1792, 2048, 2304] {
        assert_eq!(&bytes[state_offset..state_offset + 4], b"AER\x01");
        assert_eq!(&bytes[state_offset + 4..state_offset + 8], &[0; 4]);
    }
}

#[test]
fn rejected_random_writes_leave_guest_owned_stream_state_unchanged() {
    let limits = Limits {
        max_random_source_bytes: 4,
        ..Limits::default()
    };
    let mut frontplane = Frontplane::from_wat(RANDOM_FAILURE_WAT, limits).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(42, 640.0, 480.0).unwrap();
    let initial = frontplane.snapshot().unwrap().bytes;

    assert!(matches!(
        frontplane.tick(1),
        Err(FrontplaneError::InvalidPointer {
            operation: "AE_random_v1_fill"
        })
    ));
    assert_eq!(frontplane.snapshot().unwrap().bytes, initial);

    assert!(matches!(
        frontplane.tick(2),
        Err(FrontplaneError::BudgetExhausted {
            operation: "AE_random_v1_fill",
            kind: "random output",
        })
    ));
    assert_eq!(frontplane.snapshot().unwrap().bytes, initial);

    assert!(matches!(
        frontplane.tick(3),
        Err(FrontplaneError::InvalidPointer {
            operation: "AE_random_v1_fill"
        })
    ));
    assert_eq!(frontplane.snapshot().unwrap().bytes, initial);

    assert!(matches!(
        frontplane.tick(4),
        Err(FrontplaneError::BudgetExhausted {
            operation: "AE_random_v1_normal",
            kind: "random source",
        })
    ));
    assert_eq!(frontplane.snapshot().unwrap().bytes, initial);

    assert!(matches!(
        frontplane.tick(5),
        Err(FrontplaneError::BudgetExhausted {
            operation: "AE_random_v1_fill",
            kind: "random source",
        })
    ));
    assert_eq!(frontplane.snapshot().unwrap().bytes, initial);
}
