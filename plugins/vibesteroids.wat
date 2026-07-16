;; Vibesteroids behavioral conversion for the gpui-frontplane-v0 ABI.
;;
;; Canonical snapshot state occupies [1024, 1280). No gameplay state lives in
;; globals: host snapshot/restore therefore captures the whole simulation.
;; Increment fp_state_schema whenever this layout or its meaning becomes
;; incompatible; equal schema and byte length deliberately authorize live
;; state transfer into a replacement module.
;;
;; 1024 tick:i32        1028 seed:i32       1032 width:f32
;; 1036 height:f32      1040 ship.x:f32     1044 ship.y:f32
;; 1048 ship.vx:f32     1052 ship.vy:f32    1056 ship.dx:f32
;; 1060 ship.dy:f32     1064 score:i32      1068 lives:i32
;; 1072 level:i32       1076 flags:i32      1080 last-fire:i32
;; 1084 bullet.active   1088..1100 bullet x/y/vx/vy
;; 1104 bullet.life     1108 invulnerability ticks
;; 1120,1144,1168: asteroid x/y/vx/vy/radius:f32, active:i32
(module
	(import "host.v0" "title" (func $title (param i32 i32) (result i32)))
	(import "host.v0" "menu_item" (func $menu_item (param i32 i32 i32 i32 i32) (result i32)))
	(import "host.v0" "frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
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

	(func (export "fp_abi_major") (result i32) i32.const 0)
	(func (export "fp_abi_minor") (result i32) i32.const 0)
	(func (export "fp_state_ptr") (result i32) i32.const 1024)
	(func (export "fp_state_len") (result i32) i32.const 256)
	(func (export "fp_state_schema") (result i32) i32.const 1)

	(func (export "fp_configure") (result i32)
		i32.const 0 i32.const 20 call $title drop
		i32.const 1 i32.const 32 i32.const 8 i32.const 1 i32.const 0 call $menu_item drop
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 i32.const 1 call $menu_item drop
		i32.const 6 i32.const 48 i32.const 4 i32.const 6 i32.const 0 call $menu_item drop
		i32.const 0)

	;; Populate the three fixed asteroid slots from the current viewport.
	(func $spawn_asteroids
		;; upper-left
		i32.const 1120
		i32.const 1032 f32.load f32.const 0.15 f32.mul
		i32.const 1028 i32.load i32.const 31 i32.and f32.convert_i32_u f32.const 2 f32.mul
		f32.add f32.store
		i32.const 1124
		i32.const 1036 f32.load f32.const 0.20 f32.mul f32.store
		i32.const 1128 f32.const 1.20 f32.store
		i32.const 1132 f32.const 0.70 f32.store
		i32.const 1136 f32.const 30 f32.store
		i32.const 1140 i32.const 1 i32.store
		;; upper-right
		i32.const 1144
		i32.const 1032 f32.load f32.const 0.82 f32.mul f32.store
		i32.const 1148
		i32.const 1036 f32.load f32.const 0.25 f32.mul f32.store
		i32.const 1152 f32.const -0.90 f32.store
		i32.const 1156 f32.const 1.10 f32.store
		i32.const 1160 f32.const 26 f32.store
		i32.const 1164 i32.const 1 i32.store
		;; lower-right
		i32.const 1168
		i32.const 1032 f32.load f32.const 0.72 f32.mul f32.store
		i32.const 1172
		i32.const 1036 f32.load f32.const 0.78 f32.mul f32.store
		i32.const 1176 f32.const -1.00 f32.store
		i32.const 1180 f32.const -0.80 f32.store
		i32.const 1184 f32.const 34 f32.store
		i32.const 1188 i32.const 1 i32.store)

	(func $reset (param $seed i32) (param $width f32) (param $height f32)
		i32.const 1024 i32.const 0 i32.store
		i32.const 1028 local.get $seed i32.store
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
		i32.const 1084 i32.const 0 i32.store
		i32.const 1088 f32.const 0 f32.store
		i32.const 1092 f32.const 0 f32.store
		i32.const 1096 f32.const 0 f32.store
		i32.const 1100 f32.const 0 f32.store
		i32.const 1104 i32.const 0 i32.store
		i32.const 1108 i32.const 120 i32.store
		call $spawn_asteroids)

	(func (export "fp_init") (param $seed_lo i32) (param $seed_hi i32)
		(param $width f32) (param $height f32) (result i32)
		local.get $seed_lo local.get $width local.get $height call $reset
		i32.const 0)

	(func $wrap_x (param $value f32) (result f32)
		local.get $value f32.const 0 f32.lt
		(if (result f32)
			(then i32.const 1032 f32.load)
			(else
				local.get $value i32.const 1032 f32.load f32.gt
				(if (result f32)
					(then f32.const 0)
					(else local.get $value)))))

	(func $wrap_y (param $value f32) (result f32)
		local.get $value f32.const 0 f32.lt
		(if (result f32)
			(then i32.const 1036 f32.load)
			(else
				local.get $value i32.const 1036 f32.load f32.gt
				(if (result f32)
					(then f32.const 0)
					(else local.get $value)))))

	(func $update_asteroid (param $address i32)
		local.get $address i32.const 20 i32.add i32.load
		(if
			(then
				local.get $address
				local.get $address f32.load
				local.get $address i32.const 8 i32.add f32.load f32.add
				call $wrap_x f32.store
				local.get $address i32.const 4 i32.add
				local.get $address i32.const 4 i32.add f32.load
				local.get $address i32.const 12 i32.add f32.load f32.add
				call $wrap_y f32.store)))

	(func $bullet_hits (param $address i32) (result i32)
		(local $dx f32) (local $dy f32) (local $radius f32)
		i32.const 1084 i32.load
		local.get $address i32.const 20 i32.add i32.load
		i32.and
		(if (result i32)
			(then
				i32.const 1088 f32.load local.get $address f32.load f32.sub local.set $dx
				i32.const 1092 f32.load
				local.get $address i32.const 4 i32.add f32.load f32.sub local.set $dy
				local.get $address i32.const 16 i32.add f32.load
				f32.const 2 f32.add local.set $radius
				local.get $dx local.get $dx f32.mul
				local.get $dy local.get $dy f32.mul f32.add
				local.get $radius local.get $radius f32.mul f32.lt)
			(else i32.const 0)))

	(func $resolve_bullet_hit (param $address i32)
		local.get $address call $bullet_hits
		(if
			(then
				local.get $address i32.const 20 i32.add i32.const 0 i32.store
				i32.const 1084 i32.const 0 i32.store
				i32.const 1064 i32.const 1064 i32.load i32.const 80 i32.add i32.store
				i32.const 2 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))

	(func $ship_hits (param $address i32) (result i32)
		(local $dx f32) (local $dy f32) (local $radius f32)
		local.get $address i32.const 20 i32.add i32.load
		(if (result i32)
			(then
				i32.const 1040 f32.load local.get $address f32.load f32.sub local.set $dx
				i32.const 1044 f32.load
				local.get $address i32.const 4 i32.add f32.load f32.sub local.set $dy
				local.get $address i32.const 16 i32.add f32.load
				f32.const 10 f32.add local.set $radius
				local.get $dx local.get $dx f32.mul
				local.get $dy local.get $dy f32.mul f32.add
				local.get $radius local.get $radius f32.mul f32.lt)
			(else i32.const 0)))

	(func $lose_life
		i32.const 1068 i32.const 1068 i32.load i32.const 1 i32.sub i32.store
		i32.const 3 f32.const 1 f32.const 1 i32.const 0 call $audio drop
		i32.const 1040 i32.const 1032 f32.load f32.const 0.5 f32.mul f32.store
		i32.const 1044 i32.const 1036 f32.load f32.const 0.5 f32.mul f32.store
		i32.const 1048 f32.const 0 f32.store
		i32.const 1052 f32.const 0 f32.store
		i32.const 1108 i32.const 120 i32.store
		i32.const 1068 i32.load i32.const 0 i32.le_s
		(if
			(then i32.const 1076 i32.const 1076 i32.load i32.const 32 i32.or i32.store)))

	(func $step
		(local $flags i32) (local $dx f32) (local $dy f32)
		(local $next_dx f32) (local $next_dy f32)
		i32.const 1024 i32.const 1024 i32.load i32.const 1 i32.add i32.store
		i32.const 1076 i32.load local.tee $flags i32.const 16 i32.and
		(if (then return))
		local.get $flags i32.const 32 i32.and
		(if (then return))

		;; Rotate the direction vector by a fixed 0.08 radians per tick.
		i32.const 1056 f32.load local.set $dx
		i32.const 1060 f32.load local.set $dy
		local.get $flags i32.const 1 i32.and
		(if
			(then
				local.get $dx f32.const 0.9968017 f32.mul
				local.get $dy f32.const 0.0799147 f32.mul f32.add local.set $next_dx
				local.get $dy f32.const 0.9968017 f32.mul
				local.get $dx f32.const 0.0799147 f32.mul f32.sub local.set $next_dy
				local.get $next_dx local.set $dx
				local.get $next_dy local.set $dy))
		local.get $flags i32.const 2 i32.and
		(if
			(then
				local.get $dx f32.const 0.9968017 f32.mul
				local.get $dy f32.const 0.0799147 f32.mul f32.sub local.set $next_dx
				local.get $dy f32.const 0.9968017 f32.mul
				local.get $dx f32.const 0.0799147 f32.mul f32.add local.set $next_dy
				local.get $next_dx local.set $dx
				local.get $next_dy local.set $dy))
		i32.const 1056 local.get $dx f32.store
		i32.const 1060 local.get $dy f32.store

		;; Thrust and drag.
		local.get $flags i32.const 4 i32.and
		(if
			(then
				i32.const 1048 i32.const 1048 f32.load local.get $dx f32.const 0.12 f32.mul f32.add f32.store
				i32.const 1052 i32.const 1052 f32.load local.get $dy f32.const 0.12 f32.mul f32.add f32.store
				i32.const 4 f32.const 0.20 f32.const 1 i32.const 0 call $audio drop))
		i32.const 1048 i32.const 1048 f32.load f32.const 0.995 f32.mul f32.store
		i32.const 1052 i32.const 1052 f32.load f32.const 0.995 f32.mul f32.store
		i32.const 1040 i32.const 1040 f32.load i32.const 1048 f32.load f32.add call $wrap_x f32.store
		i32.const 1044 i32.const 1044 f32.load i32.const 1052 f32.load f32.add call $wrap_y f32.store

		;; Fire one fixed-capacity bullet with a twelve-tick cooldown.
		local.get $flags i32.const 8 i32.and i32.eqz i32.eqz
		i32.const 1084 i32.load i32.eqz i32.and
		i32.const 1024 i32.load i32.const 1080 i32.load i32.sub i32.const 12 i32.ge_s i32.and
		(if
			(then
				i32.const 1084 i32.const 1 i32.store
				i32.const 1088 i32.const 1040 f32.load local.get $dx f32.const 20 f32.mul f32.add f32.store
				i32.const 1092 i32.const 1044 f32.load local.get $dy f32.const 20 f32.mul f32.add f32.store
				i32.const 1096 i32.const 1048 f32.load local.get $dx f32.const 7 f32.mul f32.add f32.store
				i32.const 1100 i32.const 1052 f32.load local.get $dy f32.const 7 f32.mul f32.add f32.store
				i32.const 1104 i32.const 90 i32.store
				i32.const 1080 i32.const 1024 i32.load i32.store
				i32.const 1 f32.const 1 f32.const 1 i32.const 0 call $audio drop))

		i32.const 1084 i32.load
		(if
			(then
				i32.const 1088 i32.const 1088 f32.load i32.const 1096 f32.load f32.add call $wrap_x f32.store
				i32.const 1092 i32.const 1092 f32.load i32.const 1100 f32.load f32.add call $wrap_y f32.store
				i32.const 1104 i32.const 1104 i32.load i32.const 1 i32.sub i32.store
				i32.const 1104 i32.load i32.const 0 i32.le_s
				(if (then i32.const 1084 i32.const 0 i32.store))))

		i32.const 1120 call $update_asteroid
		i32.const 1144 call $update_asteroid
		i32.const 1168 call $update_asteroid
		i32.const 1120 call $resolve_bullet_hit
		i32.const 1144 call $resolve_bullet_hit
		i32.const 1168 call $resolve_bullet_hit

		i32.const 1108 i32.load i32.const 0 i32.gt_s
		(if
			(then i32.const 1108 i32.const 1108 i32.load i32.const 1 i32.sub i32.store)
			(else
				i32.const 1120 call $ship_hits
				i32.const 1144 call $ship_hits i32.or
				i32.const 1168 call $ship_hits i32.or
				(if (then call $lose_life))))

		;; Clearing a wave advances the level and respawns it deterministically.
		i32.const 1140 i32.load
		i32.const 1164 i32.load i32.or
		i32.const 1188 i32.load i32.or i32.eqz
		(if
			(then
				i32.const 1072 i32.const 1072 i32.load i32.const 1 i32.add i32.store
				call $spawn_asteroids)))

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
		;; key down
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
						(if (then i32.const 1076 i32.const 1076 i32.load i32.const 16 i32.xor i32.store))
						local.get $code i32.const 6 i32.eq
						(if
							(then i32.const 1028 i32.load i32.const 1032 f32.load i32.const 1036 f32.load call $reset)))
					(else i32.const 1076 i32.const 1076 i32.load local.get $mask i32.or i32.store))))
		;; key up
		local.get $kind i32.const 2 i32.eq
		(if
			(then
				local.get $code i32.const 1 i32.eq (if (then i32.const 1 local.set $mask))
				local.get $code i32.const 2 i32.eq (if (then i32.const 2 local.set $mask))
				local.get $code i32.const 3 i32.eq (if (then i32.const 4 local.set $mask))
				local.get $code i32.const 4 i32.eq (if (then i32.const 8 local.set $mask))
				i32.const 1076 i32.const 1076 i32.load local.get $mask i32.const -1 i32.xor i32.and i32.store))
		;; viewport
		local.get $kind i32.const 6 i32.eq
		(if (then i32.const 1032 local.get $a f32.store i32.const 1036 local.get $b f32.store))
		;; menu action
		local.get $kind i32.const 7 i32.eq
		(if
			(then
				local.get $code i32.const 1 i32.eq
				(if (then i32.const 1028 i32.load i32.const 1032 f32.load i32.const 1036 f32.load call $reset))
				local.get $code i32.const 6 i32.eq
				(if (then i32.const 2 i32.const 0 i32.const 0 call $effect drop))))
		;; focus loss clears every held control but preserves pause/game-over.
		local.get $kind i32.const 8 i32.eq local.get $code i32.eqz i32.and
		(if (then i32.const 1076 i32.const 1076 i32.load i32.const -16 i32.and i32.store))
		i32.const 0)

	(func $draw_asteroid (param $id i32) (param $address i32)
		local.get $address i32.const 20 i32.add i32.load
		(if
			(then
				local.get $id
				local.get $address f32.load
				local.get $address i32.const 4 i32.add f32.load
				local.get $address i32.const 16 i32.add f32.load
				f32.const 2 i32.const 0x8894a8ff i32.const 0 call $circle drop)))

	(func (export "fp_render") (result i32)
		(local $x f32) (local $y f32) (local $dx f32) (local $dy f32)
		(local $rear_x f32) (local $rear_y f32)
		(local $left_x f32) (local $left_y f32)
		(local $right_x f32) (local $right_y f32)
		f32.const 0.03137255 f32.const 0.04313725 f32.const 0.07058824 f32.const 1 call $frame_begin drop
		;; title and authorship are presentation strings, ready for catalog lookup.
		i32.const 10 i32.const 64 i32.const 12
		i32.const 1032 f32.load f32.const 0.5 f32.mul f32.const 42 f32.const 30
		i32.const 0x5ee7ffff i32.const 1 call $text drop
		i32.const 11 i32.const 80 i32.const 16
		i32.const 1032 f32.load f32.const 0.5 f32.mul f32.const 66 f32.const 14
		i32.const -1 i32.const 1 call $text drop

		;; ship: three lines derived from its direction and perpendicular vector.
		i32.const 1040 f32.load local.set $x
		i32.const 1044 f32.load local.set $y
		i32.const 1056 f32.load local.set $dx
		i32.const 1060 f32.load local.set $dy
		local.get $x local.get $dx f32.const 10 f32.mul f32.sub local.set $rear_x
		local.get $y local.get $dy f32.const 10 f32.mul f32.sub local.set $rear_y
		local.get $rear_x local.get $dy f32.const 8 f32.mul f32.sub local.set $left_x
		local.get $rear_y local.get $dx f32.const 8 f32.mul f32.add local.set $left_y
		local.get $rear_x local.get $dy f32.const 8 f32.mul f32.add local.set $right_x
		local.get $rear_y local.get $dx f32.const 8 f32.mul f32.sub local.set $right_y
		i32.const 1
		local.get $x local.get $dx f32.const 16 f32.mul f32.add
		local.get $y local.get $dy f32.const 16 f32.mul f32.add
		local.get $left_x local.get $left_y f32.const 2 i32.const -1 call $line drop
		i32.const 2 local.get $left_x local.get $left_y local.get $right_x local.get $right_y
		f32.const 2 i32.const -1 call $line drop
		i32.const 3 local.get $right_x local.get $right_y
		local.get $x local.get $dx f32.const 16 f32.mul f32.add
		local.get $y local.get $dy f32.const 16 f32.mul f32.add
		f32.const 2 i32.const -1 call $line drop

		i32.const 1084 i32.load
		(if
			(then i32.const 100 i32.const 1088 f32.load i32.const 1092 f32.load
				f32.const 2 f32.const 0 i32.const 0x58ff72ff i32.const 1 call $circle drop))
		i32.const 200 i32.const 1120 call $draw_asteroid
		i32.const 201 i32.const 1144 call $draw_asteroid
		i32.const 202 i32.const 1168 call $draw_asteroid

		i32.const 1076 i32.load i32.const 16 i32.and
		(if
			(then i32.const 20 i32.const 100 i32.const 6
				i32.const 1032 f32.load f32.const 0.5 f32.mul
				i32.const 1036 f32.load f32.const 0.5 f32.mul
				f32.const 36 i32.const 0xffcf5cff i32.const 1 call $text drop))
		i32.const 1076 i32.load i32.const 32 i32.and
		(if
			(then i32.const 21 i32.const 108 i32.const 9
				i32.const 1032 f32.load f32.const 0.5 f32.mul
				i32.const 1036 f32.load f32.const 0.5 f32.mul
				f32.const 36 i32.const 0xff5c73ff i32.const 1 call $text drop))
		call $frame_end drop
		i32.const 0)
)
