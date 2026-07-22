(module
	;; Aedicule's integer-control and fixed-point drawing profile keeps all
	;; application decisions exact until the unavoidable GPUI adapter boundary.
	(import "aedicule.v0" "AE_title" (func $title (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_menu_item" (func $menu_item (param i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_slider_i32" (func $slider_i32 (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_ui_begin" (func $ui_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_ui_end" (func $ui_end (result i32)))
	(import "aedicule.v0" "AE_control_panel_q16" (func $control_panel_q16 (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_slider_place_q16" (func $slider_place_q16 (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_button_place_q16" (func $button_place_q16 (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_sin_cos_turn" (func $sin_cos_turn (param i32) (result i32 i32)))
	(import "aedicule.v0" "AE_frame_begin_rgba" (func $frame_begin_rgba (param i32) (result i32)))
	(import "aedicule.v0" "AE_path_begin" (func $path_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_path_move_q16" (func $path_move_q16 (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_path_line_q16" (func $path_line_q16 (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_path_end_q16" (func $path_end_q16 (param i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))

	(memory (export "memory") 2)
	(data (i32.const 0) "The Most Illegal Uzumaki")
	(data (i32.const 32) "coarse t numerator/3000")
	(data (i32.const 64) "fine t offset/300000")
	(data (i32.const 96) "Play / Pause")
	(data (i32.const 112) "0.5x")
	(data (i32.const 120) "1x")
	(data (i32.const 124) "2x")

	(global $control_min i32 (i32.const 0))
	(global $control_max i32 (i32.const 3600))
	(global $control_default i32 (i32.const 1050))
	(global $fine_min i32 (i32.const -500))
	(global $fine_max i32 (i32.const 500))
	(global $fine_default i32 (i32.const 0))
	(global $step_count i32 (i32.const 2000))
	(global $vertex_count i32 (i32.const 2001))
	(global $point_base i32 (i32.const 4096))
	(global $point_stride i32 (i32.const 16))
	(global $amplitude_base i32 (i32.const 65536))
	(global $control (mut i32) (i32.const 1050))
	(global $fine_control (mut i32) (i32.const 0))
	(global $playing (mut i32) (i32.const 0))
	(global $speed_mode (mut i32) (i32.const 1)) ;; 0 = half, 1 = normal, 2 = double
	(global $playback_phase (mut i32) (i32.const 0))
	(global $viewport_width (mut i32) (i32.const 67108864)) ;; 1024 Q16.16
	(global $viewport_height (mut i32) (i32.const 50331648)) ;; 768 Q16.16
	(global $ui_revision (mut i32) (i32.const 1))
	(global $sent_ui_revision (mut i32) (i32.const 0))
	(global $min_x (mut i64) (i64.const 0))
	(global $max_x (mut i64) (i64.const 0))
	(global $min_y (mut i64) (i64.const 0))
	(global $max_y (mut i64) (i64.const 0))

	;; The durable state is two exact slider integers plus integer playback mode,
	;; speed, and sub-step playback phase. The sliders define
	;; t = (100 * control + fine_control) / 300000. Native GPUI receives their
	;; authoritative values from each changed guest-authored UI frame.

	;; Exact floor(sqrt(value)) for unsigned i64 using the restoring algorithm.
	(func $isqrt_u64 (param $value i64) (result i64)
		(local $remainder i64)
		(local $root i64)
		(local $bit i64)
		local.get $value local.set $remainder
		i64.const 0 local.set $root
		i64.const 4611686018427387904 local.set $bit
		(block $bit_ready
			(loop $shrink_bit
				local.get $bit local.get $remainder i64.le_u br_if $bit_ready
				local.get $bit i64.const 2 i64.shr_u local.set $bit
				br $shrink_bit))
		(block $done
			(loop $next
				local.get $bit i64.eqz br_if $done
				local.get $remainder
				local.get $root local.get $bit i64.add
				i64.ge_u
				if
					local.get $remainder
					local.get $root local.get $bit i64.add
					i64.sub local.set $remainder
					local.get $root i64.const 1 i64.shr_u
					local.get $bit i64.add local.set $root
				else
					local.get $root i64.const 1 i64.shr_u local.set $root
				end
				local.get $bit i64.const 2 i64.shr_u local.set $bit
				br $next))
		local.get $root)

	;; Amplitudes depend only on n, so compute them once instead of repeating
	;; 2,000 integer square roots on every continuous slider change.
	(func $compute_amplitudes
		(local $n i32)
		(local $sqrt_q16 i64)
		(local $amplitude_q16 i64)
		(block $done
			(loop $amplitude
				local.get $n global.get $step_count i32.ge_u br_if $done
				local.get $n i64.extend_i32_u
				i64.const 32 i64.shl
				call $isqrt_u64
				local.set $sqrt_q16
				local.get $n i64.extend_i32_u
				local.get $sqrt_q16 i64.mul
				local.get $n i64.extend_i32_u i64.const 1000 i64.add
				i64.div_u
				local.set $amplitude_q16
				global.get $amplitude_base
				local.get $n i32.const 8 i32.mul i32.add
				local.get $amplitude_q16 i64.store
				local.get $n i32.const 1 i32.add local.set $n
				br $amplitude)))

	;; Convert signed Q30 radians to the host's wrapping full-i32-turn domain.
	;; Reducing to [-pi, pi] first keeps the exact i64 multiply in range.
	(func $radians_q30_to_turn (param $radians i64) (result i32)
		(local $reduced i64)
		local.get $radians
		i64.const 6746518852 ;; tau * 2^30
		i64.rem_s
		local.set $reduced
		local.get $reduced i64.const 3373259426 i64.gt_s
		if
			local.get $reduced i64.const 6746518852 i64.sub local.set $reduced
		end
		local.get $reduced i64.const -3373259426 i64.lt_s
		if
			local.get $reduced i64.const 6746518852 i64.add local.set $reduced
		end
		local.get $reduced
		i64.const 683565276 ;; rounded 2^32 / tau
		i64.mul
		i64.const 30
		i64.shr_s
		i32.wrap_i64)

	;; Store all 2,001 raw Q16.16 points and their i64 bounds. The inner sine
	;; depends only on t, so it is evaluated once rather than 2,000 times.
	(func $compute_points
		(local $n i32)
		(local $address i32)
		(local $x i64)
		(local $y i64)
		(local $amplitude_q16 i64)
		(local $b_argument_q30 i64)
		(local $c_q30 i64)
		(local $angle_q30 i64)
		(local $inner_sin_q30 i32)
		(local $sin_q30 i32)
		(local $cos_q30 i32)
		(local $next_x i64)
		(local $next_y i64)
		(local $t_numerator i64)

		global.get $control i64.extend_i32_s
		i64.const 100 i64.mul
		global.get $fine_control i64.extend_i32_s i64.add
		local.set $t_numerator

		;; sin((250/3)t), with exact t = t_numerator/300000, gives
		;; sin(t_numerator/3600).
		local.get $t_numerator
		i64.const 1073741824
		i64.mul
		i64.const 3600
		i64.div_s
		call $radians_q30_to_turn
		call $sin_cos_turn
		drop
		local.set $inner_sin_q30

		global.get $point_base local.set $address
		i64.const 0 local.set $x
		i64.const 0 local.set $y
		local.get $address local.get $x i64.store
		local.get $address i32.const 8 i32.add local.get $y i64.store
		i64.const 0 global.set $min_x
		i64.const 0 global.set $max_x
		i64.const 0 global.set $min_y
		i64.const 0 global.set $max_y
		i32.const 0 local.set $n

		(block $done
			(loop $point
				local.get $n global.get $step_count i32.ge_u br_if $done

				;; a = n * sqrt(n) / (n + 1000), precomputed as Q16.16.
				global.get $amplitude_base
				local.get $n i32.const 8 i32.mul i32.add
				i64.load local.set $amplitude_q16

				;; b = sin((n/10) * sin((250/3)t)).
				local.get $n i64.extend_i32_u
				local.get $inner_sin_q30 i64.extend_i32_s
				i64.mul
				i64.const 10 i64.div_s
				local.set $b_argument_q30
				local.get $b_argument_q30
				call $radians_q30_to_turn
				call $sin_cos_turn
				local.set $cos_q30
				local.set $sin_q30

				;; c = n*t/10 = n*t_numerator/3000000 radians.
				local.get $n i64.extend_i32_u
				local.get $t_numerator
				i64.mul
				i64.const 1073741824 i64.mul
				i64.const 3000000 i64.div_s
				local.set $c_q30
				local.get $sin_q30 i64.extend_i32_s
				local.get $c_q30 i64.sub
				local.set $angle_q30

				local.get $angle_q30
				call $radians_q30_to_turn
				call $sin_cos_turn
				local.set $cos_q30
				local.set $sin_q30

				local.get $x
				local.get $amplitude_q16
				local.get $cos_q30 i64.extend_i32_s
				i64.mul i64.const 30 i64.shr_s
				i64.add local.set $next_x
				local.get $y
				local.get $amplitude_q16
				local.get $sin_q30 i64.extend_i32_s
				i64.mul i64.const 30 i64.shr_s
				i64.add local.set $next_y

				local.get $address global.get $point_stride i32.add local.set $address
				local.get $address local.get $next_x i64.store
				local.get $address i32.const 8 i32.add local.get $next_y i64.store

				local.get $next_x global.get $min_x i64.lt_s
				if local.get $next_x global.set $min_x end
				local.get $next_x global.get $max_x i64.gt_s
				if local.get $next_x global.set $max_x end
				local.get $next_y global.get $min_y i64.lt_s
				if local.get $next_y global.set $min_y end
				local.get $next_y global.get $max_y i64.gt_s
				if local.get $next_y global.set $max_y end

				local.get $next_x local.set $x
				local.get $next_y local.set $y
				local.get $n i32.const 1 i32.add local.set $n
				br $point)))

	(func $render_path
		(local $index i32)
		(local $address i32)
		(local $x i64)
		(local $y i64)
		(local $center_x i64)
		(local $center_y i64)
		(local $span_x i64)
		(local $span_y i64)
		(local $span i64)
		(local $available_width i64)
		(local $available_height i64)
		(local $extent i64)
		(local $screen_x i32)
		(local $screen_y i32)

		global.get $min_x global.get $max_x i64.add i64.const 2 i64.div_s local.set $center_x
		global.get $min_y global.get $max_y i64.add i64.const 2 i64.div_s local.set $center_y
		global.get $max_x global.get $min_x i64.sub local.set $span_x
		global.get $max_y global.get $min_y i64.sub local.set $span_y
		local.get $span_x local.set $span
		local.get $span_y local.get $span i64.gt_s
		if local.get $span_y local.set $span end
		local.get $span i64.eqz
		if i64.const 1 local.set $span end

		global.get $viewport_width i64.extend_i32_s
		i64.const 6291456 i64.sub ;; 96 logical pixels
		local.set $available_width
		global.get $viewport_height i64.extend_i32_s
		i64.const 10485760 i64.sub ;; 160 pixels leaves room for native control
		local.set $available_height
		local.get $available_width i64.const 65536 i64.lt_s
		if i64.const 65536 local.set $available_width end
		local.get $available_height i64.const 65536 i64.lt_s
		if i64.const 65536 local.set $available_height end
		local.get $available_width local.set $extent
		local.get $available_height local.get $extent i64.lt_s
		if local.get $available_height local.set $extent end

		i32.const 1 call $path_begin drop
		i32.const 0 local.set $index
		global.get $point_base local.set $address
		(block $done
			(loop $point
				local.get $index global.get $vertex_count i32.ge_u br_if $done
				local.get $address i64.load local.set $x
				local.get $address i32.const 8 i32.add i64.load local.set $y
				global.get $viewport_width i32.const 2 i32.div_s
				local.get $x local.get $center_x i64.sub
				local.get $extent i64.mul local.get $span i64.div_s
				i32.wrap_i64 i32.add
				local.set $screen_x
				global.get $viewport_height i32.const 2 i32.div_s
				local.get $y local.get $center_y i64.sub
				local.get $extent i64.mul local.get $span i64.div_s
				i32.wrap_i64 i32.add
				local.set $screen_y
				local.get $index i32.eqz
				if
					local.get $screen_x local.get $screen_y call $path_move_q16 drop
				else
					local.get $screen_x local.get $screen_y call $path_line_q16 drop
				end
				local.get $address global.get $point_stride i32.add local.set $address
				local.get $index i32.const 1 i32.add local.set $index
				br $point))
		i32.const 81920 ;; 1.25 pixels in Q16.16
		i32.const 0
		i32.const 0xf6d8a8e8
		i32.const 0
		call $path_end_q16 drop)

	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 1)

	(func (export "AE_configure") (result i32)
		i32.const 0 i32.const 24 call $title drop
		i32.const 10 i32.const 96 i32.const 12 i32.const 0 i32.const 0 call $menu_item drop
		i32.const 11 i32.const 112 i32.const 4 i32.const 0 i32.const 0 call $menu_item drop
		i32.const 12 i32.const 120 i32.const 2 i32.const 0 i32.const 0 call $menu_item drop
		i32.const 13 i32.const 124 i32.const 2 i32.const 0 i32.const 0 call $menu_item drop
		i32.const 1
		i32.const 32 i32.const 23
		global.get $control_min global.get $control_max i32.const 1 global.get $control_default
		call $slider_i32 drop
		i32.const 2
		i32.const 64 i32.const 20
		global.get $fine_min global.get $fine_max i32.const 1 global.get $fine_default
		call $slider_i32 drop
		i32.const 0)

	(func (export "AE_init_i32") (param i32 i32) (param $width_q16 i32) (param $height_q16 i32) (result i32)
		local.get $width_q16 global.set $viewport_width
		local.get $height_q16 global.set $viewport_height
		global.get $control_default global.set $control
		global.get $fine_default global.set $fine_control
		i32.const 0 global.set $playing
		i32.const 1 global.set $speed_mode
		i32.const 0 global.set $playback_phase
		i32.const 1 global.set $ui_revision
		i32.const 0 global.set $sent_ui_revision
		i32.const 256 global.get $control i32.store
		i32.const 260 global.get $fine_control i32.store
		i32.const 264 global.get $playing i32.store
		i32.const 268 global.get $speed_mode i32.store
		i32.const 272 global.get $playback_phase i32.store
		call $compute_amplitudes
		i32.const 0)

	(func (export "AE_event_i32") (param $kind i32) (param $code i32) (param $a_q16 i32) (param $b_q16 i32) (result i32)
		local.get $kind i32.const 6 i32.eq
		if
			local.get $a_q16 global.get $viewport_width i32.ne
			local.get $b_q16 global.get $viewport_height i32.ne
			i32.or
			if
				global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
			end
			local.get $a_q16 global.set $viewport_width
			local.get $b_q16 global.set $viewport_height
		end
		;; Native playback buttons arrive through the existing ordered action event.
		;; Space is an exact keyboard equivalent for the Play/Pause toggle.
		local.get $kind i32.const 7 i32.eq
		local.get $kind i32.const 1 i32.eq
		local.get $code i32.const 4 i32.eq
		i32.and i32.or
		if
			local.get $code i32.const 10 i32.eq
			local.get $kind i32.const 1 i32.eq
			local.get $code i32.const 4 i32.eq i32.and
			i32.or
			if
				global.get $playing i32.eqz global.set $playing
				global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
			else
				local.get $code i32.const 11 i32.eq
				if
					i32.const 0 global.set $speed_mode
					i32.const 0 global.set $playback_phase
					global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
				else
					local.get $code i32.const 12 i32.eq
					if
						i32.const 1 global.set $speed_mode
						i32.const 0 global.set $playback_phase
						global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
					else
						local.get $code i32.const 13 i32.eq
						if
							i32.const 2 global.set $speed_mode
							i32.const 0 global.set $playback_phase
							global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
						end
					end
				end
			end
			i32.const 264 global.get $playing i32.store
			i32.const 268 global.get $speed_mode i32.store
			i32.const 272 global.get $playback_phase i32.store
		end
		i32.const 0)

	(func (export "AE_control_event") (param $id i32) (param $value i32) (param i32) (result i32)
		local.get $id i32.const 1 i32.eq
		if
			local.get $value global.get $control_min i32.lt_s
			local.get $value global.get $control_max i32.gt_s
			i32.or
			if i32.const -1 return end
			local.get $value global.get $control i32.ne
			if
				global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
			end
			local.get $value global.set $control
			i32.const 256 local.get $value i32.store
			i32.const 0 return
		end
		local.get $id i32.const 2 i32.eq
		if
			local.get $value global.get $fine_min i32.lt_s
			local.get $value global.get $fine_max i32.gt_s
			i32.or
			if i32.const -1 return end
			local.get $value global.get $fine_control i32.ne
			if
				global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
			end
			local.get $value global.set $fine_control
			i32.const 260 local.get $value i32.store
			i32.const 0 return
		end
		i32.const -1)

	(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
		;; Keep host presentation and pointer-driven redraws at a responsive 60 Hz.
		i32.const 60 i32.const 1)

	(func (export "AE_tick") (param $ticks i32) (result i32)
		(local $steps i32)
		(local $phase_total i32)
		(local $fine_units_per_second i32)
		(local $fine_total i32)
		(local $fine_wraps i32)
		global.get $playing i32.eqz
		if i32.const 0 return end
		;; Playback timing is an integer phase accumulator independent of the
		;; host's 60 Hz presentation cadence: 10/20/40 fine units per second.
		global.get $speed_mode i32.eqz
		if
			i32.const 10 local.set $fine_units_per_second
		else
			global.get $speed_mode i32.const 1 i32.eq
			if
				i32.const 20 local.set $fine_units_per_second
			else
				i32.const 40 local.set $fine_units_per_second
			end
		end
		global.get $playback_phase
		local.get $ticks local.get $fine_units_per_second i32.mul i32.add
		local.tee $phase_total
		i32.const 60 i32.div_u local.set $steps
		local.get $phase_total i32.const 60 i32.rem_u global.set $playback_phase
		local.get $steps i32.eqz
		if
			i32.const 272 global.get $playback_phase i32.store
			i32.const 0 return
		end
		;; Advance the fine numerator itself. At its upper endpoint, carry ten
		;; coarse units and subtract 1000 fine units: both transformations are
		;; exactly equal because one coarse unit is 100 fine units. The coarse
		;; modulo then provides an exact combined-lattice wrap.
		global.get $fine_control local.get $steps i32.add local.set $fine_total
		local.get $fine_total global.get $fine_max i32.gt_s
		if
			local.get $fine_total i32.const 499 i32.add
			i32.const 1000 i32.div_u local.set $fine_wraps
			local.get $fine_total local.get $fine_wraps i32.const 1000 i32.mul i32.sub
			local.set $fine_total
			global.get $control global.get $control_min i32.sub
			local.get $fine_wraps i32.const 10 i32.mul i32.add
			global.get $control_max global.get $control_min i32.sub i32.const 1 i32.add
			i32.rem_u
			global.get $control_min i32.add global.set $control
		end
		local.get $fine_total global.set $fine_control
		global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
		i32.const 256 global.get $control i32.store
		i32.const 260 global.get $fine_control i32.store
		i32.const 272 global.get $playback_phase i32.store
		i32.const 0)

	(func (export "AE_render") (result i32)
		(local $panel_x i32)
		(local $panel_y i32)
		(local $panel_width i32)
		(local $panel_height i32)
		(local $slider_x i32)
		(local $slider_y i32)
		(local $fine_slider_y i32)
		(local $slider_width i32)
		(local $slider_height i32)
		(local $button_x i32)
		(local $button_y i32)
		(local $button_width i32)
		(local $button_height i32)
		(local $button_gap i32)
		(local $ui_status i32)
		call $compute_points

		;; This guest owns the native-control document. At ordinary window sizes the
		;; panel has 12 px side insets and the slider track has 24 px side insets,
		;; giving it essentially the full viewport width. Keep the controls above
		;; Aedicule's bottom status strip with a 32 px safety gap. The button row owns
		;; exact playback actions and the second track is an exact local fine-tune
		;; offset. Tiny viewports use a compact non-overlapping stack.
		global.get $viewport_width i32.const 1572864 i32.gt_s
		if
			i32.const 786432 local.set $panel_x
			global.get $viewport_width i32.const 1572864 i32.sub local.set $panel_width
		else
			i32.const 0 local.set $panel_x
			global.get $viewport_width local.set $panel_width
		end
		local.get $panel_width i32.const 65536 i32.lt_s
		if i32.const 65536 local.set $panel_width end
		global.get $viewport_width i32.const 3145728 i32.gt_s
		if
			i32.const 1572864 local.set $slider_x
			global.get $viewport_width i32.const 3145728 i32.sub local.set $slider_width
		else
			i32.const 0 local.set $slider_x
			global.get $viewport_width local.set $slider_width
		end
		local.get $slider_width i32.const 65536 i32.lt_s
		if i32.const 65536 local.set $slider_width end

		global.get $viewport_width i32.const 4718592 i32.gt_s
		if
			i32.const 1572864 local.set $button_x
			i32.const 524288 local.set $button_gap
			global.get $viewport_width i32.const 4718592 i32.sub
			i32.const 4 i32.div_s local.set $button_width
		else
			i32.const 0 local.set $button_x
			i32.const 0 local.set $button_gap
			global.get $viewport_width i32.const 4 i32.div_s local.set $button_width
		end
		local.get $button_width i32.const 65536 i32.lt_s
		if i32.const 65536 local.set $button_width end

		global.get $viewport_height i32.const 14680064 i32.gt_s
		if
			global.get $viewport_height i32.const 14680064 i32.sub local.set $panel_y
			i32.const 11010048 local.set $panel_height
			global.get $viewport_height i32.const 14155776 i32.sub local.set $button_y
			i32.const 2359296 local.set $button_height
			global.get $viewport_height i32.const 11534336 i32.sub local.set $slider_y
			global.get $viewport_height i32.const 8126464 i32.sub local.set $fine_slider_y
			i32.const 2883584 local.set $slider_height
		else
			i32.const 0 local.set $panel_y
			global.get $viewport_height i32.const 9437184 i32.gt_s
			if i32.const 9437184 local.set $panel_height
			else global.get $viewport_height local.set $panel_height end
			global.get $viewport_height i32.const 524288 i32.gt_s
			if i32.const 524288 local.set $button_y
			else i32.const 0 local.set $button_y end
			global.get $viewport_height i32.const 2097152 i32.gt_s
			if i32.const 2097152 local.set $button_height
			else global.get $viewport_height local.set $button_height end
			global.get $viewport_height i32.const 3145728 i32.gt_s
			if i32.const 3145728 local.set $slider_y
			else i32.const 0 local.set $slider_y end
			global.get $viewport_height i32.const 6291456 i32.gt_s
			if i32.const 6291456 local.set $fine_slider_y
			else i32.const 0 local.set $fine_slider_y end
			global.get $viewport_height i32.const 2621440 i32.gt_s
			if i32.const 2621440 local.set $slider_height
			else global.get $viewport_height local.set $slider_height end
		end
		local.get $panel_height i32.const 65536 i32.lt_s
		if i32.const 65536 local.set $panel_height end
		local.get $button_height i32.const 65536 i32.lt_s
		if i32.const 65536 local.set $button_height end
		local.get $slider_height i32.const 65536 i32.lt_s
		if i32.const 65536 local.set $slider_height end

		;; Aedicule retains the last accepted keyed UI snapshot. Submit the whole
		;; document only when viewport geometry or the exact slider value changes.
		global.get $ui_revision global.get $sent_ui_revision i32.ne
		if
			global.get $ui_revision call $ui_begin local.set $ui_status
			i32.const 5
			local.get $panel_x local.get $panel_y
			local.get $panel_width local.get $panel_height
			i32.const 0x080b12e8 i32.const 0
			call $control_panel_q16
			local.get $ui_status i32.or local.set $ui_status
			i32.const 10 i32.const 5 i32.const 10
			local.get $button_x local.get $button_y
			local.get $button_width local.get $button_height
			global.get $playing
			call $button_place_q16
			local.get $ui_status i32.or local.set $ui_status
			i32.const 11 i32.const 5 i32.const 11
			local.get $button_x local.get $button_width i32.add local.get $button_gap i32.add
			local.get $button_y local.get $button_width local.get $button_height
			global.get $speed_mode i32.eqz
			call $button_place_q16
			local.get $ui_status i32.or local.set $ui_status
			i32.const 12 i32.const 5 i32.const 12
			local.get $button_x
			local.get $button_width i32.const 2 i32.mul i32.add
			local.get $button_gap i32.const 2 i32.mul i32.add
			local.get $button_y local.get $button_width local.get $button_height
			global.get $speed_mode i32.const 1 i32.eq
			call $button_place_q16
			local.get $ui_status i32.or local.set $ui_status
			i32.const 13 i32.const 5 i32.const 13
			local.get $button_x
			local.get $button_width i32.const 3 i32.mul i32.add
			local.get $button_gap i32.const 3 i32.mul i32.add
			local.get $button_y local.get $button_width local.get $button_height
			global.get $speed_mode i32.const 2 i32.eq
			call $button_place_q16
			local.get $ui_status i32.or local.set $ui_status
			i32.const 1 i32.const 5 global.get $control
			local.get $slider_x local.get $slider_y
			local.get $slider_width local.get $slider_height
			i32.const 1 i32.const 0
			call $slider_place_q16
			local.get $ui_status i32.or local.set $ui_status
			i32.const 2 i32.const 5 global.get $fine_control
			local.get $slider_x local.get $fine_slider_y
			local.get $slider_width local.get $slider_height
			i32.const 1 i32.const 0
			call $slider_place_q16
			local.get $ui_status i32.or local.set $ui_status
			call $ui_end
			local.get $ui_status i32.or local.tee $ui_status
			i32.eqz
			if global.get $ui_revision global.set $sent_ui_revision end
		end

		i32.const 0x07090fff call $frame_begin_rgba drop
		call $render_path
		call $frame_end drop
		i32.const 0)

	(func (export "AE_state_ptr") (result i32) i32.const 256)
	(func (export "AE_state_len") (result i32) i32.const 20)
	(func (export "AE_state_schema") (result i32) i32.const 6)
	(func (export "AE_after_restore") (result i32)
		(local $restored i32)
		(local $restored_fine i32)
		(local $restored_flag i32)
		i32.const 256 i32.load local.set $restored
		local.get $restored global.get $control_min i32.lt_s
		local.get $restored global.get $control_max i32.gt_s
		i32.or
		if
			global.get $control_default local.set $restored
			i32.const 256 local.get $restored i32.store
		end
		local.get $restored global.set $control
		i32.const 260 i32.load local.set $restored_fine
		local.get $restored_fine global.get $fine_min i32.lt_s
		local.get $restored_fine global.get $fine_max i32.gt_s
		i32.or
		if
			global.get $fine_default local.set $restored_fine
			i32.const 260 local.get $restored_fine i32.store
		end
		local.get $restored_fine global.set $fine_control
		i32.const 264 i32.load local.tee $restored_flag
		i32.const 1 i32.gt_u
		if
			i32.const 0 local.set $restored_flag
			i32.const 264 local.get $restored_flag i32.store
		end
		local.get $restored_flag global.set $playing
		i32.const 268 i32.load local.tee $restored_flag
		i32.const 2 i32.gt_u
		if
			i32.const 1 local.set $restored_flag
			i32.const 268 local.get $restored_flag i32.store
		end
		local.get $restored_flag global.set $speed_mode
		i32.const 272 i32.load local.tee $restored_flag
		i32.const 59 i32.gt_u
		if
			i32.const 0 local.set $restored_flag
			i32.const 272 local.get $restored_flag i32.store
		end
		local.get $restored_flag global.set $playback_phase
		i32.const 0))
