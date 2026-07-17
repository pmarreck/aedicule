(module
	(import "aedicule.v0" "AE_title" (func $title (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "GPUI–WASM Conformance Fallback")
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 0)
	(func (export "AE_configure") (result i32)
		i32.const 0 i32.const 32 call $title drop
		i32.const 0)
	(func (export "AE_init") (param i32 i32 f32 f32) (result i32)
		i32.const 64 i64.const 0 i64.store
		i32.const 0)
	(func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "AE_tick") (param $count i32) (result i32)
		i32.const 64
		i32.const 64 i64.load local.get $count i64.extend_i32_u i64.add
		i64.store
		i32.const 0)
	(func (export "AE_render") (result i32)
		f32.const 0.03137255 f32.const 0.04313725 f32.const 0.07058824 f32.const 1
		call $frame_begin drop
		call $frame_end drop
		i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 64)
	(func (export "AE_state_len") (result i32) i32.const 8)
	(func (export "AE_state_schema") (result i32) i32.const 1)
)
