;; Vibesteroids behavioral conversion for the gpui-frontplane-v0 ABI.
;;
;; Canonical snapshot state occupies [1024, 9216). Gameplay randomness,
;; fixed-capacity entity pools, lifecycle timers, and all simulation state are
;; included so snapshot/restore and live WAT reloads remain deterministic.
;; Rendering writes only decimal display buffers below the snapshot.
;;
;; Header, relative to fp_state_ptr = 1024:
;;   0 tick:i32          4 rng-state:i32     8 width:f32
;;  12 height:f32      16 ship.x:f32       20 ship.y:f32
;;  24 ship.vx:f32     28 ship.vy:f32      32 ship.dx:f32
;;  36 ship.dy:f32     40 score:i32        44 lives:i32
;;  48 level:i32       52 input flags:i32  56 last-fire tick:i32
;;  60 invulnerability 64 next-extra-life  68 lifecycle:i32
;;  72 lifecycle ticks 76 wave-banner ticks 80 initial seed:i32
;;
;; Pools, relative to fp_state_ptr:
;;  256: 64 bullets × 24 bytes
;;       active:i32, x/y/vx/vy:f32, life:i32
;; 1792: 32 asteroids × 48 bytes
;;       active:i32, generation:i32, shape:i32, x/y/vx/vy/radius:f32,
;;       rotation x/y:f32, spin-sign:i32, reserved:i32
;; 3328: 150 particles × 24 bytes
;;       active:i32, x/y/vx/vy:f32, life:i32
;; 6928: 4 debris pieces × 48 bytes
;;       active:i32, x/y/vx/vy:f32, rotation x/y:f32, spin-sign:i32,
;;       life:i32, piece:i32, reserved:i32
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

	(func (export "fp_abi_major") (result i32) i32.const 0)
	(func (export "fp_abi_minor") (result i32) i32.const 0)
	(func (export "fp_state_ptr") (result i32) i32.const 1024)
	(func (export "fp_state_len") (result i32) i32.const 8192)
	(func (export "fp_state_schema") (result i32) i32.const 2)

	(func (export "fp_configure") (result i32)
		i32.const 0 i32.const 20 call $title drop
		i32.const 1 i32.const 32 i32.const 8 i32.const 1 i32.const 0 call $menu_item drop
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 i32.const 1 call $menu_item drop
		i32.const 6 i32.const 48 i32.const 4 i32.const 6 i32.const 0 call $menu_item drop
		i32.const 0)

	(func $bullet_address (param $index i32) (result i32)
		i32.const 1280
		local.get $index i32.const 24 i32.mul
		i32.add)

	(func $asteroid_address (param $index i32) (result i32)
		i32.const 2816
		local.get $index i32.const 48 i32.mul
		i32.add)

	(func $particle_address (param $index i32) (result i32)
		i32.const 4352
		local.get $index i32.const 24 i32.mul
		i32.add)

	(func $debris_address (param $index i32) (result i32)
		i32.const 7952
		local.get $index i32.const 48 i32.mul
		i32.add)

	;; Mulberry32 keeps seeded gameplay deterministic without render-time RNG.
	(func $rand_u32 (result i32)
		(local $state i32) (local $t i32)
		i32.const 1028 i32.load
		i32.const 0x6d2b79f5 i32.add
		local.set $state
		i32.const 1028 local.get $state i32.store
		local.get $state local.set $t
		local.get $t
		local.get $t i32.const 15 i32.shr_u i32.xor
		local.get $t i32.const 1 i32.or
		i32.mul
		local.set $t
		local.get $t
		local.get $t
		local.get $t
		local.get $t i32.const 7 i32.shr_u i32.xor
		local.get $t i32.const 61 i32.or
		i32.mul
		i32.add
		i32.xor
		local.set $t
		local.get $t
		local.get $t i32.const 14 i32.shr_u
		i32.xor)

	(func $rand_unit (result f32)
		call $rand_u32
		i32.const 0xffff i32.and
		f32.convert_i32_u
		f32.const 65536
		f32.div)

	(func $rand_signed (result f32)
		call $rand_unit
		f32.const 2 f32.mul
		f32.const 1 f32.sub)

	(func $ensure_component (param $value f32) (result f32)
		local.get $value f32.abs f32.const 0.45 f32.lt
		(if (result f32)
			(then
				local.get $value f32.const 0 f32.lt
				(if (result f32)
					(then f32.const -0.45)
					(else f32.const 0.45)))
			(else local.get $value)))

	(func $store_digit (param $address i32) (param $value i32) (param $divisor i32)
		local.get $address
		local.get $value
		local.get $divisor i32.div_u
		i32.const 10 i32.rem_u
		i32.const 48 i32.add
		i32.store8)

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

	(func $wrap_ship_x (param $value f32) (result f32)
		local.get $value f32.const 0 f32.lt
		(if (result f32)
			(then i32.const 1032 f32.load)
			(else
				local.get $value i32.const 1032 f32.load f32.gt
				(if (result f32)
					(then f32.const 0)
					(else local.get $value)))))

	(func $wrap_ship_y (param $value f32) (result f32)
		local.get $value f32.const 0 f32.lt
		(if (result f32)
			(then i32.const 1036 f32.load)
			(else
				local.get $value i32.const 1036 f32.load f32.gt
				(if (result f32)
					(then f32.const 0)
					(else local.get $value)))))

	(func $wrap_bullet_x (param $value f32) (result f32)
		local.get $value f32.const -25 f32.lt
		(if (result f32)
			(then i32.const 1032 f32.load f32.const 25 f32.add)
			(else
				local.get $value i32.const 1032 f32.load f32.const 25 f32.add f32.gt
				(if (result f32)
					(then f32.const -25)
					(else local.get $value)))))

	(func $wrap_bullet_y (param $value f32) (result f32)
		local.get $value f32.const -25 f32.lt
		(if (result f32)
			(then i32.const 1036 f32.load f32.const 25 f32.add)
			(else
				local.get $value i32.const 1036 f32.load f32.const 25 f32.add f32.gt
				(if (result f32)
					(then f32.const -25)
					(else local.get $value)))))

	(func $wrap_asteroid_x (param $value f32) (result f32)
		local.get $value f32.const -50 f32.lt
		(if (result f32)
			(then i32.const 1032 f32.load f32.const 50 f32.add)
			(else
				local.get $value i32.const 1032 f32.load f32.const 50 f32.add f32.gt
				(if (result f32)
					(then f32.const -50)
					(else local.get $value)))))

	(func $wrap_asteroid_y (param $value f32) (result f32)
		local.get $value f32.const -50 f32.lt
		(if (result f32)
			(then i32.const 1036 f32.load f32.const 50 f32.add)
			(else
				local.get $value i32.const 1036 f32.load f32.const 50 f32.add f32.gt
				(if (result f32)
					(then f32.const -50)
					(else local.get $value)))))

	(func $find_free_bullet (result i32)
		(local $index i32) (local $address i32)
		(block $none
			(loop $again
				local.get $index i32.const 64 i32.ge_u br_if $none
				local.get $index call $bullet_address local.set $address
				local.get $address i32.load i32.eqz
				(if (then local.get $address return))
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		i32.const 0)

	(func $find_free_asteroid (result i32)
		(local $index i32) (local $address i32)
		(block $none
			(loop $again
				local.get $index i32.const 32 i32.ge_u br_if $none
				local.get $index call $asteroid_address local.set $address
				local.get $address i32.load i32.eqz
				(if (then local.get $address return))
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		i32.const 0)

	(func $asteroid_count (result i32)
		(local $index i32) (local $count i32) (local $address i32)
		(block $done
			(loop $again
				local.get $index i32.const 32 i32.ge_u br_if $done
				local.get $index call $asteroid_address local.set $address
				local.get $count
				local.get $address i32.load
				i32.add
				local.set $count
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		local.get $count)

	(func $initialize_asteroid
		(param $address i32) (param $x f32) (param $y f32) (param $radius f32)
		(local $shape i32) (local $level_scale f32)
		call $rand_u32 i32.const 0xffff i32.and local.set $shape
		i32.const 1072 i32.load i32.const 1 i32.sub
		f32.convert_i32_s f32.const 0.03 f32.mul f32.const 1 f32.add
		local.set $level_scale
		local.get $address i32.const 1 i32.store
		local.get $address i32.const 4 i32.add i32.const 0 i32.store
		local.get $address i32.const 8 i32.add local.get $shape i32.store
		local.get $address i32.const 12 i32.add local.get $x f32.store
		local.get $address i32.const 16 i32.add local.get $y f32.store
		local.get $address i32.const 20 i32.add
		call $rand_signed f32.const 1.1 f32.mul local.get $level_scale f32.mul
		call $ensure_component f32.store
		local.get $address i32.const 24 i32.add
		call $rand_signed f32.const 1.1 f32.mul local.get $level_scale f32.mul
		call $ensure_component f32.store
		local.get $address i32.const 28 i32.add local.get $radius f32.store
		local.get $address i32.const 32 i32.add f32.const 1 f32.store
		local.get $address i32.const 36 i32.add f32.const 0 f32.store
		local.get $address i32.const 40 i32.add
		local.get $shape i32.const 1 i32.and
		(if (result i32) (then i32.const 1) (else i32.const -1))
		i32.store)

	(func $spawn_wave
		(local $count i32) (local $index i32) (local $address i32)
		(local $edge i32) (local $x f32) (local $y f32) (local $radius f32)
		i32.const 2816 i32.const 0 i32.const 1536 memory.fill
		i32.const 1072 i32.load i32.const 4 i32.add local.set $count
		local.get $count i32.const 32 i32.gt_u
		(if (then i32.const 32 local.set $count))
		(block $done
			(loop $again
				local.get $index local.get $count i32.ge_u br_if $done
				local.get $index call $asteroid_address local.set $address
				local.get $index i32.const 3 i32.and local.set $edge
				call $rand_unit i32.const 1032 f32.load f32.mul local.set $x
				call $rand_unit i32.const 1036 f32.load f32.mul local.set $y
				local.get $edge i32.const 0 i32.eq
				(if (then f32.const 100 local.set $y))
				local.get $edge i32.const 1 i32.eq
				(if
					(then
						i32.const 1032 f32.load f32.const 100 f32.sub local.set $x))
				local.get $edge i32.const 2 i32.eq
				(if
					(then
						i32.const 1036 f32.load f32.const 100 f32.sub local.set $y))
				local.get $edge i32.const 3 i32.eq
				(if (then f32.const 100 local.set $x))
				call $rand_unit f32.const 24 f32.mul f32.const 24 f32.add local.set $radius
				local.get $address local.get $x local.get $y local.get $radius
				call $initialize_asteroid
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		i32.const 1100 i32.const 90 i32.store)

	(func $reset (param $seed i32) (param $width f32) (param $height f32)
		(local $normalized_seed i32)
		local.get $seed i32.eqz
		(if (result i32) (then i32.const 1) (else local.get $seed))
		local.set $normalized_seed
		i32.const 1024 i32.const 0 i32.const 8192 memory.fill
		i32.const 1024 i32.const 0 i32.store
		i32.const 1028 local.get $normalized_seed i32.store
		i32.const 1032 local.get $width f32.store
		i32.const 1036 local.get $height f32.store
		i32.const 1040 local.get $width f32.const 0.5 f32.mul f32.store
		i32.const 1044 local.get $height f32.const 0.5 f32.mul f32.store
		i32.const 1048 f32.const 0 f32.store
		i32.const 1052 f32.const 0 f32.store
		i32.const 1056 f32.const 0 f32.store
		i32.const 1060 f32.const -1 f32.store
		i32.const 1064 i32.const 0 i32.store
		i32.const 1068 i32.const 3 i32.store
		i32.const 1072 i32.const 1 i32.store
		i32.const 1076 i32.const 0 i32.store
		i32.const 1080 i32.const -100 i32.store
		i32.const 1084 i32.const 120 i32.store
		i32.const 1088 i32.const 20000 i32.store
		i32.const 1092 i32.const 0 i32.store
		i32.const 1096 i32.const 0 i32.store
		i32.const 1104 local.get $normalized_seed i32.store
		call $spawn_wave)

	(func (export "fp_init") (param $seed_lo i32) (param $seed_hi i32)
		(param $width f32) (param $height f32) (result i32)
		local.get $seed_lo local.get $width local.get $height call $reset
		i32.const 0)

	(func $spawn_particles
		(param $x f32) (param $y f32) (param $count i32)
		(param $inherit_vx f32) (param $inherit_vy f32)
		(local $index i32) (local $added i32) (local $address i32)
		(local $direction i32) (local $ux f32) (local $uy f32) (local $speed f32)
		(block $done
			(loop $again
				local.get $added local.get $count i32.ge_u br_if $done
				local.get $index i32.const 150 i32.ge_u br_if $done
				local.get $index call $particle_address local.set $address
				local.get $address i32.load i32.eqz
				(if
					(then
						call $rand_u32 i32.const 7 i32.and local.set $direction
						f32.const 0 local.set $ux
						f32.const 0 local.set $uy
						local.get $direction i32.const 0 i32.eq
						(if (then f32.const 1 local.set $ux))
						local.get $direction i32.const 1 i32.eq
						(if (then f32.const 0.7071068 local.set $ux f32.const 0.7071068 local.set $uy))
						local.get $direction i32.const 2 i32.eq
						(if (then f32.const 1 local.set $uy))
						local.get $direction i32.const 3 i32.eq
						(if (then f32.const -0.7071068 local.set $ux f32.const 0.7071068 local.set $uy))
						local.get $direction i32.const 4 i32.eq
						(if (then f32.const -1 local.set $ux))
						local.get $direction i32.const 5 i32.eq
						(if (then f32.const -0.7071068 local.set $ux f32.const -0.7071068 local.set $uy))
						local.get $direction i32.const 6 i32.eq
						(if (then f32.const -1 local.set $uy))
						local.get $direction i32.const 7 i32.eq
						(if (then f32.const 0.7071068 local.set $ux f32.const -0.7071068 local.set $uy))
						call $rand_unit f32.const 2.2 f32.mul f32.const 1.2 f32.add
						local.set $speed
						local.get $address i32.const 1 i32.store
						local.get $address i32.const 4 i32.add local.get $x f32.store
						local.get $address i32.const 8 i32.add local.get $y f32.store
						local.get $address i32.const 12 i32.add
						local.get $inherit_vx local.get $ux local.get $speed f32.mul f32.add
						f32.store
						local.get $address i32.const 16 i32.add
						local.get $inherit_vy local.get $uy local.get $speed f32.mul f32.add
						f32.store
						local.get $address i32.const 20 i32.add
						call $rand_u32 i32.const 20 i32.rem_u i32.const 30 i32.add
						i32.store
						local.get $added i32.const 1 i32.add local.set $added))
				local.get $index i32.const 1 i32.add local.set $index
				br $again)))

	(func $spawn_debris (param $x f32) (param $y f32) (param $vx f32) (param $vy f32)
		(local $index i32) (local $address i32)
		i32.const 7952 i32.const 0 i32.const 192 memory.fill
		(block $done
			(loop $again
				local.get $index i32.const 4 i32.ge_u br_if $done
				local.get $index call $debris_address local.set $address
				local.get $address i32.const 1 i32.store
				local.get $address i32.const 4 i32.add local.get $x f32.store
				local.get $address i32.const 8 i32.add local.get $y f32.store
				local.get $address i32.const 12 i32.add
				local.get $vx
				local.get $index i32.const 0 i32.eq
				(if (result f32)
					(then f32.const 2.2)
					(else
						local.get $index i32.const 1 i32.eq
						(if (result f32)
							(then f32.const -1.6)
							(else
								local.get $index i32.const 2 i32.eq
								(if (result f32) (then f32.const -1.4) (else f32.const 0.8))))))
				f32.add f32.store
				local.get $address i32.const 16 i32.add
				local.get $vy
				local.get $index i32.const 0 i32.eq
				(if (result f32)
					(then f32.const -0.4)
					(else
						local.get $index i32.const 1 i32.eq
						(if (result f32)
							(then f32.const -1.7)
							(else
								local.get $index i32.const 2 i32.eq
								(if (result f32) (then f32.const 1.7) (else f32.const 2.0))))))
				f32.add f32.store
				local.get $address i32.const 20 i32.add f32.const 1 f32.store
				local.get $address i32.const 24 i32.add f32.const 0 f32.store
				local.get $address i32.const 28 i32.add
				local.get $index i32.const 1 i32.and
				(if (result i32) (then i32.const 1) (else i32.const -1))
				i32.store
				local.get $address i32.const 32 i32.add i32.const 120 i32.store
				local.get $address i32.const 36 i32.add local.get $index i32.store
				local.get $index i32.const 1 i32.add local.set $index
				br $again)))

	(func $find_free_particle (result i32)
		(local $index i32) (local $address i32)
		(block $none
			(loop $again
				local.get $index i32.const 150 i32.ge_u br_if $none
				local.get $index call $particle_address local.set $address
				local.get $address i32.load i32.eqz
				(if (then local.get $address return))
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		i32.const 0)

	(func $hit_asteroid
		(param $address i32) (param $impulse_x f32) (param $impulse_y f32)
		(param $award_score i32)
		(local $radius f32) (local $child_radius f32)
		(local $x f32) (local $y f32) (local $vx f32) (local $vy f32)
		(local $shape i32) (local $generation i32) (local $free i32)
		(local $points i32)
		local.get $address i32.const 28 i32.add f32.load local.set $radius
		local.get $address i32.const 12 i32.add f32.load local.set $x
		local.get $address i32.const 16 i32.add f32.load local.set $y
		local.get $address i32.const 20 i32.add f32.load local.set $vx
		local.get $address i32.const 24 i32.add f32.load local.set $vy
		local.get $address i32.const 8 i32.add i32.load local.set $shape
		local.get $address i32.const 4 i32.add i32.load local.set $generation
		local.get $x local.get $y i32.const 20 f32.const 0 f32.const 0 call $spawn_particles
		i32.const 2 f32.const 1 f32.const 1 i32.const 0 call $audio drop
		local.get $radius f32.const 20 f32.gt
		(if
			(then
				i32.const 80 local.set $points
				local.get $radius f32.const 0.6 f32.mul local.set $child_radius
				call $find_free_asteroid local.set $free
				local.get $address i32.const 4 i32.add
				local.get $generation i32.const 1 i32.add i32.store
				local.get $address i32.const 8 i32.add
				local.get $shape i32.const 1 i32.add i32.store
				local.get $address i32.const 12 i32.add
				local.get $x f32.const 6 f32.sub f32.store
				local.get $address i32.const 20 i32.add
				local.get $vx local.get $impulse_x f32.add f32.const 0.6 f32.sub
				call $ensure_component f32.store
				local.get $address i32.const 24 i32.add
				local.get $vy local.get $impulse_y f32.add f32.const 0.4 f32.add
				call $ensure_component f32.store
				local.get $address i32.const 28 i32.add local.get $child_radius f32.store
				local.get $free
				(if
					(then
						local.get $free i32.const 1 i32.store
						local.get $free i32.const 4 i32.add
						local.get $generation i32.const 1 i32.add i32.store
						local.get $free i32.const 8 i32.add
						local.get $shape i32.const 2 i32.add i32.store
						local.get $free i32.const 12 i32.add
						local.get $x f32.const 6 f32.add f32.store
						local.get $free i32.const 16 i32.add
						local.get $y f32.const 6 f32.add f32.store
						local.get $free i32.const 20 i32.add
						local.get $vx local.get $impulse_x f32.add f32.const 0.6 f32.add
						call $ensure_component f32.store
						local.get $free i32.const 24 i32.add
						local.get $vy local.get $impulse_y f32.add f32.const 0.4 f32.sub
						call $ensure_component f32.store
						local.get $free i32.const 28 i32.add local.get $child_radius f32.store
						local.get $free i32.const 32 i32.add f32.const 1 f32.store
						local.get $free i32.const 36 i32.add f32.const 0 f32.store
						local.get $free i32.const 40 i32.add
						local.get $address i32.const 40 i32.add i32.load
						i32.const -1 i32.mul i32.store)))
			(else
				i32.const 120 local.set $points
				local.get $address i32.const 0 i32.store))
		local.get $award_score
		(if
			(then
				i32.const 1064
				i32.const 1064 i32.load local.get $points i32.add
				i32.store
				i32.const 1064 i32.load i32.const 1088 i32.load i32.ge_u
				(if
					(then
						i32.const 1068 i32.const 1068 i32.load i32.const 1 i32.add i32.store
						i32.const 1088 i32.const 1088 i32.load i32.const 20000 i32.add i32.store
						i32.const 6 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))))

	(func $bullet_hits (param $bullet i32) (param $asteroid i32) (result i32)
		(local $dx f32) (local $dy f32) (local $radius f32)
		local.get $bullet i32.load
		local.get $asteroid i32.load
		i32.and
		(if (result i32)
			(then
				local.get $bullet i32.const 4 i32.add f32.load
				local.get $asteroid i32.const 12 i32.add f32.load f32.sub
				local.set $dx
				local.get $bullet i32.const 8 i32.add f32.load
				local.get $asteroid i32.const 16 i32.add f32.load f32.sub
				local.set $dy
				local.get $asteroid i32.const 28 i32.add f32.load
				f32.const 5 f32.add local.set $radius
				local.get $dx local.get $dx f32.mul
				local.get $dy local.get $dy f32.mul f32.add
				local.get $radius local.get $radius f32.mul
				f32.lt)
			(else i32.const 0)))

	(func $resolve_bullet_collisions
		(local $bullet_index i32) (local $asteroid_index i32)
		(local $bullet i32) (local $asteroid i32)
		(local $factor f32) (local $impulse_x f32) (local $impulse_y f32)
		(block $bullets_done
			(loop $next_bullet
				local.get $bullet_index i32.const 64 i32.ge_u br_if $bullets_done
				local.get $bullet_index call $bullet_address local.set $bullet
				local.get $bullet i32.load
				(if
					(then
						i32.const 0 local.set $asteroid_index
						(block $bullet_done
							(loop $next_asteroid
								local.get $asteroid_index i32.const 32 i32.ge_u br_if $bullet_done
								local.get $asteroid_index call $asteroid_address local.set $asteroid
								local.get $bullet local.get $asteroid call $bullet_hits
								(if
									(then
										f32.const 3.6
										local.get $asteroid i32.const 28 i32.add f32.load
										f32.div local.set $factor
										local.get $bullet i32.const 12 i32.add f32.load
										local.get $factor f32.mul local.set $impulse_x
										local.get $bullet i32.const 16 i32.add f32.load
										local.get $factor f32.mul local.set $impulse_y
										local.get $bullet i32.const 0 i32.store
										local.get $asteroid local.get $impulse_x local.get $impulse_y
										i32.const 1 call $hit_asteroid
										br $bullet_done))
								local.get $asteroid_index i32.const 1 i32.add
								local.set $asteroid_index
								br $next_asteroid))))
				local.get $bullet_index i32.const 1 i32.add local.set $bullet_index
				br $next_bullet)))

	(func $ship_hits (param $asteroid i32) (result i32)
		(local $dx f32) (local $dy f32) (local $radius f32)
		local.get $asteroid i32.load
		(if (result i32)
			(then
				i32.const 1040 f32.load
				local.get $asteroid i32.const 12 i32.add f32.load f32.sub
				local.set $dx
				i32.const 1044 f32.load
				local.get $asteroid i32.const 16 i32.add f32.load f32.sub
				local.set $dy
				local.get $asteroid i32.const 28 i32.add f32.load
				f32.const 10 f32.add local.set $radius
				local.get $dx local.get $dx f32.mul
				local.get $dy local.get $dy f32.mul f32.add
				local.get $radius local.get $radius f32.mul
				f32.lt)
			(else i32.const 0)))

	(func $find_ship_collision (result i32)
		(local $index i32) (local $address i32)
		(block $none
			(loop $again
				local.get $index i32.const 32 i32.ge_u br_if $none
				local.get $index call $asteroid_address local.set $address
				local.get $address call $ship_hits
				(if (then local.get $address return))
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		i32.const 0)

	(func $begin_ship_explosion (param $asteroid i32)
		i32.const 1040 f32.load i32.const 1044 f32.load
		i32.const 40 i32.const 1048 f32.load i32.const 1052 f32.load
		call $spawn_particles
		i32.const 1040 f32.load i32.const 1044 f32.load
		i32.const 1048 f32.load i32.const 1052 f32.load
		call $spawn_debris
		local.get $asteroid f32.const 0 f32.const 0 i32.const 0 call $hit_asteroid
		i32.const 3 f32.const 1 f32.const 1 i32.const 0 call $audio drop
		i32.const 1068 i32.const 1068 i32.load i32.const 1 i32.sub i32.store
		i32.const 1092 i32.const 1 i32.store
		i32.const 1096 i32.const 120 i32.store
		i32.const 1076 i32.const 1076 i32.load i32.const 16 i32.and i32.store)

	(func $update_direction
		(local $flags i32) (local $dx f32) (local $dy f32)
		(local $next_dx f32) (local $next_dy f32)
		i32.const 1076 i32.load local.set $flags
		i32.const 1056 f32.load local.set $dx
		i32.const 1060 f32.load local.set $dy
		local.get $flags i32.const 1 i32.and
		(if
			(then
				local.get $dx f32.const 0.9968017 f32.mul
				local.get $dy f32.const 0.0799147 f32.mul f32.add
				local.set $next_dx
				local.get $dy f32.const 0.9968017 f32.mul
				local.get $dx f32.const 0.0799147 f32.mul f32.sub
				local.set $next_dy
				local.get $next_dx local.set $dx
				local.get $next_dy local.set $dy))
		local.get $flags i32.const 2 i32.and
		(if
			(then
				local.get $dx f32.const 0.9968017 f32.mul
				local.get $dy f32.const 0.0799147 f32.mul f32.sub
				local.set $next_dx
				local.get $dy f32.const 0.9968017 f32.mul
				local.get $dx f32.const 0.0799147 f32.mul f32.add
				local.set $next_dy
				local.get $next_dx local.set $dx
				local.get $next_dy local.set $dy))
		i32.const 1056 local.get $dx f32.store
		i32.const 1060 local.get $dy f32.store)

	(func $fire_delay (result i32)
		(local $level i32)
		i32.const 1072 i32.load local.set $level
		local.get $level i32.const 20 i32.ge_u
		(if (result i32)
			(then i32.const 8)
			(else
				i32.const 15
				local.get $level i32.const 1 i32.sub
				i32.const 7 i32.mul i32.const 19 i32.div_u
				i32.sub)))

	(func $fire_if_ready
		(local $address i32) (local $speed f32)
		i32.const 1076 i32.load i32.const 8 i32.and i32.eqz
		(if (then return))
		i32.const 1024 i32.load i32.const 1080 i32.load i32.sub
		call $fire_delay i32.lt_s
		(if (then return))
		call $find_free_bullet local.tee $address i32.eqz
		(if (then return))
		i32.const 1072 i32.load i32.const 1 i32.sub
		f32.convert_i32_s f32.const 0.08 f32.mul f32.const 7 f32.add
		local.set $speed
		local.get $address i32.const 1 i32.store
		local.get $address i32.const 4 i32.add
		i32.const 1040 f32.load i32.const 1056 f32.load f32.const 20 f32.mul f32.add
		f32.store
		local.get $address i32.const 8 i32.add
		i32.const 1044 f32.load i32.const 1060 f32.load f32.const 20 f32.mul f32.add
		f32.store
		local.get $address i32.const 12 i32.add
		i32.const 1048 f32.load i32.const 1056 f32.load local.get $speed f32.mul f32.add
		f32.store
		local.get $address i32.const 16 i32.add
		i32.const 1052 f32.load i32.const 1060 f32.load local.get $speed f32.mul f32.add
		f32.store
		local.get $address i32.const 20 i32.add i32.const 90 i32.store
		i32.const 1080 i32.const 1024 i32.load i32.store
		i32.const 1 f32.const 1 f32.const 1 i32.const 0 call $audio drop)

	(func $update_live_ship
		(local $flags i32) (local $dx f32) (local $dy f32)
		i32.const 1076 i32.load local.set $flags
		i32.const 1056 f32.load local.set $dx
		i32.const 1060 f32.load local.set $dy
		local.get $flags i32.const 4 i32.and
		(if
			(then
				i32.const 1048
				i32.const 1048 f32.load local.get $dx f32.const 0.12 f32.mul f32.add
				f32.store
				i32.const 1052
				i32.const 1052 f32.load local.get $dy f32.const 0.12 f32.mul f32.add
				f32.store
				i32.const 1024 i32.load i32.const 3 i32.and i32.eqz
				(if
					(then i32.const 4 f32.const 0.18 f32.const 1 i32.const 0 call $audio drop))))
		i32.const 1048 i32.const 1048 f32.load f32.const 0.995 f32.mul f32.store
		i32.const 1052 i32.const 1052 f32.load f32.const 0.995 f32.mul f32.store
		i32.const 1040
		i32.const 1040 f32.load i32.const 1048 f32.load f32.add call $wrap_ship_x
		f32.store
		i32.const 1044
		i32.const 1044 f32.load i32.const 1052 f32.load f32.add call $wrap_ship_y
		f32.store
		call $fire_if_ready)

	(func $update_bullets
		(local $index i32) (local $address i32)
		(block $done
			(loop $again
				local.get $index i32.const 64 i32.ge_u br_if $done
				local.get $index call $bullet_address local.set $address
				local.get $address i32.load
				(if
					(then
						local.get $address i32.const 4 i32.add
						local.get $address i32.const 4 i32.add f32.load
						local.get $address i32.const 12 i32.add f32.load f32.add
						call $wrap_bullet_x f32.store
						local.get $address i32.const 8 i32.add
						local.get $address i32.const 8 i32.add f32.load
						local.get $address i32.const 16 i32.add f32.load f32.add
						call $wrap_bullet_y f32.store
						local.get $address i32.const 20 i32.add
						local.get $address i32.const 20 i32.add i32.load i32.const 1 i32.sub
						i32.store
						local.get $address i32.const 20 i32.add i32.load i32.const 0 i32.le_s
						(if (then local.get $address i32.const 0 i32.store))))
				local.get $index i32.const 1 i32.add local.set $index
				br $again)))

	(func $update_particles
		(local $index i32) (local $address i32)
		(block $done
			(loop $again
				local.get $index i32.const 150 i32.ge_u br_if $done
				local.get $index call $particle_address local.set $address
				local.get $address i32.load
				(if
					(then
						local.get $address i32.const 4 i32.add
						local.get $address i32.const 4 i32.add f32.load
						local.get $address i32.const 12 i32.add f32.load f32.add
						f32.store
						local.get $address i32.const 8 i32.add
						local.get $address i32.const 8 i32.add f32.load
						local.get $address i32.const 16 i32.add f32.load f32.add
						f32.store
						local.get $address i32.const 12 i32.add
						local.get $address i32.const 12 i32.add f32.load f32.const 0.98 f32.mul
						f32.store
						local.get $address i32.const 16 i32.add
						local.get $address i32.const 16 i32.add f32.load f32.const 0.98 f32.mul
						f32.store
						local.get $address i32.const 20 i32.add
						local.get $address i32.const 20 i32.add i32.load i32.const 1 i32.sub
						i32.store
						local.get $address i32.const 20 i32.add i32.load i32.const 0 i32.le_s
						(if (then local.get $address i32.const 0 i32.store))))
				local.get $index i32.const 1 i32.add local.set $index
				br $again)))

	(func $update_debris
		(local $index i32) (local $address i32)
		(local $dx f32) (local $dy f32) (local $next_dx f32) (local $next_dy f32)
		(block $done
			(loop $again
				local.get $index i32.const 4 i32.ge_u br_if $done
				local.get $index call $debris_address local.set $address
				local.get $address i32.load
				(if
					(then
						local.get $address i32.const 4 i32.add
						local.get $address i32.const 4 i32.add f32.load
						local.get $address i32.const 12 i32.add f32.load f32.add
						call $wrap_asteroid_x f32.store
						local.get $address i32.const 8 i32.add
						local.get $address i32.const 8 i32.add f32.load
						local.get $address i32.const 16 i32.add f32.load f32.add
						call $wrap_asteroid_y f32.store
						local.get $address i32.const 12 i32.add
						local.get $address i32.const 12 i32.add f32.load f32.const 0.99 f32.mul
						f32.store
						local.get $address i32.const 16 i32.add
						local.get $address i32.const 16 i32.add f32.load f32.const 0.99 f32.mul
						f32.store
						local.get $address i32.const 20 i32.add f32.load local.set $dx
						local.get $address i32.const 24 i32.add f32.load local.set $dy
						local.get $address i32.const 28 i32.add i32.load i32.const 0 i32.gt_s
						(if
							(then
								local.get $dx f32.const 0.9992 f32.mul
								local.get $dy f32.const 0.0399893 f32.mul f32.sub
								local.set $next_dx
								local.get $dx f32.const 0.0399893 f32.mul
								local.get $dy f32.const 0.9992 f32.mul f32.add
								local.set $next_dy)
							(else
								local.get $dx f32.const 0.9992 f32.mul
								local.get $dy f32.const 0.0399893 f32.mul f32.add
								local.set $next_dx
								local.get $dy f32.const 0.9992 f32.mul
								local.get $dx f32.const 0.0399893 f32.mul f32.sub
								local.set $next_dy))
						local.get $address i32.const 20 i32.add local.get $next_dx f32.store
						local.get $address i32.const 24 i32.add local.get $next_dy f32.store
						local.get $address i32.const 32 i32.add
						local.get $address i32.const 32 i32.add i32.load i32.const 1 i32.sub
						i32.store
						local.get $address i32.const 32 i32.add i32.load i32.const 0 i32.le_s
						(if (then local.get $address i32.const 0 i32.store))))
				local.get $index i32.const 1 i32.add local.set $index
				br $again)))

	(func $update_asteroids
		(local $index i32) (local $address i32)
		(local $dx f32) (local $dy f32) (local $next_dx f32) (local $next_dy f32)
		(block $done
			(loop $again
				local.get $index i32.const 32 i32.ge_u br_if $done
				local.get $index call $asteroid_address local.set $address
				local.get $address i32.load
				(if
					(then
						local.get $address i32.const 12 i32.add
						local.get $address i32.const 12 i32.add f32.load
						local.get $address i32.const 20 i32.add f32.load f32.add
						call $wrap_asteroid_x f32.store
						local.get $address i32.const 16 i32.add
						local.get $address i32.const 16 i32.add f32.load
						local.get $address i32.const 24 i32.add f32.load f32.add
						call $wrap_asteroid_y f32.store
						local.get $address i32.const 32 i32.add f32.load local.set $dx
						local.get $address i32.const 36 i32.add f32.load local.set $dy
						local.get $address i32.const 40 i32.add i32.load i32.const 0 i32.gt_s
						(if
							(then
								local.get $dx f32.const 0.999928 f32.mul
								local.get $dy f32.const 0.0119997 f32.mul f32.sub
								local.set $next_dx
								local.get $dx f32.const 0.0119997 f32.mul
								local.get $dy f32.const 0.999928 f32.mul f32.add
								local.set $next_dy)
							(else
								local.get $dx f32.const 0.999928 f32.mul
								local.get $dy f32.const 0.0119997 f32.mul f32.add
								local.set $next_dx
								local.get $dy f32.const 0.999928 f32.mul
								local.get $dx f32.const 0.0119997 f32.mul f32.sub
								local.set $next_dy))
						local.get $address i32.const 32 i32.add local.get $next_dx f32.store
						local.get $address i32.const 36 i32.add local.get $next_dy f32.store))
				local.get $index i32.const 1 i32.add local.set $index
				br $again)))

	(func $center_clear (param $safe_radius f32) (result i32)
		(local $index i32) (local $address i32)
		(local $dx f32) (local $dy f32) (local $radius f32)
		(block $clear
			(loop $again
				local.get $index i32.const 32 i32.ge_u br_if $clear
				local.get $index call $asteroid_address local.set $address
				local.get $address i32.load
				(if
					(then
						i32.const 1032 f32.load f32.const 0.5 f32.mul
						local.get $address i32.const 12 i32.add f32.load f32.sub
						local.set $dx
						i32.const 1036 f32.load f32.const 0.5 f32.mul
						local.get $address i32.const 16 i32.add f32.load f32.sub
						local.set $dy
						local.get $safe_radius
						local.get $address i32.const 28 i32.add f32.load f32.add
						local.set $radius
						local.get $dx local.get $dx f32.mul
						local.get $dy local.get $dy f32.mul f32.add
						local.get $radius local.get $radius f32.mul
						f32.lt
						(if (then i32.const 0 return))))
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		i32.const 1)

	(func $bomb_center
		(local $index i32) (local $address i32) (local $hit_any i32)
		(local $dx f32) (local $dy f32) (local $radius f32)
		(block $done
			(loop $again
				local.get $index i32.const 32 i32.ge_u br_if $done
				local.get $index call $asteroid_address local.set $address
				local.get $address i32.load
				(if
					(then
						i32.const 1032 f32.load f32.const 0.5 f32.mul
						local.get $address i32.const 12 i32.add f32.load f32.sub
						local.set $dx
						i32.const 1036 f32.load f32.const 0.5 f32.mul
						local.get $address i32.const 16 i32.add f32.load f32.sub
						local.set $dy
						f32.const 80
						local.get $address i32.const 28 i32.add f32.load f32.add
						local.set $radius
						local.get $dx local.get $dx f32.mul
						local.get $dy local.get $dy f32.mul f32.add
						local.get $radius local.get $radius f32.mul
						f32.lt
						(if
							(then
								local.get $address i32.const 12 i32.add f32.load
								local.get $address i32.const 16 i32.add f32.load
								i32.const 20 f32.const 0 f32.const 0 call $spawn_particles
								local.get $address i32.const 0 i32.store
								i32.const 1 local.set $hit_any))))
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		local.get $hit_any
		(if (then i32.const 2 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))

	(func $advance_lifecycle
		(local $lifecycle i32) (local $ticks i32) (local $safe_radius f32)
		i32.const 1092 i32.load local.set $lifecycle
		local.get $lifecycle i32.const 1 i32.eq
		(if
			(then
				i32.const 1096 i32.const 1096 i32.load i32.const 1 i32.sub i32.store
				i32.const 1096 i32.load i32.const 0 i32.le_s
				(if
					(then
						i32.const 1068 i32.load i32.const 0 i32.le_s
						(if
							(then i32.const 1092 i32.const 3 i32.store)
							(else
								i32.const 1092 i32.const 2 i32.store
								i32.const 1096 i32.const 0 i32.store
								i32.const 1040 i32.const 1032 f32.load f32.const 0.5 f32.mul f32.store
								i32.const 1044 i32.const 1036 f32.load f32.const 0.5 f32.mul f32.store
								i32.const 1048 f32.const 0 f32.store
								i32.const 1052 f32.const 0 f32.store))))))
		i32.const 1092 i32.load i32.const 2 i32.eq
		(if
			(then
				i32.const 1096 i32.const 1096 i32.load i32.const 1 i32.add i32.store
				i32.const 1096 i32.load local.set $ticks
				local.get $ticks i32.const 600 i32.eq
				(if (then call $bomb_center))
				local.get $ticks i32.const 300 i32.gt_u
				(if (result f32) (then f32.const 48) (else f32.const 96))
				local.set $safe_radius
				local.get $safe_radius call $center_clear
				(if
					(then
						i32.const 1092 i32.const 0 i32.store
						i32.const 1096 i32.const 0 i32.store
						i32.const 1084 i32.const 120 i32.store
						i32.const 1100 i32.const 60 i32.store)))))

	(func $step
		(local $lifecycle i32) (local $collision i32)
		i32.const 1024 i32.const 1024 i32.load i32.const 1 i32.add i32.store
		i32.const 1076 i32.load i32.const 16 i32.and
		(if (then return))
		i32.const 1092 i32.load local.tee $lifecycle i32.const 3 i32.eq
		(if (then return))
		local.get $lifecycle i32.const 0 i32.eq
		local.get $lifecycle i32.const 2 i32.eq i32.or
		(if (then call $update_direction))
		local.get $lifecycle i32.const 0 i32.eq
		(if
			(then call $update_live_ship)
			(else
				local.get $lifecycle i32.const 2 i32.eq
				(if
					(then
						i32.const 1040 i32.const 1032 f32.load f32.const 0.5 f32.mul f32.store
						i32.const 1044 i32.const 1036 f32.load f32.const 0.5 f32.mul f32.store
						i32.const 1048 f32.const 0 f32.store
						i32.const 1052 f32.const 0 f32.store))))
		call $update_bullets
		call $update_particles
		call $update_debris
		call $update_asteroids
		call $resolve_bullet_collisions
		i32.const 1092 i32.load i32.eqz
		(if
			(then
				i32.const 1084 i32.load i32.const 0 i32.gt_s
				(if
					(then
						i32.const 1084 i32.const 1084 i32.load i32.const 1 i32.sub i32.store)
					(else
						call $find_ship_collision local.set $collision
						local.get $collision
						(if (then local.get $collision call $begin_ship_explosion))))))
		call $advance_lifecycle
		call $asteroid_count i32.eqz
		i32.const 1092 i32.load i32.const 3 i32.ne
		i32.and
		(if
			(then
				i32.const 1072 i32.const 1072 i32.load i32.const 1 i32.add i32.store
				call $spawn_wave))
		i32.const 1100 i32.load i32.const 0 i32.gt_s
		(if (then i32.const 1100 i32.const 1100 i32.load i32.const 1 i32.sub i32.store)))

	(func (export "fp_tick") (param $count i32) (result i32)
		(local $index i32)
		(block $done
			(loop $again
				local.get $index local.get $count i32.ge_u br_if $done
				call $step
				local.get $index i32.const 1 i32.add local.set $index
				br $again))
		i32.const 1 i32.const 0 i32.const 0 call $effect drop
		i32.const 0)

	(func (export "fp_event") (param $kind i32) (param $code i32)
		(param $a f32) (param $b f32) (result i32)
		(local $mask i32)
		local.get $kind i32.const 1 i32.eq
		(if
			(then
				local.get $code i32.const 1 i32.eq (if (then i32.const 1 local.set $mask))
				local.get $code i32.const 2 i32.eq (if (then i32.const 2 local.set $mask))
				local.get $code i32.const 3 i32.eq (if (then i32.const 4 local.set $mask))
				local.get $code i32.const 4 i32.eq (if (then i32.const 8 local.set $mask))
				local.get $mask i32.eqz
				(if
					(then
						local.get $code i32.const 5 i32.eq
						(if
							(then
								i32.const 1076
								i32.const 1076 i32.load i32.const 16 i32.xor
								i32.store))
						local.get $code i32.const 6 i32.eq
						(if
							(then
								i32.const 1104 i32.load
								i32.const 1032 f32.load i32.const 1036 f32.load
								call $reset)))
					(else
						i32.const 1076
						i32.const 1076 i32.load local.get $mask i32.or
						i32.store))))
		local.get $kind i32.const 2 i32.eq
		(if
			(then
				local.get $code i32.const 1 i32.eq (if (then i32.const 1 local.set $mask))
				local.get $code i32.const 2 i32.eq (if (then i32.const 2 local.set $mask))
				local.get $code i32.const 3 i32.eq (if (then i32.const 4 local.set $mask))
				local.get $code i32.const 4 i32.eq (if (then i32.const 8 local.set $mask))
				i32.const 1076
				i32.const 1076 i32.load local.get $mask i32.const -1 i32.xor i32.and
				i32.store))
		local.get $kind i32.const 6 i32.eq
		(if
			(then
				i32.const 1032 local.get $a f32.store
				i32.const 1036 local.get $b f32.store))
		local.get $kind i32.const 7 i32.eq
		(if
			(then
				local.get $code i32.const 1 i32.eq
				(if
					(then
						i32.const 1104 i32.load
						i32.const 1032 f32.load i32.const 1036 f32.load
						call $reset))
				local.get $code i32.const 6 i32.eq
				(if (then i32.const 2 i32.const 0 i32.const 0 call $effect drop))))
		local.get $kind i32.const 8 i32.eq
		local.get $code i32.eqz i32.and
		(if
			(then
				i32.const 1076
				i32.const 1076 i32.load i32.const 16 i32.and
				i32.store))
		i32.const 0)

	(func $rock_scale (param $address i32) (param $vertex i32) (result f32)
		local.get $address i32.const 8 i32.add i32.load
		local.get $vertex i32.const 3 i32.mul
		i32.add i32.const 7 i32.and
		f32.convert_i32_u f32.const 0.06 f32.mul
		f32.const 0.68 f32.add)

	(func $emit_rock_point (param $address i32) (param $vertex i32) (param $first i32)
		(local $ux f32) (local $uy f32) (local $scale f32) (local $radius f32)
		local.get $vertex i32.const 0 i32.eq
		(if (then f32.const 1 local.set $ux))
		local.get $vertex i32.const 1 i32.eq
		(if (then f32.const 0.809017 local.set $ux f32.const 0.5877852 local.set $uy))
		local.get $vertex i32.const 2 i32.eq
		(if (then f32.const 0.309017 local.set $ux f32.const 0.9510565 local.set $uy))
		local.get $vertex i32.const 3 i32.eq
		(if (then f32.const -0.309017 local.set $ux f32.const 0.9510565 local.set $uy))
		local.get $vertex i32.const 4 i32.eq
		(if (then f32.const -0.809017 local.set $ux f32.const 0.5877852 local.set $uy))
		local.get $vertex i32.const 5 i32.eq
		(if (then f32.const -1 local.set $ux))
		local.get $vertex i32.const 6 i32.eq
		(if (then f32.const -0.809017 local.set $ux f32.const -0.5877852 local.set $uy))
		local.get $vertex i32.const 7 i32.eq
		(if (then f32.const -0.309017 local.set $ux f32.const -0.9510565 local.set $uy))
		local.get $vertex i32.const 8 i32.eq
		(if (then f32.const 0.309017 local.set $ux f32.const -0.9510565 local.set $uy))
		local.get $vertex i32.const 9 i32.eq
		(if (then f32.const 0.809017 local.set $ux f32.const -0.5877852 local.set $uy))
		local.get $address i32.const 28 i32.add f32.load local.set $radius
		local.get $address local.get $vertex call $rock_scale local.set $scale
		local.get $first
		(if
			(then
				local.get $ux local.get $radius f32.mul local.get $scale f32.mul
				local.get $uy local.get $radius f32.mul local.get $scale f32.mul
				call $path_move drop)
			(else
				local.get $ux local.get $radius f32.mul local.get $scale f32.mul
				local.get $uy local.get $radius f32.mul local.get $scale f32.mul
				call $path_line drop)))

	(func $draw_ship
		(param $id i32) (param $x f32) (param $y f32)
		(param $dx f32) (param $dy f32) (param $scale f32)
		(param $color i32) (param $flame i32)
		local.get $dx local.get $scale f32.mul
		local.get $dy local.get $scale f32.mul
		local.get $dy f32.neg local.get $scale f32.mul
		local.get $dx local.get $scale f32.mul
		local.get $x local.get $y call $transform_push drop
		local.get $id call $path_begin drop
		f32.const 15 f32.const 0 call $path_move drop
		f32.const -10 f32.const -8 call $path_line drop
		f32.const -5 f32.const 0 call $path_line drop
		f32.const -10 f32.const 8 call $path_line drop
		call $path_close drop
		f32.const 2 i32.const 0 local.get $color i32.const 0 call $path_end drop
		local.get $id i32.const 1000 i32.add
		f32.const 0 f32.const 0 f32.const 3 f32.const 0
		local.get $color i32.const 1 call $circle drop
		local.get $flame
		(if
			(then
				local.get $id i32.const 3 i32.add call $path_begin drop
				f32.const -5 f32.const -3 call $path_move drop
				i32.const 1024 i32.load i32.const 7 i32.and
				f32.convert_i32_u f32.const 2 f32.mul f32.const 12 f32.add f32.neg
				f32.const 0 call $path_line drop
				f32.const -5 f32.const 3 call $path_line drop
				f32.const 2 i32.const 0 i32.const 0xff8a2bff i32.const 0 call $path_end drop))
		call $transform_pop drop)

	(func $draw_asteroid (param $index i32) (param $address i32)
		(local $radius f32)
		local.get $address i32.load
		(if
			(then
				local.get $address i32.const 28 i32.add f32.load local.set $radius
				local.get $address i32.const 32 i32.add f32.load
				local.get $address i32.const 36 i32.add f32.load
				local.get $address i32.const 36 i32.add f32.load f32.neg
				local.get $address i32.const 32 i32.add f32.load
				local.get $address i32.const 12 i32.add f32.load
				local.get $address i32.const 16 i32.add f32.load
				call $transform_push drop
				i32.const 200 local.get $index i32.add call $path_begin drop
				local.get $address i32.const 0 i32.const 1 call $emit_rock_point
				local.get $address i32.const 1 i32.const 0 call $emit_rock_point
				local.get $address i32.const 2 i32.const 0 call $emit_rock_point
				local.get $address i32.const 3 i32.const 0 call $emit_rock_point
				local.get $address i32.const 4 i32.const 0 call $emit_rock_point
				local.get $address i32.const 5 i32.const 0 call $emit_rock_point
				local.get $address i32.const 6 i32.const 0 call $emit_rock_point
				local.get $address i32.const 7 i32.const 0 call $emit_rock_point
				local.get $address i32.const 8 i32.const 0 call $emit_rock_point
				local.get $address i32.const 9 i32.const 0 call $emit_rock_point
				call $path_close drop
				f32.const 2 i32.const 0 i32.const 0x8894a8ff i32.const 0 call $path_end drop
				i32.const 300 local.get $index i32.add
				local.get $radius f32.const -0.55 f32.mul f32.const 0
				local.get $radius f32.const 0.45 f32.mul
				local.get $radius f32.const 0.25 f32.mul
				f32.const 1 i32.const 0x536178ff call $line drop
				call $transform_pop drop)))

	(func $draw_debris (param $index i32) (param $address i32)
		(local $piece i32)
		local.get $address i32.load
		(if
			(then
				local.get $address i32.const 36 i32.add i32.load local.set $piece
				local.get $address i32.const 20 i32.add f32.load
				local.get $address i32.const 24 i32.add f32.load
				local.get $address i32.const 24 i32.add f32.load f32.neg
				local.get $address i32.const 20 i32.add f32.load
				local.get $address i32.const 4 i32.add f32.load
				local.get $address i32.const 8 i32.add f32.load
				call $transform_push drop
				i32.const 700 local.get $index i32.add call $path_begin drop
				local.get $piece i32.const 0 i32.eq
				(if
					(then
						f32.const 15 f32.const 0 call $path_move drop
						f32.const 5 f32.const 0 call $path_line drop
						f32.const 0 f32.const -3 call $path_line drop)
					(else
						local.get $piece i32.const 1 i32.eq
						(if
							(then
								f32.const -10 f32.const -8 call $path_move drop
								f32.const -5 f32.const 0 call $path_line drop
								f32.const 0 f32.const -3 call $path_line drop)
							(else
								local.get $piece i32.const 2 i32.eq
								(if
									(then
										f32.const -10 f32.const 8 call $path_move drop
										f32.const -5 f32.const 0 call $path_line drop
										f32.const 0 f32.const 3 call $path_line drop)
									(else
										f32.const -5 f32.const 0 call $path_move drop
										f32.const 3 f32.const -3 call $path_line drop
									f32.const 3 f32.const 3 call $path_line drop))))))
				call $path_close drop
				f32.const 2 i32.const 0 i32.const 0xffffffff i32.const 0 call $path_end drop
				call $transform_pop drop)))

	(func $draw_stars
		(local $index i32) (local $x f32) (local $y f32)
		(block $done
			(loop $again
				local.get $index i32.const 48 i32.ge_u br_if $done
				local.get $index i32.const 83 i32.mul i32.const 47 i32.add
				i32.const 1000 i32.rem_u
				f32.convert_i32_u f32.const 1000 f32.div
				i32.const 1032 f32.load f32.mul local.set $x
				local.get $index i32.const 47 i32.mul i32.const 29 i32.add
				i32.const 1000 i32.rem_u
				f32.convert_i32_u f32.const 1000 f32.div
				i32.const 1036 f32.load f32.mul local.set $y
				i32.const 800 local.get $index i32.add
				local.get $x local.get $y
				f32.const 0.85 f32.const 0 i32.const 0x9bb8d199 i32.const 1
				call $circle drop
				local.get $index i32.const 1 i32.add local.set $index
				br $again)))

	(func (export "fp_render") (result i32)
		(local $index i32) (local $address i32) (local $reserve_count i32)
		f32.const 0.03137255 f32.const 0.04313725 f32.const 0.07058824 f32.const 1
		call $frame_begin drop
		call $draw_stars
		i32.const 160 i32.const 1064 i32.load call $write_six_digits
		i32.const 168 i32.const 1072 i32.load call $write_two_digits

		i32.const 10 i32.const 64 i32.const 12
		i32.const 1032 f32.load f32.const 0.5 f32.mul f32.const 28 f32.const 28
		i32.const 0x5ee7ffff i32.const 1 call $text drop
		i32.const 11 i32.const 80 i32.const 16
		i32.const 1032 f32.load f32.const 0.5 f32.mul
		i32.const 1036 f32.load f32.const 18 f32.sub
		f32.const 13 i32.const 0x7f91a8ff i32.const 1 call $text drop

		i32.const 20 i32.const 128 i32.const 5
		f32.const 24 f32.const 24 f32.const 15
		i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 21 i32.const 160 i32.const 6
		f32.const 24 f32.const 48 f32.const 22
		i32.const 0x58ff72ff i32.const 0 call $text drop
		i32.const 22 i32.const 136 i32.const 5
		i32.const 1032 f32.load f32.const 0.5 f32.mul f32.const 50 f32.sub
		f32.const 54 f32.const 15 i32.const 0x9bb8d1ff i32.const 1 call $text drop
		i32.const 23 i32.const 168 i32.const 2
		i32.const 1032 f32.load f32.const 0.5 f32.mul f32.const 30 f32.add
		f32.const 54 f32.const 22 i32.const 0xffcf5cff i32.const 1 call $text drop
		i32.const 24 i32.const 144 i32.const 5
		i32.const 1032 f32.load f32.const 62 f32.sub
		f32.const 24 f32.const 15 i32.const 0x9bb8d1ff i32.const 1 call $text drop

		i32.const 1068 i32.load i32.const 1 i32.sub local.set $reserve_count
		local.get $reserve_count i32.const 5 i32.gt_u
		(if (then i32.const 5 local.set $reserve_count))
		i32.const 0 local.set $index
		(block $reserves_done
			(loop $next_reserve
				local.get $index local.get $reserve_count i32.ge_u br_if $reserves_done
				i32.const 50 local.get $index i32.add
				i32.const 1032 f32.load f32.const 40 f32.sub
				local.get $index f32.convert_i32_u f32.const 28 f32.mul f32.sub
				f32.const 48 f32.const 1 f32.const 0 f32.const 0.55
				i32.const 0x58ff72ff i32.const 0 call $draw_ship
				local.get $index i32.const 1 i32.add local.set $index
				br $next_reserve))

		i32.const 1092 i32.load local.set $index
		local.get $index i32.eqz
		(if
			(then
				i32.const 1084 i32.load i32.eqz
				i32.const 1024 i32.load i32.const 8 i32.and i32.eqz
				i32.or
				(if
					(then
						i32.const 1 i32.const 1040 f32.load i32.const 1044 f32.load
						i32.const 1056 f32.load i32.const 1060 f32.load f32.const 1
						i32.const -1
						i32.const 1076 i32.load i32.const 4 i32.and i32.eqz i32.eqz
						call $draw_ship)))
			(else
				local.get $index i32.const 2 i32.eq
				(if
					(then
						i32.const 1 i32.const 1040 f32.load i32.const 1044 f32.load
						i32.const 1056 f32.load i32.const 1060 f32.load f32.const 1
						i32.const 0x777f8c99 i32.const 0 call $draw_ship))))

		i32.const 0 local.set $index
		(block $bullets_done
			(loop $next_bullet
				local.get $index i32.const 64 i32.ge_u br_if $bullets_done
				local.get $index call $bullet_address local.set $address
				local.get $address i32.load
				(if
					(then
						i32.const 100 local.get $index i32.add
						local.get $address i32.const 4 i32.add f32.load
						local.get $address i32.const 8 i32.add f32.load
						f32.const 2 f32.const 0 i32.const 0x58ff72ff i32.const 1
						call $circle drop))
				local.get $index i32.const 1 i32.add local.set $index
				br $next_bullet))

		i32.const 0 local.set $index
		(block $asteroids_done
			(loop $next_asteroid
				local.get $index i32.const 32 i32.ge_u br_if $asteroids_done
				local.get $index call $asteroid_address local.set $address
				local.get $index local.get $address call $draw_asteroid
				local.get $index i32.const 1 i32.add local.set $index
				br $next_asteroid))

		i32.const 0 local.set $index
		(block $particles_done
			(loop $next_particle
				local.get $index i32.const 150 i32.ge_u br_if $particles_done
				local.get $index call $particle_address local.set $address
				local.get $address i32.load
				(if
					(then
						i32.const 500 local.get $index i32.add
						local.get $address i32.const 4 i32.add f32.load
						local.get $address i32.const 8 i32.add f32.load
						f32.const 2 f32.const 0 i32.const 0xff9b2fff i32.const 1
						call $circle drop))
				local.get $index i32.const 1 i32.add local.set $index
				br $next_particle))

		i32.const 0 local.set $index
		(block $debris_done
			(loop $next_debris
				local.get $index i32.const 4 i32.ge_u br_if $debris_done
				local.get $index call $debris_address local.set $address
				local.get $index local.get $address call $draw_debris
				local.get $index i32.const 1 i32.add local.set $index
				br $next_debris))

		i32.const 1100 i32.load i32.const 0 i32.gt_s
		(if
			(then
				i32.const 30 i32.const 176 i32.const 4
				i32.const 1032 f32.load f32.const 0.5 f32.mul f32.const 35 f32.sub
				i32.const 1036 f32.load f32.const 0.68 f32.mul
				f32.const 24 i32.const 0x5ee7ffff i32.const 1 call $text drop
				i32.const 31 i32.const 168 i32.const 2
				i32.const 1032 f32.load f32.const 0.5 f32.mul f32.const 36 f32.add
				i32.const 1036 f32.load f32.const 0.68 f32.mul
				f32.const 24 i32.const 0xffcf5cff i32.const 1 call $text drop))
		i32.const 1092 i32.load i32.const 1 i32.eq
		(if
			(then
				i32.const 32 i32.const 184 i32.const 14
				i32.const 1032 f32.load f32.const 0.5 f32.mul
				i32.const 1036 f32.load f32.const 0.5 f32.mul
				f32.const 26 i32.const 0xff8a2bff i32.const 1 call $text drop))
		i32.const 1076 i32.load i32.const 16 i32.and
		(if
			(then
				i32.const 33 i32.const 100 i32.const 6
				i32.const 1032 f32.load f32.const 0.5 f32.mul
				i32.const 1036 f32.load f32.const 0.5 f32.mul
				f32.const 36 i32.const 0xffcf5cff i32.const 1 call $text drop))
		i32.const 1092 i32.load i32.const 3 i32.eq
		(if
			(then
				i32.const 34 i32.const 108 i32.const 9
				i32.const 1032 f32.load f32.const 0.5 f32.mul
				i32.const 1036 f32.load f32.const 0.5 f32.mul
				f32.const 40 i32.const 0xff5c73ff i32.const 1 call $text drop))
		call $frame_end drop
		i32.const 0)
)
