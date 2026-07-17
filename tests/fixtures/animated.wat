;; Neutral stateful fixture shared by generic SVG and CLI adapter tests.
(module
	(import "aedicule.v0" "AE_frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
	(import "aedicule.v0" "AE_circle" (func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
	(memory (export "memory") 1)
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 0)
	(func (export "AE_configure") (result i32) i32.const 0)
	(func (export "AE_init") (param i32 i32 f32 f32) (result i32)
		i32.const 32 i32.const 0 i32.store
		i32.const 0)
	(func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "AE_tick") (param $count i32) (result i32)
		i32.const 32 i32.const 32 i32.load local.get $count i32.add i32.store
		i32.const 0)
	(func (export "AE_render") (result i32)
		f32.const 0.03137255 f32.const 0.04313725 f32.const 0.07058824 f32.const 1 call $begin drop
		i32.const 1 i32.const 32 i32.load f32.convert_i32_s f32.const 40
		f32.const 8 f32.const 2 i32.const 0x58ff72ff i32.const 1 call $circle drop
		call $end drop
		i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 32)
	(func (export "AE_state_len") (result i32) i32.const 4)
	(func (export "AE_state_schema") (result i32) i32.const 1)
)
