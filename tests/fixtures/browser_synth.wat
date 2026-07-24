(module
	(import "aedicule.v0" "AE_synth_voice"
		(func $synth_voice
			(param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
			(result i32)))
	(import "aedicule.v0" "AE_audio"
		(func $audio (param i32 f32 f32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin_rgba"
		(func $frame_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(global $played (mut i32) (i32.const 0))
	(global $background (mut i32) (i32.const 0x081020ff))

	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 0)
	(func (export "AE_configure") (result i32)
		i32.const 7       ;; program ID
		i32.const 1       ;; sine
		i32.const 0       ;; delay ms
		i32.const 60      ;; duration ms
		i32.const 440000  ;; start frequency, millihertz
		i32.const 440000  ;; midpoint frequency
		i32.const 440000  ;; end frequency
		i32.const 0       ;; start gain, ppm
		i32.const 1000000 ;; peak gain
		i32.const 0       ;; end gain
		i32.const 0       ;; no filter
		i32.const 0       ;; filter start
		i32.const 0       ;; filter end
		i32.const 0       ;; cooldown
		call $synth_voice)
	(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "AE_event") (param $kind i32) (param i32) (param f32 f32) (result i32)
		local.get $kind i32.const 4 i32.eq
		global.get $played i32.eqz
		i32.and
		if
			i32.const 7 f32.const 1 f32.const 1 i32.const 0 call $audio drop
			i32.const 1 global.set $played
		end
		local.get $kind i32.const 1 i32.eq
		if i32.const 0xd94b64ff global.set $background end
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
		i32.const 60 i32.const 1)
	(func (export "AE_render") (result i32)
		global.get $background call $frame_begin drop
		call $frame_end drop
		i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 0)
	(func (export "AE_state_len") (result i32) i32.const 0)
	(func (export "AE_state_schema") (result i32) i32.const 1))
