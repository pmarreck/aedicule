;; Vibesteroids behavioral conversion for the gpui-frontplane-v0 ABI.
;;
;; Schema 4 stores every gameplay scalar as an integer. Spatial quantities use
;; signed decimal fixed point with SCALE = 1,000,000. IEEE-754 values exist
;; only at the host ABI boundary: viewport scalars enter through $from_host,
;; and completed draw scalars leave through $to_host. They never feed back.
;; Linear velocity is stored in logical pixels/second and angular velocity in
;; radians/second; only named integration helpers know the fixed tick rate.
;;
;; Canonical state occupies [1024, 17408):
;;   0 tick:i32          4 rng:i32            8 width:i64
;;  16 height:i64      24 ship.x:i64         32 ship.y:i64
;;  40 ship.vx:i64     48 ship.vy:i64        56 ship.dx:i64
;;  64 ship.dy:i64     72 score:i32          76 lives:i32
;;  80 level:i32       84 flags:i32          88 last-fire:i32
;;  92 invulnerable    96 next-life:i32     100 lifecycle:i32
;; 104 lifecycle-ticks 108 banner:i32       112 seed:i32
;; 120 blossom-rotation:i64
;;
;; Pools, relative to fp_state_ptr:
;;   256: 64 bullets x 48 bytes
;;        active:i32, pad:i32, x/y/vx/vy/distance-traveled:i64
;;  3328: 32 asteroids x 80 bytes
;;        active/generation/shape/pad:i32, x/y/vx/vy/radius/dx/dy:i64,
;;        spin:i32, points:i32
;;  5888: 150 particles x 48 bytes
;;        active/life:i32, x/y/vx/vy:i64, max-life:i32
;; 13088: 4 debris pieces x 80 bytes
;;        active/life:i32, x/y/vx/vy/dx/dy:i64, spin/piece:i32
;; 13408: 100 stars x 16 bytes, x/y:i64
(module
	(import "host.v0" "title" (func $title (param i32 i32) (result i32)))
	(import "host.v0" "menu_item" (func $menu_item (param i32 i32 i32 i32 i32) (result i32)))
	(import "host.v0" "frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
	(import "host.v0" "transform_push" (func $transform_push (param f32 f32 f32 f32 f32 f32) (result i32)))
	(import "host.v0" "transform_pop" (func $transform_pop (result i32)))
	(import "host.v0" "path_begin" (func $path_begin (param i32) (result i32)))
	(import "host.v0" "path_move" (func $path_move (param f32 f32) (result i32)))
	(import "host.v0" "path_line" (func $path_line (param f32 f32) (result i32)))
	(import "host.v0" "path_close" (func $path_close (result i32)))
	(import "host.v0" "path_end" (func $path_end (param f32 i32 i32 i32) (result i32)))
	(import "host.v0" "line" (func $line (param i32 f32 f32 f32 f32 f32 i32) (result i32)))
	(import "host.v0" "circle" (func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
	(import "host.v0" "text" (func $text (param i32 i32 i32 f32 f32 f32 i32 i32) (result i32)))
	(import "host.v0" "frame_end" (func $frame_end (result i32)))
	(import "host.v0" "audio" (func $audio (param i32 f32 f32 i32) (result i32)))
	(import "host.v0" "synth_voice"
		(func $synth_voice
			(param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
			(result i32)))
	(import "host.v0" "effect" (func $effect (param i32 i32 i32) (result i32)))

	(memory (export "memory") 1 64)
	(data (i32.const 0) "Vibesteroids \e2\80\94 WAT")
	(data (i32.const 32) "New Game")
	(data (i32.const 48) "Quit")
	(data (i32.const 64) "VIBESTEROIDS")
	(data (i32.const 80) "by Peter Marreck")
	(data (i32.const 100) "PAUSED")
	(data (i32.const 108) "GAME OVER")
	(data (i32.const 128) "SCORE")
	(data (i32.const 136) "LEVEL")
	(data (i32.const 144) "LIVES")
	(data (i32.const 160) "000000")
	(data (i32.const 168) "00")
	(data (i32.const 176) "WAVE")
	(data (i32.const 184) "SHIP DESTROYED")
	(data (i32.const 200) "Help / Controls")
	(data (i32.const 224) "CONTROLS")
	(data (i32.const 240) "LEFT / RIGHT   ROTATE")
	(data (i32.const 264) "UP             THRUST")
	(data (i32.const 288) "SPACE          FIRE")
	(data (i32.const 312) "F              AUTO-FIRE")
	(data (i32.const 340) "K              KID MODE")
	(data (i32.const 368) "B              DEATH BLOSSOM")
	(data (i32.const 400) "P / ESC        PAUSE")
	(data (i32.const 424) "R              RESTART")
	(data (i32.const 448) "F1 / H         HELP")
	(data (i32.const 472) "DEATH BLOSSOM!")
	(data (i32.const 488) "\f0\9f\94\86")

	(global $scale i64 (i64.const 1000000))
	(global $tick_hz i64 (i64.const 60))

	(func (export "fp_abi_major") (result i32) i32.const 0)
	(func (export "fp_abi_minor") (result i32) i32.const 0)
	(func (export "fp_state_ptr") (result i32) i32.const 1024)
	(func (export "fp_state_len") (result i32) i32.const 16384)
	(func (export "fp_state_schema") (result i32) i32.const 4)
	(func (export "fp_tick_hz") (result i32) global.get $tick_hz i32.wrap_i64)

	(func (export "fp_configure") (result i32)
		i32.const 0 i32.const 20 call $title drop
		i32.const 1 i32.const 32 i32.const 8 i32.const 1 i32.const 0 call $menu_item drop
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 i32.const 1 call $menu_item drop
		i32.const 7 i32.const 200 i32.const 15 i32.const 7 i32.const 0 call $menu_item drop
		i32.const 6 i32.const 48 i32.const 4 i32.const 6 i32.const 0 call $menu_item drop
		;; Shot: 800 -> 400 -> 200 Hz sine, 0.3 -> 0.01 gain.
		i32.const 1 i32.const 1 i32.const 0 i32.const 100
		i32.const 800000 i32.const 400000 i32.const 200000
		i32.const 300000 i32.const 300000 i32.const 10000
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $synth_voice drop
		;; Asteroid explosion: swept low-pass noise plus a restrained bass layer.
		i32.const 2 i32.const 3 i32.const 0 i32.const 500
		i32.const 0 i32.const 0 i32.const 0
		i32.const 500000 i32.const 500000 i32.const 10000
		i32.const 1 i32.const 1500000 i32.const 80000 i32.const 0 call $synth_voice drop
		i32.const 2 i32.const 1 i32.const 0 i32.const 500
		i32.const 120000 i32.const 120000 i32.const 80000
		i32.const 80000 i32.const 80000 i32.const 1000
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $synth_voice drop
		;; Ship explosion: white/brown noise layers and a short low impulse.
		i32.const 3 i32.const 3 i32.const 0 i32.const 1200
		i32.const 0 i32.const 0 i32.const 0
		i32.const 700000 i32.const 700000 i32.const 1000
		i32.const 1 i32.const 3000000 i32.const 100000 i32.const 0 call $synth_voice drop
		i32.const 3 i32.const 4 i32.const 0 i32.const 1200
		i32.const 0 i32.const 0 i32.const 0
		i32.const 300000 i32.const 300000 i32.const 1000
		i32.const 2 i32.const 300000 i32.const 300000 i32.const 0 call $synth_voice drop
		i32.const 3 i32.const 1 i32.const 0 i32.const 200
		i32.const 120000 i32.const 60000 i32.const 40000
		i32.const 500000 i32.const 500000 i32.const 0
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $synth_voice drop
		;; Thrust: rate-limited 60 Hz saw through a 200 Hz low-pass.
		i32.const 4 i32.const 2 i32.const 0 i32.const 50
		i32.const 60000 i32.const 60000 i32.const 60000
		i32.const 100000 i32.const 100000 i32.const 10000
		i32.const 1 i32.const 200000 i32.const 200000 i32.const 55 call $synth_voice drop
		;; Death Blossom: three scheduled 400 -> 800 -> 400 Hz whoops.
		i32.const 5 i32.const 1 i32.const 0 i32.const 300 i32.const 400000 i32.const 800000 i32.const 400000 i32.const 300000 i32.const 300000 i32.const 1000 i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $synth_voice drop
		i32.const 5 i32.const 1 i32.const 400 i32.const 300 i32.const 400000 i32.const 800000 i32.const 400000 i32.const 300000 i32.const 300000 i32.const 1000 i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $synth_voice drop
		i32.const 5 i32.const 1 i32.const 800 i32.const 300 i32.const 400000 i32.const 800000 i32.const 400000 i32.const 300000 i32.const 300000 i32.const 1000 i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $synth_voice drop
		;; Extra life: five delayed sawtooth chimes with note-relative filters.
		i32.const 6 i32.const 2 i32.const 0 i32.const 400 i32.const 523250 i32.const 523250 i32.const 523250 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 1569750 i32.const 1569750 i32.const 0 call $synth_voice drop
		i32.const 6 i32.const 2 i32.const 150 i32.const 400 i32.const 588656 i32.const 588656 i32.const 588656 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 1765968 i32.const 1765968 i32.const 0 call $synth_voice drop
		i32.const 6 i32.const 2 i32.const 300 i32.const 400 i32.const 654063 i32.const 654063 i32.const 654063 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 1962189 i32.const 1962189 i32.const 0 call $synth_voice drop
		i32.const 6 i32.const 2 i32.const 450 i32.const 400 i32.const 784875 i32.const 784875 i32.const 784875 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 2354625 i32.const 2354625 i32.const 0 call $synth_voice drop
		i32.const 6 i32.const 2 i32.const 600 i32.const 400 i32.const 1046500 i32.const 1046500 i32.const 1046500 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 3139500 i32.const 3139500 i32.const 0 call $synth_voice drop
		i32.const 0)

	;; FLOAT ADAPTER BEGIN
	(func $from_host (param $value f32) (result i64)
		local.get $value f32.const 1000000 f32.mul i64.trunc_sat_f32_s)
	(func $to_host (param $value i64) (result f32)
		local.get $value f32.convert_i64_s f32.const 1000000 f32.div)
	;; FLOAT ADAPTER END

	(func $fixed_mul (param $left i64) (param $right i64) (result i64)
		local.get $left local.get $right i64.mul global.get $scale i64.div_s)

	(func $per_tick (param $per_second i64) (result i64)
		local.get $per_second global.get $tick_hz i64.div_s)

	(func $ticks_from_sixty (param $ticks i32) (result i32)
		local.get $ticks i64.extend_i32_u global.get $tick_hz i64.mul
		i64.const 60 i64.div_u i32.wrap_i64)

	(func $fixed_abs (param $value i64) (result i64)
		local.get $value i64.const 0 i64.lt_s
		(if (result i64) (then i64.const 0 local.get $value i64.sub) (else local.get $value)))

	(func $difficulty_index (result i64)
		i32.const 1104 i32.load i32.const 1 i32.sub
		i32.const 0 i32.lt_s (if (then i64.const 0 return))
		i32.const 1104 i32.load i32.const 20 i32.ge_s (if (then i64.const 19 return))
		i32.const 1104 i32.load i32.const 1 i32.sub i64.extend_i32_s)

	(func $difficulty_value (param $base i64) (param $delta i64) (result i64)
		local.get $base local.get $delta call $difficulty_index i64.mul i64.const 19 i64.div_s i64.add)

	(func $ship_acceleration_per_second (result i64)
		i64.const 300000000 i64.const 200000000 call $difficulty_value)

	(func $rotation_per_second (result i64)
		i64.const 5000000 i64.const 5000000 call $difficulty_value)

	(func $bullet_speed_per_second (result i64)
		i64.const 337500000 i64.const 427500000 call $difficulty_value)

	(func $fire_interval_ticks (result i32)
		;; Source uses elapsed_ms > delay, so the first integral fixed tick after
		;; the delay is floor(delay * 60 / 1000) + 1.
		i64.const 250000000 i64.const -125000000 call $difficulty_value
		global.get $tick_hz i64.mul i64.const 1000000000 i64.div_s i32.wrap_i64 i32.const 1 i32.add)

	(func $small_sine (param $angle i64) (result i64)
		(local $square i64)
		local.get $angle local.get $angle call $fixed_mul local.set $square
		local.get $angle local.get $square local.get $angle call $fixed_mul i64.const 6 i64.div_s i64.sub)

	(func $small_cosine (param $angle i64) (result i64)
		(local $square i64)
		local.get $angle local.get $angle call $fixed_mul local.set $square
		global.get $scale local.get $square i64.const 2 i64.div_s i64.sub
		local.get $square local.get $square call $fixed_mul i64.const 24 i64.div_s i64.add)

	(func $integer_sqrt (param $value i64) (result i64)
		(local $estimate i64) (local $next i64)
		local.get $value i64.const 0 i64.le_s (if (then i64.const 0 return))
		local.get $value local.set $estimate
		local.get $value i64.const 1 i64.add i64.const 2 i64.div_u local.set $next
		(block $done
			(loop $refine
				local.get $next local.get $estimate i64.ge_u br_if $done
				local.get $next local.set $estimate
				local.get $estimate local.get $value local.get $estimate i64.div_u i64.add
				i64.const 2 i64.div_u local.set $next
				br $refine))
		local.get $estimate)

	(func $fixed_hypot (param $x i64) (param $y i64) (result i64)
		(local $milli_x i64) (local $milli_y i64)
		local.get $x i64.const 1000 i64.div_s local.set $milli_x
		local.get $y i64.const 1000 i64.div_s local.set $milli_y
		local.get $milli_x local.get $milli_x i64.mul
		local.get $milli_y local.get $milli_y i64.mul i64.add
		call $integer_sqrt i64.const 1000 i64.mul)

	(func $bullet_address (param $index i32) (result i32)
		i32.const 1280 local.get $index i32.const 48 i32.mul i32.add)
	(func $asteroid_address (param $index i32) (result i32)
		i32.const 4352 local.get $index i32.const 80 i32.mul i32.add)
	(func $particle_address (param $index i32) (result i32)
		i32.const 6912 local.get $index i32.const 48 i32.mul i32.add)
	(func $debris_address (param $index i32) (result i32)
		i32.const 14112 local.get $index i32.const 80 i32.mul i32.add)
	(func $star_address (param $index i32) (result i32)
		i32.const 14432 local.get $index i32.const 16 i32.mul i32.add)

	;; Mulberry32 keeps gameplay randomness deterministic and render-independent.
	(func $rand_u32 (result i32)
		(local $state i32) (local $t i32)
		i32.const 1028 i32.load i32.const 0x6d2b79f5 i32.add local.set $state
		i32.const 1028 local.get $state i32.store
		local.get $state local.set $t
		local.get $t local.get $t i32.const 15 i32.shr_u i32.xor
		local.get $t i32.const 1 i32.or i32.mul local.set $t
		local.get $t
		local.get $t local.get $t local.get $t i32.const 7 i32.shr_u i32.xor
		local.get $t i32.const 61 i32.or i32.mul i32.add i32.xor local.set $t
		local.get $t local.get $t i32.const 14 i32.shr_u i32.xor)

	(func $rand_unit (result i64)
		call $rand_u32 i32.const 0xffff i32.and i64.extend_i32_u
		global.get $scale i64.mul i64.const 65536 i64.div_u)
	(func $rand_signed (result i64)
		call $rand_unit i64.const 2 i64.mul global.get $scale i64.sub)

	(func $ensure_component (param $value i64) (result i64)
		(local $adjusted i64) (local $cap i64)
		local.get $value local.set $adjusted
		local.get $value call $fixed_abs i64.const 15000000 i64.lt_u
		(if (then
			local.get $value i64.const 0 i64.lt_s
			(if (then i64.const -15000000 local.set $adjusted)
				(else local.get $value i64.eqz
					(if (then call $rand_u32 i32.const 1 i32.and
						(if (then i64.const -15000000 local.set $adjusted)
							(else i64.const 15000000 local.set $adjusted)))
						(else i64.const 15000000 local.set $adjusted))))))
		i32.const 1104 i32.load i32.const 20 i32.ge_s (if (then local.get $adjusted return))
		i64.const 60000000 call $difficulty_index i64.const 7500000 i64.mul i64.add local.set $cap
		local.get $adjusted local.get $cap i64.gt_s (if (then local.get $cap return))
		local.get $adjusted i64.const 0 local.get $cap i64.sub i64.lt_s
		(if (then i64.const 0 local.get $cap i64.sub return))
		local.get $adjusted)

	(func $store_digit (param $address i32) (param $value i32) (param $divisor i32)
		local.get $address local.get $value local.get $divisor i32.div_u
		i32.const 10 i32.rem_u i32.const 48 i32.add i32.store8)
	(func $write_six_digits (param $address i32) (param $value i32)
		local.get $address local.get $value i32.const 100000 call $store_digit
		local.get $address i32.const 1 i32.add local.get $value i32.const 10000 call $store_digit
		local.get $address i32.const 2 i32.add local.get $value i32.const 1000 call $store_digit
		local.get $address i32.const 3 i32.add local.get $value i32.const 100 call $store_digit
		local.get $address i32.const 4 i32.add local.get $value i32.const 10 call $store_digit
		local.get $address i32.const 5 i32.add local.get $value i32.const 1 call $store_digit)
	(func $write_two_digits (param $address i32) (param $value i32)
		local.get $address local.get $value i32.const 10 call $store_digit
		local.get $address i32.const 1 i32.add local.get $value i32.const 1 call $store_digit)

	(func $splash_alpha (result i32)
		(local $remaining i32) (local $elapsed i32) (local $one_second i32) (local $three_seconds i32)
		i32.const 1132 i32.load local.set $remaining
		i32.const 60 call $ticks_from_sixty local.set $one_second
		i32.const 180 call $ticks_from_sixty local.set $three_seconds
		i32.const 240 call $ticks_from_sixty local.get $remaining i32.sub local.set $elapsed
		local.get $elapsed local.get $one_second i32.lt_u
		(if (result i32)
			(then local.get $elapsed i32.const 1 i32.add i32.const 255 i32.mul local.get $one_second i32.div_u)
			(else local.get $elapsed local.get $three_seconds i32.lt_u
				(if (result i32)
					(then i32.const 255)
					(else local.get $remaining i32.const 255 i32.mul local.get $one_second i32.div_u)))))

	(func $wrap (param $value i64) (param $minimum i64) (param $maximum i64) (result i64)
		local.get $value local.get $minimum i64.lt_s
		(if (result i64)
			(then local.get $maximum)
			(else local.get $value local.get $maximum i64.gt_s
				(if (result i64) (then local.get $minimum) (else local.get $value)))))

	(func $regenerate_stars
		(local $index i32) (local $address i32)
		(block $done
			(loop $again
				local.get $index i32.const 100 i32.ge_u br_if $done
				local.get $index call $star_address local.set $address
				local.get $address
				local.get $index i32.const 83 i32.mul i32.const 47 i32.add i32.const 1000 i32.rem_u
				i64.extend_i32_u i32.const 1032 i64.load i64.mul i64.const 1000 i64.div_u i64.store
				local.get $address i32.const 8 i32.add
				local.get $index i32.const 47 i32.mul i32.const 29 i32.add i32.const 1000 i32.rem_u
				i64.extend_i32_u i32.const 1040 i64.load i64.mul i64.const 1000 i64.div_u i64.store
				local.get $index i32.const 1 i32.add local.set $index br $again)))

	(func $spawn_asteroid_at (param $address i32) (param $x i64) (param $y i64)
		(param $radius i64) (param $generation i32)
		local.get $address i32.const 1 i32.store
		local.get $address i32.const 4 i32.add local.get $generation i32.store
		local.get $address i32.const 8 i32.add call $rand_u32 i32.store
		local.get $address i32.const 16 i32.add local.get $x i64.store
		local.get $address i32.const 24 i32.add local.get $y i64.store
		local.get $address i32.const 32 i32.add
		call $rand_signed i64.const 50000000 call $fixed_mul
		call $ensure_component i64.store
		local.get $address i32.const 40 i32.add
		call $rand_signed i64.const 50000000 call $fixed_mul
		call $ensure_component i64.store
		local.get $address i32.const 48 i32.add local.get $radius i64.store
		local.get $address i32.const 56 i32.add i64.const 1000000 i64.store
		local.get $address i32.const 64 i32.add i64.const 0 i64.store
		local.get $address i32.const 72 i32.add
		call $rand_signed i32.wrap_i64 i32.store
		local.get $address i32.const 76 i32.add
		call $rand_u32 i32.const 5 i32.rem_u i32.const 8 i32.add i32.store)

	(func $spawn_wave
		(local $index i32) (local $count i32) (local $address i32)
		(local $edge i32) (local $x i64) (local $y i64) (local $radius i64)
		i32.const 1104 i32.load i32.const 4 i32.add local.set $count
		local.get $count i32.const 32 i32.gt_u (if (then i32.const 32 local.set $count))
		(block $done
			(loop $again
				local.get $index local.get $count i32.ge_u br_if $done
				local.get $index call $asteroid_address local.set $address
				call $rand_u32 i32.const 4 i32.rem_u local.set $edge
				call $rand_unit i32.const 1032 i64.load call $fixed_mul local.set $x
				call $rand_unit i32.const 1040 i64.load call $fixed_mul local.set $y
				local.get $edge i32.eqz (if (then i64.const 100000000 local.set $y))
				local.get $edge i32.const 1 i32.eq
				(if (then i32.const 1032 i64.load i64.const 100000000 i64.sub local.set $x))
				local.get $edge i32.const 2 i32.eq
				(if (then i32.const 1040 i64.load i64.const 100000000 i64.sub local.set $y))
				local.get $edge i32.const 3 i32.eq (if (then i64.const 100000000 local.set $x))
				call $rand_unit i64.const 30000000 call $fixed_mul i64.const 20000000 i64.add local.set $radius
				local.get $address local.get $x local.get $y local.get $radius i32.const 0
				call $spawn_asteroid_at
				local.get $index i32.const 1 i32.add local.set $index br $again))
	)

	(func $reset (param $seed i32) (param $width i64) (param $height i64)
		(local $normalized_seed i32)
		local.get $seed local.set $normalized_seed
		local.get $normalized_seed i32.eqz (if (then i32.const 1 local.set $normalized_seed))
		i32.const 1024 i32.const 0 i32.const 16384 memory.fill
		i32.const 1028 local.get $normalized_seed i32.store
		i32.const 1032 local.get $width i64.store
		i32.const 1040 local.get $height i64.store
		i32.const 1048 local.get $width i64.const 2 i64.div_s i64.store
		i32.const 1056 local.get $height i64.const 2 i64.div_s i64.store
		i32.const 1064 i64.const 0 i64.store
		i32.const 1072 i64.const 0 i64.store
		i32.const 1080 i64.const 0 i64.store
		i32.const 1088 i64.const -1000000 i64.store
		i32.const 1096 i32.const 0 i32.store
		i32.const 1100 i32.const 3 i32.store
		i32.const 1104 i32.const 1 i32.store
		i32.const 1108 i32.const 256 i32.store
		i32.const 1112 i32.const -100 i32.store
		i32.const 1116 i32.const 120 call $ticks_from_sixty i32.store
		i32.const 1120 i32.const 20000 i32.store
		i32.const 1124 i32.const 0 i32.store
		i32.const 1128 i32.const 0 i32.store
		i32.const 1136 local.get $normalized_seed i32.store
		i32.const 1144 i64.const 0 i64.store
		i32.const 1132 i32.const 240 call $ticks_from_sixty i32.store
		call $regenerate_stars
		call $spawn_wave)

	(func (export "fp_init") (param $seed_low i32) (param $seed_high i32)
		(param $width f32) (param $height f32) (result i32)
		local.get $seed_low local.get $seed_high i32.xor
		local.get $width call $from_host local.get $height call $from_host call $reset
		i32.const 0)

	(func $asteroid_count (result i32)
		(local $index i32) (local $count i32)
		(block $done (loop $again
			local.get $index i32.const 32 i32.ge_u br_if $done
			local.get $index call $asteroid_address i32.load
			(if (then local.get $count i32.const 1 i32.add local.set $count))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		local.get $count)

	(func $find_free_bullet (result i32)
		(local $index i32) (local $address i32)
		(block $none
			(loop $again
				local.get $index i32.const 64 i32.ge_u br_if $none
				local.get $index call $bullet_address local.set $address
				local.get $address i32.load i32.eqz
				(if (then local.get $address return))
				local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	(func $fire
		(local $address i32) (local $speed i64) (local $interval i32)
		call $fire_interval_ticks local.set $interval
		i32.const 1108 i32.load i32.const 128 i32.and
		(if (then i32.const 2 local.set $interval))
		i32.const 1124 i32.load i32.eqz
		i32.const 1108 i32.load i32.const 16 i32.and i32.eqz i32.and
		i32.const 1024 i32.load i32.const 1112 i32.load i32.sub local.get $interval i32.ge_s i32.and
		(if
			(then
				call $find_free_bullet local.tee $address
				(if
					(then
						i32.const 1108 i32.load i32.const 128 i32.and
						(if (result i64)
							(then i64.const 450000000)
							(else call $bullet_speed_per_second))
						local.set $speed
						local.get $address i32.const 1 i32.store
						local.get $address i32.const 8 i32.add
						i32.const 1048 i64.load i32.const 1080 i64.load i64.const 20000000 call $fixed_mul i64.add i64.store
						local.get $address i32.const 16 i32.add
						i32.const 1056 i64.load i32.const 1088 i64.load i64.const 20000000 call $fixed_mul i64.add i64.store
						local.get $address i32.const 24 i32.add
						i32.const 1064 i64.load i32.const 1080 i64.load local.get $speed call $fixed_mul i64.add i64.store
						local.get $address i32.const 32 i32.add
						i32.const 1072 i64.load i32.const 1088 i64.load local.get $speed call $fixed_mul i64.add i64.store
						local.get $address i32.const 40 i32.add i64.const 0 i64.store
						i32.const 1112 i32.const 1024 i32.load i32.store
						i32.const 1 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))))

	(func $spawn_particles (param $x i64) (param $y i64) (param $count i32)
		(param $inherit_vx i64) (param $inherit_vy i64)
		(local $index i32) (local $created i32) (local $address i32)
		(local $sector i32) (local $ux i64) (local $uy i64) (local $speed i64)
		(block $done
			(loop $again
				local.get $index i32.const 150 i32.ge_u br_if $done
				local.get $created local.get $count i32.ge_u br_if $done
				local.get $index call $particle_address local.set $address
				local.get $address i32.load i32.eqz
				(if
					(then
						local.get $created i32.const 7 i32.and local.set $sector
						i64.const 0 local.set $ux i64.const 0 local.set $uy
						local.get $sector i32.eqz (if (then i64.const 1000000 local.set $ux))
						local.get $sector i32.const 1 i32.eq (if (then i64.const 707107 local.set $ux i64.const 707107 local.set $uy))
						local.get $sector i32.const 2 i32.eq (if (then i64.const 1000000 local.set $uy))
						local.get $sector i32.const 3 i32.eq (if (then i64.const -707107 local.set $ux i64.const 707107 local.set $uy))
						local.get $sector i32.const 4 i32.eq (if (then i64.const -1000000 local.set $ux))
						local.get $sector i32.const 5 i32.eq (if (then i64.const -707107 local.set $ux i64.const -707107 local.set $uy))
						local.get $sector i32.const 6 i32.eq (if (then i64.const -1000000 local.set $uy))
						local.get $sector i32.const 7 i32.eq (if (then i64.const 707107 local.set $ux i64.const -707107 local.set $uy))
						call $rand_unit i64.const 132000000 call $fixed_mul i64.const 72000000 i64.add local.set $speed
						local.get $address i32.const 1 i32.store
						local.get $address i32.const 4 i32.add i32.const 40 call $ticks_from_sixty i32.store
						local.get $address i32.const 8 i32.add local.get $x i64.store
						local.get $address i32.const 16 i32.add local.get $y i64.store
						local.get $address i32.const 24 i32.add
						local.get $inherit_vx local.get $ux local.get $speed call $fixed_mul i64.add i64.store
						local.get $address i32.const 32 i32.add
						local.get $inherit_vy local.get $uy local.get $speed call $fixed_mul i64.add i64.store
						local.get $address i32.const 40 i32.add i32.const 40 call $ticks_from_sixty i32.store
						local.get $created i32.const 1 i32.add local.set $created))
				local.get $index i32.const 1 i32.add local.set $index br $again)))

	(func $spawn_debris (param $x i64) (param $y i64) (param $vx i64) (param $vy i64)
		(local $index i32) (local $address i32)
		(block $done (loop $again
			local.get $index i32.const 4 i32.ge_u br_if $done
			local.get $index call $debris_address local.set $address
			local.get $address i32.const 1 i32.store
			local.get $address i32.const 4 i32.add i32.const 120 call $ticks_from_sixty i32.store
			local.get $address i32.const 8 i32.add local.get $x i64.store
			local.get $address i32.const 16 i32.add local.get $y i64.store
			local.get $address i32.const 24 i32.add
			local.get $vx
			local.get $index i32.const 2 i32.lt_u (if (result i64) (then i64.const 108000000) (else i64.const -90000000))
			i64.add i64.store
			local.get $address i32.const 32 i32.add
			local.get $vy
			local.get $index i32.const 1 i32.and (if (result i64) (then i64.const 102000000) (else i64.const -96000000))
			i64.add i64.store
			local.get $address i32.const 40 i32.add i64.const 1000000 i64.store
			local.get $address i32.const 48 i32.add i64.const 0 i64.store
			local.get $address i32.const 56 i32.add
			local.get $index i32.const 1 i32.and (if (result i32) (then i32.const 1) (else i32.const -1)) i32.store
			local.get $address i32.const 60 i32.add local.get $index i32.store
			local.get $index i32.const 1 i32.add local.set $index br $again)))

	(func $distance_lt (param $x1 i64) (param $y1 i64) (param $x2 i64) (param $y2 i64)
		(param $radius i64) (result i32)
		(local $dx i64) (local $dy i64) (local $r i64)
		;; Divide microunits to milli-fixed before squaring. At an 8K viewport,
		;; each squared term stays below 2^63 while retaining 0.001-pixel detail.
		local.get $x1 local.get $x2 i64.sub i64.const 1000 i64.div_s local.set $dx
		local.get $y1 local.get $y2 i64.sub i64.const 1000 i64.div_s local.set $dy
		local.get $radius i64.const 1000 i64.div_s local.set $r
		local.get $dx local.get $dx i64.mul
		local.get $dy local.get $dy i64.mul i64.add
		local.get $r local.get $r i64.mul i64.lt_s)

	(func $find_free_asteroid (result i32)
		(local $index i32) (local $address i32)
		(block $none (loop $again
			local.get $index i32.const 32 i32.ge_u br_if $none
			local.get $index call $asteroid_address local.set $address
			local.get $address i32.load i32.eqz (if (then local.get $address return))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	(func $hit_asteroid (param $address i32) (param $impulse_x i64) (param $impulse_y i64)
		(param $score_hit i32)
		(local $x i64) (local $y i64) (local $vx i64) (local $vy i64)
		(local $radius i64) (local $child_radius i64) (local $half_boost i64)
		(local $child_x i64) (local $child_y i64)
		(local $generation i32) (local $free i32) (local $points i32)
		local.get $address i32.const 16 i32.add i64.load local.set $x
		local.get $address i32.const 24 i32.add i64.load local.set $y
		local.get $address i32.const 32 i32.add i64.load local.set $vx
		local.get $address i32.const 40 i32.add i64.load local.set $vy
		local.get $address i32.const 48 i32.add i64.load local.set $radius
		local.get $x local.get $y i32.const 20 i64.const 0 i64.const 0 call $spawn_particles
		i32.const 2 f32.const 1 f32.const 1 i32.const 0 call $audio drop
		local.get $radius i64.const 20000000 i64.gt_s
		(if
			(then
				local.get $radius i64.const 600000 call $fixed_mul local.set $child_radius
				local.get $address i32.const 4 i32.add i32.load i32.const 1 i32.add local.set $generation
				i64.const 4800000000 global.get $scale i64.mul local.get $child_radius i64.div_s
				i64.const 2 i64.div_s local.set $half_boost
				local.get $x call $rand_signed i64.const 20000000 call $fixed_mul i64.add local.set $child_x
				local.get $y call $rand_signed i64.const 20000000 call $fixed_mul i64.add local.set $child_y
				local.get $address local.get $child_x local.get $child_y local.get $child_radius local.get $generation
				call $spawn_asteroid_at
				local.get $address i32.const 32 i32.add
				local.get $vx local.get $impulse_x i64.add call $rand_signed local.get $half_boost call $fixed_mul i64.add call $ensure_component i64.store
				local.get $address i32.const 40 i32.add
				local.get $vy local.get $impulse_y i64.add call $rand_signed local.get $half_boost call $fixed_mul i64.add call $ensure_component i64.store
				local.get $address i32.const 72 i32.add call $rand_signed i64.const 2 i64.mul i32.wrap_i64 i32.store
				call $find_free_asteroid local.tee $free
				(if
					(then
						local.get $x call $rand_signed i64.const 20000000 call $fixed_mul i64.add local.set $child_x
						local.get $y call $rand_signed i64.const 20000000 call $fixed_mul i64.add local.set $child_y
						local.get $free local.get $child_x local.get $child_y local.get $child_radius local.get $generation
						call $spawn_asteroid_at
						local.get $free i32.const 32 i32.add
						local.get $vx local.get $impulse_x i64.add call $rand_signed local.get $half_boost call $fixed_mul i64.add call $ensure_component i64.store
						local.get $free i32.const 40 i32.add
						local.get $vy local.get $impulse_y i64.add call $rand_signed local.get $half_boost call $fixed_mul i64.add call $ensure_component i64.store
						local.get $free i32.const 72 i32.add call $rand_signed i64.const 2 i64.mul i32.wrap_i64 i32.store))
				i32.const 80 local.set $points)
			(else local.get $address i32.const 0 i32.store i32.const 120 local.set $points))
		local.get $score_hit i32.const 1108 i32.load i32.const 64 i32.and i32.eqz i32.and
		(if
			(then
				i32.const 1096 i32.const 1096 i32.load local.get $points i32.add i32.store
				i32.const 1096 i32.load i32.const 1120 i32.load i32.ge_u
				(if (then
					i32.const 1100 i32.const 1100 i32.load i32.const 1 i32.add i32.store
					i32.const 1120 i32.const 1120 i32.load i32.const 20000 i32.add i32.store
					i32.const 6 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))))

	(func $check_bullet_collisions
		(local $bullet_index i32) (local $asteroid_index i32)
		(local $bullet i32) (local $asteroid i32) (local $factor i64)
		(local $impulse_x i64) (local $impulse_y i64)
		(block $bullets_done (loop $next_bullet
			local.get $bullet_index i32.const 64 i32.ge_u br_if $bullets_done
			local.get $bullet_index call $bullet_address local.set $bullet
			local.get $bullet i32.load
			(if
				(then
					i32.const 0 local.set $asteroid_index
					(block $asteroids_done (loop $next_asteroid
						local.get $asteroid_index i32.const 32 i32.ge_u br_if $asteroids_done
						local.get $asteroid_index call $asteroid_address local.set $asteroid
						local.get $asteroid i32.load
						(if
							(then
								local.get $bullet i32.const 8 i32.add i64.load
								local.get $bullet i32.const 16 i32.add i64.load
								local.get $asteroid i32.const 16 i32.add i64.load
								local.get $asteroid i32.const 24 i32.add i64.load
								local.get $asteroid i32.const 48 i32.add i64.load i64.const 5000000 i64.add
								call $distance_lt
								(if
									(then
										local.get $bullet i32.const 0 i32.store
						i64.const 3600000 global.get $scale i64.mul
						local.get $asteroid i32.const 48 i32.add i64.load i64.div_s local.set $factor
										local.get $bullet i32.const 24 i32.add i64.load local.get $factor call $fixed_mul local.set $impulse_x
										local.get $bullet i32.const 32 i32.add i64.load local.get $factor call $fixed_mul local.set $impulse_y
										local.get $asteroid local.get $impulse_x local.get $impulse_y i32.const 1 call $hit_asteroid
										br $asteroids_done))))
						local.get $asteroid_index i32.const 1 i32.add local.set $asteroid_index br $next_asteroid))))
			local.get $bullet_index i32.const 1 i32.add local.set $bullet_index br $next_bullet)))

	(func $find_ship_collision (result i32)
		(local $index i32) (local $address i32)
		(block $none (loop $again
			local.get $index i32.const 32 i32.ge_u br_if $none
			local.get $index call $asteroid_address local.set $address
			local.get $address i32.load
			(if (then
				i32.const 1048 i64.load i32.const 1056 i64.load
				local.get $address i32.const 16 i32.add i64.load local.get $address i32.const 24 i32.add i64.load
				local.get $address i32.const 48 i32.add i64.load i64.const 10000000 i64.add call $distance_lt
				(if (then local.get $address return))))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	(func $begin_ship_explosion (param $asteroid i32)
		i32.const 1048 i64.load i32.const 1056 i64.load i32.const 40
		i32.const 1064 i64.load i32.const 1072 i64.load call $spawn_particles
		i32.const 1048 i64.load i32.const 1056 i64.load i32.const 1064 i64.load i32.const 1072 i64.load call $spawn_debris
		local.get $asteroid i64.const 0 i64.const 0 i32.const 0 call $hit_asteroid
		i32.const 3 f32.const 1 f32.const 1 i32.const 0 call $audio drop
		i32.const 1108 i32.load i32.const 64 i32.and i32.eqz
		(if (then i32.const 1100 i32.const 1100 i32.load i32.const 1 i32.sub i32.store))
		i32.const 1124 i32.const 1 i32.store
		i32.const 1128 i32.const 120 call $ticks_from_sixty i32.store
		i32.const 1108 i32.const 1108 i32.load i32.const -129 i32.and i32.store)

	(func $rotate_ship
		(local $dx i64) (local $dy i64) (local $next_dx i64) (local $next_dy i64)
		(local $angle i64) (local $sine i64) (local $cosine i64)
		i32.const 1080 i64.load local.set $dx i32.const 1088 i64.load local.set $dy
		call $rotation_per_second call $per_tick local.tee $angle call $small_sine local.set $sine
		local.get $angle call $small_cosine local.set $cosine
		i32.const 1108 i32.load i32.const 2 i32.and
		(if (then
			local.get $dx local.get $cosine call $fixed_mul local.get $dy local.get $sine call $fixed_mul i64.sub local.set $next_dx
			local.get $dy local.get $cosine call $fixed_mul local.get $dx local.get $sine call $fixed_mul i64.add local.set $next_dy
			local.get $next_dx local.set $dx local.get $next_dy local.set $dy))
		i32.const 1108 i32.load i32.const 1 i32.and
		(if (then
			local.get $dx local.get $cosine call $fixed_mul local.get $dy local.get $sine call $fixed_mul i64.add local.set $next_dx
			local.get $dy local.get $cosine call $fixed_mul local.get $dx local.get $sine call $fixed_mul i64.sub local.set $next_dy
			local.get $next_dx local.set $dx local.get $next_dy local.set $dy))
		i32.const 1080 local.get $dx i64.store i32.const 1088 local.get $dy i64.store)

	(func $activate_death_blossom
		(local $flags i32)
		i32.const 1108 i32.load local.set $flags
		i32.const 1124 i32.load i32.eqz
		local.get $flags i32.const 656 i32.and i32.eqz i32.and
		local.get $flags i32.const 256 i32.and i32.eqz i32.eqz i32.and
		(if (then
			i32.const 1108 local.get $flags i32.const 128 i32.or i32.const -257 i32.and i32.store
			i32.const 1144 i64.const 0 i64.store
			i32.const 5 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))

	(func $update_death_blossom
		(local $dx i64) (local $dy i64) (local $next_dx i64) (local $next_dy i64)
		(local $angle i64) (local $sine i64) (local $cosine i64) (local $rotation i64)
		i32.const 1080 i64.load local.set $dx i32.const 1088 i64.load local.set $dy
		i64.const 7200000 call $per_tick local.tee $angle call $small_sine local.set $sine
		local.get $angle call $small_cosine local.set $cosine
		local.get $dx local.get $cosine call $fixed_mul local.get $dy local.get $sine call $fixed_mul i64.add local.set $next_dx
		local.get $dy local.get $cosine call $fixed_mul local.get $dx local.get $sine call $fixed_mul i64.sub local.set $next_dy
		i32.const 1080 local.get $next_dx i64.store i32.const 1088 local.get $next_dy i64.store
		i32.const 1144 i64.load local.get $angle i64.add local.set $rotation
		local.get $rotation i64.const 75398224 i64.ge_s
		(if
			(then
				i32.const 1144 i64.const 75398224 i64.store
				call $fire
				i32.const 1108 i32.const 1108 i32.load i32.const -129 i32.and i32.store)
			(else
				i32.const 1144 local.get $rotation i64.store
				call $fire)))

	(func $update_ship
		(local $flags i32)
		i32.const 1108 i32.load local.set $flags
		local.get $flags i32.const 128 i32.and
		(if
			(then call $update_death_blossom)
			(else
				call $rotate_ship
				local.get $flags i32.const 4 i32.and
				(if (then
					i32.const 1064 i32.const 1064 i64.load i32.const 1080 i64.load call $ship_acceleration_per_second call $per_tick call $fixed_mul i64.add i64.store
					i32.const 1072 i32.const 1072 i64.load i32.const 1088 i64.load call $ship_acceleration_per_second call $per_tick call $fixed_mul i64.add i64.store
					i32.const 1024 i32.load i32.const 3 i32.and i32.eqz
					(if (then i32.const 4 f32.const 0.18 f32.const 1 i32.const 0 call $audio drop))))
				local.get $flags i32.const 8 i32.and local.get $flags i32.const 32 i32.and i32.or
				(if (then call $fire))))
		i32.const 1064 i32.const 1064 i64.load i64.const 995000 call $fixed_mul i64.store
		i32.const 1072 i32.const 1072 i64.load i64.const 995000 call $fixed_mul i64.store
		i32.const 1048
		i32.const 1048 i64.load i32.const 1064 i64.load call $per_tick i64.add i64.const 0 i32.const 1032 i64.load call $wrap i64.store
		i32.const 1056
		i32.const 1056 i64.load i32.const 1072 i64.load call $per_tick i64.add i64.const 0 i32.const 1040 i64.load call $wrap i64.store)

	(func $update_bullets
		(local $index i32) (local $address i32) (local $vx i64) (local $vy i64)
		(block $done (loop $again
			local.get $index i32.const 64 i32.ge_u br_if $done
			local.get $index call $bullet_address local.set $address
			local.get $address i32.load
			(if (then
				local.get $address i32.const 24 i32.add i64.load local.set $vx
				local.get $address i32.const 32 i32.add i64.load local.set $vy
				local.get $address i32.const 8 i32.add
				local.get $address i32.const 8 i32.add i64.load local.get $vx call $per_tick i64.add
				i64.const -25000000 i32.const 1032 i64.load i64.const 25000000 i64.add call $wrap i64.store
				local.get $address i32.const 16 i32.add
				local.get $address i32.const 16 i32.add i64.load local.get $vy call $per_tick i64.add
				i64.const -25000000 i32.const 1040 i64.load i64.const 25000000 i64.add call $wrap i64.store
				local.get $address i32.const 40 i32.add
				local.get $address i32.const 40 i32.add i64.load local.get $vx local.get $vy call $fixed_hypot call $per_tick i64.add i64.store
				local.get $address i32.const 40 i32.add i64.load
				i32.const 1032 i64.load i32.const 1040 i64.load call $fixed_hypot i64.const 2 i64.div_u i64.ge_u
				(if (then local.get $address i32.const 0 i32.store))))
			local.get $index i32.const 1 i32.add local.set $index br $again)))

	(func $update_particles
		(local $index i32) (local $address i32)
		(block $done (loop $again
			local.get $index i32.const 150 i32.ge_u br_if $done
			local.get $index call $particle_address local.set $address
			local.get $address i32.load
			(if (then
				local.get $address i32.const 8 i32.add local.get $address i32.const 8 i32.add i64.load local.get $address i32.const 24 i32.add i64.load call $per_tick i64.add i64.store
				local.get $address i32.const 16 i32.add local.get $address i32.const 16 i32.add i64.load local.get $address i32.const 32 i32.add i64.load call $per_tick i64.add i64.store
				local.get $address i32.const 24 i32.add local.get $address i32.const 24 i32.add i64.load i64.const 980000 call $fixed_mul i64.store
				local.get $address i32.const 32 i32.add local.get $address i32.const 32 i32.add i64.load i64.const 980000 call $fixed_mul i64.store
				local.get $address i32.const 4 i32.add local.get $address i32.const 4 i32.add i32.load i32.const 1 i32.sub i32.store
				local.get $address i32.const 4 i32.add i32.load i32.const 0 i32.le_s (if (then local.get $address i32.const 0 i32.store))))
			local.get $index i32.const 1 i32.add local.set $index br $again)))

	(func $update_asteroids
		(local $index i32) (local $address i32) (local $dx i64) (local $dy i64)
		(local $next_dx i64) (local $next_dy i64) (local $angle i64)
		(local $sine i64) (local $cosine i64)
		(block $done (loop $again
			local.get $index i32.const 32 i32.ge_u br_if $done
			local.get $index call $asteroid_address local.set $address
			local.get $address i32.load
			(if (then
				local.get $address i32.const 16 i32.add
				local.get $address i32.const 16 i32.add i64.load local.get $address i32.const 32 i32.add i64.load call $per_tick i64.add
				i64.const -50000000 i32.const 1032 i64.load i64.const 50000000 i64.add call $wrap i64.store
				local.get $address i32.const 24 i32.add
				local.get $address i32.const 24 i32.add i64.load local.get $address i32.const 40 i32.add i64.load call $per_tick i64.add
				i64.const -50000000 i32.const 1040 i64.load i64.const 50000000 i64.add call $wrap i64.store
				local.get $address i32.const 56 i32.add i64.load local.set $dx
				local.get $address i32.const 64 i32.add i64.load local.set $dy
				local.get $address i32.const 72 i32.add i32.load i64.extend_i32_s call $per_tick local.tee $angle call $small_sine local.set $sine
				local.get $angle call $small_cosine local.set $cosine
				local.get $dx local.get $cosine call $fixed_mul local.get $dy local.get $sine call $fixed_mul i64.sub local.set $next_dx
				local.get $dy local.get $cosine call $fixed_mul local.get $dx local.get $sine call $fixed_mul i64.add local.set $next_dy
				local.get $address i32.const 56 i32.add local.get $next_dx i64.store
				local.get $address i32.const 64 i32.add local.get $next_dy i64.store))
			local.get $index i32.const 1 i32.add local.set $index br $again)))

	(func $update_debris
		(local $index i32) (local $address i32) (local $dx i64) (local $dy i64)
		(local $next_dx i64) (local $next_dy i64)
		(block $done (loop $again
			local.get $index i32.const 4 i32.ge_u br_if $done
			local.get $index call $debris_address local.set $address
			local.get $address i32.load
			(if (then
				local.get $address i32.const 8 i32.add local.get $address i32.const 8 i32.add i64.load local.get $address i32.const 24 i32.add i64.load call $per_tick i64.add i64.store
				local.get $address i32.const 16 i32.add local.get $address i32.const 16 i32.add i64.load local.get $address i32.const 32 i32.add i64.load call $per_tick i64.add i64.store
				local.get $address i32.const 24 i32.add local.get $address i32.const 24 i32.add i64.load i64.const 990000 call $fixed_mul i64.store
				local.get $address i32.const 32 i32.add local.get $address i32.const 32 i32.add i64.load i64.const 990000 call $fixed_mul i64.store
				local.get $address i32.const 40 i32.add i64.load local.set $dx
				local.get $address i32.const 48 i32.add i64.load local.set $dy
				local.get $address i32.const 56 i32.add i32.load i32.const 0 i32.gt_s
				(if
					(then
						local.get $dx i64.const 720000 call $per_tick call $small_cosine call $fixed_mul local.get $dy i64.const 720000 call $per_tick call $small_sine call $fixed_mul i64.sub local.set $next_dx
						local.get $dx i64.const 720000 call $per_tick call $small_sine call $fixed_mul local.get $dy i64.const 720000 call $per_tick call $small_cosine call $fixed_mul i64.add local.set $next_dy)
					(else
						local.get $dx i64.const 720000 call $per_tick call $small_cosine call $fixed_mul local.get $dy i64.const 720000 call $per_tick call $small_sine call $fixed_mul i64.add local.set $next_dx
						local.get $dy i64.const 720000 call $per_tick call $small_cosine call $fixed_mul local.get $dx i64.const 720000 call $per_tick call $small_sine call $fixed_mul i64.sub local.set $next_dy))
				local.get $address i32.const 40 i32.add local.get $next_dx i64.store
				local.get $address i32.const 48 i32.add local.get $next_dy i64.store
				local.get $address i32.const 4 i32.add local.get $address i32.const 4 i32.add i32.load i32.const 1 i32.sub i32.store
				local.get $address i32.const 4 i32.add i32.load i32.const 0 i32.le_s (if (then local.get $address i32.const 0 i32.store))))
			local.get $index i32.const 1 i32.add local.set $index br $again)))

	(func $respawn_radius (result i64)
		i32.const 1128 i32.load i32.const 300 call $ticks_from_sixty i32.ge_s
		(if (result i64) (then i64.const 48000000) (else i64.const 96000000)))

	(func $respawn_safe (result i32)
		(local $index i32) (local $address i32)
		(block $done (loop $again
			local.get $index i32.const 32 i32.ge_u
			(if (then i32.const 1 return))
			local.get $index call $asteroid_address local.set $address
			local.get $address i32.load
			(if (then
				i32.const 1032 i64.load i64.const 2 i64.div_s i32.const 1040 i64.load i64.const 2 i64.div_s
				local.get $address i32.const 16 i32.add i64.load local.get $address i32.const 24 i32.add i64.load
				call $respawn_radius local.get $address i32.const 48 i32.add i64.load i64.add call $distance_lt
				(if (then i32.const 0 return))))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	(func $clear_respawn_zone
		(local $index i32) (local $address i32) (local $hit i32)
		(block $done (loop $again
			local.get $index i32.const 32 i32.ge_u br_if $done
			local.get $index call $asteroid_address local.set $address
			local.get $address i32.load
			(if (then
				i32.const 1032 i64.load i64.const 2 i64.div_s i32.const 1040 i64.load i64.const 2 i64.div_s
				local.get $address i32.const 16 i32.add i64.load local.get $address i32.const 24 i32.add i64.load
				i64.const 80000000 local.get $address i32.const 48 i32.add i64.load i64.add call $distance_lt
				(if (then local.get $address i64.const 0 i64.const 0 i32.const 0 call $hit_asteroid i32.const 1 local.set $hit))))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		local.get $hit (if (then i32.const 2 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))

	(func $advance_lifecycle
		i32.const 1124 i32.load i32.const 1 i32.eq
		(if (then
			i32.const 1128 i32.const 1128 i32.load i32.const 1 i32.sub i32.store
			i32.const 1128 i32.load i32.const 0 i32.le_s
			(if (then
				i32.const 1100 i32.load i32.const 0 i32.le_s
				(if (then i32.const 1124 i32.const 3 i32.store)
					(else
						i32.const 1124 i32.const 2 i32.store i32.const 1128 i32.const 0 i32.store
						i32.const 1048 i32.const 1032 i64.load i64.const 2 i64.div_s i64.store
						i32.const 1056 i32.const 1040 i64.load i64.const 2 i64.div_s i64.store
						i32.const 1064 i64.const 0 i64.store i32.const 1072 i64.const 0 i64.store
						call $respawn_safe
						(if (then
							i32.const 1124 i32.const 0 i32.store
							i32.const 1116 i32.const 120 call $ticks_from_sixty i32.store
							i32.const 1132 i32.const 240 call $ticks_from_sixty i32.store
							i32.const 1108 i32.const 1108 i32.load i32.const 256 i32.or i32.store))))))))
		i32.const 1124 i32.load i32.const 2 i32.eq
		(if (then
			i32.const 1128 i32.const 1128 i32.load i32.const 1 i32.add i32.store
			i32.const 1128 i32.load i32.const 600 call $ticks_from_sixty i32.eq (if (then call $clear_respawn_zone))
			call $respawn_safe
			(if (then
				i32.const 1124 i32.const 0 i32.store i32.const 1128 i32.const 0 i32.store
				i32.const 1116 i32.const 120 call $ticks_from_sixty i32.store i32.const 1132 i32.const 240 call $ticks_from_sixty i32.store
				i32.const 1108 i32.const 1108 i32.load i32.const 256 i32.or i32.store))))
	)

	(func $step
		(local $collision i32)
		i32.const 1024 i32.const 1024 i32.load i32.const 1 i32.add i32.store
		i32.const 1108 i32.load i32.const 528 i32.and (if (then return))
		i32.const 1124 i32.load i32.const 3 i32.eq
		(if (then i32.const 1108 i32.load i32.const 8 i32.and (if (then i32.const 1136 i32.load i32.const 1032 i64.load i32.const 1040 i64.load call $reset)) return))
		i32.const 1124 i32.load i32.eqz
		(if (then
			call $update_ship call $update_bullets call $update_asteroids call $update_particles call $update_debris
			call $check_bullet_collisions
			i32.const 1116 i32.load i32.const 0 i32.gt_s
			(if (then i32.const 1116 i32.const 1116 i32.load i32.const 1 i32.sub i32.store)
				(else call $find_ship_collision local.tee $collision (if (then local.get $collision call $begin_ship_explosion)))))
			(else
				i32.const 1124 i32.load i32.const 3 i32.ne
				(if (then
					i32.const 1124 i32.load i32.const 2 i32.eq (if (then call $rotate_ship))
					call $update_bullets call $update_asteroids call $update_particles call $update_debris
					call $check_bullet_collisions))))
		call $advance_lifecycle
		call $asteroid_count i32.eqz i32.const 1124 i32.load i32.const 3 i32.ne i32.and
		(if (then i32.const 1104 i32.const 1104 i32.load i32.const 1 i32.add i32.store call $spawn_wave))
		i32.const 1132 i32.load i32.const 0 i32.gt_s
		(if (then i32.const 1132 i32.const 1132 i32.load i32.const 1 i32.sub i32.store)))

	(func (export "fp_tick") (param $count i32) (result i32)
		(local $index i32)
		(block $done (loop $again
			local.get $index local.get $count i32.ge_u br_if $done
			call $step local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 1 i32.const 0 i32.const 0 call $effect drop i32.const 0)

	(func $translate_entities (param $dx i64) (param $dy i64)
		(local $index i32) (local $address i32)
		;; All records reserve x/y at +8/+16 except asteroids, which use +16/+24.
		(block $bullets_done (loop $bullets
			local.get $index i32.const 64 i32.ge_u br_if $bullets_done
			local.get $index call $bullet_address local.set $address
			local.get $address i32.load (if (then
				local.get $address i32.const 8 i32.add local.get $address i32.const 8 i32.add i64.load local.get $dx i64.add i64.store
				local.get $address i32.const 16 i32.add local.get $address i32.const 16 i32.add i64.load local.get $dy i64.add i64.store))
			local.get $index i32.const 1 i32.add local.set $index br $bullets))
		i32.const 0 local.set $index
		(block $asteroids_done (loop $asteroids
			local.get $index i32.const 32 i32.ge_u br_if $asteroids_done
			local.get $index call $asteroid_address local.set $address
			local.get $address i32.load (if (then
				local.get $address i32.const 16 i32.add local.get $address i32.const 16 i32.add i64.load local.get $dx i64.add i64.store
				local.get $address i32.const 24 i32.add local.get $address i32.const 24 i32.add i64.load local.get $dy i64.add i64.store))
			local.get $index i32.const 1 i32.add local.set $index br $asteroids))
		i32.const 0 local.set $index
		(block $particles_done (loop $particles
			local.get $index i32.const 150 i32.ge_u br_if $particles_done
			local.get $index call $particle_address local.set $address
			local.get $address i32.load (if (then
				local.get $address i32.const 8 i32.add local.get $address i32.const 8 i32.add i64.load local.get $dx i64.add i64.store
				local.get $address i32.const 16 i32.add local.get $address i32.const 16 i32.add i64.load local.get $dy i64.add i64.store))
			local.get $index i32.const 1 i32.add local.set $index br $particles))
		i32.const 0 local.set $index
		(block $debris_done (loop $debris
			local.get $index i32.const 4 i32.ge_u br_if $debris_done
			local.get $index call $debris_address local.set $address
			local.get $address i32.load (if (then
				local.get $address i32.const 8 i32.add local.get $address i32.const 8 i32.add i64.load local.get $dx i64.add i64.store
				local.get $address i32.const 16 i32.add local.get $address i32.const 16 i32.add i64.load local.get $dy i64.add i64.store))
			local.get $index i32.const 1 i32.add local.set $index br $debris)))

	(func $resize (param $new_width i64) (param $new_height i64)
		(local $dx i64) (local $dy i64)
		local.get $new_width i64.const 2 i64.div_s i32.const 1032 i64.load i64.const 2 i64.div_s i64.sub local.set $dx
		local.get $new_height i64.const 2 i64.div_s i32.const 1040 i64.load i64.const 2 i64.div_s i64.sub local.set $dy
		i32.const 1048 i32.const 1048 i64.load local.get $dx i64.add i64.store
		i32.const 1056 i32.const 1056 i64.load local.get $dy i64.add i64.store
		local.get $dx local.get $dy call $translate_entities
		i32.const 1032 local.get $new_width i64.store i32.const 1040 local.get $new_height i64.store
		call $regenerate_stars)

	(func (export "fp_event") (param $kind i32) (param $code i32)
		(param $a f32) (param $b f32) (result i32)
		(local $mask i32)
		local.get $kind i32.const 1 i32.eq
		(if (then
			local.get $code i32.const 11 i32.eq (if (then i32.const 5 local.set $code))
			local.get $code i32.const 12 i32.eq (if (then i32.const 10 local.set $code))
			local.get $code i32.const 1 i32.eq (if (then i32.const 1 local.set $mask))
			local.get $code i32.const 2 i32.eq (if (then i32.const 2 local.set $mask))
			local.get $code i32.const 3 i32.eq (if (then i32.const 4 local.set $mask))
			local.get $code i32.const 4 i32.eq (if (then i32.const 8 local.set $mask))
			local.get $mask i32.eqz
			(if (then
				local.get $code i32.const 5 i32.eq (if (then i32.const 1108 i32.const 1108 i32.load i32.const 16 i32.xor i32.store))
				local.get $code i32.const 6 i32.eq (if (then i32.const 1136 i32.load i32.const 1032 i64.load i32.const 1040 i64.load call $reset))
				local.get $code i32.const 7 i32.eq (if (then i32.const 1108 i32.const 1108 i32.load i32.const 32 i32.xor i32.store))
				local.get $code i32.const 8 i32.eq (if (then i32.const 1108 i32.const 1108 i32.load i32.const 64 i32.xor i32.store))
				local.get $code i32.const 9 i32.eq (if (then call $activate_death_blossom))
				local.get $code i32.const 10 i32.eq (if (then i32.const 1108 i32.const 1108 i32.load i32.const 512 i32.xor i32.store)))
				(else i32.const 1108 i32.const 1108 i32.load local.get $mask i32.or i32.store))))
		local.get $kind i32.const 2 i32.eq
		(if (then
			local.get $code i32.const 11 i32.eq (if (then i32.const 5 local.set $code))
			local.get $code i32.const 12 i32.eq (if (then i32.const 10 local.set $code))
			local.get $code i32.const 1 i32.eq (if (then i32.const 1 local.set $mask))
			local.get $code i32.const 2 i32.eq (if (then i32.const 2 local.set $mask))
			local.get $code i32.const 3 i32.eq (if (then i32.const 4 local.set $mask))
			local.get $code i32.const 4 i32.eq (if (then i32.const 8 local.set $mask))
			i32.const 1108 i32.const 1108 i32.load local.get $mask i32.const -1 i32.xor i32.and i32.store))
		local.get $kind i32.const 6 i32.eq
		(if (then local.get $a call $from_host local.get $b call $from_host call $resize))
		local.get $kind i32.const 7 i32.eq
		(if (then
			local.get $code i32.const 1 i32.eq (if (then i32.const 1136 i32.load i32.const 1032 i64.load i32.const 1040 i64.load call $reset))
			local.get $code i32.const 7 i32.eq (if (then i32.const 1108 i32.const 1108 i32.load i32.const 512 i32.xor i32.store))
			local.get $code i32.const 6 i32.eq (if (then i32.const 2 i32.const 0 i32.const 0 call $effect drop))))
		local.get $kind i32.const 8 i32.eq local.get $code i32.eqz i32.and
		(if (then i32.const 1108 i32.const 1108 i32.load i32.const 16 i32.and i32.store))
		i32.const 0)

	(func $rock_scale (param $address i32) (param $vertex i32) (result i64)
		local.get $address i32.const 8 i32.add i32.load local.get $vertex i32.const 3 i32.mul i32.add i32.const 7 i32.and
		i64.extend_i32_u i64.const 60000 i64.mul i64.const 680000 i64.add)

	(func $unit_x (param $vertex i32) (result i64)
		local.get $vertex i32.const 0 i32.eq (if (then i64.const 1000000 return))
		local.get $vertex i32.const 1 i32.eq (if (then i64.const 866025 return))
		local.get $vertex i32.const 2 i32.eq (if (then i64.const 500000 return))
		local.get $vertex i32.const 3 i32.eq (if (then i64.const 0 return))
		local.get $vertex i32.const 4 i32.eq (if (then i64.const -500000 return))
		local.get $vertex i32.const 5 i32.eq (if (then i64.const -866025 return))
		local.get $vertex i32.const 6 i32.eq (if (then i64.const -1000000 return))
		local.get $vertex i32.const 7 i32.eq (if (then i64.const -866025 return))
		local.get $vertex i32.const 8 i32.eq (if (then i64.const -500000 return))
		local.get $vertex i32.const 9 i32.eq (if (then i64.const 0 return))
		local.get $vertex i32.const 10 i32.eq (if (then i64.const 500000 return))
		i64.const 866025)
	(func $unit_y (param $vertex i32) (result i64)
		local.get $vertex i32.const 0 i32.eq (if (then i64.const 0 return))
		local.get $vertex i32.const 1 i32.eq (if (then i64.const 500000 return))
		local.get $vertex i32.const 2 i32.eq (if (then i64.const 866025 return))
		local.get $vertex i32.const 3 i32.eq (if (then i64.const 1000000 return))
		local.get $vertex i32.const 4 i32.eq (if (then i64.const 866025 return))
		local.get $vertex i32.const 5 i32.eq (if (then i64.const 500000 return))
		local.get $vertex i32.const 6 i32.eq (if (then i64.const 0 return))
		local.get $vertex i32.const 7 i32.eq (if (then i64.const -500000 return))
		local.get $vertex i32.const 8 i32.eq (if (then i64.const -866025 return))
		local.get $vertex i32.const 9 i32.eq (if (then i64.const -1000000 return))
		local.get $vertex i32.const 10 i32.eq (if (then i64.const -866025 return))
		i64.const -500000)

	(func $emit_rock_point (param $address i32) (param $vertex i32) (param $first i32)
		(local $x i64) (local $y i64) (local $scaled_radius i64) (local $direction i32)
		local.get $address i32.const 48 i32.add i64.load local.get $address local.get $vertex call $rock_scale call $fixed_mul local.set $scaled_radius
		local.get $vertex i32.const 12 i32.mul local.get $address i32.const 76 i32.add i32.load i32.div_u local.set $direction
		local.get $direction call $unit_x local.get $scaled_radius call $fixed_mul local.set $x
		local.get $direction call $unit_y local.get $scaled_radius call $fixed_mul local.set $y
		local.get $first
		(if (then local.get $x call $to_host local.get $y call $to_host call $path_move drop)
			(else local.get $x call $to_host local.get $y call $to_host call $path_line drop)))

	(func $draw_ship (param $id i32) (param $x i64) (param $y i64)
		(param $dx i64) (param $dy i64) (param $visual_scale i64)
		(param $color i32) (param $flame i32)
		(local $a i64) (local $b i64) (local $c i64) (local $d i64) (local $flame_x i64)
		local.get $dx local.get $visual_scale call $fixed_mul local.set $a
		local.get $dy local.get $visual_scale call $fixed_mul local.set $b
		i64.const 0 local.get $b i64.sub local.set $c local.get $a local.set $d
		local.get $a call $to_host local.get $b call $to_host local.get $c call $to_host local.get $d call $to_host
		local.get $x call $to_host local.get $y call $to_host call $transform_push drop
		local.get $id call $path_begin drop
		f32.const 15 f32.const 0 call $path_move drop f32.const -10 f32.const -8 call $path_line drop
		f32.const -5 f32.const 0 call $path_line drop f32.const -10 f32.const 8 call $path_line drop
		call $path_close drop f32.const 2 i32.const 0 local.get $color i32.const 0 call $path_end drop
		local.get $id i32.const 1000 i32.add f32.const 0 f32.const 0 f32.const 3 f32.const 0 local.get $color i32.const 1 call $circle drop
		local.get $flame
		(if (then
			local.get $id i32.const 3 i32.add call $path_begin drop f32.const -5 f32.const -3 call $path_move drop
			i64.const 0
			i32.const 1024 i32.load i32.const 7 i32.and i64.extend_i32_u i64.const 2000000 i64.mul i64.const 12000000 i64.add
			i64.sub local.set $flame_x
			local.get $flame_x call $to_host f32.const 0 call $path_line drop f32.const -5 f32.const 3 call $path_line drop
			f32.const 2 i32.const 0 i32.const 0xff8a2bff i32.const 0 call $path_end drop))
		call $transform_pop drop)

	(func $draw_asteroid (param $index i32) (param $address i32)
		(local $vertex i32) (local $points i32) (local $radius i64)
		local.get $address i32.load
		(if (then
			local.get $address i32.const 48 i32.add i64.load local.set $radius
			local.get $address i32.const 76 i32.add i32.load local.set $points
			local.get $address i32.const 56 i32.add i64.load call $to_host
			local.get $address i32.const 64 i32.add i64.load call $to_host
			i64.const 0 local.get $address i32.const 64 i32.add i64.load i64.sub call $to_host
			local.get $address i32.const 56 i32.add i64.load call $to_host
			local.get $address i32.const 16 i32.add i64.load call $to_host
			local.get $address i32.const 24 i32.add i64.load call $to_host call $transform_push drop
			i32.const 200 local.get $index i32.add call $path_begin drop
			(block $vertices_done (loop $vertices
				local.get $vertex local.get $points i32.ge_u br_if $vertices_done
				local.get $address local.get $vertex local.get $vertex i32.eqz call $emit_rock_point
				local.get $vertex i32.const 1 i32.add local.set $vertex br $vertices))
			call $path_close drop f32.const 2 i32.const 0 i32.const 0x8894a8ff i32.const 0 call $path_end drop
			i32.const 300 local.get $index i32.add
			local.get $radius i64.const -550000 call $fixed_mul call $to_host f32.const 0
			local.get $radius i64.const 450000 call $fixed_mul call $to_host
			local.get $radius i64.const 250000 call $fixed_mul call $to_host
			f32.const 1 i32.const 0x536178ff call $line drop call $transform_pop drop)))

	(func $draw_debris (param $index i32) (param $address i32)
		(local $piece i32)
		local.get $address i32.load
		(if (then
			local.get $address i32.const 60 i32.add i32.load local.set $piece
			local.get $address i32.const 40 i32.add i64.load call $to_host
			local.get $address i32.const 48 i32.add i64.load call $to_host
			i64.const 0 local.get $address i32.const 48 i32.add i64.load i64.sub call $to_host
			local.get $address i32.const 40 i32.add i64.load call $to_host
			local.get $address i32.const 8 i32.add i64.load call $to_host
			local.get $address i32.const 16 i32.add i64.load call $to_host call $transform_push drop
			i32.const 700 local.get $index i32.add call $path_begin drop
			local.get $piece i32.eqz
			(if (then f32.const 15 f32.const 0 call $path_move drop f32.const 5 f32.const 0 call $path_line drop f32.const 0 f32.const -3 call $path_line drop)
				(else local.get $piece i32.const 1 i32.eq
					(if (then f32.const -10 f32.const -8 call $path_move drop f32.const -5 f32.const 0 call $path_line drop f32.const 0 f32.const -3 call $path_line drop)
						(else local.get $piece i32.const 2 i32.eq
							(if (then f32.const -10 f32.const 8 call $path_move drop f32.const -5 f32.const 0 call $path_line drop f32.const 0 f32.const 3 call $path_line drop)
								(else f32.const -5 f32.const 0 call $path_move drop f32.const 3 f32.const -3 call $path_line drop f32.const 3 f32.const 3 call $path_line drop))))))
			call $path_close drop f32.const 2 i32.const 0 i32.const 0xffffffff i32.const 0 call $path_end drop call $transform_pop drop)))

	(func $draw_stars
		(local $index i32) (local $address i32)
		(block $done (loop $again
			local.get $index i32.const 100 i32.ge_u br_if $done
			local.get $index call $star_address local.set $address
			i32.const 800 local.get $index i32.add
			local.get $address i64.load call $to_host local.get $address i32.const 8 i32.add i64.load call $to_host
			f32.const 0.85 f32.const 0 i32.const 0x9bb8d199 i32.const 1 call $circle drop
			local.get $index i32.const 1 i32.add local.set $index br $again)))

	(func (export "fp_render") (result i32)
		(local $index i32) (local $address i32) (local $reserve_count i32) (local $alpha i32)
		f32.const 0.03137255 f32.const 0.04313725 f32.const 0.07058824 f32.const 1 call $frame_begin drop
		i32.const 1132 i32.load i32.const 0 i32.gt_s
		(if (then
			call $splash_alpha local.set $alpha
			i32.const 10 i32.const 64 i32.const 12
			i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host
			i32.const 1040 i64.load i64.const 333333 call $fixed_mul call $to_host
			f32.const 54 i32.const 0xff5cf400 local.get $alpha i32.or i32.const 1 call $text drop
			i32.const 12 i32.const 64 i32.const 12
			i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host
			i32.const 1040 i64.load i64.const 333333 call $fixed_mul call $to_host
			f32.const 52 i32.const 0x5ee7ff00 local.get $alpha i32.or i32.const 1 call $text drop
			i32.const 13 i32.const 64 i32.const 12
			i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host
			i32.const 1040 i64.load i64.const 333333 call $fixed_mul call $to_host
			f32.const 50 i32.const 0xffffff00 local.get $alpha i32.or i32.const 1 call $text drop
			i32.const 11 i32.const 80 i32.const 16
			i32.const 1032 i64.load i64.const 2 i64.div_s i64.const 100000000 i64.sub call $to_host
			i32.const 1040 i64.load i64.const 333333 call $fixed_mul i64.const 58000000 i64.add call $to_host
			f32.const 18 i32.const 0x58ff7200 local.get $alpha i32.or i32.const 1 call $text drop))
		call $draw_stars
		i32.const 160 i32.const 1096 i32.load call $write_six_digits
		i32.const 168 i32.const 1104 i32.load call $write_two_digits
		i32.const 1108 i32.load i32.const 64 i32.and i32.eqz
		(if (then
		i32.const 20 i32.const 128 i32.const 5 f32.const 24 f32.const 24 f32.const 15 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 21 i32.const 160 i32.const 6 f32.const 24 f32.const 48 f32.const 22 i32.const 0x58ff72ff i32.const 0 call $text drop
		i32.const 22 i32.const 136 i32.const 5
		i32.const 1032 i64.load i64.const 2 i64.div_s i64.const 50000000 i64.sub call $to_host f32.const 54 f32.const 15 i32.const 0x9bb8d1ff i32.const 1 call $text drop
		i32.const 23 i32.const 168 i32.const 2
		i32.const 1032 i64.load i64.const 2 i64.div_s i64.const 30000000 i64.add call $to_host f32.const 54 f32.const 22 i32.const 0xffcf5cff i32.const 1 call $text drop
		i32.const 24 i32.const 144 i32.const 5
		i32.const 1032 i64.load i64.const 62000000 i64.sub call $to_host f32.const 24 f32.const 15 i32.const 0x9bb8d1ff i32.const 1 call $text drop
		i32.const 1100 i32.load i32.const 1 i32.sub local.set $reserve_count
		local.get $reserve_count i32.const 5 i32.gt_u (if (then i32.const 5 local.set $reserve_count))
		(block $reserves_done (loop $reserves
			local.get $index local.get $reserve_count i32.ge_u br_if $reserves_done
			i32.const 50 local.get $index i32.add
			i32.const 1032 i64.load i64.const 40000000 i64.sub local.get $index i64.extend_i32_u i64.const 28000000 i64.mul i64.sub
			i64.const 48000000 i64.const 1000000 i64.const 0 i64.const 550000 i32.const 0x58ff72ff i32.const 0 call $draw_ship
			local.get $index i32.const 1 i32.add local.set $index br $reserves))))
		i32.const 1124 i32.load local.set $index
		local.get $index i32.eqz
		(if (then
			i32.const 1116 i32.load i32.eqz i32.const 1024 i32.load i32.const 8 i32.and i32.eqz i32.or
			(if (then i32.const 1 i32.const 1048 i64.load i32.const 1056 i64.load i32.const 1080 i64.load i32.const 1088 i64.load i64.const 1000000 i32.const -1 i32.const 1108 i32.load i32.const 4 i32.and i32.eqz i32.eqz call $draw_ship)))
			(else local.get $index i32.const 2 i32.eq
				(if (then i32.const 1 i32.const 1048 i64.load i32.const 1056 i64.load i32.const 1080 i64.load i32.const 1088 i64.load i64.const 1000000 i32.const 0x777f8c99 i32.const 0 call $draw_ship))))
		i32.const 0 local.set $index
		(block $bullets_done (loop $bullets
			local.get $index i32.const 64 i32.ge_u br_if $bullets_done
			local.get $index call $bullet_address local.set $address
			local.get $address i32.load (if (then
				i32.const 100 local.get $index i32.add local.get $address i32.const 8 i32.add i64.load call $to_host local.get $address i32.const 16 i32.add i64.load call $to_host
				f32.const 2 f32.const 0 i32.const 0x58ff72ff i32.const 1 call $circle drop))
			local.get $index i32.const 1 i32.add local.set $index br $bullets))
		i32.const 0 local.set $index
		(block $asteroids_done (loop $asteroids
			local.get $index i32.const 32 i32.ge_u br_if $asteroids_done
			local.get $index call $asteroid_address local.set $address local.get $index local.get $address call $draw_asteroid
			local.get $index i32.const 1 i32.add local.set $index br $asteroids))
		i32.const 0 local.set $index
		(block $particles_done (loop $particles
			local.get $index i32.const 150 i32.ge_u br_if $particles_done
			local.get $index call $particle_address local.set $address
			local.get $address i32.load (if (then
				i32.const 500 local.get $index i32.add local.get $address i32.const 8 i32.add i64.load call $to_host local.get $address i32.const 16 i32.add i64.load call $to_host
				f32.const 2 f32.const 0 i32.const 0xff9b2fff i32.const 1 call $circle drop))
			local.get $index i32.const 1 i32.add local.set $index br $particles))
		i32.const 0 local.set $index
		(block $debris_done (loop $debris
			local.get $index i32.const 4 i32.ge_u br_if $debris_done
			local.get $index call $debris_address local.set $address local.get $index local.get $address call $draw_debris
			local.get $index i32.const 1 i32.add local.set $index br $debris))
		i32.const 1124 i32.load i32.const 1 i32.eq
		(if (then i32.const 32 i32.const 184 i32.const 14 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i32.const 1040 i64.load i64.const 2 i64.div_s call $to_host f32.const 26 i32.const 0xff8a2bff i32.const 1 call $text drop))
		i32.const 1108 i32.load i32.const 256 i32.and
		(if (then i32.const 35 i32.const 488 i32.const 4 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 82000000 call $to_host f32.const 24 i32.const 0xffcf5cff i32.const 1 call $text drop))
		i32.const 1108 i32.load i32.const 128 i32.and
		(if (then i32.const 36 i32.const 472 i32.const 14 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i32.const 1040 i64.load i64.const 666667 call $fixed_mul call $to_host f32.const 36 i32.const 0xff5cf4ff i32.const 1 call $text drop))
		i32.const 1108 i32.load i32.const 16 i32.and
		(if (then i32.const 33 i32.const 100 i32.const 6 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i32.const 1040 i64.load i64.const 2 i64.div_s call $to_host f32.const 36 i32.const 0xffcf5cff i32.const 1 call $text drop))
		i32.const 1108 i32.load i32.const 512 i32.and
		(if (then
			i32.const 40 i32.const 224 i32.const 8 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 140000000 call $to_host f32.const 36 i32.const 0x5ee7ffff i32.const 1 call $text drop
			i32.const 41 i32.const 240 i32.const 21 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 200000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop
			i32.const 42 i32.const 264 i32.const 21 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 232000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop
			i32.const 43 i32.const 288 i32.const 19 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 264000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop
			i32.const 44 i32.const 312 i32.const 24 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 296000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop
			i32.const 45 i32.const 340 i32.const 23 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 328000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop
			i32.const 46 i32.const 368 i32.const 28 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 360000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop
			i32.const 47 i32.const 400 i32.const 20 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 392000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop
			i32.const 48 i32.const 424 i32.const 22 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 424000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop
			i32.const 49 i32.const 448 i32.const 19 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i64.const 456000000 call $to_host f32.const 20 i32.const 0xffffffff i32.const 1 call $text drop))
		i32.const 1124 i32.load i32.const 3 i32.eq
		(if (then i32.const 34 i32.const 108 i32.const 9 i32.const 1032 i64.load i64.const 2 i64.div_s call $to_host i32.const 1040 i64.load i64.const 2 i64.div_s call $to_host f32.const 40 i32.const 0xff5c73ff i32.const 1 call $text drop))
		call $frame_end drop i32.const 0)
)
