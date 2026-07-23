(module
	(import "aedicule.v0" "AE_sample_asset"
		(func $sample_asset (param i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_sample_play"
		(func $sample_play (param i32 f32 f32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin_rgba"
		(func $frame_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "assets/audio/satellite-destroyed.flac")
	(global $played (mut i32) (i32.const 0))

	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 1)
	(func (export "AE_configure") (result i32)
		i32.const 77 i32.const 0 i32.const 37 i32.const 0 call $sample_asset)
	(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "AE_event") (param $kind i32) (param i32) (param f32 f32) (result i32)
		local.get $kind i32.const 4 i32.eq
		global.get $played i32.eqz
		i32.and
		if
			i32.const 77 f32.const 1 f32.const 1 i32.const 0 call $sample_play drop
			i32.const 1 global.set $played
		end
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
		i32.const 60 i32.const 1)
	(func (export "AE_render") (result i32)
		i32.const 0x081020ff call $frame_begin drop
		call $frame_end drop
		i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 0)
	(func (export "AE_state_len") (result i32) i32.const 0)
	(func (export "AE_state_schema") (result i32) i32.const 1))
