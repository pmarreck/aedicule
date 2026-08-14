(module
	(import "aedicule.v0" "AE_touch_interest"
		(func $touch_interest (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_action"
		(func $action (param i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_ui_begin" (func $ui_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_ui_end" (func $ui_end (result i32)))
	(import "aedicule.v0" "AE_control_panel_q16"
		(func $control_panel (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_button_place_q16"
		(func $button_place (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin_rgba"
		(func $frame_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "Touch-safe")
	(global $starts (mut i32) (i32.const 0))
	(global $moves (mut i32) (i32.const 0))
	(global $ends (mut i32) (i32.const 0))
	(global $cancels (mut i32) (i32.const 0))
	(global $ui_sent (mut i32) (i32.const 0))

	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 10)
	(func (export "AE_configure") (result i32)
		i32.const 3 i32.const 0 call $touch_interest drop
		i32.const 42 i32.const 0 i32.const 10 i32.const 0 call $action)
	(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "AE_event")
		(param $kind i32) (param i32) (param f32 f32) (result i32)
		local.get $kind i32.const 11 i32.eq
		(if (then global.get $starts i32.const 1 i32.add global.set $starts))
		local.get $kind i32.const 12 i32.eq
		(if (then global.get $moves i32.const 1 i32.add global.set $moves))
		local.get $kind i32.const 13 i32.eq
		(if (then global.get $ends i32.const 1 i32.add global.set $ends))
		local.get $kind i32.const 14 i32.eq
		(if (then global.get $cancels i32.const 1 i32.add global.set $cancels))
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
		i32.const 60 i32.const 1)
	(func (export "AE_render") (result i32)
		global.get $ui_sent i32.eqz
		if
			i32.const 1 call $ui_begin drop
			i32.const 5
			i32.const 6553600 i32.const 6553600
			i32.const 26214400 i32.const 7864320
			i32.const 0x101820e8 i32.const 0
			call $control_panel drop
			i32.const 9 i32.const 5 i32.const 42
			i32.const 7864320 i32.const 9175040
			i32.const 10485760 i32.const 2621440
			i32.const 0
			call $button_place drop
			call $ui_end drop
			i32.const 1 global.set $ui_sent
		end
		global.get $starts i32.const 3 i32.ge_u
		global.get $moves i32.const 2 i32.ge_u i32.and
		global.get $ends i32.const 2 i32.ge_u i32.and
		global.get $cancels i32.const 1 i32.ge_u i32.and
		(if (result i32)
			(then i32.const 0x00ff00ff)
			(else i32.const 0xff0000ff))
		call $frame_begin drop
		call $frame_end)
	(func (export "AE_state_ptr") (result i32) i32.const 0)
	(func (export "AE_state_len") (result i32) i32.const 0)
	(func (export "AE_state_schema") (result i32) i32.const 1))
