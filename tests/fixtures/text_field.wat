(module
	;; Declares one guest-owned text-entry control at configure time. The guest
	;; never receives a byte buffer: the current value arrives as ordered integer
	;; scalar events terminated by an exact count, so the host never writes into
	;; guest memory and no string decoding crosses the boundary.
	;;
	;; Observable state region (AE_state_ptr = 16):
	;;   16  last terminating scalar count
	;;   20  last terminating phase
	;;   24  total AE_text_event calls received
	;;   64  received scalar at index N, at 64 + N*4
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
		i32.const 24
		i32.const 24
		i32.load
		i32.const 1
		i32.add
		i32.store
		local.get $index
		i32.const -1
		i32.eq
		if
			i32.const 16 local.get $scalar i32.store
			i32.const 20 local.get $phase i32.store
		else
			i32.const 64
			local.get $index
			i32.const 4
			i32.mul
			i32.add
			local.get $scalar
			i32.store
		end
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32)
		i32.const 1
		global.get $submitted_revision
		i32.ne
		if
			i32.const 1 call $ui_begin drop
			i32.const 5
			i32.const 655360 i32.const 1310720
			i32.const 19660800 i32.const 3145728
			i32.const 0x101820e8 i32.const 0
			call $control_panel drop
			i32.const 9 i32.const 5
			i32.const 1048576 i32.const 1572864
			i32.const 18350080 i32.const 2097152
			i32.const 0
			call $text_field_place drop
			call $ui_end drop
			i32.const 1 global.set $submitted_revision
		end
		i32.const 0x081020ff call $frame_begin drop
		call $frame_end)
	(func (export "AE_state_ptr") (result i32) i32.const 16)
	(func (export "AE_state_len") (result i32) i32.const 304)
	(func (export "AE_state_schema") (result i32) i32.const 1))
