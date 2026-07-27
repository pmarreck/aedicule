(module
	;; Browser acceptance fixture for the software-keyboard classifier's second
	;; half: a placed text field that a real browser can focus and type into.
	;;
	;; The guest turns its canvas background green only after the host has
	;; delivered the exact scalar run for "hi" — first scalar 'h' at index 0,
	;; last scalar 'i' at index 1, terminating count 2. The published frame
	;; background is therefore an end-to-end oracle spanning DOM keyboard input,
	;; GPUI's input handler, the platform text widget, and the WAT ABI.
	(import "aedicule.v0" "AE_text_field"
		(func $text_field (param i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_ui_begin" (func $ui_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_ui_end" (func $ui_end (result i32)))
	(import "aedicule.v0" "AE_control_panel_q16"
		(func $control_panel (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_text_field_place_q16"
		(func $text_field_place (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin_rgba" (func $frame_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "Name")
	(global $submitted_revision (mut i32) (i32.const 0))
	(global $first (mut i32) (i32.const 0))
	(global $last (mut i32) (i32.const 0))
	(global $count (mut i32) (i32.const 0))
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 5)
	(func (export "AE_configure") (result i32)
		i32.const 9
		i32.const 0 i32.const 4
		i32.const 64
		i32.const 0
		call $text_field)
	(func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
	(func (export "AE_event_i32") (param i32) (param i32) (param i32 i32) (result i32) i32.const 0)
	(func (export "AE_text_event")
		(param $id i32) (param $index i32) (param $scalar i32) (param $phase i32) (result i32)
		local.get $index
		i32.const -1
		i32.eq
		if
			local.get $scalar global.set $count
		else
			local.get $index i32.eqz
			if local.get $scalar global.set $first end
			local.get $index i32.const 1 i32.eq
			if local.get $scalar global.set $last end
		end
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32)
		i32.const 1
		global.get $submitted_revision
		i32.ne
		if
			i32.const 1 call $ui_begin drop
			;; Panel at (100, 100) sized 400x120 in Q16.16 logical pixels.
			i32.const 5
			i32.const 6553600 i32.const 6553600
			i32.const 26214400 i32.const 7864320
			i32.const 0x101820e8 i32.const 0
			call $control_panel drop
			;; Text field at (120, 140) sized 360x40.
			i32.const 9 i32.const 5
			i32.const 7864320 i32.const 9175040
			i32.const 23592960 i32.const 2621440
			i32.const 0
			call $text_field_place drop
			call $ui_end drop
			i32.const 1 global.set $submitted_revision
		end
		global.get $first i32.const 0x68 i32.eq
		global.get $last i32.const 0x69 i32.eq
		i32.and
		global.get $count i32.const 2 i32.eq
		i32.and
		if (result i32) i32.const 0x00ff00ff else i32.const 0xff0000ff end
		call $frame_begin drop
		call $frame_end)
	(func (export "AE_state_ptr") (result i32) i32.const 0)
	(func (export "AE_state_len") (result i32) i32.const 0)
	(func (export "AE_state_schema") (result i32) i32.const 1))
