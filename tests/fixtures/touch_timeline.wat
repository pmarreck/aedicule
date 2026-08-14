(module
	;; Turns green only when one contact remains held for exactly three ticks and
	;; then ends before two further ticks. This makes event counts insufficient:
	;; the headless adapter must preserve touch/tick timeline order.
	(import "aedicule.v0" "AE_touch_interest"
		(func $touch_interest (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin_rgba"
		(func $frame_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(global $held (mut i32) (i32.const 0))
	(global $held_ticks (mut i32) (i32.const 0))

	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 10)
	(func (export "AE_configure") (result i32)
		i32.const 1 i32.const 0 call $touch_interest)
	(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "AE_event")
		(param $kind i32) (param i32) (param f32 f32) (result i32)
		local.get $kind i32.const 11 i32.eq
		(if (then i32.const 1 global.set $held))
		local.get $kind i32.const 13 i32.eq
		local.get $kind i32.const 14 i32.eq i32.or
		(if (then i32.const 0 global.set $held))
		i32.const 0)
	(func (export "AE_tick") (param $ticks i32) (result i32)
		global.get $held
		(if (then
			global.get $held_ticks local.get $ticks i32.add
			global.set $held_ticks))
		i32.const 0)
	(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
		i32.const 60 i32.const 1)
	(func (export "AE_render") (result i32)
		global.get $held_ticks i32.const 3 i32.eq
		global.get $held i32.eqz i32.and
		(if (result i32)
			(then i32.const 0x00ff00ff)
			(else i32.const 0xff0000ff))
		call $frame_begin drop
		call $frame_end)
	(func (export "AE_state_ptr") (result i32) i32.const 0)
	(func (export "AE_state_len") (result i32) i32.const 0)
	(func (export "AE_state_schema") (result i32) i32.const 1))
