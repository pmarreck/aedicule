(module
	(import "aedicule.v0" "AE_external_link"
		(func $external_link (param i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_ui_begin" (func $ui_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_ui_end" (func $ui_end (result i32)))
	(import "aedicule.v0" "AE_control_panel_q16"
		(func $control_panel (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_external_link_place_q16"
		(func $external_link_place (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin_rgba" (func $frame_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "What is this?")
	(data (i32.const 32) "https://example.com/readme#ulam")
	(global $invalid (mut i32) (i32.const 0))
	(global $submitted_revision (mut i32) (i32.const 0))
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 4)
	(func (export "AE_configure") (result i32)
		i32.const 7
		i32.const 0 i32.const 13
		i32.const 32 i32.const 31
		i32.const 0
		call $external_link)
	(func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
	(func (export "AE_event_i32") (param $kind i32) (param $code i32) (param i32 i32) (result i32)
		local.get $kind i32.const 1 i32.eq
		local.get $code i32.const 11 i32.eq
		i32.and
		if i32.const 1 global.set $invalid end
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32)
		;; AVP revisions are retained: submit only when desired UI changes.
		global.get $invalid i32.const 1 i32.add
		global.get $submitted_revision i32.ne
		if
			global.get $invalid i32.const 1 i32.add call $ui_begin drop
			global.get $invalid
			if (result i32) i32.const 99 else i32.const 5 end
			i32.const 655360 i32.const 1310720
			i32.const 19660800 i32.const 3145728
			i32.const 0x101820e8 i32.const 0
			call $control_panel drop
			i32.const 7 i32.const 5
			i32.const 1048576 i32.const 1572864
			i32.const 18350080 i32.const 2097152
			i32.const 0
			call $external_link_place drop
			call $ui_end drop
			global.get $invalid i32.const 1 i32.add global.set $submitted_revision
		end
		i32.const 0x081020ff call $frame_begin drop
		call $frame_end)
	(func (export "AE_state_ptr") (result i32) i32.const 0)
	(func (export "AE_state_len") (result i32) i32.const 0)
	(func (export "AE_state_schema") (result i32) i32.const 1))
