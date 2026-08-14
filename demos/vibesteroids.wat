;; Vibesteroids behavioral conversion for the gpui-frontplane-v0 ABI.
;;
;; Schema 11 stores every gameplay scalar as an integer. Spatial quantities use
;; signed decimal fixed point with SCALE = 1,000,000. IEEE-754 values exist
;; only at the host ABI boundary: viewport scalars enter through $from_host,
;; and completed draw scalars leave through $to_host. They never feed back.
;; Linear velocity is stored in logical pixels/second and angular velocity in
;; radians/second; only named integration helpers know the fixed tick rate.
;;
;; Canonical state occupies [1024, 33792):
;;   0 tick:i32          4 rng:i32            8 width:i64
;;  16 height:i64      24 ship.x:i64         32 ship.y:i64
;;  40 ship.vx:i64     48 ship.vy:i64        56 ship.dx:i64
;;  64 ship.dy:i64     72 score:i32          76 lives:i32
;;  80 level:i32       84 flags:i32          88 last-fire:i32
;;  92 invulnerable    96 next-life:i32     100 lifecycle:i32
;; 104 lifecycle-ticks 108 banner:i32       112 seed:i32
;; 120 blossom-rotation:i64
;;
;; Pools, relative to AE_state_ptr:
;;   256: 64 legacy bullets x 48 bytes
;;        active:i32, pad:i32, x/y/vx/vy/lifetime-ticks:i64
;;  3328: 32 asteroids x 80 bytes
;;        active/generation/shape/pad:i32, x/y/vx/vy/radius/dx/dy:i64,
;;        spin:i32, points:i32
;;  5888: 150 particles x 48 bytes
;;        active/life:i32, x/y/vx/vy:i64, max-life:i32
;; 13088: 4 debris pieces x 80 bytes
;;        active/life:i32, x/y/vx/vy/dx/dy:i64, spin/piece:i32
;; 13408: 100 stars x 16 bytes, x/y:i64
;; 15008: enemy saucer: active/direction:i32, x/y/vx/radius:i64,
;;        shot-countdown:i32
;; 15056: 8 enemy shots x 48 bytes, same kinematic layout as player bullets
;; 15440: powerup package: active/direction:i32, x/y/vx/vy/radius:i64
;; 15488: ufo/package spawn countdowns:i32, laser/beam timers:i32,
;;        beam start/end x/y:i64
;; 15536: completed UFO appearances:i32
;; 15544: pointer target x/y:i64, pointer-heading authority:i32
;; 15568: temporary power kind:i32 (0 laser, 1 doubled fire rate)
;; 16384: 192 overflow player bullets x 48 bytes, preserving legacy addresses
;; 25600: hazardous blast active/attribution:i32, x/y:i64, elapsed ticks:i32
;; 26656: derelict satellite: active/direction:i32, x/y/vx/vy/dx/dy/radius:i64,
;;        spin/ping/pulse/spawn-countdown:i32
;; 26736: player-attributed satellite quote countdown:i32
(module
	(import "aedicule.v0" "AE_title" (func $title (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_menu_item" (func $menu_item (param i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_action" (func $action (param i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_touch_interest"
		(func $touch_interest (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_motion_interest"
		(func $motion_interest (param i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_ui_begin" (func $ui_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_ui_end" (func $ui_end (result i32)))
	(import "aedicule.v0" "AE_control_panel_q16"
		(func $control_panel_q16 (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_button_place_q16"
		(func $button_place_q16 (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
	(import "aedicule.v0" "AE_transform_push" (func $transform_push (param f32 f32 f32 f32 f32 f32) (result i32)))
	(import "aedicule.v0" "AE_transform_pop" (func $transform_pop (result i32)))
	(import "aedicule.v0" "AE_path_begin" (func $path_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_path_move" (func $path_move (param f32 f32) (result i32)))
	(import "aedicule.v0" "AE_path_line" (func $path_line (param f32 f32) (result i32)))
	(import "aedicule.v0" "AE_path_close" (func $path_close (result i32)))
	(import "aedicule.v0" "AE_path_end" (func $path_end (param f32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_line" (func $line (param i32 f32 f32 f32 f32 f32 i32) (result i32)))
	(import "aedicule.v0" "AE_circle" (func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_text" (func $text (param i32 i32 i32 f32 f32 f32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(import "aedicule.v0" "AE_audio" (func $audio (param i32 f32 f32 i32) (result i32)))
	(import "aedicule.v0" "AE_sample_asset"
		(func $sample_asset (param i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_sample_play"
		(func $sample_play (param i32 f32 f32 i32) (result i32)))
	(import "aedicule.v0" "AE_synth_voice"
		(func $synth_voice
			(param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
			(result i32)))
	(import "aedicule.v0" "AE_effect" (func $effect (param i32 i32 i32) (result i32)))

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
	(data (i32.const 184) "SHIP DESTROYED")
	(data (i32.const 200) "Help / Controls")
	(data (i32.const 224) "CONTROLS")
	(data (i32.const 240) "LEFT/A RIGHT/D ROTATE")
	(data (i32.const 264) "UP/W           THRUST")
	(data (i32.const 288) "SPACE          FIRE")
	(data (i32.const 312) "F              AUTO-FIRE")
	(data (i32.const 340) "K              KID MODE")
	(data (i32.const 368) "B              DEATH BLOSSOM")
	(data (i32.const 400) "P / ESC        PAUSE")
	(data (i32.const 424) "R              RESTART")
	(data (i32.const 448) "F1 / H         HELP")
	(data (i32.const 472) "DEATH BLOSSOM!")
	(data (i32.const 512) "ONE DEATH BLOSSOM PER LIFE")
	(data (i32.const 544) "000000")
	(data (i32.const 560) "KEYBOARD")
	(data (i32.const 568) "POINTER")
	(data (i32.const 576) "MOVE         AIM")
	(data (i32.const 592) "HOLD LEFT    FIRE")
	(data (i32.const 616) "HOLD RIGHT   THRUST")
	(data (i32.const 640) "SCROLL       BLOSSOM")
	(data (i32.const 672) "LASER  00.0")
	(data (i32.const 688) "RAPID  00.0")
	(data (i32.const 704) "assets/audio/satellite-destroyed.flac")
	(data (i32.const 744) "START GAME")
	(data (i32.const 760) "RESUME")
	(data (i32.const 768) "TOUCH")
	(data (i32.const 776) "LEFT/RIGHT EDGE")
	(data (i32.const 792) "FIRE")
	(data (i32.const 800) "EDGE STROKE")
	(data (i32.const 812) "ROTATE")
	(data (i32.const 820) "MIDDLE HOLD")
	(data (i32.const 832) "THRUST")
	(data (i32.const 840) "TOP CENTER")
	(data (i32.const 852) "PAUSE / RESUME")

	(global $scale i64 (i64.const 1000000))
	;; Fraction of the viewport reference spanned by a UFO/Voyager explosion. The
	;; historical extent was a flat 240 units, chosen at the 1024x768 design
	;; viewport whose reference length is 886.81, so 0.27 reproduces it to within
	;; a quarter of a percent while letting the blast shrink with a phone-sized
	;; screen instead of swallowing it.
	(global $hazardous_blast_fraction i64 (i64.const 270000))
	(global $tick_numerator i64 (i64.const 120))
	(global $tick_denominator i64 (i64.const 1))
	(global $player_bullet_capacity i32 (i32.const 256))
	(global $wave_sine i32 (i32.const 1))
	(global $wave_saw i32 (i32.const 2))
	(global $wave_white_noise i32 (i32.const 3))
	(global $filter_none i32 (i32.const 0))
	(global $filter_low_pass i32 (i32.const 1))
	(global $state_base_address i32 (i32.const 1024))
	(global $state_tick_address i32 (i32.const 1024))
	(global $state_rng_address i32 (i32.const 1028))
	(global $state_width_address i32 (i32.const 1032))
	(global $state_height_address i32 (i32.const 1040))
	(global $state_ship_x_address i32 (i32.const 1048))
	(global $state_ship_y_address i32 (i32.const 1056))
	(global $state_ship_vx_address i32 (i32.const 1064))
	(global $state_ship_vy_address i32 (i32.const 1072))
	(global $state_ship_dx_address i32 (i32.const 1080))
	(global $state_ship_dy_address i32 (i32.const 1088))
	(global $state_score_address i32 (i32.const 1096))
	(global $state_lives_address i32 (i32.const 1100))
	(global $state_level_address i32 (i32.const 1104))
	(global $state_flags_address i32 (i32.const 1108))
	(global $state_last_fire_address i32 (i32.const 1112))
	(global $state_invulnerability_address i32 (i32.const 1116))
	(global $state_next_life_address i32 (i32.const 1120))
	(global $state_lifecycle_address i32 (i32.const 1124))
	(global $state_lifecycle_ticks_address i32 (i32.const 1128))
	(global $state_banner_ticks_address i32 (i32.const 1132))
	(global $state_seed_address i32 (i32.const 1136))
	(global $state_blossom_rotation_address i32 (i32.const 1144))
	(global $state_satellite_active_address i32 (i32.const 26656))
	(global $state_satellite_direction_address i32 (i32.const 26660))
	(global $state_satellite_x_address i32 (i32.const 26664))
	(global $state_satellite_y_address i32 (i32.const 26672))
	(global $state_satellite_vx_address i32 (i32.const 26680))
	(global $state_satellite_vy_address i32 (i32.const 26688))
	(global $state_satellite_dx_address i32 (i32.const 26696))
	(global $state_satellite_dy_address i32 (i32.const 26704))
	(global $state_satellite_radius_address i32 (i32.const 26712))
	(global $state_satellite_spin_address i32 (i32.const 26720))
	(global $state_satellite_ping_countdown_address i32 (i32.const 26724))
	(global $state_satellite_pulse_ticks_address i32 (i32.const 26728))
	(global $state_satellite_spawn_countdown_address i32 (i32.const 26732))
	(global $state_satellite_quote_countdown_address i32 (i32.const 26736))
	(global $flag_rotate_left i32 (i32.const 1))
	(global $flag_rotate_right i32 (i32.const 2))
	(global $flag_thrust i32 (i32.const 4))
	(global $flag_fire i32 (i32.const 8))
	(global $flag_held_controls i32 (i32.const 15))
	(global $flag_paused i32 (i32.const 16))
	(global $flag_auto_fire i32 (i32.const 32))
	(global $flag_kid_mode i32 (i32.const 64))
	(global $flag_blossom_active i32 (i32.const 128))
	(global $flag_blossom_available i32 (i32.const 256))
	(global $flag_help_visible i32 (i32.const 512))
	(global $flag_suspends_tick i32 (i32.const 528))
	(global $flag_blocks_blossom_activation i32 (i32.const 656))
	;; Start and Resume share one visible gate. Resume has a second bit because it
	;; freezes simulation, while the boot gate deliberately lets the world drift
	;; behind the no-ship lifecycle until the player's completed tap.
	(global $flag_gate i32 (i32.const 1024))
	(global $flag_resume_gate i32 (i32.const 2048))
	(global $flag_suspends_simulation i32 (i32.const 2576))
	;; Retained host UI state is process-local and deliberately excluded from the
	;; snapshot. A sent mode of -1 forces the next render to publish the complete
	;; gate document after initialization or hot restore.
	(global $ui_revision (mut i32) (i32.const 0))
	(global $ui_sent_gate_mode (mut i32) (i32.const -1))
	(global $ui_sent_width (mut i64) (i64.const -1))
	(global $ui_sent_height (mut i64) (i64.const -1))
	;; Host input edges are intentionally not snapshotted: reload clears them.
	;; Each thrust source owns one bit so releasing an alias or pointer cannot
	;; cancel another physical source that remains held.
	(global $thrust_source_up i32 (i32.const 1))
	(global $thrust_source_pointer i32 (i32.const 2))
	(global $thrust_source_letter i32 (i32.const 4))
	(global $thrust_source_touch i32 (i32.const 8))
	(global $held_thrust_sources (mut i32) (i32.const 0))
	;; Fire also has independent physical owners. The touch bit represents the
	;; derived set of all live edge contacts, not any contact's opaque ID.
	(global $fire_source_key i32 (i32.const 1))
	(global $fire_source_pointer i32 (i32.const 2))
	(global $fire_source_touch i32 (i32.const 4))
	(global $held_fire_sources (mut i32) (i32.const 0))
	;; Rotation likewise has two physical sources per direction, the arrow key
	;; and its letter alias, so each direction needs its own held-source set.
	(global $rotate_source_arrow i32 (i32.const 1))
	(global $rotate_source_letter i32 (i32.const 2))
	(global $held_rotate_left_sources (mut i32) (i32.const 0))
	(global $held_rotate_right_sources (mut i32) (i32.const 0))
	;; Touch contacts are transient input adapter state outside AE_state_ptr/len.
	;; Eight records are ample for human fingers while keeping all loops bounded.
	;; Record: active/id/zone/pad:i32, start-y/start-dx/start-dy:i64.
	(global $touch_contacts_base i32 (i32.const 33792))
	(global $touch_contact_capacity i32 (i32.const 8))
	(global $touch_contact_stride i32 (i32.const 40))
	(global $touch_zone_left i32 (i32.const 1))
	(global $touch_zone_thrust i32 (i32.const 2))
	(global $touch_zone_right i32 (i32.const 3))
	(global $touch_zone_pause i32 (i32.const 4))
	;; Device modality is host state, not gameplay snapshot state. An observed raw
	;; contact remains useful on a hybrid device even if its primary pointer later
	;; changes from coarse to fine.
	(global $coarse_pointer_mode (mut i32) (i32.const 0))
	(global $touch_mode_seen (mut i32) (i32.const 0))
	(global $pause_opened_help (mut i32) (i32.const 0))

	(func $load_flags (result i32)
		global.get $state_flags_address i32.load)

	(func $store_flags (param $flags i32)
		global.get $state_flags_address local.get $flags i32.store)

	(func $clear_flag (param $mask i32)
		call $load_flags local.get $mask i32.const -1 i32.xor i32.and call $store_flags)

	(func $set_flag (param $mask i32)
		call $load_flags local.get $mask i32.or call $store_flags)

	(func $toggle_flag (param $mask i32)
		call $load_flags local.get $mask i32.xor call $store_flags)

	(func $refresh_thrust_flag
		global.get $held_thrust_sources i32.eqz
		(if
			(then global.get $flag_thrust call $clear_flag)
			(else global.get $flag_thrust call $set_flag)))

	(func $hold_thrust_source (param $source i32)
		global.get $held_thrust_sources local.get $source i32.or
		global.set $held_thrust_sources
		call $refresh_thrust_flag)

	(func $release_thrust_source (param $source i32)
		global.get $held_thrust_sources local.get $source i32.const -1 i32.xor i32.and
		global.set $held_thrust_sources
		call $refresh_thrust_flag)

	(func $refresh_fire_flag
		global.get $held_fire_sources i32.eqz
		(if
			(then global.get $flag_fire call $clear_flag)
			(else global.get $flag_fire call $set_flag)))

	(func $hold_fire_source (param $source i32)
		global.get $held_fire_sources local.get $source i32.or
		global.set $held_fire_sources
		call $refresh_fire_flag)

	(func $release_fire_source (param $source i32)
		global.get $held_fire_sources local.get $source i32.const -1 i32.xor i32.and
		global.set $held_fire_sources
		call $refresh_fire_flag)

	(func $refresh_rotate_left_flag
		global.get $held_rotate_left_sources i32.eqz
		(if
			(then global.get $flag_rotate_left call $clear_flag)
			(else global.get $flag_rotate_left call $set_flag)))

	(func $hold_rotate_left_source (param $source i32)
		global.get $held_rotate_left_sources local.get $source i32.or
		global.set $held_rotate_left_sources
		call $refresh_rotate_left_flag)

	(func $release_rotate_left_source (param $source i32)
		global.get $held_rotate_left_sources local.get $source i32.const -1 i32.xor i32.and
		global.set $held_rotate_left_sources
		call $refresh_rotate_left_flag)

	(func $refresh_rotate_right_flag
		global.get $held_rotate_right_sources i32.eqz
		(if
			(then global.get $flag_rotate_right call $clear_flag)
			(else global.get $flag_rotate_right call $set_flag)))

	(func $hold_rotate_right_source (param $source i32)
		global.get $held_rotate_right_sources local.get $source i32.or
		global.set $held_rotate_right_sources
		call $refresh_rotate_right_flag)

	(func $release_rotate_right_source (param $source i32)
		global.get $held_rotate_right_sources local.get $source i32.const -1 i32.xor i32.and
		global.set $held_rotate_right_sources
		call $refresh_rotate_right_flag)

	(func $touch_contact_address (param $index i32) (result i32)
		global.get $touch_contacts_base
		local.get $index global.get $touch_contact_stride i32.mul i32.add)

	;; Returns zero when an opaque contact ID is not currently active.
	(func $find_touch_contact (param $id i32) (result i32)
		(local $index i32) (local $address i32)
		(block $none
			(loop $again
				local.get $index global.get $touch_contact_capacity i32.ge_u br_if $none
				local.get $index call $touch_contact_address local.set $address
				local.get $address i32.load
				local.get $address i32.const 4 i32.add i32.load local.get $id i32.eq
				i32.and (if (then local.get $address return))
				local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	(func $find_free_touch_contact (result i32)
		(local $index i32) (local $address i32)
		(block $none
			(loop $again
				local.get $index global.get $touch_contact_capacity i32.ge_u br_if $none
				local.get $index call $touch_contact_address local.set $address
				local.get $address i32.load i32.eqz
				(if (then local.get $address return))
				local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	;; Recomputes action ownership over the complete active-contact set. This is
	;; what lets one finger lift without cancelling another finger in that zone.
	(func $refresh_touch_action_sources
		(local $index i32) (local $address i32)
		(local $has_edge i32) (local $has_thrust i32) (local $zone i32)
		(block $done
			(loop $again
				local.get $index global.get $touch_contact_capacity i32.ge_u br_if $done
				local.get $index call $touch_contact_address local.set $address
				local.get $address i32.load
				(if (then
					local.get $address i32.const 8 i32.add i32.load local.set $zone
					local.get $zone global.get $touch_zone_thrust i32.eq
					(if (then i32.const 1 local.set $has_thrust))
					local.get $zone global.get $touch_zone_left i32.eq
					local.get $zone global.get $touch_zone_right i32.eq i32.or
					(if (then i32.const 1 local.set $has_edge))))
				local.get $index i32.const 1 i32.add local.set $index br $again))
		local.get $has_edge
		(if
			(then global.get $fire_source_touch call $hold_fire_source)
			(else global.get $fire_source_touch call $release_fire_source))
		local.get $has_thrust
		(if
			(then global.get $thrust_source_touch call $hold_thrust_source)
			(else global.get $thrust_source_touch call $release_thrust_source)))

	(func $clear_touch_contacts
		global.get $touch_contacts_base i32.const 0
		global.get $touch_contact_capacity global.get $touch_contact_stride i32.mul
		memory.fill
		global.get $fire_source_touch call $release_fire_source
		global.get $thrust_source_touch call $release_thrust_source)

	(func $touch_zone (param $x i64) (param $y i64) (result i32)
		local.get $x i64.const 5 i64.mul
		global.get $state_width_address i64.load i64.const 2 i64.mul i64.ge_s
		local.get $x i64.const 5 i64.mul
		global.get $state_width_address i64.load i64.const 3 i64.mul i64.le_s i32.and
		local.get $y i64.const 70000000 i64.le_s i32.and
		(if (then global.get $touch_zone_pause return))
		local.get $x i64.const 5 i64.mul
		global.get $state_width_address i64.load i64.lt_s
		(if (then global.get $touch_zone_left return))
		local.get $x i64.const 5 i64.mul
		global.get $state_width_address i64.load i64.const 4 i64.mul i64.gt_s
		(if (then global.get $touch_zone_right return))
		global.get $touch_zone_thrust)

	;; A start claims one free bounded record. Duplicate active IDs and overflow
	;; starts are inert, so neither can steal another contact's zone ownership.
	(func $handle_touch_start (param $id i32) (param $x i64) (param $y i64)
		(local $address i32) (local $zone i32)
		i32.const 1 global.set $touch_mode_seen
		local.get $id call $find_touch_contact i32.eqz
		(if (then) (else return))
		local.get $x local.get $y call $touch_zone local.set $zone
		local.get $zone global.get $touch_zone_pause i32.eq
		(if (then call $toggle_pause return))
		call $load_flags global.get $flag_paused i32.and
		(if (then return))
		call $find_free_touch_contact local.tee $address i32.eqz
		(if (then return))
		local.get $address i32.const 1 i32.store
		local.get $address i32.const 4 i32.add local.get $id i32.store
		local.get $address i32.const 8 i32.add local.get $zone i32.store
		local.get $address i32.const 16 i32.add local.get $y i64.store
		local.get $address i32.const 24 i32.add
		global.get $state_ship_dx_address i64.load i64.store
		local.get $address i32.const 32 i32.add
		global.get $state_ship_dy_address i64.load i64.store
		local.get $zone global.get $touch_zone_left i32.eq
		local.get $zone global.get $touch_zone_right i32.eq i32.or
		(if (then i32.const 16584 i32.const 0 i32.store))
		call $refresh_touch_action_sources)

	(func $handle_touch_terminal (param $id i32)
		(local $address i32)
		local.get $id call $find_touch_contact local.tee $address i32.eqz
		(if (then return))
		local.get $address i32.const 0 i32.store
		call $refresh_touch_action_sources)

	(func $handle_touch_move (param $id i32) (param $y i64)
		(local $address i32)
		local.get $id call $find_touch_contact local.tee $address i32.eqz
		(if (then return))
		local.get $address local.get $y call $apply_touch_heading)

	;; Input-down edges are transient host state. Clear them as a class while
	;; preserving modal gameplay choices across focus loss and hot reload.
	(func $clear_held_controls
		call $clear_touch_contacts
		i32.const 0 global.set $held_thrust_sources
		i32.const 0 global.set $held_fire_sources
		i32.const 0 global.set $held_rotate_left_sources
		i32.const 0 global.set $held_rotate_right_sources
		global.get $flag_held_controls call $clear_flag)

	(func $touch_help_mode (result i32)
		global.get $coarse_pointer_mode global.get $touch_mode_seen i32.or)

	;; Pause is an input boundary: entering it releases every held gameplay edge
	;; so a later resume cannot resurrect thrust, fire, or rotation. Touch Help is
	;; paired with Pause and removed on resume only when Pause opened it.
	(func $toggle_pause
		global.get $flag_paused call $toggle_flag
		call $load_flags global.get $flag_paused i32.and
		(if
			(then
				call $clear_held_controls
				i32.const 0 global.set $pause_opened_help
				call $touch_help_mode
				(if (then
					call $load_flags global.get $flag_help_visible i32.and i32.eqz
					(if (then
						global.get $flag_help_visible call $set_flag
						i32.const 1 global.set $pause_opened_help)))))
			(else
				global.get $pause_opened_help
				(if (then global.get $flag_help_visible call $clear_flag))
				i32.const 0 global.set $pause_opened_help)))

	;; Classifies the only events that remain meaningful behind Pause. Native
	;; commands, viewport/focus housekeeping, and the unpause key stay live;
	;; every gameplay-control family is rejected before it can mutate state.
	(func $event_allowed_while_paused (param $kind i32) (param $code i32) (result i32)
		call $load_flags global.get $flag_paused i32.and i32.eqz
		(if (result i32)
			(then i32.const 1)
			(else
				local.get $kind i32.const 6 i32.eq
				local.get $kind i32.const 7 i32.eq i32.or
				local.get $kind i32.const 8 i32.eq i32.or
				local.get $kind i32.const 11 i32.eq i32.or
				local.get $kind i32.const 1 i32.eq
				local.get $code i32.const 5 i32.eq
				local.get $code i32.const 11 i32.eq i32.or
				i32.and i32.or)))

	;; Keeps modal presentation frozen without discarding held-edge state needed
	;; by Help; Pause itself has already cleared gameplay edges on entry.
	(func $thrust_is_presented (result i32)
		(local $flags i32)
		call $load_flags local.set $flags
		local.get $flags global.get $flag_thrust i32.and i32.eqz i32.eqz
		local.get $flags global.get $flag_suspends_tick i32.and i32.eqz
		i32.and)

	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 10)
	(func (export "AE_state_ptr") (result i32) global.get $state_base_address)
	(func (export "AE_state_len") (result i32) i32.const 32768)
	(func (export "AE_state_schema") (result i32) i32.const 11)
	(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
		global.get $tick_numerator i32.wrap_i64
		global.get $tick_denominator i32.wrap_i64)

	;; Centralizes the host's positional voice ABI behind semantic parameter
	;; names, so call sites cannot silently drift when new cues are added.
	(func $declare_voice
		(param $program i32) (param $waveform i32) (param $delay_ms i32) (param $duration_ms i32)
		(param $frequency_start i32) (param $frequency_mid i32) (param $frequency_end i32)
		(param $gain_start i32) (param $gain_peak i32) (param $gain_end i32)
		(param $filter i32) (param $filter_start i32) (param $filter_end i32) (param $cooldown_ms i32)
		local.get $program local.get $waveform local.get $delay_ms local.get $duration_ms
		local.get $frequency_start local.get $frequency_mid local.get $frequency_end
		local.get $gain_start local.get $gain_peak local.get $gain_end
		local.get $filter local.get $filter_start local.get $filter_end local.get $cooldown_ms
		call $synth_voice drop)

	;; Declares a swept tone with a 0 -> peak -> 0 envelope. Naming
	;; the reduced parameter surface prevents filter enums from being mistaken
	;; for gain or cooldown values in the host's 14-scalar ABI call.
	(func $declare_swept_voice
		(param $program i32) (param $waveform i32) (param $delay_ms i32) (param $duration_ms i32)
		(param $frequency_start i32) (param $frequency_mid i32) (param $frequency_end i32)
		(param $gain_peak i32) (param $filter i32)
		(param $filter_start i32) (param $filter_end i32)
		local.get $program local.get $waveform local.get $delay_ms local.get $duration_ms
		local.get $frequency_start local.get $frequency_mid local.get $frequency_end
		i32.const 0 local.get $gain_peak i32.const 0
		local.get $filter local.get $filter_start local.get $filter_end i32.const 0
		call $declare_voice)

	(func $configure_menu
		i32.const 0 i32.const 20 call $title drop
		i32.const 1 i32.const 32 i32.const 8 i32.const 1 i32.const 0 call $menu_item drop
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 i32.const 1 call $menu_item drop
		i32.const 7 i32.const 200 i32.const 15 i32.const 7 i32.const 0 call $menu_item drop
		i32.const 6 i32.const 48 i32.const 4 i32.const 6 i32.const 0 call $menu_item drop)
	;; These actions exist only for retained buttons. Menu presentation remains an
	;; explicit, separate declaration owned by $configure_menu.
	(func $configure_gate_actions
		i32.const 8 i32.const 744 i32.const 10 i32.const 0 call $action drop
		i32.const 9 i32.const 760 i32.const 6 i32.const 0 call $action drop)
	(func $configure_shot_sound
		;; Shot: 800 -> 400 -> 200 Hz sine, 0.3 -> 0.01 gain.
		i32.const 1 i32.const 1 i32.const 0 i32.const 100
		i32.const 800000 i32.const 400000 i32.const 200000
		i32.const 300000 i32.const 300000 i32.const 10000
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $declare_voice)
	(func $configure_asteroid_explosion_sound
		;; Asteroid explosion: swept low-pass noise plus a restrained bass layer.
		i32.const 2 i32.const 3 i32.const 0 i32.const 500
		i32.const 0 i32.const 0 i32.const 0
		i32.const 500000 i32.const 500000 i32.const 10000
		i32.const 1 i32.const 1500000 i32.const 80000 i32.const 0 call $declare_voice
		i32.const 2 i32.const 1 i32.const 0 i32.const 500
		i32.const 120000 i32.const 120000 i32.const 80000
		i32.const 80000 i32.const 80000 i32.const 1000
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $declare_voice)
	(func $configure_ship_explosion_sound
		;; Ship explosion: white/brown noise layers and a short low impulse.
		i32.const 3 i32.const 3 i32.const 0 i32.const 1200
		i32.const 0 i32.const 0 i32.const 0
		i32.const 700000 i32.const 700000 i32.const 1000
		i32.const 1 i32.const 3000000 i32.const 100000 i32.const 0 call $declare_voice
		i32.const 3 i32.const 4 i32.const 0 i32.const 1200
		i32.const 0 i32.const 0 i32.const 0
		i32.const 300000 i32.const 300000 i32.const 1000
		i32.const 2 i32.const 300000 i32.const 300000 i32.const 0 call $declare_voice
		i32.const 3 i32.const 1 i32.const 0 i32.const 200
		i32.const 120000 i32.const 60000 i32.const 40000
		i32.const 500000 i32.const 500000 i32.const 0
		i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $declare_voice)
	(func $configure_thrust_sound
		;; Thrust uses one low-passed white-noise bed: removing the low band-pass
		;; and overlapping second voice avoids mechanical rattle while retaining
		;; the non-tonal static texture of classic LFSR-based engine effects.
		i32.const 4 global.get $wave_white_noise i32.const 0 i32.const 120
		i32.const 0 i32.const 0 i32.const 0
		i32.const 90000 i32.const 120000 i32.const 40000
		global.get $filter_low_pass i32.const 1800000 i32.const 1200000 i32.const 55
		call $declare_voice)
	(func $configure_death_blossom_sound
		;; Death Blossom: three scheduled 400 -> 800 -> 400 Hz whoops.
		i32.const 5 i32.const 1 i32.const 0 i32.const 300 i32.const 400000 i32.const 800000 i32.const 400000 i32.const 300000 i32.const 300000 i32.const 1000 i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $declare_voice
		i32.const 5 i32.const 1 i32.const 400 i32.const 300 i32.const 400000 i32.const 800000 i32.const 400000 i32.const 300000 i32.const 300000 i32.const 1000 i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $declare_voice
		i32.const 5 i32.const 1 i32.const 800 i32.const 300 i32.const 400000 i32.const 800000 i32.const 400000 i32.const 300000 i32.const 300000 i32.const 1000 i32.const 0 i32.const 0 i32.const 0 i32.const 0 call $declare_voice)
	(func $configure_extra_life_sound
		;; Extra life: five delayed sawtooth chimes with note-relative filters.
		i32.const 6 i32.const 2 i32.const 0 i32.const 400 i32.const 523250 i32.const 523250 i32.const 523250 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 1569750 i32.const 1569750 i32.const 0 call $declare_voice
		i32.const 6 i32.const 2 i32.const 150 i32.const 400 i32.const 588656 i32.const 588656 i32.const 588656 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 1765968 i32.const 1765968 i32.const 0 call $declare_voice
		i32.const 6 i32.const 2 i32.const 300 i32.const 400 i32.const 654063 i32.const 654063 i32.const 654063 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 1962189 i32.const 1962189 i32.const 0 call $declare_voice
		i32.const 6 i32.const 2 i32.const 450 i32.const 400 i32.const 784875 i32.const 784875 i32.const 784875 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 2354625 i32.const 2354625 i32.const 0 call $declare_voice
		i32.const 6 i32.const 2 i32.const 600 i32.const 400 i32.const 1046500 i32.const 1046500 i32.const 1046500 i32.const 0 i32.const 300000 i32.const 1000 i32.const 1 i32.const 3139500 i32.const 3139500 i32.const 0 call $declare_voice)
	(func $configure_laser_sound
		;; Laser: sfxr-style descending saw/sine sweep with a zero attack and
		;; short envelope, layered behind a fast high-frequency transient.
		i32.const 7 global.get $wave_saw i32.const 0 i32.const 180
		i32.const 1400000 i32.const 650000 i32.const 120000
		i32.const 700000 global.get $filter_low_pass i32.const 4000000 i32.const 500000
		call $declare_swept_voice
		i32.const 7 global.get $wave_sine i32.const 0 i32.const 120
		i32.const 900000 i32.const 450000 i32.const 160000
		i32.const 350000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice)
	(func $configure_ufo_alert_sound
		;; Red alert: two filtered saw sweeps form a compact oscillating siren.
		i32.const 8 global.get $wave_saw i32.const 0 i32.const 220
		i32.const 900000 i32.const 560000 i32.const 900000
		i32.const 350000 global.get $filter_low_pass i32.const 1800000 i32.const 1100000
		call $declare_swept_voice
		i32.const 8 global.get $wave_saw i32.const 260 i32.const 220
		i32.const 900000 i32.const 560000 i32.const 900000
		i32.const 350000 global.get $filter_low_pass i32.const 1800000 i32.const 1100000
		call $declare_swept_voice)
	(func $configure_package_notification_sound
		;; Package notification: a quick ascending two-note sine chime.
		i32.const 9 global.get $wave_sine i32.const 0 i32.const 150
		i32.const 659255 i32.const 659255 i32.const 659255
		i32.const 250000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice
		i32.const 9 global.get $wave_sine i32.const 110 i32.const 180
		i32.const 987767 i32.const 987767 i32.const 987767
		i32.const 280000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice)
	(func $configure_package_collection_sound
		;; Package collection: an ascending major arpeggio rewards the risky pickup.
		i32.const 10 global.get $wave_sine i32.const 0 i32.const 180
		i32.const 523251 i32.const 523251 i32.const 523251
		i32.const 260000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice
		i32.const 10 global.get $wave_sine i32.const 90 i32.const 180
		i32.const 659255 i32.const 659255 i32.const 659255
		i32.const 280000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice
		i32.const 10 global.get $wave_sine i32.const 180 i32.const 240
		i32.const 783991 i32.const 783991 i32.const 783991
		i32.const 320000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice)
	(func $configure_package_loss_sound
		;; Package loss: layered descending tones make a compact negative buzzer.
		i32.const 11 global.get $wave_saw i32.const 0 i32.const 360
		i32.const 440000 i32.const 220000 i32.const 110000
		i32.const 300000 global.get $filter_low_pass i32.const 1000000 i32.const 180000
		call $declare_swept_voice
		i32.const 11 global.get $wave_sine i32.const 80 i32.const 300
		i32.const 311127 i32.const 207652 i32.const 155563
		i32.const 220000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice)
	(func $configure_hazardous_blast_sound
		;; Hazardous blast: three full-length layers make a broad, loud BOOM whose
		;; audible envelope cannot finish before one second has elapsed.
		i32.const 12 i32.const 3 i32.const 0 i32.const 1500
		i32.const 0 i32.const 0 i32.const 0
		i32.const 1000000 global.get $filter_low_pass i32.const 3000000 i32.const 60000
		call $declare_swept_voice
		i32.const 12 i32.const 4 i32.const 0 i32.const 1400
		i32.const 0 i32.const 0 i32.const 0
		i32.const 850000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice
		i32.const 12 global.get $wave_sine i32.const 0 i32.const 1200
		i32.const 110000 i32.const 65000 i32.const 35000
		i32.const 850000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice)
	(func $configure_satellite_ping_sound
		;; Deep-space sonar: one pitch-stable sine transient and two progressively
		;; quieter delayed reflections imply reverb without host-side effects.
		i32.const 13 global.get $wave_sine i32.const 0 i32.const 420
		i32.const 520000 i32.const 520000 i32.const 520000
		i32.const 220000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice
		i32.const 13 global.get $wave_sine i32.const 160 i32.const 520
		i32.const 520000 i32.const 520000 i32.const 520000
		i32.const 85000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice
		i32.const 13 global.get $wave_sine i32.const 340 i32.const 620
		i32.const 520000 i32.const 520000 i32.const 520000
		i32.const 35000 global.get $filter_none i32.const 0 i32.const 0
		call $declare_swept_voice)
	;; Binds the packaged digitized quote once during configuration so runtime
	;; destruction emits only a bounded playback event, never asset I/O.
	(func $configure_satellite_quote
		i32.const 1 i32.const 704 i32.const 37 i32.const 0 call $sample_asset drop)

	(func (export "AE_configure") (result i32)
		(local $status i32)
		;; Raw contacts are required for independent touch ownership. A failed
		;; opt-in must reject configuration instead of silently restoring the
		;; compatibility-pointer path and its duplicate/one-contact semantics.
		i32.const 8 i32.const 0 call $touch_interest local.tee $status
		(if (then local.get $status return))
		;; Shake is an optional physical route to the same once-per-life action.
		;; Registration failure is explicit; visible keyboard, wheel, and touch
		;; routes still document the action when a device supplies no events.
		i32.const 1 i32.const 0 i32.const 0 call $motion_interest local.tee $status
		(if (then local.get $status return))
		call $configure_menu
		call $configure_gate_actions
		call $configure_shot_sound
		call $configure_asteroid_explosion_sound
		call $configure_ship_explosion_sound
		call $configure_thrust_sound
		call $configure_death_blossom_sound
		call $configure_extra_life_sound
		call $configure_laser_sound
		call $configure_ufo_alert_sound
		call $configure_package_notification_sound
		call $configure_package_collection_sound
		call $configure_package_loss_sound
		call $configure_hazardous_blast_sound
		call $configure_satellite_ping_sound
		call $configure_satellite_quote
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
		local.get $per_second global.get $tick_denominator i64.mul
		global.get $tick_numerator i64.div_s)

	;; Converts a source-authored 60-Hz retention factor into per-second linear
	;; damping, preventing higher simulation rates from multiplying decay.
	(func $retention_factor_per_tick (param $at_sixty i64) (result i64)
		global.get $scale
		global.get $scale local.get $at_sixty i64.sub i64.const 60 i64.mul
		global.get $tick_denominator i64.mul global.get $tick_numerator i64.div_u
		i64.sub)

	(func $ticks_from_sixty (param $ticks i32) (result i32)
		local.get $ticks i64.extend_i32_u global.get $tick_numerator i64.mul
		i64.const 60 global.get $tick_denominator i64.mul i64.div_u i32.wrap_i64)

	(func $fixed_abs (param $value i64) (result i64)
		local.get $value i64.const 0 i64.lt_s
		(if (result i64) (then i64.const 0 local.get $value i64.sub) (else local.get $value)))

	(func $difficulty_index (result i64)
		global.get $state_level_address i32.load i32.const 1 i32.sub
		i32.const 0 i32.lt_s (if (then i64.const 0 return))
		global.get $state_level_address i32.load i32.const 20 i32.ge_s (if (then i64.const 19 return))
		global.get $state_level_address i32.load i32.const 1 i32.sub i64.extend_i32_s)

	(func $difficulty_value (param $base i64) (param $delta i64) (result i64)
		local.get $base local.get $delta call $difficulty_index i64.mul i64.const 19 i64.div_s i64.add)

	;; Produces 1.0 + 0.2 per completed level, capped at level 20 with the other
	;; difficulty controls so projectile and asteroid pacing share one contract.
	(func $level_scale (result i64)
		global.get $scale call $difficulty_index i64.const 200000 i64.mul i64.add)

	(func $scale_per_level (param $base i64) (result i64)
		local.get $base call $level_scale call $fixed_mul)

	(func $ship_acceleration_per_second (result i64)
		i64.const 300000000 i64.const 200000000 call $difficulty_value)

	(func $rotation_per_second (result i64)
		i64.const 5000000 i64.const 5000000 call $difficulty_value)

	(func $bullet_speed_per_second (result i64)
		i64.const 337500000 call $scale_per_level)

	(func $fire_interval_ticks (result i32)
		;; Source uses elapsed_ms > delay, so the first integral fixed tick after
		;; the delay is floor(delay * 60 / 1000) + 1.
		i64.const 250000000 i64.const -125000000 call $difficulty_value
		global.get $tick_numerator i64.mul
		i64.const 1000000000 global.get $tick_denominator i64.mul i64.div_s
		i32.wrap_i64 i32.const 1 i32.add)

	(func $ufo_visit_index (result i64)
		(local $visit i64)
		i32.const 16560 i32.load i64.extend_i32_u i64.const 1 i64.sub local.set $visit
		local.get $visit i64.const 0 i64.lt_s (if (then i64.const 0 return))
		local.get $visit i64.const 20 i64.gt_s (if (then i64.const 20 return))
		local.get $visit)

	;; Each appearance after the first adds 5% offensive/traversal pressure while
	;; the visit cap keeps long-running sessions within bounded integer ranges.
	(func $ufo_scale (result i64)
		global.get $scale call $ufo_visit_index i64.const 50000 i64.mul i64.add)

	(func $ufo_scale_per_visit (param $base i64) (result i64)
		local.get $base call $ufo_scale call $fixed_mul)

	(func $ufo_radius (result i64)
		(local $radius i64)
		i64.const 20000000 call $ufo_visit_index i64.const 500000 i64.mul i64.sub
		local.tee $radius i64.const 10000000 i64.lt_s
		(if (result i64) (then i64.const 10000000) (else local.get $radius)))

	(func $ufo_projectile_speed (result i64)
		i64.const 280000000 call $ufo_scale_per_visit)

	;; Scales the UFO alongside the player's level rate and successive visits,
	;; then enforces an interval at least twice the player's current interval.
	(func $ufo_fire_interval_ticks (result i32)
		(local $interval i32) (local $minimum i32)
		call $rand_u32 i32.const 46 i32.rem_u i32.const 45 i32.add
		call $ticks_from_sixty i64.extend_i32_u
		global.get $scale i64.mul call $level_scale i64.div_s
		global.get $scale i64.mul call $ufo_scale i64.div_s i32.wrap_i64
		local.set $interval
		call $fire_interval_ticks i32.const 2 i32.mul local.set $minimum
		local.get $interval local.get $minimum i32.lt_s
		(if (then local.get $minimum return))
		local.get $interval)

	(func $small_sine (param $angle i64) (result i64)
		(local $square i64)
		local.get $angle local.get $angle call $fixed_mul local.set $square
		local.get $angle local.get $square local.get $angle call $fixed_mul i64.const 6 i64.div_s i64.sub)

	(func $small_cosine (param $angle i64) (result i64)
		(local $square i64)
		local.get $angle local.get $angle call $fixed_mul local.set $square
		global.get $scale local.get $square i64.const 2 i64.div_s i64.sub
		local.get $square local.get $square call $fixed_mul i64.const 24 i64.div_s i64.add)

	;; Applies the original `4pi`-per-viewport-height stroke mapping relative to
	;; one contact's captured heading. Reducing modulo one turn and subdividing
	;; the residual angle keeps the existing small-angle polynomial accurate and
	;; bounds a worst-case move to 32 fixed-point rotations.
	(func $apply_touch_heading (param $address i32) (param $y i64)
		(local $zone i32) (local $angle i64) (local $magnitude i64)
		(local $remaining i64) (local $step i64) (local $sine i64) (local $cosine i64)
		(local $dx i64) (local $dy i64) (local $next_dx i64) (local $next_dy i64)
		(local $length i64) (local $steps i32) (local $index i32)
		local.get $address i32.const 8 i32.add i32.load local.set $zone
		local.get $zone global.get $touch_zone_left i32.eq
		local.get $zone global.get $touch_zone_right i32.eq i32.or i32.eqz
		(if (then return))
		call $load_flags global.get $flag_blossom_active i32.and
		(if (then return))
		global.get $state_height_address i64.load i64.const 0 i64.le_s
		(if (then return))
		local.get $y local.get $address i32.const 16 i32.add i64.load i64.sub
		i64.const 12566371 i64.mul
		global.get $state_height_address i64.load i64.div_s local.set $angle
		local.get $zone global.get $touch_zone_left i32.eq
		(if (then i64.const 0 local.get $angle i64.sub local.set $angle))
		local.get $angle i64.const 6283185 i64.rem_s local.set $angle
		local.get $angle i64.const 3141593 i64.gt_s
		(if (then local.get $angle i64.const 6283185 i64.sub local.set $angle))
		local.get $angle i64.const -3141593 i64.lt_s
		(if (then local.get $angle i64.const 6283185 i64.add local.set $angle))
		local.get $address i32.const 24 i32.add i64.load local.set $dx
		local.get $address i32.const 32 i32.add i64.load local.set $dy
		local.get $angle local.set $remaining
		local.get $angle i64.const 0 i64.lt_s
		(if (result i64)
			(then i64.const 0 local.get $angle i64.sub)
			(else local.get $angle))
		local.set $magnitude
		local.get $magnitude i64.const 99999 i64.add i64.const 100000 i64.div_u
		i32.wrap_i64 local.set $steps
		(block $done
			(loop $again
				local.get $index local.get $steps i32.ge_u br_if $done
				local.get $remaining
				local.get $steps local.get $index i32.sub i64.extend_i32_u i64.div_s
				local.set $step
				local.get $remaining local.get $step i64.sub local.set $remaining
				local.get $step call $small_sine local.set $sine
				local.get $step call $small_cosine local.set $cosine
				local.get $dx local.get $cosine call $fixed_mul
				local.get $dy local.get $sine call $fixed_mul i64.sub local.set $next_dx
				local.get $dy local.get $cosine call $fixed_mul
				local.get $dx local.get $sine call $fixed_mul i64.add local.set $next_dy
				local.get $next_dx local.set $dx
				local.get $next_dy local.set $dy
				local.get $index i32.const 1 i32.add local.set $index br $again))
		local.get $dx local.get $dy call $fixed_hypot local.tee $length i64.eqz
		(if (then return))
		global.get $state_ship_dx_address
		local.get $dx global.get $scale i64.mul local.get $length i64.div_s i64.store
		global.get $state_ship_dy_address
		local.get $dy global.get $scale i64.mul local.get $length i64.div_s i64.store
		i32.const 16584 i32.const 0 i32.store)

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

	;; Reference length for every viewport-derived gameplay extent: the geometric
	;; mean of width and height, which is the side of the square with the same
	;; area. It tracks the smaller dimension far more closely than the diagonal
	;; does, so a portrait phone no longer inflates each extent by its dominant
	;; height. Milli-scaling before the multiply keeps an 8K viewport clear of
	;; i64 overflow, the same guard $fixed_hypot uses.
	(func $viewport_reference (result i64)
		global.get $state_width_address i64.load i64.const 1000 i64.div_s
		global.get $state_height_address i64.load i64.const 1000 i64.div_s
		i64.mul call $integer_sqrt i64.const 1000 i64.mul)

	;; Shared visual and damage extent for UFO/Voyager explosions; target hull
	;; radii are added at collision sites so the drawn danger area stays honest.
	;; Recomputed rather than cached because a blast is rare and short-lived.
	(func $hazardous_blast_radius (result i64)
		call $viewport_reference global.get $hazardous_blast_fraction call $fixed_mul)

	;; Leads a moving target with two deterministic fixed-point time-of-flight
	;; refinements, then adds the shooter's velocity to projectile world motion.
	(func $aim_projectile_velocity
		(param $source_x i64) (param $source_y i64)
		(param $source_vx i64) (param $source_vy i64)
		(param $target_x i64) (param $target_y i64)
		(param $target_vx i64) (param $target_vy i64)
		(param $speed i64) (result i64 i64)
		(local $base_x i64) (local $base_y i64) (local $relative_vx i64) (local $relative_vy i64)
		(local $lead_x i64) (local $lead_y i64) (local $length i64) (local $time i64)
		local.get $target_x local.get $source_x i64.sub local.set $base_x
		local.get $target_y local.get $source_y i64.sub local.set $base_y
		local.get $target_vx local.get $source_vx i64.sub local.set $relative_vx
		local.get $target_vy local.get $source_vy i64.sub local.set $relative_vy
		local.get $base_x local.get $base_y call $fixed_hypot local.set $length
		local.get $length i64.eqz
		(if (then
			local.get $source_vx local.get $speed i64.add local.get $source_vy return))
		local.get $length global.get $scale i64.mul local.get $speed i64.div_s local.set $time
		local.get $base_x local.get $relative_vx local.get $time call $fixed_mul i64.add local.set $lead_x
		local.get $base_y local.get $relative_vy local.get $time call $fixed_mul i64.add local.set $lead_y
		local.get $lead_x local.get $lead_y call $fixed_hypot local.set $length
		local.get $length global.get $scale i64.mul local.get $speed i64.div_s local.set $time
		local.get $base_x local.get $relative_vx local.get $time call $fixed_mul i64.add local.set $lead_x
		local.get $base_y local.get $relative_vy local.get $time call $fixed_mul i64.add local.set $lead_y
		local.get $lead_x local.get $lead_y call $fixed_hypot local.set $length
		local.get $length i64.eqz
		(if (then
			local.get $source_vx local.get $speed i64.add local.get $source_vy return))
		local.get $source_vx local.get $lead_x local.get $speed i64.mul local.get $length i64.div_s i64.add
		local.get $source_vy local.get $lead_y local.get $speed i64.mul local.get $length i64.div_s i64.add)

	;; Extends the projectile pool without moving any of the 64 legacy records,
	;; keeping old slot offsets stable while the current schema adds a tail pool.
	(func $bullet_address (param $index i32) (result i32)
		local.get $index i32.const 64 i32.lt_u
		(if (result i32)
			(then i32.const 1280 local.get $index i32.const 48 i32.mul i32.add)
			(else i32.const 17408 local.get $index i32.const 64 i32.sub i32.const 48 i32.mul i32.add)))
	;; Keeps new player projectile IDs out of the asteroid command-key range.
	(func $bullet_draw_key (param $index i32) (result i32)
		local.get $index i32.const 64 i32.lt_u
		(if (result i32)
			(then i32.const 100 local.get $index i32.add)
			(else i32.const 1100 local.get $index i32.const 64 i32.sub i32.add)))
	(func $enemy_bullet_address (param $index i32) (result i32)
		i32.const 16080 local.get $index i32.const 48 i32.mul i32.add)
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
		global.get $state_rng_address i32.load i32.const 0x6d2b79f5 i32.add local.set $state
		global.get $state_rng_address local.get $state i32.store
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

	;; Selects an inclusive 45--120 second interval in canonical 60-Hz ticks,
	;; then converts it exactly to the configured rational simulation rate.
	(func $random_spawn_ticks (result i32)
		call $rand_u32 i32.const 4501 i32.rem_u i32.const 2700 i32.add
		call $ticks_from_sixty)

	;; Destructible gifts recur sooner than hazardous actors: scaling both old
	;; endpoints by 80% yields an inclusive 36--96 second independent interval.
	(func $random_package_spawn_ticks (result i32)
		call $rand_u32 i32.const 3601 i32.rem_u i32.const 2160 i32.add
		call $ticks_from_sixty)

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
		i64.const 60000000 call $scale_per_level local.set $cap
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
	;; Selects the visible suffix of a six-digit HUD buffer so reserve counts do
	;; not acquire misleading leading zeroes as extra ships accumulate.
	(func $decimal_digits (param $value i32) (result i32)
		local.get $value i32.const 100000 i32.ge_u (if (then i32.const 6 return))
		local.get $value i32.const 10000 i32.ge_u (if (then i32.const 5 return))
		local.get $value i32.const 1000 i32.ge_u (if (then i32.const 4 return))
		local.get $value i32.const 100 i32.ge_u (if (then i32.const 3 return))
		local.get $value i32.const 10 i32.ge_u (if (then i32.const 2 return))
		i32.const 1)

	(func $splash_alpha (result i32)
		(local $remaining i32) (local $elapsed i32) (local $one_second i32) (local $three_seconds i32)
		global.get $state_banner_ticks_address i32.load local.set $remaining
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

	;; Scatters one star deterministically from the world seed and its index
	;; without drawing from the gameplay stream, so a resize cannot perturb
	;; asteroid spawns and every seed still paints its own sky. The body is the
	;; mulberry32 finalizer applied to a seed/index/axis mix rather than to
	;; running state, which is why it needs no stored stream of its own. The
	;; multipliers are the golden-ratio and MurmurHash3 constants, chosen so
	;; adjacent indices and the two axes decorrelate rather than march in step.
	(func $star_hash (param $index i32) (param $axis i32) (result i32)
		(local $t i32)
		global.get $state_seed_address i32.load
		local.get $index i32.const 0x9e3779b9 i32.mul i32.add
		local.get $axis i32.const 0x85ebca6b i32.mul i32.add
		i32.const 0x6d2b79f5 i32.add local.set $t
		local.get $t local.get $t i32.const 15 i32.shr_u i32.xor
		local.get $t i32.const 1 i32.or i32.mul local.set $t
		local.get $t
		local.get $t local.get $t local.get $t i32.const 7 i32.shr_u i32.xor
		local.get $t i32.const 61 i32.or i32.mul i32.add i32.xor local.set $t
		local.get $t local.get $t i32.const 14 i32.shr_u i32.xor)

	(func $regenerate_stars
		(local $index i32) (local $address i32)
		(block $done
			(loop $again
				local.get $index i32.const 100 i32.ge_u br_if $done
				local.get $index call $star_address local.set $address
				local.get $address
				local.get $index i32.const 0 call $star_hash i32.const 0xffff i32.and
				i64.extend_i32_u global.get $state_width_address i64.load i64.mul
				i64.const 65536 i64.div_u i64.store
				local.get $address i32.const 8 i32.add
				local.get $index i32.const 1 call $star_hash i32.const 0xffff i32.and
				i64.extend_i32_u global.get $state_height_address i64.load i64.mul
				i64.const 65536 i64.div_u i64.store
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

	;; Five initial parents plus one per completed level is exactly a linear 20%
	;; increase from the baseline, capped only by the physical 32-record pool.
	(func $spawn_wave
		(local $index i32) (local $count i32) (local $address i32)
		(local $edge i32) (local $x i64) (local $y i64) (local $radius i64)
		global.get $state_level_address i32.load i32.const 4 i32.add local.set $count
		local.get $count i32.const 32 i32.gt_u (if (then i32.const 32 local.set $count))
		(block $done
			(loop $again
				local.get $index local.get $count i32.ge_u br_if $done
				local.get $index call $asteroid_address local.set $address
				call $rand_u32 i32.const 4 i32.rem_u local.set $edge
				call $rand_unit global.get $state_width_address i64.load call $fixed_mul local.set $x
				call $rand_unit global.get $state_height_address i64.load call $fixed_mul local.set $y
				local.get $edge i32.eqz (if (then i64.const 100000000 local.set $y))
				local.get $edge i32.const 1 i32.eq
				(if (then global.get $state_width_address i64.load i64.const 100000000 i64.sub local.set $x))
				local.get $edge i32.const 2 i32.eq
				(if (then global.get $state_height_address i64.load i64.const 100000000 i64.sub local.set $y))
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
		global.get $state_tick_address i32.const 0 i32.const 32768 memory.fill
		global.get $touch_contacts_base i32.const 0
		global.get $touch_contact_capacity global.get $touch_contact_stride i32.mul
		memory.fill
		i32.const 0 global.set $held_thrust_sources
		i32.const 0 global.set $held_fire_sources
		i32.const 0 global.set $held_rotate_left_sources
		i32.const 0 global.set $held_rotate_right_sources
		i32.const -1 global.set $ui_sent_gate_mode
		global.get $state_rng_address local.get $normalized_seed i32.store
		global.get $state_width_address local.get $width i64.store
		global.get $state_height_address local.get $height i64.store
		global.get $state_ship_x_address local.get $width i64.const 2 i64.div_s i64.store
		global.get $state_ship_y_address local.get $height i64.const 2 i64.div_s i64.store
		global.get $state_ship_vx_address i64.const 0 i64.store
		global.get $state_ship_vy_address i64.const 0 i64.store
		global.get $state_ship_dx_address i64.const 0 i64.store
		global.get $state_ship_dy_address i64.const -1000000 i64.store
		global.get $state_score_address i32.const 0 i32.store
		global.get $state_lives_address i32.const 3 i32.store
		global.get $state_level_address i32.const 1 i32.store
		global.get $flag_blossom_available call $store_flags
		global.get $state_last_fire_address i32.const -100 i32.store
		global.get $state_invulnerability_address i32.const 120 call $ticks_from_sixty i32.store
		global.get $state_next_life_address i32.const 30000 i32.store
		global.get $state_lifecycle_address i32.const 2 i32.store
		global.get $flag_gate call $set_flag
		global.get $state_lifecycle_ticks_address i32.const 0 i32.store
		global.get $state_seed_address local.get $normalized_seed i32.store
		global.get $state_blossom_rotation_address i64.const 0 i64.store
		global.get $state_banner_ticks_address i32.const 240 call $ticks_from_sixty i32.store
		call $regenerate_stars
		call $spawn_wave
		i32.const 16512 call $random_spawn_ticks i32.store
		i32.const 16516 call $random_package_spawn_ticks i32.store
		global.get $state_satellite_spawn_countdown_address call $random_spawn_ticks i32.store)

	(func (export "AE_init") (param $seed_low i32) (param $seed_high i32)
		(param $width f32) (param $height f32) (result i32)
		;; These describe the current host instance, not snapshotted game state.
		;; A fresh instance relearns them from device-change or raw-touch events.
		i32.const 0 global.set $coarse_pointer_mode
		i32.const 0 global.set $touch_mode_seen
		i32.const 0 global.set $pause_opened_help
		local.get $seed_low local.get $seed_high i32.xor
		local.get $width call $from_host local.get $height call $from_host call $reset
		i32.const 0)

	;; A reload may replace the guest while an input is held, after the host has
	;; delivered its down edge but before its matching release. Clear only the
	;; four edge-latched controls; persistent mode toggles remain snapshotted.
	(func (export "AE_after_restore") (result i32)
		call $clear_held_controls
		i32.const -1 global.set $ui_sent_gate_mode
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
				local.get $index global.get $player_bullet_capacity i32.ge_u br_if $none
				local.get $index call $bullet_address local.set $address
				local.get $address i32.load i32.eqz
				(if (then local.get $address return))
				local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	(func $fire
		(local $address i32) (local $speed i64) (local $interval i32)
		call $fire_interval_ticks local.set $interval
		call $load_flags global.get $flag_blossom_active i32.and
		(if (then i32.const 2 local.set $interval))
		;; Ceiling division bounds the reward at no more than exactly twice the
		;; ordinary rate even when a difficulty interval contains an odd tick.
		i32.const 16520 i32.load i32.const 0 i32.gt_s
		i32.const 16592 i32.load i32.const 1 i32.eq i32.and
		(if (then local.get $interval i32.const 1 i32.add i32.const 2 i32.div_u local.set $interval))
		global.get $state_lifecycle_address i32.load i32.eqz
		call $load_flags global.get $flag_paused i32.and i32.eqz i32.and
		global.get $state_tick_address i32.load global.get $state_last_fire_address i32.load i32.sub local.get $interval i32.ge_s i32.and
		(if
			(then
				i32.const 16520 i32.load i32.const 0 i32.gt_s
				i32.const 16592 i32.load i32.eqz i32.and
				(if
					(then call $fire_laser)
					(else
						call $find_free_bullet local.tee $address
						(if
							(then
								call $load_flags global.get $flag_blossom_active i32.and
								(if (result i64)
									(then i64.const 450000000)
									(else call $bullet_speed_per_second))
								local.set $speed
								local.get $address i32.const 1 i32.store
								local.get $address i32.const 8 i32.add
								global.get $state_ship_x_address i64.load global.get $state_ship_dx_address i64.load i64.const 20000000 call $fixed_mul i64.add i64.store
								local.get $address i32.const 16 i32.add
								global.get $state_ship_y_address i64.load global.get $state_ship_dy_address i64.load i64.const 20000000 call $fixed_mul i64.add i64.store
								local.get $address i32.const 24 i32.add
								global.get $state_ship_vx_address i64.load global.get $state_ship_dx_address i64.load local.get $speed call $fixed_mul i64.add i64.store
								local.get $address i32.const 32 i32.add
								global.get $state_ship_vy_address i64.load global.get $state_ship_dy_address i64.load local.get $speed call $fixed_mul i64.add i64.store
								local.get $address i32.const 40 i32.add i64.const 0 i64.store
								global.get $state_last_fire_address global.get $state_tick_address i32.load i32.store
								i32.const 1 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))))))

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

	;; Replays the small geometric threshold sequence from its canonical origin,
	;; avoiding a second mutable gap field that could diverge after restoration.
	(func $next_free_ship_threshold (param $current i32) (result i32)
		(local $threshold i64) (local $gap i64)
		i64.const 30000 local.set $threshold
		i64.const 30000 local.set $gap
		(block $found (loop $advance
			local.get $threshold local.get $current i64.extend_i32_u i64.ge_u br_if $found
			local.get $gap i64.const 3 i64.mul i64.const 1 i64.add i64.const 2 i64.div_u local.set $gap
			local.get $threshold local.get $gap i64.add local.set $threshold
			br $advance))
		local.get $gap i64.const 3 i64.mul i64.const 1 i64.add i64.const 2 i64.div_u local.set $gap
		local.get $threshold local.get $gap i64.add local.set $threshold
		local.get $threshold i64.const 4294967295 i64.gt_u
		(if (then i32.const -1 return))
		local.get $threshold i32.wrap_i64)

	;; Centralizes score and extra-life accounting so non-asteroid targets obey
	;; the same Kid Mode and geometrically increasing reserve-ship rules.
	(func $add_score (param $points i32)
		call $load_flags global.get $flag_kid_mode i32.and i32.eqz
		(if
			(then
				global.get $state_score_address global.get $state_score_address i32.load local.get $points i32.add i32.store
				global.get $state_score_address i32.load global.get $state_next_life_address i32.load i32.ge_u
				(if (then
					global.get $state_lives_address global.get $state_lives_address i32.load i32.const 1 i32.add i32.store
					global.get $state_next_life_address global.get $state_next_life_address i32.load call $next_free_ship_threshold i32.store
					i32.const 6 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))))

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
		local.get $score_hit (if (then local.get $points call $add_score)))

	;; Clips the ship's forward ray to the nearest viewport boundary. The result
	;; is finite by construction, so laser fire can never wrap across an edge.
	(func $compute_laser_end (result i64 i64)
		(local $dx i64) (local $dy i64) (local $tx i64) (local $ty i64) (local $time i64)
		global.get $state_ship_dx_address i64.load local.set $dx global.get $state_ship_dy_address i64.load local.set $dy
		i64.const 9000000000000 local.set $tx i64.const 9000000000000 local.set $ty
		local.get $dx i64.const 0 i64.gt_s
		(if (then global.get $state_width_address i64.load global.get $state_ship_x_address i64.load i64.sub global.get $scale i64.mul local.get $dx i64.div_s local.set $tx))
		local.get $dx i64.const 0 i64.lt_s
		(if (then i64.const 0 global.get $state_ship_x_address i64.load i64.sub global.get $scale i64.mul local.get $dx i64.div_s local.set $tx))
		local.get $dy i64.const 0 i64.gt_s
		(if (then global.get $state_height_address i64.load global.get $state_ship_y_address i64.load i64.sub global.get $scale i64.mul local.get $dy i64.div_s local.set $ty))
		local.get $dy i64.const 0 i64.lt_s
		(if (then i64.const 0 global.get $state_ship_y_address i64.load i64.sub global.get $scale i64.mul local.get $dy i64.div_s local.set $ty))
		local.get $tx local.get $ty i64.lt_s
		(if (result i64) (then local.get $tx) (else local.get $ty)) local.set $time
		global.get $state_ship_x_address i64.load local.get $dx local.get $time call $fixed_mul i64.add
		global.get $state_ship_y_address i64.load local.get $dy local.get $time call $fixed_mul i64.add)

	;; Tests a rock against the finite beam using whole-pixel projection; this
	;; avoids fixed-point cross-product overflow while retaining ample precision.
	(func $segment_circle_hit
		(param $start_x i64) (param $start_y i64) (param $end_x i64) (param $end_y i64)
		(param $center_x i64) (param $center_y i64) (param $radius i64) (result i32)
		(local $sx i64) (local $sy i64) (local $cx i64) (local $cy i64)
		(local $abx i64) (local $aby i64) (local $apx i64) (local $apy i64)
		(local $dot i64) (local $length_squared i64) (local $nearest_x i64) (local $nearest_y i64)
		(local $distance_x i64) (local $distance_y i64) (local $r i64)
		local.get $start_x global.get $scale i64.div_s local.set $sx
		local.get $start_y global.get $scale i64.div_s local.set $sy
		local.get $end_x global.get $scale i64.div_s local.get $sx i64.sub local.set $abx
		local.get $end_y global.get $scale i64.div_s local.get $sy i64.sub local.set $aby
		local.get $center_x global.get $scale i64.div_s local.set $cx
		local.get $center_y global.get $scale i64.div_s local.set $cy
		local.get $cx local.get $sx i64.sub local.set $apx
		local.get $cy local.get $sy i64.sub local.set $apy
		local.get $abx local.get $abx i64.mul local.get $aby local.get $aby i64.mul i64.add local.set $length_squared
		local.get $length_squared i64.eqz (if (then i32.const 0 return))
		local.get $apx local.get $abx i64.mul local.get $apy local.get $aby i64.mul i64.add local.set $dot
		local.get $dot i64.const 0 i64.le_s
		(if
			(then local.get $sx local.set $nearest_x local.get $sy local.set $nearest_y)
			(else
				local.get $dot local.get $length_squared i64.ge_s
				(if
					(then local.get $sx local.get $abx i64.add local.set $nearest_x local.get $sy local.get $aby i64.add local.set $nearest_y)
					(else
						local.get $sx local.get $abx local.get $dot i64.mul local.get $length_squared i64.div_s i64.add local.set $nearest_x
						local.get $sy local.get $aby local.get $dot i64.mul local.get $length_squared i64.div_s i64.add local.set $nearest_y))))
		local.get $cx local.get $nearest_x i64.sub local.set $distance_x
		local.get $cy local.get $nearest_y i64.sub local.set $distance_y
		local.get $radius global.get $scale i64.div_s i64.const 2 i64.add local.set $r
		local.get $distance_x local.get $distance_x i64.mul
		local.get $distance_y local.get $distance_y i64.mul i64.add
		local.get $r local.get $r i64.mul i64.le_s)

	;; Compares one actor/asteroid relative trajectory over the next two seconds.
	;; The explicit current-position check also covers a zero-length relative path,
	;; which the projection helper intentionally treats as non-segmental.
	(func $foreign_trajectory_unsafe
		(param $actor_x i64) (param $actor_y i64)
		(param $actor_end_x i64) (param $actor_end_y i64)
		(param $asteroid_x i64) (param $asteroid_y i64)
		(param $asteroid_end_x i64) (param $asteroid_end_y i64)
		(param $radius i64) (result i32)
		local.get $actor_x local.get $actor_y
		local.get $asteroid_x local.get $asteroid_y local.get $radius call $distance_lt
		(if (then i32.const 1 return))
		local.get $asteroid_x local.get $actor_x i64.sub
		local.get $asteroid_y local.get $actor_y i64.sub
		local.get $asteroid_end_x local.get $actor_end_x i64.sub
		local.get $asteroid_end_y local.get $actor_end_y i64.sub
		i64.const 0 i64.const 0 local.get $radius call $segment_circle_hit)

	;; Classifies the complete active asteroid set against a proposed foreign
	;; actor entry. Nine toroidal asteroid images preserve near-future wraparound;
	;; a ten-pixel fairness margin absorbs whole-pixel projection and small turns.
	(func $foreign_spawn_safe
		(param $candidate_x i64) (param $candidate_y i64)
		(param $candidate_vx i64) (param $candidate_vy i64)
		(param $candidate_radius i64) (result i32)
		(local $index i32) (local $asteroid i32)
		(local $actor_end_x i64) (local $actor_end_y i64)
		(local $asteroid_x i64) (local $asteroid_y i64)
		(local $asteroid_end_x i64) (local $asteroid_end_y i64)
		(local $period_x i64) (local $period_y i64) (local $radius i64)
		(local $x_offset i32) (local $y_offset i32)
		local.get $candidate_x local.get $candidate_vx i64.const 2 i64.mul i64.add local.set $actor_end_x
		local.get $candidate_y local.get $candidate_vy i64.const 2 i64.mul i64.add local.set $actor_end_y
		global.get $state_width_address i64.load i64.const 100000000 i64.add local.set $period_x
		global.get $state_height_address i64.load i64.const 100000000 i64.add local.set $period_y
		(block $done (loop $asteroids
			local.get $index i32.const 32 i32.ge_u br_if $done
			local.get $index call $asteroid_address local.set $asteroid
			local.get $asteroid i32.load
			(if (then
				local.get $candidate_radius
				local.get $asteroid i32.const 48 i32.add i64.load i64.add
				i64.const 10000000 i64.add local.set $radius
				i32.const -1 local.set $x_offset
				(block $x_done (loop $x_images
					local.get $x_offset i32.const 1 i32.gt_s br_if $x_done
					i32.const -1 local.set $y_offset
					(block $y_done (loop $y_images
						local.get $y_offset i32.const 1 i32.gt_s br_if $y_done
						local.get $asteroid i32.const 16 i32.add i64.load
						local.get $x_offset i64.extend_i32_s local.get $period_x i64.mul i64.add
						local.set $asteroid_x
						local.get $asteroid i32.const 24 i32.add i64.load
						local.get $y_offset i64.extend_i32_s local.get $period_y i64.mul i64.add
						local.set $asteroid_y
						local.get $asteroid_x
						local.get $asteroid i32.const 32 i32.add i64.load i64.const 2 i64.mul i64.add
						local.set $asteroid_end_x
						local.get $asteroid_y
						local.get $asteroid i32.const 40 i32.add i64.load i64.const 2 i64.mul i64.add
						local.set $asteroid_end_y
						local.get $candidate_x local.get $candidate_y
						local.get $actor_end_x local.get $actor_end_y
						local.get $asteroid_x local.get $asteroid_y
						local.get $asteroid_end_x local.get $asteroid_end_y local.get $radius
						call $foreign_trajectory_unsafe
						(if (then i32.const 0 return))
						local.get $y_offset i32.const 1 i32.add local.set $y_offset
						br $y_images))
					local.get $x_offset i32.const 1 i32.add local.set $x_offset
					br $x_images))))
			local.get $index i32.const 1 i32.add local.set $index
			br $asteroids))
		i32.const 1)

	;; An unsafe proposal waits one simulated second before drawing a fresh
	;; candidate, bounding retry work without eventually forcing an unfair entry.
	(func $foreign_spawn_retry_ticks (result i32)
		i32.const 60 call $ticks_from_sixty)

	;; Snapshots the 32 pre-fire asteroid slots, then lets one finite beam pierce
	;; all members without recursively targeting children created by splitting.
	(func $fire_laser
		(local $index i32) (local $address i32) (local $snapshot i32)
		(local $package_snapshot i32) (local $satellite_snapshot i32)
		(local $end_x i64) (local $end_y i64)
		i32.const 16528 global.get $state_ship_x_address i64.load i64.store
		i32.const 16536 global.get $state_ship_y_address i64.load i64.store
		call $compute_laser_end local.set $end_y local.set $end_x
		i32.const 16544 local.get $end_x i64.store i32.const 16552 local.get $end_y i64.store
		i32.const 16524 i32.const 4 call $ticks_from_sixty i32.store
		i32.const 16464 i32.load local.set $package_snapshot
		global.get $state_satellite_active_address i32.load local.set $satellite_snapshot
		(block $snapshot_done (loop $snapshot_loop
			local.get $index i32.const 32 i32.ge_u br_if $snapshot_done
			local.get $index call $asteroid_address i32.load
			(if (then local.get $snapshot i32.const 1 local.get $index i32.shl i32.or local.set $snapshot))
			local.get $index i32.const 1 i32.add local.set $index br $snapshot_loop))
		i32.const 0 local.set $index
		(block $hits_done (loop $hits
			local.get $index i32.const 32 i32.ge_u br_if $hits_done
			local.get $snapshot i32.const 1 local.get $index i32.shl i32.and
			(if (then
				local.get $index call $asteroid_address local.set $address
				local.get $address i32.load
				(if (then
					i32.const 16528 i64.load i32.const 16536 i64.load
					local.get $end_x local.get $end_y
					local.get $address i32.const 16 i32.add i64.load
					local.get $address i32.const 24 i32.add i64.load
					local.get $address i32.const 48 i32.add i64.load call $segment_circle_hit
					(if (then local.get $address i64.const 0 i64.const 0 i32.const 1 call $hit_asteroid))))))
			local.get $index i32.const 1 i32.add local.set $index br $hits))
		local.get $package_snapshot i32.const 16464 i32.load i32.and
		(if (then
			i32.const 16528 i64.load i32.const 16536 i64.load
			local.get $end_x local.get $end_y
			i32.const 16472 i64.load i32.const 16480 i64.load i32.const 16504 i64.load
			call $segment_circle_hit
			(if (then call $lose_package))))
		i32.const 16032 i32.load
		(if (then
			i32.const 16528 i64.load i32.const 16536 i64.load
			local.get $end_x local.get $end_y
			i32.const 16040 i64.load i32.const 16048 i64.load i32.const 16064 i64.load
			call $segment_circle_hit
			(if (then i32.const 1 call $destroy_ufo))))
		local.get $satellite_snapshot global.get $state_satellite_active_address i32.load i32.and
		(if (then
			i32.const 16528 i64.load i32.const 16536 i64.load
			local.get $end_x local.get $end_y
			global.get $state_satellite_x_address i64.load
			global.get $state_satellite_y_address i64.load
			global.get $state_satellite_radius_address i64.load
			call $segment_circle_hit
			(if (then i32.const 1 call $destroy_satellite))))
		 global.get $state_last_fire_address global.get $state_tick_address i32.load i32.store
		i32.const 7 f32.const 1 f32.const 1 i32.const 0 call $audio drop)

	;; Tests a precomputed bullet segment against one asteroid. The expanded-AABB
	;; classifier rejects most pairs before the projection's divisions; its five-
	;; pixel margin exactly includes the three-pixel bullet allowance and the
	;; two-pixel whole-coordinate allowance inside $segment_circle_hit.
	(func $bullet_segment_hits_asteroid
		(param $start_x i64) (param $start_y i64) (param $end_x i64) (param $end_y i64)
		(param $use_segment i32) (param $asteroid i32) (result i32)
		(local $center_x i64) (local $center_y i64) (local $radius i64)
		(local $minimum i64) (local $maximum i64)
		local.get $asteroid i32.const 16 i32.add i64.load local.set $center_x
		local.get $asteroid i32.const 24 i32.add i64.load local.set $center_y
		local.get $asteroid i32.const 48 i32.add i64.load i64.const 5000000 i64.add local.set $radius
		local.get $use_segment i32.eqz
		(if (then
			local.get $end_x local.get $end_y
			local.get $center_x local.get $center_y local.get $radius call $distance_lt return))
		local.get $start_x local.get $end_x i64.lt_s
		(if (result i64) (then local.get $start_x) (else local.get $end_x)) local.set $minimum
		local.get $start_x local.get $end_x i64.gt_s
		(if (result i64) (then local.get $start_x) (else local.get $end_x)) local.set $maximum
		local.get $center_x local.get $radius i64.add local.get $minimum i64.lt_s (if (then i32.const 0 return))
		local.get $center_x local.get $radius i64.sub local.get $maximum i64.gt_s (if (then i32.const 0 return))
		local.get $start_y local.get $end_y i64.lt_s
		(if (result i64) (then local.get $start_y) (else local.get $end_y)) local.set $minimum
		local.get $start_y local.get $end_y i64.gt_s
		(if (result i64) (then local.get $start_y) (else local.get $end_y)) local.set $maximum
		local.get $center_y local.get $radius i64.add local.get $minimum i64.lt_s (if (then i32.const 0 return))
		local.get $center_y local.get $radius i64.sub local.get $maximum i64.gt_s (if (then i32.const 0 return))
		local.get $start_x local.get $start_y local.get $end_x local.get $end_y
		local.get $center_x local.get $center_y
		;; The projection helper adds the remaining two pixels after rescaling.
		local.get $radius i64.const 2000000 i64.sub call $segment_circle_hit)

	(func $check_bullet_collisions
		(local $bullet_index i32) (local $asteroid_index i32)
		(local $bullet i32) (local $asteroid i32) (local $factor i64)
		(local $impulse_x i64) (local $impulse_y i64)
		(local $start_x i64) (local $start_y i64) (local $end_x i64) (local $end_y i64)
		(local $use_segment i32)
		(block $bullets_done (loop $next_bullet
			local.get $bullet_index global.get $player_bullet_capacity i32.ge_u br_if $bullets_done
			local.get $bullet_index call $bullet_address local.set $bullet
			local.get $bullet i32.load
			(if
				(then
					local.get $bullet i32.const 8 i32.add i64.load local.set $end_x
					local.get $bullet i32.const 16 i32.add i64.load local.set $end_y
					local.get $end_x local.get $bullet i32.const 24 i32.add i64.load call $per_tick i64.sub local.set $start_x
					local.get $end_y local.get $bullet i32.const 32 i32.add i64.load call $per_tick i64.sub local.set $start_y
					i32.const 1 local.set $use_segment
					local.get $start_x i64.const -25000000 i64.lt_s
					local.get $start_x global.get $state_width_address i64.load i64.const 25000000 i64.add i64.gt_s i32.or
					local.get $start_y i64.const -25000000 i64.lt_s i32.or
					local.get $start_y global.get $state_height_address i64.load i64.const 25000000 i64.add i64.gt_s i32.or
					local.get $start_x local.get $end_x i64.eq
					local.get $start_y local.get $end_y i64.eq i32.and i32.or
					(if (then i32.const 0 local.set $use_segment))
					i32.const 0 local.set $asteroid_index
					(block $asteroids_done (loop $next_asteroid
						local.get $asteroid_index i32.const 32 i32.ge_u br_if $asteroids_done
						local.get $asteroid_index call $asteroid_address local.set $asteroid
						local.get $asteroid i32.load
						(if
							(then
								local.get $start_x local.get $start_y local.get $end_x local.get $end_y
								local.get $use_segment local.get $asteroid call $bullet_segment_hits_asteroid
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
				global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load
				local.get $address i32.const 16 i32.add i64.load local.get $address i32.const 24 i32.add i64.load
				local.get $address i32.const 48 i32.add i64.load i64.const 10000000 i64.add call $distance_lt
				(if (then local.get $address return))))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	;; Starts the ship death lifecycle without assuming what caused the impact.
	(func $begin_ship_destruction
		global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load i32.const 40
		global.get $state_ship_vx_address i64.load global.get $state_ship_vy_address i64.load call $spawn_particles
		global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load global.get $state_ship_vx_address i64.load global.get $state_ship_vy_address i64.load call $spawn_debris
		i32.const 3 f32.const 1 f32.const 1 i32.const 0 call $audio drop
		call $load_flags global.get $flag_kid_mode i32.and i32.eqz
		(if (then global.get $state_lives_address global.get $state_lives_address i32.load i32.const 1 i32.sub i32.store))
		global.get $state_lifecycle_address i32.const 1 i32.store
		global.get $state_lifecycle_ticks_address i32.const 120 call $ticks_from_sixty i32.store
		global.get $flag_blossom_active call $clear_flag)

	(func $begin_ship_explosion (param $asteroid i32)
		local.get $asteroid i64.const 0 i64.const 0 i32.const 0 call $hit_asteroid
		call $begin_ship_destruction)

	;; Rotates toward the latest pointer at the ordinary keyboard turn rate. A
	;; cross-product sign chooses the shortest direction; a sign change after
	;; one step detects overshoot and snaps to the exact normalized target.
	(func $rotate_toward_pointer
		(local $dx i64) (local $dy i64) (local $target_x i64) (local $target_y i64)
		(local $target_dx i64) (local $target_dy i64) (local $length i64)
		(local $angle i64) (local $sine i64) (local $cosine i64)
		(local $cross i64) (local $dot i64) (local $next_cross i64)
		(local $next_dx i64) (local $next_dy i64)
		global.get $state_ship_dx_address i64.load local.set $dx
		global.get $state_ship_dy_address i64.load local.set $dy
		i32.const 16568 i64.load global.get $state_ship_x_address i64.load i64.sub local.set $target_x
		i32.const 16576 i64.load global.get $state_ship_y_address i64.load i64.sub local.set $target_y
		local.get $target_x local.get $target_y call $fixed_hypot local.tee $length i64.eqz
		(if (then return))
		local.get $target_x global.get $scale i64.mul local.get $length i64.div_s local.set $target_dx
		local.get $target_y global.get $scale i64.mul local.get $length i64.div_s local.set $target_dy
		local.get $dx local.get $target_dy i64.mul
		local.get $dy local.get $target_dx i64.mul i64.sub local.set $cross
		local.get $cross i64.eqz
		(if (then
			local.get $dx local.get $target_dx i64.mul
			local.get $dy local.get $target_dy i64.mul i64.add local.set $dot
			local.get $dot i64.const 0 i64.ge_s
			(if (then
				global.get $state_ship_dx_address local.get $target_dx i64.store
				global.get $state_ship_dy_address local.get $target_dy i64.store
				return))))
		call $rotation_per_second call $per_tick local.tee $angle call $small_sine local.set $sine
		local.get $angle call $small_cosine local.set $cosine
		local.get $cross i64.const 0 i64.ge_s
		(if
			(then
				local.get $dx local.get $cosine call $fixed_mul
				local.get $dy local.get $sine call $fixed_mul i64.sub local.set $next_dx
				local.get $dy local.get $cosine call $fixed_mul
				local.get $dx local.get $sine call $fixed_mul i64.add local.set $next_dy
				local.get $next_dx local.get $target_dy i64.mul
				local.get $next_dy local.get $target_dx i64.mul i64.sub local.set $next_cross
				local.get $next_cross i64.const 0 i64.le_s
				(if (then local.get $target_dx local.set $next_dx local.get $target_dy local.set $next_dy)))
			(else
				local.get $dx local.get $cosine call $fixed_mul
				local.get $dy local.get $sine call $fixed_mul i64.add local.set $next_dx
				local.get $dy local.get $cosine call $fixed_mul
				local.get $dx local.get $sine call $fixed_mul i64.sub local.set $next_dy
				local.get $next_dx local.get $target_dy i64.mul
				local.get $next_dy local.get $target_dx i64.mul i64.sub local.set $next_cross
				local.get $next_cross i64.const 0 i64.ge_s
				(if (then local.get $target_dx local.set $next_dx local.get $target_dy local.set $next_dy))))
		global.get $state_ship_dx_address local.get $next_dx i64.store
		global.get $state_ship_dy_address local.get $next_dy i64.store)

	(func $rotate_ship
		(local $dx i64) (local $dy i64) (local $next_dx i64) (local $next_dy i64)
		(local $angle i64) (local $sine i64) (local $cosine i64)
		i32.const 16584 i32.load
		(if (then call $rotate_toward_pointer return))
		global.get $state_ship_dx_address i64.load local.set $dx global.get $state_ship_dy_address i64.load local.set $dy
		call $rotation_per_second call $per_tick local.tee $angle call $small_sine local.set $sine
		local.get $angle call $small_cosine local.set $cosine
		call $load_flags global.get $flag_rotate_right i32.and
		(if (then
			local.get $dx local.get $cosine call $fixed_mul local.get $dy local.get $sine call $fixed_mul i64.sub local.set $next_dx
			local.get $dy local.get $cosine call $fixed_mul local.get $dx local.get $sine call $fixed_mul i64.add local.set $next_dy
			local.get $next_dx local.set $dx local.get $next_dy local.set $dy))
		call $load_flags global.get $flag_rotate_left i32.and
		(if (then
			local.get $dx local.get $cosine call $fixed_mul local.get $dy local.get $sine call $fixed_mul i64.add local.set $next_dx
			local.get $dy local.get $cosine call $fixed_mul local.get $dx local.get $sine call $fixed_mul i64.sub local.set $next_dy
			local.get $next_dx local.set $dx local.get $next_dy local.set $dy))
		global.get $state_ship_dx_address local.get $dx i64.store global.get $state_ship_dy_address local.get $dy i64.store)

	(func $activate_death_blossom
		(local $flags i32)
		call $load_flags local.set $flags
		global.get $state_lifecycle_address i32.load i32.eqz
		local.get $flags global.get $flag_blocks_blossom_activation i32.and i32.eqz i32.and
		local.get $flags global.get $flag_blossom_available i32.and i32.eqz i32.eqz i32.and
		(if (then
			local.get $flags global.get $flag_blossom_active i32.or
			global.get $flag_blossom_available i32.const -1 i32.xor i32.and call $store_flags
			global.get $state_blossom_rotation_address i64.const 0 i64.store
			i32.const 5 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))

	(func $update_death_blossom
		(local $dx i64) (local $dy i64) (local $next_dx i64) (local $next_dy i64)
		(local $angle i64) (local $sine i64) (local $cosine i64) (local $rotation i64)
		global.get $state_ship_dx_address i64.load local.set $dx global.get $state_ship_dy_address i64.load local.set $dy
		i64.const 7200000 call $per_tick local.tee $angle call $small_sine local.set $sine
		local.get $angle call $small_cosine local.set $cosine
		local.get $dx local.get $cosine call $fixed_mul local.get $dy local.get $sine call $fixed_mul i64.add local.set $next_dx
		local.get $dy local.get $cosine call $fixed_mul local.get $dx local.get $sine call $fixed_mul i64.sub local.set $next_dy
		global.get $state_ship_dx_address local.get $next_dx i64.store global.get $state_ship_dy_address local.get $next_dy i64.store
		global.get $state_blossom_rotation_address i64.load local.get $angle i64.add local.set $rotation
		local.get $rotation i64.const 75398224 i64.ge_s
		(if
			(then
				global.get $state_blossom_rotation_address i64.const 75398224 i64.store
				call $fire
				global.get $flag_blossom_active call $clear_flag)
			(else
				global.get $state_blossom_rotation_address local.get $rotation i64.store
				call $fire)))

	(func $update_ship
		(local $flags i32)
		call $load_flags local.set $flags
		local.get $flags global.get $flag_blossom_active i32.and
		(if
			(then call $update_death_blossom)
			(else
				call $rotate_ship
				local.get $flags global.get $flag_thrust i32.and
				(if (then
					global.get $state_ship_vx_address global.get $state_ship_vx_address i64.load global.get $state_ship_dx_address i64.load call $ship_acceleration_per_second call $per_tick call $fixed_mul i64.add i64.store
					global.get $state_ship_vy_address global.get $state_ship_vy_address i64.load global.get $state_ship_dy_address i64.load call $ship_acceleration_per_second call $per_tick call $fixed_mul i64.add i64.store
					global.get $state_tick_address i32.load
					i32.const 4 call $ticks_from_sixty i32.rem_u i32.eqz
					(if (then i32.const 4 f32.const 0.18 f32.const 1 i32.const 0 call $audio drop))))
				local.get $flags global.get $flag_fire i32.and
				local.get $flags global.get $flag_auto_fire i32.and i32.or
				(if (then call $fire))))
		global.get $state_ship_vx_address global.get $state_ship_vx_address i64.load i64.const 995000 call $retention_factor_per_tick call $fixed_mul i64.store
		global.get $state_ship_vy_address global.get $state_ship_vy_address i64.load i64.const 995000 call $retention_factor_per_tick call $fixed_mul i64.store
		global.get $state_ship_x_address
		global.get $state_ship_x_address i64.load global.get $state_ship_vx_address i64.load call $per_tick i64.add i64.const 0 global.get $state_width_address i64.load call $wrap i64.store
		global.get $state_ship_y_address
		global.get $state_ship_y_address i64.load global.get $state_ship_vy_address i64.load call $per_tick i64.add i64.const 0 global.get $state_height_address i64.load call $wrap i64.store)

	(func $update_bullets
		(local $index i32) (local $address i32) (local $vx i64) (local $vy i64)
		(block $done (loop $again
			local.get $index global.get $player_bullet_capacity i32.ge_u br_if $done
			local.get $index call $bullet_address local.set $address
			local.get $address i32.load
			(if (then
				local.get $address i32.const 24 i32.add i64.load local.set $vx
				local.get $address i32.const 32 i32.add i64.load local.set $vy
				local.get $address i32.const 8 i32.add
				local.get $address i32.const 8 i32.add i64.load local.get $vx call $per_tick i64.add
				i64.store
				local.get $address i32.const 16 i32.add
				local.get $address i32.const 16 i32.add i64.load local.get $vy call $per_tick i64.add
				i64.store
				;; A projectile ends on a hit or at the viewport edge and never
				;; wraps, so a shot leaves the screen instead of evaporating in
				;; open space. The margin lets it exit fully rather than blinking
				;; out while still partly drawn.
				local.get $address i32.const 8 i32.add i64.load
				i64.const -25000000 i64.lt_s
				local.get $address i32.const 8 i32.add i64.load
				global.get $state_width_address i64.load i64.const 25000000 i64.add
				i64.gt_s i32.or
				local.get $address i32.const 16 i32.add i64.load
				i64.const -25000000 i64.lt_s i32.or
				local.get $address i32.const 16 i32.add i64.load
				global.get $state_height_address i64.load i64.const 25000000 i64.add
				i64.gt_s i32.or
				(if (then local.get $address i32.const 0 i32.store))))
			local.get $index i32.const 1 i32.add local.set $index br $again)))

	;; Retires a zero-world-velocity shot after every player-target collision
	;; pass, giving an overlapping target one chance to claim a physically
	;; reachable cancellation shot without leaking the record forever.
	(func $retire_inert_bullets
		(local $index i32) (local $address i32)
		(block $done (loop $again
			local.get $index global.get $player_bullet_capacity i32.ge_u br_if $done
			local.get $index call $bullet_address local.set $address
			local.get $address i32.load
			(if (then
				local.get $address i32.const 24 i32.add i64.load i64.eqz
				local.get $address i32.const 32 i32.add i64.load i64.eqz i32.and
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
				local.get $address i32.const 24 i32.add local.get $address i32.const 24 i32.add i64.load i64.const 980000 call $retention_factor_per_tick call $fixed_mul i64.store
				local.get $address i32.const 32 i32.add local.get $address i32.const 32 i32.add i64.load i64.const 980000 call $retention_factor_per_tick call $fixed_mul i64.store
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
				i64.const -50000000 global.get $state_width_address i64.load i64.const 50000000 i64.add call $wrap i64.store
				local.get $address i32.const 24 i32.add
				local.get $address i32.const 24 i32.add i64.load local.get $address i32.const 40 i32.add i64.load call $per_tick i64.add
				i64.const -50000000 global.get $state_height_address i64.load i64.const 50000000 i64.add call $wrap i64.store
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
				local.get $address i32.const 24 i32.add local.get $address i32.const 24 i32.add i64.load i64.const 990000 call $retention_factor_per_tick call $fixed_mul i64.store
				local.get $address i32.const 32 i32.add local.get $address i32.const 32 i32.add i64.load i64.const 990000 call $retention_factor_per_tick call $fixed_mul i64.store
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

	(func $find_free_enemy_bullet (result i32)
		(local $index i32) (local $address i32)
		(block $none (loop $again
			local.get $index i32.const 8 i32.ge_u br_if $none
			local.get $index call $enemy_bullet_address local.set $address
			local.get $address i32.load i32.eqz (if (then local.get $address return))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 0)

	;; Selects the closest asteroid inside a 300-pixel forward safety corridor;
	;; this defensive target takes precedence over opportunistic player fire.
	(func $find_ufo_threat (result i32)
		(local $index i32) (local $address i32) (local $best i32)
		(local $dx i64) (local $abs_dx i64) (local $best_dx i64) (local $dy i64)
		i64.const 301000000 local.set $best_dx
		(block $done (loop $again
			local.get $index i32.const 32 i32.ge_u br_if $done
			local.get $index call $asteroid_address local.set $address
			local.get $address i32.load
			(if (then
				local.get $address i32.const 16 i32.add i64.load i32.const 16040 i64.load i64.sub local.tee $dx
				i64.const 0 i64.lt_s (if (result i64) (then i64.const 0 local.get $dx i64.sub) (else local.get $dx)) local.set $abs_dx
				local.get $address i32.const 24 i32.add i64.load i32.const 16048 i64.load i64.sub local.tee $dy
				i64.const 0 i64.lt_s (if (result i64) (then i64.const 0 local.get $dy i64.sub) (else local.get $dy)) local.set $dy
				local.get $dx i32.const 16036 i32.load i64.extend_i32_s i64.mul i64.const 0 i64.gt_s
				local.get $abs_dx i64.const 300000000 i64.le_s i32.and
				local.get $dy local.get $address i32.const 48 i32.add i64.load i64.const 30000000 i64.add i64.le_s i32.and
				local.get $abs_dx local.get $best_dx i64.lt_s i32.and
				(if (then local.get $address local.set $best local.get $abs_dx local.set $best_dx))))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		local.get $best)

	;; Fires one finite enemy projectile. Immediate asteroid defense wins first,
	;; then an active package is predictively targeted before player/random aim.
	(func $fire_ufo
		(local $bullet i32) (local $target i32) (local $vx i64) (local $vy i64)
		(local $dx i64) (local $dy i64) (local $length i64)
		call $find_free_enemy_bullet local.set $bullet
		local.get $bullet
		(if
			(then
				call $find_ufo_threat local.set $target
				local.get $target
				(if
					(then
						i32.const 16040 i64.load i32.const 16048 i64.load
						i32.const 16056 i64.load i64.const 0
						local.get $target i32.const 16 i32.add i64.load local.get $target i32.const 24 i32.add i64.load
						local.get $target i32.const 32 i32.add i64.load local.get $target i32.const 40 i32.add i64.load
						call $ufo_projectile_speed call $aim_projectile_velocity
						local.set $vy local.set $vx)
					(else
						i32.const 16464 i32.load
						(if
							(then
								i32.const 16040 i64.load i32.const 16048 i64.load
								i32.const 16056 i64.load i64.const 0
								i32.const 16472 i64.load i32.const 16480 i64.load
								i32.const 16488 i64.load i32.const 16496 i64.load
								call $ufo_projectile_speed call $aim_projectile_velocity
								local.set $vy local.set $vx)
							(else
								call $rand_u32 i32.const 1 i32.and i32.eqz
								(if
									(then
										i32.const 16040 i64.load i32.const 16048 i64.load
										i32.const 16056 i64.load i64.const 0
										global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load
										global.get $state_ship_vx_address i64.load global.get $state_ship_vy_address i64.load
										call $ufo_projectile_speed call $aim_projectile_velocity
										local.set $vy local.set $vx)
									(else
										call $rand_signed local.set $dx call $rand_signed local.set $dy
										local.get $dx local.get $dy call $fixed_hypot local.set $length
										local.get $length i64.eqz
										(if (then i64.const 1000000 local.set $dx i64.const 1000000 local.set $length))
										i32.const 16056 i64.load local.get $dx call $ufo_projectile_speed i64.mul local.get $length i64.div_s i64.add local.set $vx
										local.get $dy call $ufo_projectile_speed i64.mul local.get $length i64.div_s local.set $vy))))))
				local.get $bullet i32.const 1 i32.store
				local.get $bullet i32.const 8 i32.add i32.const 16040 i64.load i64.store
				local.get $bullet i32.const 16 i32.add i32.const 16048 i64.load i64.store
				local.get $bullet i32.const 24 i32.add local.get $vx i64.store
				local.get $bullet i32.const 32 i32.add local.get $vy i64.store
				local.get $bullet i32.const 40 i32.add i64.const 0 i64.store
				i32.const 1 f32.const 0.55 f32.const 0.72 i32.const 0 call $audio drop))
		i32.const 16072 call $ufo_fire_interval_ticks i32.store)

	;; Advances non-wrapping hostile projectiles and retires them once outside a
	;; small viewport margin, keeping their lifetime naturally bounded.
	(func $update_enemy_bullets
		(local $index i32) (local $address i32) (local $x i64) (local $y i64)
		(block $done (loop $again
			local.get $index i32.const 8 i32.ge_u br_if $done
			local.get $index call $enemy_bullet_address local.set $address
			local.get $address i32.load
			(if (then
				local.get $address i32.const 8 i32.add
				local.get $address i32.const 8 i32.add i64.load
				local.get $address i32.const 24 i32.add i64.load call $per_tick i64.add local.tee $x i64.store
				local.get $address i32.const 16 i32.add
				local.get $address i32.const 16 i32.add i64.load
				local.get $address i32.const 32 i32.add i64.load call $per_tick i64.add local.tee $y i64.store
				local.get $x i64.const -25000000 i64.lt_s
				local.get $x global.get $state_width_address i64.load i64.const 25000000 i64.add i64.gt_s i32.or
				local.get $y i64.const -25000000 i64.lt_s i32.or
				local.get $y global.get $state_height_address i64.load i64.const 25000000 i64.add i64.gt_s i32.or
				(if (then local.get $address i32.const 0 i32.store))))
			local.get $index i32.const 1 i32.add local.set $index br $again)))

	;; Resolves a hazardous explosion against a pre-hit asteroid snapshot so
	;; split children survive; the attribution bit controls all resulting score.
	(func $start_hazardous_blast (param $x i64) (param $y i64) (param $score_hit i32)
		(local $index i32) (local $asteroid i32) (local $hit_mask i32)
		i32.const 26624 i32.const 1 i32.store
		i32.const 26628 local.get $score_hit i32.store
		i32.const 26632 local.get $x i64.store
		i32.const 26640 local.get $y i64.store
		i32.const 26648 i32.const 0 i32.store
		(block $snapshot_done (loop $snapshot
			local.get $index i32.const 32 i32.ge_u br_if $snapshot_done
			local.get $index call $asteroid_address local.set $asteroid
			local.get $asteroid i32.load
			(if (then
				local.get $x local.get $y
				local.get $asteroid i32.const 16 i32.add i64.load
				local.get $asteroid i32.const 24 i32.add i64.load
				call $hazardous_blast_radius
				local.get $asteroid i32.const 48 i32.add i64.load i64.add
				call $distance_lt
				(if (then
					local.get $hit_mask i32.const 1 local.get $index i32.shl i32.or local.set $hit_mask))))
			local.get $index i32.const 1 i32.add local.set $index br $snapshot))
		i32.const 0 local.set $index
		(block $hits_done (loop $hits
			local.get $index i32.const 32 i32.ge_u br_if $hits_done
			local.get $hit_mask i32.const 1 local.get $index i32.shl i32.and
			(if (then
				local.get $index call $asteroid_address
				i64.const 0 i64.const 0 local.get $score_hit call $hit_asteroid))
			local.get $index i32.const 1 i32.add local.set $index br $hits))
		global.get $state_lifecycle_address i32.load i32.eqz
		global.get $state_invulnerability_address i32.load i32.const 0 i32.le_s i32.and
			(if (then
				local.get $x local.get $y global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load
				call $hazardous_blast_radius i64.const 10000000 i64.add call $distance_lt
				(if (then call $begin_ship_destruction))))
		i32.const 12 f32.const 1 f32.const 1 i32.const 0 call $audio drop)

	;; Advances the 1.2-second expanding/contracting blast presentation from the
	;; fixed simulation clock; collision damage remains an immediate snapshot.
	(func $update_hazardous_blast
		i32.const 26624 i32.load
		(if (then
			i32.const 26648 i32.const 26648 i32.load i32.const 1 i32.add i32.store
			i32.const 26648 i32.load i32.const 72 call $ticks_from_sixty i32.ge_u
			(if (then i32.const 26624 i32.const 0 i32.store)))))

	;; Removes the saucer, starts its next independent appearance interval, and
	;; optionally awards the fixed player-kill bounty.
	(func $destroy_ufo (param $score_hit i32)
		i32.const 16032 i32.load
		(if (then
			i32.const 16040 i64.load i32.const 16048 i64.load i32.const 20
			i32.const 16056 i64.load i64.const 0 call $spawn_particles
			i32.const 16040 i64.load i32.const 16048 i64.load local.get $score_hit call $start_hazardous_blast
			i32.const 16032 i32.const 0 i32.store
			i32.const 16512 call $random_spawn_ticks i32.store
			local.get $score_hit (if (then i32.const 2000 call $add_score)))))

	;; Retires the reactor-bearing craft through one path so player attribution
	;; controls only collateral asteroid score, never an intrinsic bounty.
	(func $destroy_satellite (param $score_hit i32)
		global.get $state_satellite_active_address i32.load
		(if (then
			global.get $state_satellite_x_address i64.load
			global.get $state_satellite_y_address i64.load i32.const 30
			global.get $state_satellite_vx_address i64.load
			global.get $state_satellite_vy_address i64.load call $spawn_particles
			global.get $state_satellite_x_address i64.load
			global.get $state_satellite_y_address i64.load local.get $score_hit call $start_hazardous_blast
			local.get $score_hit
			(if (then
				global.get $state_satellite_quote_countdown_address
				i32.const 60 call $ticks_from_sixty i32.store))
			global.get $state_satellite_active_address i32.const 0 i32.store
			global.get $state_satellite_spawn_countdown_address call $random_spawn_ticks i32.store)))

	;; Defers the player-attributed digitized quote by one simulated second;
	;; collision deaths never arm this schema-backed countdown.
	(func $update_satellite_quote
		global.get $state_satellite_quote_countdown_address i32.load i32.const 0 i32.le_s
		(if (then return))
		global.get $state_satellite_quote_countdown_address i32.load i32.const 1 i32.le_s
		(if
			(then
				global.get $state_satellite_quote_countdown_address i32.const 0 i32.store
				i32.const 1 f32.const 1 f32.const 1 i32.const 0 call $sample_play drop)
			(else
				global.get $state_satellite_quote_countdown_address
				global.get $state_satellite_quote_countdown_address i32.load i32.const 1 i32.sub i32.store)))

	;; Creates a slow Voyager-like traversal from a seeded horizontal edge, with
	;; an independently seeded rotation sign and no appearance notification.
	(func $spawn_satellite
		(local $direction i32) (local $spin i32) (local $height_range i64)
		(local $x i64) (local $y i64) (local $vx i64) (local $vy i64)
		call $rand_u32 i32.const 1 i32.and
		(if (result i32) (then i32.const 1) (else i32.const -1)) local.set $direction
		call $rand_u32 i32.const 1 i32.and
		(if (result i32) (then i32.const 1) (else i32.const -1)) local.set $spin
		local.get $direction i32.const 1 i32.eq
		(if (result i64)
			(then i64.const -80000000)
			(else global.get $state_width_address i64.load i64.const 80000000 i64.add))
		local.set $x
		global.get $state_height_address i64.load i64.const 200000000 i64.sub local.set $height_range
		local.get $height_range i64.const 0 i64.lt_s (if (then i64.const 0 local.set $height_range))
		call $rand_unit local.get $height_range call $fixed_mul i64.const 100000000 i64.add local.set $y
		local.get $direction i64.extend_i32_s i64.const 42000000 i64.mul local.set $vx
		call $rand_signed i64.const 12000000 call $fixed_mul local.set $vy
		local.get $x local.get $y local.get $vx local.get $vy i64.const 60000000
		call $foreign_spawn_safe i32.eqz
		(if (then
			global.get $state_satellite_spawn_countdown_address
			call $foreign_spawn_retry_ticks i32.store
			return))
		global.get $state_satellite_active_address i32.const 1 i32.store
		global.get $state_satellite_direction_address local.get $direction i32.store
		global.get $state_satellite_x_address local.get $x i64.store
		global.get $state_satellite_y_address local.get $y i64.store
		global.get $state_satellite_vx_address local.get $vx i64.store
		global.get $state_satellite_vy_address local.get $vy i64.store
		global.get $state_satellite_dx_address i64.const 1000000 i64.store
		global.get $state_satellite_dy_address i64.const 0 i64.store
		global.get $state_satellite_radius_address i64.const 60000000 i64.store
		global.get $state_satellite_spin_address local.get $spin i32.store
		global.get $state_satellite_ping_countdown_address i32.const 90 call $ticks_from_sixty i32.store
		global.get $state_satellite_pulse_ticks_address i32.const 0 i32.store)

	;; Defers a due Voyager while fifteen or more asteroids are active, then
	;; retries every fixed tick without imposing a once-per-level appearance cap.
	(func $update_satellite_schedule
		(local $y i64) (local $bottom i64) (local $angle i64)
		(local $sine i64) (local $cosine i64) (local $dx i64) (local $dy i64)
		(local $next_dx i64) (local $next_dy i64)
		global.get $state_satellite_active_address i32.load
		(if
			(then
				global.get $state_satellite_x_address global.get $state_satellite_x_address i64.load
				global.get $state_satellite_vx_address i64.load call $per_tick i64.add i64.store
				global.get $state_satellite_y_address global.get $state_satellite_y_address i64.load
				global.get $state_satellite_vy_address i64.load call $per_tick i64.add local.tee $y i64.store
				global.get $state_height_address i64.load i64.const 80000000 i64.sub local.set $bottom
				local.get $y i64.const 80000000 i64.lt_s
				(if (then
					global.get $state_satellite_y_address i64.const 80000000 i64.store
					global.get $state_satellite_vy_address
					global.get $state_satellite_vy_address i64.load call $fixed_abs i64.store))
				local.get $y local.get $bottom i64.gt_s
				(if (then
					global.get $state_satellite_y_address local.get $bottom i64.store
					global.get $state_satellite_vy_address i64.const 0
					global.get $state_satellite_vy_address i64.load call $fixed_abs i64.sub i64.store))
				global.get $state_satellite_spin_address i32.load i64.extend_i32_s
				i64.const 180000 i64.mul call $per_tick local.tee $angle call $small_sine local.set $sine
				local.get $angle call $small_cosine local.set $cosine
				global.get $state_satellite_dx_address i64.load local.set $dx
				global.get $state_satellite_dy_address i64.load local.set $dy
				local.get $dx local.get $cosine call $fixed_mul
				local.get $dy local.get $sine call $fixed_mul i64.sub local.set $next_dx
				local.get $dx local.get $sine call $fixed_mul
				local.get $dy local.get $cosine call $fixed_mul i64.add local.set $next_dy
				global.get $state_satellite_dx_address local.get $next_dx i64.store
				global.get $state_satellite_dy_address local.get $next_dy i64.store
				global.get $state_satellite_pulse_ticks_address
				global.get $state_satellite_pulse_ticks_address i32.load i32.const 1 i32.add i32.store
				global.get $state_satellite_ping_countdown_address i32.load i32.const 1 i32.le_s
				(if
					(then
						i32.const 13 f32.const 0.28 f32.const 1 i32.const 0 call $audio drop
						global.get $state_satellite_ping_countdown_address
						i32.const 180 call $ticks_from_sixty i32.store)
					(else
						global.get $state_satellite_ping_countdown_address
						global.get $state_satellite_ping_countdown_address i32.load i32.const 1 i32.sub i32.store))
				global.get $state_satellite_x_address i64.load i64.const -90000000 i64.lt_s
				global.get $state_satellite_x_address i64.load global.get $state_width_address i64.load
				i64.const 90000000 i64.add i64.gt_s i32.or
				(if (then
					global.get $state_satellite_active_address i32.const 0 i32.store
					global.get $state_satellite_spawn_countdown_address call $random_spawn_ticks i32.store)))
			(else
				global.get $state_satellite_spawn_countdown_address i32.load i32.const 0 i32.gt_s
				(if (then
					global.get $state_satellite_spawn_countdown_address
					global.get $state_satellite_spawn_countdown_address i32.load i32.const 1 i32.sub i32.store))
				global.get $state_satellite_spawn_countdown_address i32.load i32.eqz
				call $asteroid_count i32.const 15 i32.lt_u i32.and
				(if (then call $spawn_satellite)))))

	;; Creates an independently scheduled package with slow horizontal travel and
	;; a seeded vertical drift component, distinct from the faster hostile UFO.
	(func $spawn_package
		(local $direction i32) (local $height_range i64)
		call $rand_u32 i32.const 1 i32.and
		(if (result i32) (then i32.const 1) (else i32.const -1)) local.set $direction
		i32.const 16464 i32.const 1 i32.store
		i32.const 16468 local.get $direction i32.store
		i32.const 16472
		local.get $direction i32.const 1 i32.eq
		(if (result i64)
			(then i64.const -30000000)
			(else global.get $state_width_address i64.load i64.const 30000000 i64.add))
		i64.store
		global.get $state_height_address i64.load i64.const 200000000 i64.sub local.set $height_range
		local.get $height_range i64.const 0 i64.lt_s (if (then i64.const 0 local.set $height_range))
		i32.const 16480 call $rand_unit local.get $height_range call $fixed_mul i64.const 100000000 i64.add i64.store
		i32.const 16488 local.get $direction i64.extend_i32_s i64.const 80000000 i64.mul i64.store
		i32.const 16496 call $rand_signed i64.const 35000000 call $fixed_mul i64.store
		i32.const 16504 i64.const 14000000 i64.store
		i32.const 9 f32.const 0.75 f32.const 1 i32.const 0 call $audio drop)

	;; Centralizes every missed or destroyed package outcome so exactly one
	;; negative cue and exactly one independent respawn interval are produced.
	(func $lose_package
		i32.const 16464 i32.load
		(if (then
			i32.const 16464 i32.const 0 i32.store
			i32.const 16516 call $random_package_spawn_ticks i32.store
			i32.const 11 f32.const 0.8 f32.const 1 i32.const 0 call $audio drop)))

	;; Moves one finite package traversal, reflecting its small vertical drift at
	;; safe margins while selecting the next interval only after it leaves.
	(func $update_package_schedule
		(local $y i64) (local $bottom i64)
		i32.const 16464 i32.load
		(if
			(then
				i32.const 16472 i32.const 16472 i64.load i32.const 16488 i64.load call $per_tick i64.add i64.store
				i32.const 16480 i32.const 16480 i64.load i32.const 16496 i64.load call $per_tick i64.add local.tee $y i64.store
				global.get $state_height_address i64.load i64.const 40000000 i64.sub local.set $bottom
				local.get $y i64.const 40000000 i64.lt_s
				(if (then
					i32.const 16480 i64.const 40000000 i64.store
					i32.const 16496 i32.const 16496 i64.load call $fixed_abs i64.store))
				local.get $y local.get $bottom i64.gt_s
				(if (then
					i32.const 16480 local.get $bottom i64.store
					i32.const 16496 i64.const 0 i32.const 16496 i64.load call $fixed_abs i64.sub i64.store))
				i32.const 16472 i64.load i64.const -40000000 i64.lt_s
				i32.const 16472 i64.load global.get $state_width_address i64.load i64.const 40000000 i64.add i64.gt_s i32.or
				(if (then call $lose_package)))
			(else
				i32.const 16516 i32.load i32.const 0 i32.gt_s
				(if (then i32.const 16516 i32.const 16516 i32.load i32.const 1 i32.sub i32.store))
				i32.const 16516 i32.load i32.eqz (if (then call $spawn_package)))))

	;; Collection atomically retires the package, draws a deterministic reward,
	;; grants it for 20 simulated seconds, and begins a fresh package schedule.
	(func $check_package_collection
		i32.const 16464 i32.load
		global.get $state_lifecycle_address i32.load i32.eqz i32.and
		(if (then
			i32.const 16472 i64.load i32.const 16480 i64.load
			global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load
			i32.const 16504 i64.load i64.const 10000000 i64.add call $distance_lt
			(if (then
				i32.const 16464 i32.const 0 i32.store
				i32.const 16592 call $rand_u32 i32.const 1 i32.and i32.store
				i32.const 16516 call $random_package_spawn_ticks i32.store
				i32.const 16520 i32.const 1200 call $ticks_from_sixty i32.store
				i32.const 10 f32.const 0.8 f32.const 1 i32.const 0 call $audio drop)))))

	;; Advances temporary power and beam-afterimage durations solely from
	;; simulation ticks, keeping pause and tests independent of wall-clock time.
	(func $update_power_timers
		i32.const 16520 i32.load i32.const 0 i32.gt_s
		(if (then i32.const 16520 i32.const 16520 i32.load i32.const 1 i32.sub i32.store))
		i32.const 16524 i32.load i32.const 0 i32.gt_s
		(if (then i32.const 16524 i32.const 16524 i32.load i32.const 1 i32.sub i32.store)))

	;; Creates one classic edge-to-edge saucer. Its next interval is not selected
	;; until this instance leaves play, preventing overlapping UFOs.
	(func $spawn_ufo
		(local $direction i32) (local $height_range i64)
		(local $x i64) (local $y i64) (local $vx i64) (local $radius i64)
		i32.const 16560 i32.const 16560 i32.load i32.const 1 i32.add i32.store
		call $rand_u32 i32.const 1 i32.and
		(if (result i32) (then i32.const 1) (else i32.const -1)) local.set $direction
		local.get $direction i32.const 1 i32.eq
		(if (result i64)
			(then i64.const -30000000)
			(else global.get $state_width_address i64.load i64.const 30000000 i64.add))
		local.set $x
		global.get $state_height_address i64.load i64.const 200000000 i64.sub local.set $height_range
		local.get $height_range i64.const 0 i64.lt_s
		(if (then i64.const 0 local.set $height_range))
		call $rand_unit local.get $height_range call $fixed_mul
		i64.const 100000000 i64.add local.set $y
		local.get $direction i64.extend_i32_s
		i64.const 140000000 call $ufo_scale_per_visit i64.mul local.set $vx
		call $ufo_radius local.set $radius
		local.get $x local.get $y local.get $vx i64.const 0 local.get $radius
		call $foreign_spawn_safe i32.eqz
		(if (then
			i32.const 16560 i32.const 16560 i32.load i32.const 1 i32.sub i32.store
			i32.const 16512 call $foreign_spawn_retry_ticks i32.store
			return))
		i32.const 16032 i32.const 1 i32.store
		i32.const 16036 local.get $direction i32.store
		i32.const 16040 local.get $x i64.store
		i32.const 16048 local.get $y i64.store
		i32.const 16056 local.get $vx i64.store
		i32.const 16064 local.get $radius i64.store
		i32.const 16072 call $ufo_fire_interval_ticks i32.store
		i32.const 8 f32.const 0.8 f32.const 1 i32.const 0 call $audio drop)

	;; Advances the finite horizontal traversal and begins a fresh independent
	;; schedule only after the saucer has cleared the opposite edge.
	(func $update_ufo_schedule
		i32.const 16032 i32.load
		(if
			(then
				i32.const 16040 i32.const 16040 i64.load i32.const 16056 i64.load call $per_tick i64.add i64.store
				i32.const 16072 i32.load i32.const 0 i32.gt_s
				(if (then i32.const 16072 i32.const 16072 i32.load i32.const 1 i32.sub i32.store))
				i32.const 16072 i32.load i32.eqz (if (then call $fire_ufo))
				i32.const 16040 i64.load i64.const -40000000 i64.lt_s
				i32.const 16040 i64.load global.get $state_width_address i64.load i64.const 40000000 i64.add i64.gt_s i32.or
				(if (then
					i32.const 16032 i32.const 0 i32.store
					i32.const 16512 call $random_spawn_ticks i32.store)))
			(else
				i32.const 16512 i32.load i32.const 0 i32.gt_s
				(if (then i32.const 16512 i32.const 16512 i32.load i32.const 1 i32.sub i32.store))
				i32.const 16512 i32.load i32.eqz (if (then call $spawn_ufo)))))

	;; Gives player projectiles first claim on the saucer and awards exactly one
	;; bounty by deactivating both participants before ending the scan.
	(func $check_player_bullet_ufo_collision
		(local $index i32) (local $bullet i32)
		i32.const 16032 i32.load
		(if (then
			(block $done (loop $again
				local.get $index global.get $player_bullet_capacity i32.ge_u br_if $done
				local.get $index call $bullet_address local.set $bullet
				local.get $bullet i32.load
				(if (then
					local.get $bullet i32.const 8 i32.add i64.load
					local.get $bullet i32.const 16 i32.add i64.load
					i32.const 16040 i64.load i32.const 16048 i64.load
					i32.const 16064 i64.load i64.const 5000000 i64.add call $distance_lt
					(if (then
						local.get $bullet i32.const 0 i32.store
						i32.const 1 call $destroy_ufo
						br $done))))
				local.get $index i32.const 1 i32.add local.set $index br $again)))))

	;; Makes indiscriminate player fire costly: packages claim a colliding bullet
	;; before other special targets and retire through the shared failure path.
	(func $check_player_bullet_package_collision
		(local $index i32) (local $bullet i32)
		i32.const 16464 i32.load
		(if (then
			(block $done (loop $again
				local.get $index global.get $player_bullet_capacity i32.ge_u br_if $done
				local.get $index call $bullet_address local.set $bullet
				local.get $bullet i32.load
				(if (then
					local.get $bullet i32.const 8 i32.add i64.load
					local.get $bullet i32.const 16 i32.add i64.load
					i32.const 16472 i64.load i32.const 16480 i64.load
					i32.const 16504 i64.load i64.const 4000000 i64.add call $distance_lt
					(if (then
						local.get $bullet i32.const 0 i32.store
						call $lose_package
						br $done))))
				local.get $index i32.const 1 i32.add local.set $index br $again)))))

	;; Gives an ordinary player projectile score attribution for the reactor
	;; blast while consuming the bullet before later special-target scans.
	(func $check_player_bullet_satellite_collision
		(local $index i32) (local $bullet i32)
		global.get $state_satellite_active_address i32.load
		(if (then
			(block $done (loop $again
				local.get $index global.get $player_bullet_capacity i32.ge_u br_if $done
				local.get $index call $bullet_address local.set $bullet
				local.get $bullet i32.load
				(if (then
					local.get $bullet i32.const 8 i32.add i64.load
					local.get $bullet i32.const 16 i32.add i64.load
					global.get $state_satellite_x_address i64.load
					global.get $state_satellite_y_address i64.load
					global.get $state_satellite_radius_address i64.load i64.const 5000000 i64.add
					call $distance_lt
					(if (then
						local.get $bullet i32.const 0 i32.store
						i32.const 1 call $destroy_satellite
						br $done))))
				local.get $index i32.const 1 i32.add local.set $index br $again)))))

	;; Resolves hostile shots against shielding asteroids, then packages, then the
	;; ship; no impact awards score and each bullet has exactly one victim.
	(func $check_enemy_bullet_collisions
		(local $bullet_index i32) (local $asteroid_index i32)
		(local $bullet i32) (local $asteroid i32)
		(block $bullets_done (loop $next_bullet
			local.get $bullet_index i32.const 8 i32.ge_u br_if $bullets_done
			local.get $bullet_index call $enemy_bullet_address local.set $bullet
			local.get $bullet i32.load
			(if (then
				i32.const 0 local.set $asteroid_index
				(block $asteroids_done (loop $next_asteroid
					local.get $asteroid_index i32.const 32 i32.ge_u br_if $asteroids_done
					local.get $asteroid_index call $asteroid_address local.set $asteroid
					local.get $asteroid i32.load
					(if (then
						local.get $bullet i32.const 8 i32.add i64.load
						local.get $bullet i32.const 16 i32.add i64.load
						local.get $asteroid i32.const 16 i32.add i64.load
						local.get $asteroid i32.const 24 i32.add i64.load
						local.get $asteroid i32.const 48 i32.add i64.load i64.const 4000000 i64.add
						call $distance_lt
						(if (then
							local.get $bullet i32.const 0 i32.store
							local.get $asteroid i64.const 0 i64.const 0 i32.const 0 call $hit_asteroid
							br $asteroids_done))))
					local.get $asteroid_index i32.const 1 i32.add local.set $asteroid_index br $next_asteroid))
				local.get $bullet i32.load i32.const 16464 i32.load i32.and
				(if (then
					local.get $bullet i32.const 8 i32.add i64.load
					local.get $bullet i32.const 16 i32.add i64.load
					i32.const 16472 i64.load i32.const 16480 i64.load
					i32.const 16504 i64.load i64.const 4000000 i64.add call $distance_lt
					(if (then
						local.get $bullet i32.const 0 i32.store
						call $lose_package))))
				local.get $bullet i32.load
				global.get $state_lifecycle_address i32.load i32.eqz i32.and
				global.get $state_invulnerability_address i32.load i32.const 0 i32.le_s i32.and
				(if (then
					local.get $bullet i32.const 8 i32.add i64.load
					local.get $bullet i32.const 16 i32.add i64.load
					global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load i64.const 14000000 call $distance_lt
					(if (then
						local.get $bullet i32.const 0 i32.store
						call $begin_ship_destruction))))))
			local.get $bullet_index i32.const 1 i32.add local.set $bullet_index br $next_bullet)))

	;; Treats the saucer as a physical actor: asteroid contact destroys both, and
	;; vulnerable player contact destroys both without granting a bounty.
	(func $check_ufo_collisions
		(local $index i32) (local $asteroid i32)
		i32.const 16032 i32.load
		(if (then
			(block $asteroids_done (loop $asteroids
				local.get $index i32.const 32 i32.ge_u br_if $asteroids_done
				local.get $index call $asteroid_address local.set $asteroid
				local.get $asteroid i32.load
				(if (then
					i32.const 16040 i64.load i32.const 16048 i64.load
					local.get $asteroid i32.const 16 i32.add i64.load
					local.get $asteroid i32.const 24 i32.add i64.load
					i32.const 16064 i64.load local.get $asteroid i32.const 48 i32.add i64.load i64.add
					call $distance_lt
					(if (then
						;; The blast snapshots the impact rock before splitting it; a
						;; separate pre-hit would expose newborn children to the blast.
						i32.const 0 call $destroy_ufo
						br $asteroids_done))))
				local.get $index i32.const 1 i32.add local.set $index br $asteroids))
			i32.const 16032 i32.load
			global.get $state_lifecycle_address i32.load i32.eqz i32.and
			global.get $state_invulnerability_address i32.load i32.const 0 i32.le_s i32.and
			(if (then
				i32.const 16040 i64.load i32.const 16048 i64.load
				global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load
				i32.const 16064 i64.load i64.const 10000000 i64.add call $distance_lt
				(if (then
						i32.const 0 call $destroy_ufo)))))))

	;; Asteroid contact ruptures the reactor without player attribution. The
	;; blast itself snapshots the impact rock, so no separate pre-hit is needed.
	(func $check_satellite_collisions
		(local $index i32) (local $asteroid i32)
		global.get $state_satellite_active_address i32.load
		(if (then
			(block $done (loop $asteroids
				local.get $index i32.const 32 i32.ge_u br_if $done
				local.get $index call $asteroid_address local.set $asteroid
				local.get $asteroid i32.load
				(if (then
					global.get $state_satellite_x_address i64.load
					global.get $state_satellite_y_address i64.load
					local.get $asteroid i32.const 16 i32.add i64.load
					local.get $asteroid i32.const 24 i32.add i64.load
					global.get $state_satellite_radius_address i64.load
					local.get $asteroid i32.const 48 i32.add i64.load i64.add
					call $distance_lt
					(if (then i32.const 0 call $destroy_satellite br $done))))
				local.get $index i32.const 1 i32.add local.set $index br $asteroids)))))

	(func $respawn_radius (result i64)
		global.get $state_lifecycle_ticks_address i32.load i32.const 300 call $ticks_from_sixty i32.ge_s
		(if (result i64) (then i64.const 48000000) (else i64.const 96000000)))

	(func $respawn_safe (result i32)
		(local $index i32) (local $address i32)
		(block $done (loop $again
			local.get $index i32.const 32 i32.ge_u
			(if (then i32.const 1 return))
			local.get $index call $asteroid_address local.set $address
			local.get $address i32.load
			(if (then
				global.get $state_width_address i64.load i64.const 2 i64.div_s global.get $state_height_address i64.load i64.const 2 i64.div_s
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
				global.get $state_width_address i64.load i64.const 2 i64.div_s global.get $state_height_address i64.load i64.const 2 i64.div_s
				local.get $address i32.const 16 i32.add i64.load local.get $address i32.const 24 i32.add i64.load
				i64.const 80000000 local.get $address i32.const 48 i32.add i64.load i64.add call $distance_lt
				(if (then local.get $address i64.const 0 i64.const 0 i32.const 0 call $hit_asteroid i32.const 1 local.set $hit))))
			local.get $index i32.const 1 i32.add local.set $index br $again))
		local.get $hit (if (then i32.const 2 f32.const 1 f32.const 1 i32.const 0 call $audio drop)))

	;; Clears the boot gate and spawns the ship under the ordinary invulnerability
	;; window. Spawning immediately rather than waiting for $respawn_safe is
	;; deliberate: the player chose this moment, and the invulnerability window is
	;; already the fairness mechanism for arriving next to a rock.
	(func $dismiss_gate
		;; A game-over Start begins from the canonical reset path. Reset re-arms
		;; the boot gate, which this same dismissal then clears before spawning.
		global.get $state_lifecycle_address i32.load i32.const 3 i32.eq
		(if (then
			global.get $state_seed_address i32.load
			global.get $state_width_address i64.load
			global.get $state_height_address i64.load
			call $reset))
		global.get $flag_gate call $clear_flag
		global.get $flag_resume_gate call $clear_flag
		global.get $state_lifecycle_address i32.load i32.const 2 i32.eq
		(if (then
			global.get $state_lifecycle_address i32.const 0 i32.store
			global.get $state_lifecycle_ticks_address i32.const 0 i32.store
			global.get $state_invulnerability_address i32.const 120 call $ticks_from_sixty i32.store)))

	(func $advance_lifecycle
		global.get $state_lifecycle_address i32.load i32.const 1 i32.eq
		(if (then
			global.get $state_lifecycle_ticks_address global.get $state_lifecycle_ticks_address i32.load i32.const 1 i32.sub i32.store
			global.get $state_lifecycle_ticks_address i32.load i32.const 0 i32.le_s
			(if (then
				global.get $state_lives_address i32.load i32.const 0 i32.le_s
				(if (then
					global.get $state_lifecycle_address i32.const 3 i32.store
					global.get $flag_gate call $set_flag)
					(else
						global.get $state_lifecycle_address i32.const 2 i32.store global.get $state_lifecycle_ticks_address i32.const 0 i32.store
						global.get $state_ship_x_address global.get $state_width_address i64.load i64.const 2 i64.div_s i64.store
						global.get $state_ship_y_address global.get $state_height_address i64.load i64.const 2 i64.div_s i64.store
						global.get $state_ship_vx_address i64.const 0 i64.store global.get $state_ship_vy_address i64.const 0 i64.store
						call $respawn_safe
						(if (then
							global.get $state_lifecycle_address i32.const 0 i32.store
							global.get $state_invulnerability_address i32.const 120 call $ticks_from_sixty i32.store
							global.get $state_banner_ticks_address i32.const 240 call $ticks_from_sixty i32.store
							global.get $flag_blossom_available call $set_flag))))))))
		global.get $state_lifecycle_address i32.load i32.const 2 i32.eq
		call $load_flags global.get $flag_gate i32.and i32.eqz i32.and
		(if (then
			global.get $state_lifecycle_ticks_address global.get $state_lifecycle_ticks_address i32.load i32.const 1 i32.add i32.store
			global.get $state_lifecycle_ticks_address i32.load i32.const 600 call $ticks_from_sixty i32.eq (if (then call $clear_respawn_zone))
			call $respawn_safe
			(if (then
				global.get $state_lifecycle_address i32.const 0 i32.store global.get $state_lifecycle_ticks_address i32.const 0 i32.store
				global.get $state_invulnerability_address i32.const 120 call $ticks_from_sixty i32.store global.get $state_banner_ticks_address i32.const 240 call $ticks_from_sixty i32.store
				global.get $flag_blossom_available call $set_flag))))
	)

	(func $step
		(local $collision i32)
		global.get $state_tick_address global.get $state_tick_address i32.load i32.const 1 i32.add i32.store
		call $load_flags global.get $flag_suspends_simulation i32.and (if (then return))
		global.get $state_lifecycle_address i32.load i32.const 3 i32.eq
		(if (then call $load_flags global.get $flag_fire i32.and (if (then global.get $state_seed_address i32.load global.get $state_width_address i64.load global.get $state_height_address i64.load call $reset)) return))
		call $update_hazardous_blast
		call $update_satellite_quote
		call $update_power_timers
		global.get $state_lifecycle_address i32.load i32.eqz
		(if (then
			call $update_ship call $update_bullets call $update_enemy_bullets call $update_asteroids call $update_particles call $update_debris
			call $check_bullet_collisions
			global.get $state_invulnerability_address i32.load i32.const 0 i32.gt_s
			(if (then global.get $state_invulnerability_address global.get $state_invulnerability_address i32.load i32.const 1 i32.sub i32.store)
				(else call $find_ship_collision local.tee $collision (if (then local.get $collision call $begin_ship_explosion)))))
			(else
				global.get $state_lifecycle_address i32.load i32.const 3 i32.ne
				(if (then
					global.get $state_lifecycle_address i32.load i32.const 2 i32.eq (if (then call $rotate_ship))
					call $update_bullets call $update_enemy_bullets call $update_asteroids call $update_particles call $update_debris
					call $check_bullet_collisions))))
		call $update_ufo_schedule
		call $update_package_schedule
		call $update_satellite_schedule
		call $check_player_bullet_package_collision
		call $check_player_bullet_ufo_collision
		call $check_player_bullet_satellite_collision
		call $retire_inert_bullets
		call $check_enemy_bullet_collisions
		call $check_ufo_collisions
		call $check_satellite_collisions
		call $check_package_collection
		call $advance_lifecycle
		call $asteroid_count i32.eqz global.get $state_lifecycle_address i32.load i32.const 3 i32.ne i32.and
		(if (then global.get $state_level_address global.get $state_level_address i32.load i32.const 1 i32.add i32.store call $spawn_wave))
		global.get $state_banner_ticks_address i32.load i32.const 0 i32.gt_s
		(if (then global.get $state_banner_ticks_address global.get $state_banner_ticks_address i32.load i32.const 1 i32.sub i32.store)))

	(func (export "AE_tick") (param $count i32) (result i32)
		(local $index i32)
		(block $done (loop $again
			local.get $index local.get $count i32.ge_u br_if $done
			call $step local.get $index i32.const 1 i32.add local.set $index br $again))
		i32.const 1 i32.const 0 i32.const 0 call $effect drop i32.const 0)

	(func $translate_entities (param $dx i64) (param $dy i64)
		(local $index i32) (local $address i32)
		;; All records reserve x/y at +8/+16 except asteroids, which use +16/+24.
		(block $bullets_done (loop $bullets
			local.get $index global.get $player_bullet_capacity i32.ge_u br_if $bullets_done
			local.get $index call $bullet_address local.set $address
			local.get $address i32.load (if (then
				local.get $address i32.const 8 i32.add local.get $address i32.const 8 i32.add i64.load local.get $dx i64.add i64.store
				local.get $address i32.const 16 i32.add local.get $address i32.const 16 i32.add i64.load local.get $dy i64.add i64.store))
			local.get $index i32.const 1 i32.add local.set $index br $bullets))
		i32.const 0 local.set $index
		(block $enemy_bullets_done (loop $enemy_bullets
			local.get $index i32.const 8 i32.ge_u br_if $enemy_bullets_done
			local.get $index call $enemy_bullet_address local.set $address
			local.get $address i32.load (if (then
				local.get $address i32.const 8 i32.add local.get $address i32.const 8 i32.add i64.load local.get $dx i64.add i64.store
				local.get $address i32.const 16 i32.add local.get $address i32.const 16 i32.add i64.load local.get $dy i64.add i64.store))
			local.get $index i32.const 1 i32.add local.set $index br $enemy_bullets))
		i32.const 16032 i32.load (if (then
			i32.const 16040 i32.const 16040 i64.load local.get $dx i64.add i64.store
			i32.const 16048 i32.const 16048 i64.load local.get $dy i64.add i64.store))
		i32.const 16464 i32.load (if (then
			i32.const 16472 i32.const 16472 i64.load local.get $dx i64.add i64.store
			i32.const 16480 i32.const 16480 i64.load local.get $dy i64.add i64.store))
		global.get $state_satellite_active_address i32.load (if (then
			global.get $state_satellite_x_address global.get $state_satellite_x_address i64.load local.get $dx i64.add i64.store
			global.get $state_satellite_y_address global.get $state_satellite_y_address i64.load local.get $dy i64.add i64.store))
		i32.const 16524 i32.load i32.const 0 i32.gt_s (if (then
			i32.const 16528 i32.const 16528 i64.load local.get $dx i64.add i64.store
			i32.const 16536 i32.const 16536 i64.load local.get $dy i64.add i64.store
			i32.const 16544 i32.const 16544 i64.load local.get $dx i64.add i64.store
			i32.const 16552 i32.const 16552 i64.load local.get $dy i64.add i64.store))
		;; The hazardous blast persists for presentation after its collision
		;; snapshot, so its center must follow the same resize translation.
		i32.const 26624 i32.load (if (then
			i32.const 26632 i32.const 26632 i64.load local.get $dx i64.add i64.store
			i32.const 26640 i32.const 26640 i64.load local.get $dy i64.add i64.store))
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
		local.get $new_width i64.const 2 i64.div_s global.get $state_width_address i64.load i64.const 2 i64.div_s i64.sub local.set $dx
		local.get $new_height i64.const 2 i64.div_s global.get $state_height_address i64.load i64.const 2 i64.div_s i64.sub local.set $dy
		global.get $state_ship_x_address global.get $state_ship_x_address i64.load local.get $dx i64.add i64.store
		global.get $state_ship_y_address global.get $state_ship_y_address i64.load local.get $dy i64.add i64.store
		local.get $dx local.get $dy call $translate_entities
		global.get $state_width_address local.get $new_width i64.store global.get $state_height_address local.get $new_height i64.store
		call $regenerate_stars)

	(func (export "AE_event") (param $kind i32) (param $code i32)
		(param $a f32) (param $b f32) (result i32)
		;; A gate consumes gameplay input. Any key-down dismisses it. The native
		;; control delivers its own semantic action; raw canvas pointer events never
		;; duplicate the host's click classifier or pressed-state mechanics.
		call $load_flags global.get $flag_gate i32.and
		(if (then
			local.get $kind i32.const 1 i32.eq
			(if (then call $dismiss_gate i32.const 0 return))
			local.get $kind i32.const 7 i32.eq
			(if (then
				local.get $code i32.const 8 i32.eq
				call $load_flags global.get $flag_resume_gate i32.and i32.eqz i32.and
				(if (then call $dismiss_gate i32.const 0 return))
				local.get $code i32.const 9 i32.eq
				call $load_flags global.get $flag_resume_gate i32.and i32.eqz i32.eqz i32.and
				(if (then call $dismiss_gate i32.const 0 return))))
			local.get $kind i32.const 2 i32.eq
			local.get $kind i32.const 3 i32.eq i32.or
			local.get $kind i32.const 4 i32.eq i32.or
			local.get $kind i32.const 5 i32.eq i32.or
			local.get $kind i32.const 10 i32.eq i32.or
			local.get $kind i32.const 11 i32.eq i32.or
			local.get $kind i32.const 12 i32.eq i32.or
			local.get $kind i32.const 13 i32.eq i32.or
			local.get $kind i32.const 14 i32.eq i32.or
			local.get $kind i32.const 16 i32.eq i32.or
			(if (then i32.const 0 return))))
		local.get $kind local.get $code call $event_allowed_while_paused i32.eqz
		(if (then i32.const 0 return))
		local.get $kind i32.const 1 i32.eq
			(if (then
				local.get $code i32.const 11 i32.eq (if (then i32.const 5 local.set $code))
				local.get $code i32.const 12 i32.eq (if (then i32.const 10 local.set $code))
				local.get $code i32.const 1 i32.eq
				(if (then i32.const 16584 i32.const 0 i32.store
					global.get $rotate_source_arrow call $hold_rotate_left_source))
				local.get $code i32.const 14 i32.eq
				(if (then i32.const 16584 i32.const 0 i32.store
					global.get $rotate_source_letter call $hold_rotate_left_source))
				local.get $code i32.const 2 i32.eq
				(if (then i32.const 16584 i32.const 0 i32.store
					global.get $rotate_source_arrow call $hold_rotate_right_source))
				local.get $code i32.const 15 i32.eq
				(if (then i32.const 16584 i32.const 0 i32.store
					global.get $rotate_source_letter call $hold_rotate_right_source))
				local.get $code i32.const 3 i32.eq
				(if (then global.get $thrust_source_up call $hold_thrust_source))
				local.get $code i32.const 13 i32.eq
				(if (then global.get $thrust_source_letter call $hold_thrust_source))
				local.get $code i32.const 4 i32.eq
				(if (then global.get $fire_source_key call $hold_fire_source))
			local.get $code i32.const 5 i32.eq (if (then call $toggle_pause))
			local.get $code i32.const 6 i32.eq (if (then global.get $state_seed_address i32.load global.get $state_width_address i64.load global.get $state_height_address i64.load call $reset))
			local.get $code i32.const 7 i32.eq (if (then global.get $flag_auto_fire call $toggle_flag))
			local.get $code i32.const 8 i32.eq (if (then global.get $flag_kid_mode call $toggle_flag))
			local.get $code i32.const 9 i32.eq (if (then call $activate_death_blossom))
			local.get $code i32.const 10 i32.eq (if (then global.get $flag_help_visible call $toggle_flag))))
		local.get $kind i32.const 2 i32.eq
		(if (then
				local.get $code i32.const 11 i32.eq (if (then i32.const 5 local.set $code))
				local.get $code i32.const 12 i32.eq (if (then i32.const 10 local.set $code))
				local.get $code i32.const 1 i32.eq
				(if (then global.get $rotate_source_arrow call $release_rotate_left_source))
				local.get $code i32.const 14 i32.eq
				(if (then global.get $rotate_source_letter call $release_rotate_left_source))
				local.get $code i32.const 2 i32.eq
				(if (then global.get $rotate_source_arrow call $release_rotate_right_source))
				local.get $code i32.const 15 i32.eq
				(if (then global.get $rotate_source_letter call $release_rotate_right_source))
				local.get $code i32.const 3 i32.eq
				(if (then global.get $thrust_source_up call $release_thrust_source))
				local.get $code i32.const 13 i32.eq
				(if (then global.get $thrust_source_letter call $release_thrust_source))
				local.get $code i32.const 4 i32.eq
				(if (then global.get $fire_source_key call $release_fire_source))))
		;; Pointer motion owns heading until a keyboard turn key is pressed.
		local.get $kind i32.const 3 i32.eq
		(if (then
			i32.const 16568 local.get $a call $from_host i64.store
			i32.const 16576 local.get $b call $from_host i64.store
			i32.const 16584 i32.const 1 i32.store))
		;; Stable pointer down/up edges make button 1 held fire and button 2 held
		;; thrust; both refresh aim even without prior pointer motion.
		local.get $kind i32.const 4 i32.eq
		(if (then
			i32.const 16568 local.get $a call $from_host i64.store
				i32.const 16576 local.get $b call $from_host i64.store
				i32.const 16584 i32.const 1 i32.store
				local.get $code i32.const 1 i32.eq
				(if (then global.get $fire_source_pointer call $hold_fire_source))
				local.get $code i32.const 2 i32.eq
				(if (then global.get $thrust_source_pointer call $hold_thrust_source))))
		local.get $kind i32.const 5 i32.eq
		(if (then
			i32.const 16568 local.get $a call $from_host i64.store
				i32.const 16576 local.get $b call $from_host i64.store
				i32.const 16584 i32.const 1 i32.store
				local.get $code i32.const 1 i32.eq
				(if (then global.get $fire_source_pointer call $release_fire_source))
				local.get $code i32.const 2 i32.eq
				(if (then global.get $thrust_source_pointer call $release_thrust_source))))
		;; Aedicule omits zero-delta scroll phases, so every delivered scroll event
		;; represents an intentional wheel gesture without inspecting f32 payloads.
		local.get $kind i32.const 10 i32.eq
		(if (then call $activate_death_blossom))
		;; Registered shake kind 1 spends the same once-per-life charge; routing
		;; through the shared eligibility function keeps modal and lifecycle rules
		;; identical across keyboard, wheel, and sensor inputs.
		local.get $kind i32.const 16 i32.eq local.get $code i32.const 1 i32.eq i32.and
		(if (then call $activate_death_blossom))
		local.get $kind i32.const 11 i32.eq
		(if (then
			local.get $code local.get $a call $from_host local.get $b call $from_host
			call $handle_touch_start))
		local.get $kind i32.const 12 i32.eq
		(if (then local.get $code local.get $b call $from_host call $handle_touch_move))
		local.get $kind i32.const 13 i32.eq
		local.get $kind i32.const 14 i32.eq i32.or
		(if (then local.get $code call $handle_touch_terminal))
		local.get $kind i32.const 6 i32.eq
		(if (then
			local.get $code i32.const 1 i32.and global.set $coarse_pointer_mode
			local.get $a call $from_host local.get $b call $from_host call $resize))
		local.get $kind i32.const 7 i32.eq
		(if (then
			local.get $code i32.const 1 i32.eq (if (then global.get $state_seed_address i32.load global.get $state_width_address i64.load global.get $state_height_address i64.load call $reset))
			local.get $code i32.const 7 i32.eq (if (then global.get $flag_help_visible call $toggle_flag))
			local.get $code i32.const 6 i32.eq (if (then i32.const 2 i32.const 0 i32.const 0 call $effect drop))))
		local.get $kind i32.const 8 i32.eq local.get $code i32.eqz i32.and
		;; Losing focus can strand held edges. Arm Resume only when another gate is
		;; not already explaining the modal state, and preserve player modes,
		;; overlays, pause, and the once-per-life weapon charge beneath it.
		(if (then
			call $clear_held_controls
			call $load_flags global.get $flag_gate i32.and i32.eqz
			(if (then
				global.get $flag_gate call $set_flag
				global.get $flag_resume_gate call $set_flag))))
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
			global.get $state_tick_address i32.load i32.const 7 i32.and i64.extend_i32_u i64.const 2000000 i64.mul i64.const 12000000 i64.add
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

	;; Reproduces the approved compact Voyager silhouette from local-space vector
	;; primitives. Both bent science-boom rails remain one continuous path, while
	;; the fixed-clock glow pulse and orientation enter only through snapshot state.
	(func $draw_satellite
		(local $phase i32) (local $half i32) (local $duration i32)
		(local $glow_extra i64) (local $glow_radius i64)
		global.get $state_satellite_active_address i32.load i32.eqz (if (then return))
		i32.const 60 call $ticks_from_sixty local.set $half
		i32.const 120 call $ticks_from_sixty local.set $duration
		global.get $state_satellite_pulse_ticks_address i32.load local.get $duration i32.rem_u local.set $phase
		local.get $phase local.get $half i32.le_u
		(if (then
			local.get $phase i64.extend_i32_u i64.const 4000000 i64.mul
			local.get $half i64.extend_i32_u i64.div_u local.set $glow_extra)
		(else
			local.get $duration local.get $phase i32.sub i64.extend_i32_u i64.const 4000000 i64.mul
			local.get $half i64.extend_i32_u i64.div_u local.set $glow_extra))
		i64.const 52000000 local.get $glow_extra i64.add local.set $glow_radius
		global.get $state_satellite_dx_address i64.load call $to_host
		global.get $state_satellite_dy_address i64.load call $to_host
		i64.const 0 global.get $state_satellite_dy_address i64.load i64.sub call $to_host
		global.get $state_satellite_dx_address i64.load call $to_host
		global.get $state_satellite_x_address i64.load call $to_host
		global.get $state_satellite_y_address i64.load call $to_host call $transform_push drop
		i32.const 932 f32.const 0 f32.const 0 local.get $glow_radius call $to_host
		f32.const 0 i32.const 0x4ddff21c i32.const 1 call $circle drop
		i32.const 933 f32.const 0 f32.const 0 local.get $glow_radius i64.const 5000000 i64.sub call $to_host
		f32.const 1 i32.const 0x4ddff244 i32.const 0 call $circle drop

		;; Dish bowl plus broken upper rim.
		i32.const 934 call $path_begin drop
		f32.const -26 f32.const -8 call $path_move drop
		f32.const -23 f32.const -4 call $path_line drop
		f32.const -18 f32.const 0 call $path_line drop
		f32.const -10 f32.const 3 call $path_line drop
		f32.const 0 f32.const 4 call $path_line drop
		f32.const 10 f32.const 3 call $path_line drop
		f32.const 18 f32.const 0 call $path_line drop
		f32.const 23 f32.const -4 call $path_line drop
		f32.const 26 f32.const -8 call $path_line drop
		call $path_close drop f32.const 2 i32.const 0x102133ff i32.const 0x9bb8d1ff i32.const 0 call $path_end drop
		i32.const 935 call $path_begin drop
		f32.const -26 f32.const -8 call $path_move drop
		f32.const -23 f32.const -12 call $path_line drop
		f32.const -18 f32.const -16 call $path_line drop
		f32.const -10 f32.const -19 call $path_line drop
		f32.const 0 f32.const -20 call $path_line drop
		f32.const 10 f32.const -19 call $path_line drop
		f32.const 18 f32.const -16 call $path_line drop
		f32.const 2 i32.const 0 i32.const 0xb7cfdfff i32.const 0 call $path_end drop
		i32.const 936 call $path_begin drop
		f32.const 23 f32.const -12 call $path_move drop
		f32.const 26 f32.const -8 call $path_line drop
		f32.const 2 i32.const 0 i32.const 0xb7cfdfff i32.const 0 call $path_end drop

		;; Equipment bus, continuous bent science boom, and RTG boom.
		i32.const 937 call $path_begin drop
		f32.const -15 f32.const 2 call $path_move drop
		f32.const -6 f32.const -5 call $path_line drop
		f32.const 6 f32.const -5 call $path_line drop
		f32.const 16 f32.const 2 call $path_line drop
		f32.const 17 f32.const 12 call $path_line drop
		f32.const 9 f32.const 20 call $path_line drop
		f32.const -3 f32.const 23 call $path_line drop
		f32.const -13 f32.const 18 call $path_line drop
		f32.const -18 f32.const 9 call $path_line drop
		call $path_close drop f32.const 2 i32.const 0x152436ff i32.const 0x9bb8d1ff i32.const 0 call $path_end drop
		i32.const 938 call $path_begin drop
		f32.const 14 f32.const 9 call $path_move drop
		f32.const 43 f32.const -7 call $path_line drop
		f32.const 49 f32.const -6 call $path_line drop
		f32.const 54 f32.const -10 call $path_line drop
		f32.const 72 f32.const -20 call $path_line drop
		f32.const 2 i32.const 0 i32.const 0x7893acff i32.const 0 call $path_end drop
		i32.const 939 call $path_begin drop
		f32.const 16 f32.const 12 call $path_move drop
		f32.const 44 f32.const -4 call $path_line drop
		f32.const 50 f32.const -3 call $path_line drop
		f32.const 55 f32.const -8 call $path_line drop
		f32.const 74 f32.const -18 call $path_line drop
		f32.const 1.4 i32.const 0 i32.const 0x53687dff i32.const 0 call $path_end drop
		i32.const 940 call $path_begin drop
		f32.const -12 f32.const 11 call $path_move drop
		f32.const -32 f32.const 18 call $path_line drop
		f32.const -61 f32.const 23 call $path_line drop
		f32.const 2 i32.const 0 i32.const 0x7893acff i32.const 0 call $path_end drop

		;; Three still-warm RTGs remain attached to the historical long boom.
		i32.const 941 call $path_begin drop
		f32.const -29 f32.const 14 call $path_move drop
		f32.const -25 f32.const 20 call $path_line drop
		f32.const -34 f32.const 23 call $path_line drop
		f32.const -38 f32.const 17 call $path_line drop
		call $path_close drop f32.const 1.5 i32.const 0x172536ff i32.const 0x8eb0c9ff i32.const 0 call $path_end drop
		i32.const 942 call $path_begin drop
		f32.const -41 f32.const 17 call $path_move drop
		f32.const -37 f32.const 23 call $path_line drop
		f32.const -46 f32.const 26 call $path_line drop
		f32.const -50 f32.const 20 call $path_line drop
		call $path_close drop f32.const 1.5 i32.const 0x172536ff i32.const 0x8eb0c9ff i32.const 0 call $path_end drop
		i32.const 943 call $path_begin drop
		f32.const -53 f32.const 19 call $path_move drop
		f32.const -49 f32.const 25 call $path_line drop
		f32.const -58 f32.const 27 call $path_line drop
		f32.const -62 f32.const 22 call $path_line drop
		call $path_close drop f32.const 1.5 i32.const 0x172536ff i32.const 0x8eb0c9ff i32.const 0 call $path_end drop
		i32.const 944 f32.const -31 f32.const 19 f32.const 1.7 f32.const 0.5 i32.const 0xff9b3dff i32.const 1 call $circle drop
		i32.const 945 f32.const -43 f32.const 22 f32.const 1.7 f32.const 0.5 i32.const 0xe8792bff i32.const 1 call $circle drop
		i32.const 946 f32.const -55 f32.const 24.5 f32.const 1.7 f32.const 0.5 i32.const 0xbb5724ff i32.const 1 call $circle drop

		;; Gold record, cracked panel, dish feed/supports, end joint, and cable.
		i32.const 947 f32.const 9 f32.const 11 f32.const 4 f32.const 1 i32.const 0xb88731ff i32.const 1 call $circle drop
		i32.const 948 call $path_begin drop
		f32.const 3 f32.const 2 call $path_move drop
		f32.const -1 f32.const 8 call $path_line drop
		f32.const 4 f32.const 12 call $path_line drop
		f32.const 0 f32.const 19 call $path_line drop
		f32.const 1.3 i32.const 0 i32.const 0x53687dff i32.const 0 call $path_end drop
		i32.const 949 f32.const 0 f32.const -18 f32.const 2.3 f32.const 1 i32.const 0x5c7185ff i32.const 0 call $circle drop
		i32.const 950 f32.const -12 f32.const 2 f32.const 0 f32.const -17 f32.const 1 i32.const 0x7893acff call $line drop
		i32.const 951 f32.const 12 f32.const 2 f32.const 0 f32.const -17 f32.const 1 i32.const 0x7893acff call $line drop
		i32.const 952 f32.const 0 f32.const -6 f32.const 0 f32.const -17 f32.const 1.2 i32.const 0x8eb0c9ff call $line drop
		i32.const 953 f32.const 74 f32.const -18 f32.const 2.5 f32.const 1 i32.const 0x8eb0c9ff i32.const 0 call $circle drop
		i32.const 954 call $path_begin drop
		f32.const 21 f32.const -5 call $path_move drop
		f32.const 26 f32.const -1 call $path_line drop
		f32.const 23 f32.const 4 call $path_line drop
		f32.const 1.3 i32.const 0 i32.const 0xff8a3dff i32.const 0 call $path_end drop
		call $transform_pop drop)

	;; Emits a low hull and raised dome so the enemy reads as a classic disc UFO
	;; using only stable vector-command IDs from the generic Aedicule ABI.
	(func $draw_ufo
		(local $x i64) (local $y i64)
		i32.const 16032 i32.load
		(if (then
			i32.const 16040 i64.load local.set $x i32.const 16048 i64.load local.set $y
			i32.const 900 call $path_begin drop
			local.get $x i64.const 24000000 i64.sub call $to_host local.get $y call $to_host call $path_move drop
			local.get $x i64.const 12000000 i64.sub call $to_host local.get $y i64.const 8000000 i64.sub call $to_host call $path_line drop
			local.get $x i64.const 12000000 i64.add call $to_host local.get $y i64.const 8000000 i64.sub call $to_host call $path_line drop
			local.get $x i64.const 24000000 i64.add call $to_host local.get $y call $to_host call $path_line drop
			local.get $x i64.const 12000000 i64.add call $to_host local.get $y i64.const 8000000 i64.add call $to_host call $path_line drop
			local.get $x i64.const 12000000 i64.sub call $to_host local.get $y i64.const 8000000 i64.add call $to_host call $path_line drop
			call $path_close drop f32.const 2 i32.const 0x182033ff i32.const 0xffcf5cff i32.const 0 call $path_end drop
			i32.const 901 local.get $x call $to_host local.get $y i64.const 8000000 i64.sub call $to_host
			f32.const 10 f32.const 2 i32.const 0x5ee7ffff i32.const 0 call $circle drop
			i32.const 902 local.get $x i64.const 24000000 i64.sub call $to_host local.get $y call $to_host
			local.get $x i64.const 24000000 i64.add call $to_host local.get $y call $to_host
			f32.const 1 i32.const 0xffffffff call $line drop)))

	;; Draws a locale-neutral wrapped parcel glyph with ribbon crossbars and two
	;; closed bow loops; vector-only decoration avoids untranslated status text.
	(func $draw_package
		(local $x i64) (local $y i64)
		i32.const 16464 i32.load
		(if (then
			i32.const 16472 i64.load local.set $x i32.const 16480 i64.load local.set $y
			i32.const 910 call $path_begin drop
			local.get $x i64.const 12000000 i64.sub call $to_host local.get $y i64.const 10000000 i64.sub call $to_host call $path_move drop
			local.get $x i64.const 12000000 i64.add call $to_host local.get $y i64.const 10000000 i64.sub call $to_host call $path_line drop
			local.get $x i64.const 12000000 i64.add call $to_host local.get $y i64.const 10000000 i64.add call $to_host call $path_line drop
			local.get $x i64.const 12000000 i64.sub call $to_host local.get $y i64.const 10000000 i64.add call $to_host call $path_line drop
			call $path_close drop f32.const 2 i32.const 0x182033ff i32.const 0xffcf5cff i32.const 0 call $path_end drop
			i32.const 911 local.get $x call $to_host local.get $y i64.const 10000000 i64.sub call $to_host
			local.get $x call $to_host local.get $y i64.const 10000000 i64.add call $to_host
			f32.const 2 i32.const 0x5ee7ffff call $line drop
			i32.const 912 local.get $x i64.const 12000000 i64.sub call $to_host local.get $y call $to_host
			local.get $x i64.const 12000000 i64.add call $to_host local.get $y call $to_host
			f32.const 2 i32.const 0x5ee7ffff call $line drop
			;; Mirrored closed loops stay attached to the knot and parcel lid, so the
			;; decoration reads as a tied bow rather than unrelated particles.
			i32.const 913 call $path_begin drop
			local.get $x i64.const 2000000 i64.sub call $to_host local.get $y i64.const 12000000 i64.sub call $to_host call $path_move drop
			local.get $x i64.const 10000000 i64.sub call $to_host local.get $y i64.const 19000000 i64.sub call $to_host call $path_line drop
			local.get $x i64.const 9000000 i64.sub call $to_host local.get $y i64.const 10000000 i64.sub call $to_host call $path_line drop
			local.get $x i64.const 2000000 i64.sub call $to_host local.get $y i64.const 12000000 i64.sub call $to_host call $path_line drop
			call $path_close drop f32.const 1.5 i32.const 0xffcf5cff i32.const 0x5ee7ffff i32.const 0 call $path_end drop
			i32.const 914 call $path_begin drop
			local.get $x i64.const 2000000 i64.add call $to_host local.get $y i64.const 12000000 i64.sub call $to_host call $path_move drop
			local.get $x i64.const 10000000 i64.add call $to_host local.get $y i64.const 19000000 i64.sub call $to_host call $path_line drop
			local.get $x i64.const 9000000 i64.add call $to_host local.get $y i64.const 10000000 i64.sub call $to_host call $path_line drop
			local.get $x i64.const 2000000 i64.add call $to_host local.get $y i64.const 12000000 i64.sub call $to_host call $path_line drop
			call $path_close drop f32.const 1.5 i32.const 0xffcf5cff i32.const 0x5ee7ffff i32.const 0 call $path_end drop
			i32.const 915 local.get $x call $to_host local.get $y i64.const 12000000 i64.sub call $to_host
			f32.const 3 f32.const 1 i32.const 0xffcf5cff i32.const 0x5ee7ffff call $circle drop)))

	;; Renders a brief cyan-white afterimage from the exact finite segment used
	;; for collision resolution, making the no-wrap boundary visually explicit.
	(func $draw_laser
		i32.const 16524 i32.load i32.const 0 i32.gt_s
		(if (then
			i32.const 980 i32.const 16528 i64.load call $to_host i32.const 16536 i64.load call $to_host
			i32.const 16544 i64.load call $to_host i32.const 16552 i64.load call $to_host
			f32.const 7 i32.const 0x5ee7ff55 call $line drop
			i32.const 981 i32.const 16528 i64.load call $to_host i32.const 16536 i64.load call $to_host
			i32.const 16544 i64.load call $to_host i32.const 16552 i64.load call $to_host
			f32.const 2 i32.const 0xd9fbffff call $line drop)))

	;; Keeps the once-per-life Death Blossom charge visible without depending on
	;; an emoji glyph being present in the host's chosen font.
	(func $draw_blossom_available
		(local $x i64)
		call $load_flags global.get $flag_blossom_available i32.and i32.eqz (if (then return))
		global.get $state_width_address i64.load i64.const 2 i64.div_s local.set $x
		i32.const 920 local.get $x call $to_host f32.const 82 f32.const 4 f32.const 0
		i32.const 0xffcf5cff i32.const 1 call $circle drop
		i32.const 921 local.get $x i64.const 14000000 i64.sub call $to_host f32.const 82 local.get $x i64.const 7000000 i64.sub call $to_host f32.const 82 f32.const 2 i32.const 0xffcf5cff call $line drop
		i32.const 922 local.get $x i64.const 7000000 i64.add call $to_host f32.const 82 local.get $x i64.const 14000000 i64.add call $to_host f32.const 82 f32.const 2 i32.const 0xffcf5cff call $line drop
		i32.const 923 local.get $x call $to_host f32.const 68 local.get $x call $to_host f32.const 75 f32.const 2 i32.const 0xffcf5cff call $line drop
		i32.const 924 local.get $x call $to_host f32.const 89 local.get $x call $to_host f32.const 96 f32.const 2 i32.const 0xffcf5cff call $line drop
		i32.const 925 local.get $x i64.const 10000000 i64.sub call $to_host f32.const 72 local.get $x i64.const 5000000 i64.sub call $to_host f32.const 77 f32.const 2 i32.const 0xffcf5cff call $line drop
		i32.const 926 local.get $x i64.const 5000000 i64.add call $to_host f32.const 87 local.get $x i64.const 10000000 i64.add call $to_host f32.const 92 f32.const 2 i32.const 0xffcf5cff call $line drop
		i32.const 927 local.get $x i64.const 10000000 i64.sub call $to_host f32.const 92 local.get $x i64.const 5000000 i64.sub call $to_host f32.const 87 f32.const 2 i32.const 0xffcf5cff call $line drop
		i32.const 928 local.get $x i64.const 5000000 i64.add call $to_host f32.const 77 local.get $x i64.const 10000000 i64.add call $to_host f32.const 72 f32.const 2 i32.const 0xffcf5cff call $line drop)

	;; Writes a ceil-rounded tenths countdown into either approved power-badge
	;; label without mutating the canonical snapshot during rendering.
	(func $write_power_timer_text (param $ptr i32)
		(local $tenths i32)
		i32.const 16520 i32.load i64.extend_i32_u global.get $tick_denominator i64.mul i64.const 10 i64.mul
		global.get $tick_numerator i64.const 1 i64.sub i64.add
		global.get $tick_numerator i64.div_u i32.wrap_i64 local.set $tenths
		local.get $ptr i32.const 7 i32.add local.get $tenths i32.const 100 i32.div_u i32.const 48 i32.add i32.store8
		local.get $ptr i32.const 8 i32.add local.get $tenths i32.const 10 i32.div_u i32.const 10 i32.rem_u i32.const 48 i32.add i32.store8
		local.get $ptr i32.const 10 i32.add local.get $tenths i32.const 10 i32.rem_u i32.const 48 i32.add i32.store8)

	;; Renders the Peter-approved non-color-only active reward badge. A textual
	;; kind/countdown is paired with either a beam or double-chevron icon.
	(func $draw_power_hud
		(local $kind i32) (local $ptr i32) (local $key i32) (local $color i32) (local $fill i32)
		i32.const 16520 i32.load i32.const 0 i32.le_s (if (then return))
		i32.const 16592 i32.load local.set $kind
		local.get $kind i32.eqz
		(if
			(then i32.const 672 local.set $ptr i32.const 60 local.set $key i32.const 0x5ee7ffff local.set $color i32.const 0x071a25ee local.set $fill)
			(else i32.const 688 local.set $ptr i32.const 61 local.set $key i32.const 0xffcf5cff local.set $color i32.const 0x211807ee local.set $fill))
		local.get $ptr call $write_power_timer_text
		i32.const 2019 call $path_begin drop
		f32.const 24 f32.const 72 call $path_move drop
		f32.const 284 f32.const 72 call $path_line drop
		f32.const 284 f32.const 118 call $path_line drop
		f32.const 24 f32.const 118 call $path_line drop
		call $path_close drop f32.const 0 local.get $fill i32.const 0 i32.const 0 call $path_end drop
		i32.const 2020 f32.const 24 f32.const 72 f32.const 284 f32.const 72 f32.const 1 local.get $color call $line drop
		i32.const 2021 f32.const 284 f32.const 72 f32.const 284 f32.const 118 f32.const 1 local.get $color call $line drop
		i32.const 2022 f32.const 284 f32.const 118 f32.const 24 f32.const 118 f32.const 1 local.get $color call $line drop
		i32.const 2023 f32.const 24 f32.const 118 f32.const 24 f32.const 72 f32.const 1 local.get $color call $line drop
		local.get $kind i32.eqz
		(if
			(then
				i32.const 2024 f32.const 44 f32.const 95 f32.const 72 f32.const 95 f32.const 3 local.get $color call $line drop
				i32.const 2025 f32.const 44 f32.const 95 f32.const 4 f32.const 0 i32.const 0xffffffff i32.const 1 call $circle drop)
			(else
				i32.const 2024 f32.const 42 f32.const 84 f32.const 54 f32.const 95 f32.const 3 local.get $color call $line drop
				i32.const 2025 f32.const 54 f32.const 95 f32.const 42 f32.const 106 f32.const 3 local.get $color call $line drop
				i32.const 2026 f32.const 56 f32.const 84 f32.const 68 f32.const 95 f32.const 3 local.get $color call $line drop
				i32.const 2027 f32.const 68 f32.const 95 f32.const 56 f32.const 106 f32.const 3 local.get $color call $line drop))
		local.get $key local.get $ptr i32.const 11 f32.const 88 f32.const 95 f32.const 20 local.get $color i32.const 0 call $text drop)

	;; Maps approved 768px Help geometry into a fixed-point live coordinate,
	;; preserving exact reference spacing while compacting only short windows.
	(func $help_fixed_y (param $reference i64) (result i64)
		(local $span i64)
		global.get $state_height_address i64.load i64.const 78000000 i64.sub local.set $span
		local.get $span i64.const 0 i64.lt_s (if (then i64.const 0 local.set $span))
		local.get $span i64.const 690000000 i64.gt_s
		(if (then i64.const 690000000 local.set $span))
		i64.const 62000000
		local.get $reference i64.const 62 i64.sub local.get $span i64.mul i64.const 690 i64.div_s
		i64.add)

	(func $help_y (param $reference i64) (result f32)
		local.get $reference call $help_fixed_y call $to_host)

	;; Game Over owns the viewport centre, so its Start button moves down by 96
	;; logical pixels. Start-at-boot and Resume use the centre directly.
	(func $gate_button_center_y (result i64)
		global.get $state_lifecycle_address i32.load i32.const 3 i32.eq
		(if (result i64)
			(then global.get $state_height_address i64.load i64.const 2 i64.div_s i64.const 96000000 i64.add)
			(else global.get $state_height_address i64.load i64.const 2 i64.div_s)))

	;; Draws the approved two-column Help panel after the world. Its second column
	;; describes the active touch grammar after coarse-device or raw-contact
	;; evidence; ordinary fine-only desktop keeps the pointer grammar.
	(func $draw_help_overlay
		(local $center i64) (local $keyboard_input i64) (local $keyboard_action i64)
		(local $pointer_input i64) (local $pointer_action i64)
		(local $far_x i64) (local $bottom f32)
		global.get $state_width_address i64.load i64.const 2 i64.div_s local.set $center
		global.get $state_width_address i64.load i64.const 98 i64.mul i64.const 1024 i64.div_u local.set $keyboard_input
		global.get $state_width_address i64.load i64.const 285 i64.mul i64.const 1024 i64.div_u local.set $keyboard_action
		global.get $state_width_address i64.load i64.const 470 i64.mul i64.const 1024 i64.div_u local.set $pointer_input
		global.get $state_width_address i64.load i64.const 610 i64.mul i64.const 1024 i64.div_u local.set $pointer_action
		local.get $pointer_action local.get $pointer_input i64.const 130000000 i64.add i64.lt_s
		(if (then local.get $pointer_input i64.const 130000000 i64.add local.set $pointer_action))
		global.get $state_width_address i64.load i64.const 40000000 i64.sub local.set $far_x
		;; Size the panel from its last content baseline, not the viewport edge:
		;; this removes dead space on tall windows and carries the same 32px
		;; content padding through the compact layout used by short windows.
		i64.const 636 call $help_fixed_y i64.const 32000000 i64.add call $to_host
		local.set $bottom
		i32.const 2009 call $path_begin drop
		f32.const 40 f32.const 62 call $path_move drop
		local.get $far_x call $to_host f32.const 62 call $path_line drop
		local.get $far_x call $to_host local.get $bottom call $path_line drop
		f32.const 40 local.get $bottom call $path_line drop
		call $path_close drop f32.const 0 i32.const 0x081021f5 i32.const 0 i32.const 0 call $path_end drop
		i32.const 2010 f32.const 40 f32.const 62 local.get $far_x call $to_host f32.const 62 f32.const 2 i32.const 0x5ee7ffff call $line drop
		i32.const 2011 local.get $far_x call $to_host f32.const 62 local.get $far_x call $to_host local.get $bottom f32.const 2 i32.const 0x5ee7ffff call $line drop
		i32.const 2012 local.get $far_x call $to_host local.get $bottom f32.const 40 local.get $bottom f32.const 2 i32.const 0x5ee7ffff call $line drop
		i32.const 2013 f32.const 40 local.get $bottom f32.const 40 f32.const 62 f32.const 2 i32.const 0x5ee7ffff call $line drop
		i32.const 2014 f32.const 88 i64.const 150 call $help_y global.get $state_width_address i64.load i64.const 88000000 i64.sub call $to_host i64.const 150 call $help_y f32.const 1 i32.const 0x31536bff call $line drop
		i32.const 40 i32.const 224 i32.const 8 local.get $center call $to_host i64.const 125 call $help_y f32.const 36 i32.const 0x5ee7ffff i32.const 1 call $text drop
		i32.const 54 i32.const 560 i32.const 8 local.get $keyboard_input call $to_host i64.const 190 call $help_y f32.const 18 i32.const 0xffcf5cff i32.const 0 call $text drop
		call $touch_help_mode
		(if
			(then i32.const 55 i32.const 768 i32.const 5 local.get $pointer_input call $to_host i64.const 190 call $help_y f32.const 18 i32.const 0xffcf5cff i32.const 0 call $text drop)
			(else i32.const 55 i32.const 568 i32.const 7 local.get $pointer_input call $to_host i64.const 190 call $help_y f32.const 18 i32.const 0xffcf5cff i32.const 0 call $text drop))
		i32.const 41 i32.const 240 i32.const 14 local.get $keyboard_input call $to_host i64.const 230 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 62 i32.const 255 i32.const 6 local.get $keyboard_action call $to_host i64.const 230 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 42 i32.const 264 i32.const 4 local.get $keyboard_input call $to_host i64.const 266 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 63 i32.const 279 i32.const 6 local.get $keyboard_action call $to_host i64.const 266 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 43 i32.const 288 i32.const 5 local.get $keyboard_input call $to_host i64.const 302 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 64 i32.const 303 i32.const 4 local.get $keyboard_action call $to_host i64.const 302 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 44 i32.const 312 i32.const 1 local.get $keyboard_input call $to_host i64.const 338 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 65 i32.const 327 i32.const 9 local.get $keyboard_action call $to_host i64.const 338 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 45 i32.const 340 i32.const 1 local.get $keyboard_input call $to_host i64.const 374 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 66 i32.const 355 i32.const 8 local.get $keyboard_action call $to_host i64.const 374 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 46 i32.const 368 i32.const 1 local.get $keyboard_input call $to_host i64.const 410 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 67 i32.const 383 i32.const 13 local.get $keyboard_action call $to_host i64.const 410 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 47 i32.const 400 i32.const 7 local.get $keyboard_input call $to_host i64.const 446 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 68 i32.const 415 i32.const 5 local.get $keyboard_action call $to_host i64.const 446 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 48 i32.const 424 i32.const 1 local.get $keyboard_input call $to_host i64.const 482 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 69 i32.const 439 i32.const 7 local.get $keyboard_action call $to_host i64.const 482 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 49 i32.const 448 i32.const 6 local.get $keyboard_input call $to_host i64.const 518 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
		i32.const 70 i32.const 463 i32.const 4 local.get $keyboard_action call $to_host i64.const 518 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		call $touch_help_mode
		(if
			(then
				i32.const 56 i32.const 776 i32.const 15 local.get $pointer_input call $to_host i64.const 230 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
				i32.const 71 i32.const 792 i32.const 4 local.get $pointer_action call $to_host i64.const 230 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
				i32.const 57 i32.const 800 i32.const 11 local.get $pointer_input call $to_host i64.const 266 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
				i32.const 72 i32.const 812 i32.const 6 local.get $pointer_action call $to_host i64.const 266 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
				i32.const 58 i32.const 820 i32.const 11 local.get $pointer_input call $to_host i64.const 302 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
				i32.const 73 i32.const 832 i32.const 6 local.get $pointer_action call $to_host i64.const 302 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
				i32.const 59 i32.const 840 i32.const 10 local.get $pointer_input call $to_host i64.const 338 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
				i32.const 74 i32.const 852 i32.const 14 local.get $pointer_action call $to_host i64.const 338 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop)
			(else
				i32.const 56 i32.const 576 i32.const 4 local.get $pointer_input call $to_host i64.const 230 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
				i32.const 71 i32.const 589 i32.const 3 local.get $pointer_action call $to_host i64.const 230 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
				i32.const 57 i32.const 592 i32.const 9 local.get $pointer_input call $to_host i64.const 266 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
				i32.const 72 i32.const 605 i32.const 4 local.get $pointer_action call $to_host i64.const 266 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
				i32.const 58 i32.const 616 i32.const 10 local.get $pointer_input call $to_host i64.const 302 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
				i32.const 73 i32.const 629 i32.const 6 local.get $pointer_action call $to_host i64.const 302 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop
				i32.const 59 i32.const 640 i32.const 6 local.get $pointer_input call $to_host i64.const 338 call $help_y f32.const 17 i32.const 0xffffffff i32.const 0 call $text drop
				i32.const 74 i32.const 653 i32.const 7 local.get $pointer_action call $to_host i64.const 338 call $help_y f32.const 17 i32.const 0x9bb8d1ff i32.const 0 call $text drop))
		i32.const 2030 local.get $center call $to_host i64.const 585 call $help_y f32.const 6 f32.const 0 i32.const 0xff5cf4ff i32.const 1 call $circle drop
		i32.const 2031 local.get $center i64.const 18000000 i64.sub call $to_host i64.const 585 call $help_y local.get $center i64.const 7000000 i64.sub call $to_host i64.const 585 call $help_y f32.const 2 i32.const 0xff5cf4ff call $line drop
		i32.const 2032 local.get $center i64.const 7000000 i64.add call $to_host i64.const 585 call $help_y local.get $center i64.const 18000000 i64.add call $to_host i64.const 585 call $help_y f32.const 2 i32.const 0xff5cf4ff call $line drop
		i32.const 2033 local.get $center call $to_host i64.const 567 call $help_y local.get $center call $to_host i64.const 578 call $help_y f32.const 2 i32.const 0xff5cf4ff call $line drop
		i32.const 2034 local.get $center call $to_host i64.const 592 call $help_y local.get $center call $to_host i64.const 603 call $help_y f32.const 2 i32.const 0xff5cf4ff call $line drop
		i32.const 2035 local.get $center i64.const 13000000 i64.sub call $to_host i64.const 572 call $help_y local.get $center i64.const 5000000 i64.sub call $to_host i64.const 580 call $help_y f32.const 2 i32.const 0xff5cf4ff call $line drop
		i32.const 2036 local.get $center i64.const 5000000 i64.add call $to_host i64.const 590 call $help_y local.get $center i64.const 13000000 i64.add call $to_host i64.const 598 call $help_y f32.const 2 i32.const 0xff5cf4ff call $line drop
		i32.const 2037 local.get $center i64.const 13000000 i64.sub call $to_host i64.const 598 call $help_y local.get $center i64.const 5000000 i64.sub call $to_host i64.const 590 call $help_y f32.const 2 i32.const 0xff5cf4ff call $line drop
		i32.const 2038 local.get $center i64.const 5000000 i64.add call $to_host i64.const 580 call $help_y local.get $center i64.const 13000000 i64.add call $to_host i64.const 572 call $help_y f32.const 2 i32.const 0xff5cf4ff call $line drop
		i32.const 53 i32.const 512 i32.const 26 local.get $center call $to_host i64.const 636 call $help_y f32.const 17 i32.const 0xffcf5cff i32.const 1 call $text drop)

	;; Converts canonical millionths-of-a-logical-pixel geometry to the retained
	;; UI protocol's signed Q16.16 coordinates without introducing new float math.
	(func $fixed_to_q16 (param $value i64) (result i32)
		local.get $value i64.const 65536 i64.mul global.get $scale i64.div_s i32.wrap_i64)

	(func $gate_mode (result i32)
		call $load_flags global.get $flag_gate i32.and i32.eqz
		(if (then i32.const 0 return))
		call $load_flags global.get $flag_resume_gate i32.and
		(if (result i32) (then i32.const 2) (else i32.const 1)))

	;; Publishes a complete retained document only when gate mode or viewport
	;; geometry changes. Aedicule's real button owns centering, rounded styling,
	;; hover/press feedback, accessibility, click classification, and occlusion.
	(func $publish_gate_ui
		(local $mode i32) (local $status i32) (local $action_id i32)
		(local $width i64) (local $height i64) (local $x i32) (local $y i32)
		call $gate_mode local.set $mode
		global.get $state_width_address i64.load local.set $width
		global.get $state_height_address i64.load local.set $height
		local.get $mode global.get $ui_sent_gate_mode i32.eq
		local.get $width global.get $ui_sent_width i64.eq i32.and
		local.get $height global.get $ui_sent_height i64.eq i32.and
		(if (then return))
		global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
		global.get $ui_revision call $ui_begin local.set $status
		local.get $mode i32.eqz i32.eqz
		(if (then
			local.get $width i64.const 2 i64.div_s i64.const 160000000 i64.sub
			call $fixed_to_q16 local.set $x
			call $gate_button_center_y i64.const 44000000 i64.sub
			call $fixed_to_q16 local.set $y
			i32.const 1 local.get $x local.get $y
			i32.const 20971520 i32.const 5767168 i32.const 0 i32.const 0
			call $control_panel_q16 local.get $status i32.or local.set $status
			i32.const 8 local.set $action_id
			local.get $mode i32.const 2 i32.eq
			(if (then i32.const 9 local.set $action_id))
			i32.const 1 i32.const 1 local.get $action_id local.get $x local.get $y
			i32.const 20971520 i32.const 5767168 i32.const 0
			call $button_place_q16 local.get $status i32.or local.set $status))
		call $ui_end local.get $status i32.or local.set $status
		local.get $status i32.eqz
		(if (then
			local.get $mode global.set $ui_sent_gate_mode
			local.get $width global.set $ui_sent_width
			local.get $height global.set $ui_sent_height)))

	;; Draws the blast as a triangular 1.2-second pulse with alternating hot
	;; orange cores, giving deterministic expansion, contraction, and flicker.
	(func $draw_hazardous_blast
		(local $elapsed i32) (local $half i32) (local $duration i32)
		(local $radius i64) (local $color i32)
		i32.const 26624 i32.load i32.eqz (if (then return))
		i32.const 26648 i32.load local.set $elapsed
		i32.const 36 call $ticks_from_sixty local.set $half
		i32.const 72 call $ticks_from_sixty local.set $duration
		local.get $elapsed local.get $half i32.le_u
		(if (then
			local.get $elapsed i64.extend_i32_u call $hazardous_blast_radius i64.mul
			local.get $half i64.extend_i32_u i64.div_u local.set $radius)
		(else
			local.get $duration local.get $elapsed i32.sub i64.extend_i32_u
			call $hazardous_blast_radius i64.mul
			local.get $half i64.extend_i32_u i64.div_u local.set $radius))
		local.get $radius i64.const 4000000 i64.lt_u
		(if (then i64.const 4000000 local.set $radius))
		local.get $elapsed i32.const 2 i32.div_u i32.const 1 i32.and
		(if (result i32) (then i32.const 0xffcf5cff) (else i32.const 0xff8a2bff))
		local.set $color
		i32.const 930 i32.const 26632 i64.load call $to_host i32.const 26640 i64.load call $to_host
		local.get $radius call $to_host f32.const 0 local.get $color i32.const 1 call $circle drop
		i32.const 931 i32.const 26632 i64.load call $to_host i32.const 26640 i64.load call $to_host
		local.get $radius i64.const 8000000 i64.add call $to_host f32.const 4 i32.const 0xff9b2f99 i32.const 0 call $circle drop)

	(func (export "AE_render") (result i32)
		(local $index i32) (local $address i32) (local $reserve_count i32)
		(local $reserve_icons i32) (local $reserve_digits i32) (local $alpha i32)
		i32.const 26624 i32.load i32.const 26648 i32.load i32.eqz i32.and
		(if
			(then f32.const 0.96862745 f32.const 0.95686275 f32.const 0.92941176 f32.const 1 call $frame_begin drop)
			(else f32.const 0.03137255 f32.const 0.04313725 f32.const 0.07058824 f32.const 1 call $frame_begin drop))
		global.get $state_banner_ticks_address i32.load i32.const 0 i32.gt_s
		(if (then
			call $splash_alpha local.set $alpha
			i32.const 10 i32.const 64 i32.const 12
			global.get $state_width_address i64.load i64.const 2 i64.div_s call $to_host
			global.get $state_height_address i64.load i64.const 333333 call $fixed_mul call $to_host
			f32.const 54 i32.const 0xff5cf400 local.get $alpha i32.or i32.const 1 call $text drop
			i32.const 12 i32.const 64 i32.const 12
			global.get $state_width_address i64.load i64.const 2 i64.div_s call $to_host
			global.get $state_height_address i64.load i64.const 333333 call $fixed_mul call $to_host
			f32.const 52 i32.const 0x5ee7ff00 local.get $alpha i32.or i32.const 1 call $text drop
			i32.const 13 i32.const 64 i32.const 12
			global.get $state_width_address i64.load i64.const 2 i64.div_s call $to_host
			global.get $state_height_address i64.load i64.const 333333 call $fixed_mul call $to_host
			f32.const 50 i32.const 0xffffff00 local.get $alpha i32.or i32.const 1 call $text drop
			i32.const 11 i32.const 80 i32.const 16
			global.get $state_width_address i64.load i64.const 2 i64.div_s i64.const 100000000 i64.sub call $to_host
			global.get $state_height_address i64.load i64.const 333333 call $fixed_mul i64.const 58000000 i64.add call $to_host
			f32.const 18 i32.const 0x58ff7200 local.get $alpha i32.or i32.const 1 call $text drop))
		call $draw_stars
		call $draw_satellite
		call $draw_ufo
		call $draw_package
		call $draw_laser
		i32.const 160 global.get $state_score_address i32.load call $write_six_digits
		i32.const 168 global.get $state_level_address i32.load call $write_two_digits
		call $load_flags global.get $flag_kid_mode i32.and i32.eqz
		(if (then
		i32.const 20 i32.const 128 i32.const 5 f32.const 24 f32.const 24 f32.const 15 i32.const 0x9bb8d1ff i32.const 0 call $text drop
		i32.const 21 i32.const 160 i32.const 6 f32.const 24 f32.const 48 f32.const 22 i32.const 0x58ff72ff i32.const 0 call $text drop
		i32.const 22 i32.const 136 i32.const 5
		global.get $state_width_address i64.load i64.const 2 i64.div_s i64.const 50000000 i64.sub call $to_host f32.const 54 f32.const 15 i32.const 0x9bb8d1ff i32.const 1 call $text drop
		i32.const 23 i32.const 168 i32.const 2
		global.get $state_width_address i64.load i64.const 2 i64.div_s i64.const 30000000 i64.add call $to_host f32.const 54 f32.const 22 i32.const 0xffcf5cff i32.const 1 call $text drop
		i32.const 24 i32.const 144 i32.const 5
		global.get $state_width_address i64.load i64.const 62000000 i64.sub call $to_host f32.const 24 f32.const 15 i32.const 0x9bb8d1ff i32.const 1 call $text drop
		global.get $state_lives_address i32.load i32.const 1 i32.sub local.set $reserve_count
		local.get $reserve_count local.set $reserve_icons
		local.get $reserve_icons i32.const 3 i32.gt_u (if (then i32.const 3 local.set $reserve_icons))
		(block $reserves_done (loop $reserves
			local.get $index local.get $reserve_icons i32.ge_u br_if $reserves_done
			i32.const 50 local.get $index i32.add
			global.get $state_width_address i64.load i64.const 40000000 i64.sub local.get $index i64.extend_i32_u i64.const 28000000 i64.mul i64.sub
			i64.const 48000000 i64.const 1000000 i64.const 0 i64.const 550000 i32.const 0x58ff72ff i32.const 0 call $draw_ship
			local.get $index i32.const 1 i32.add local.set $index br $reserves))
		local.get $reserve_count i32.const 4 i32.ge_u
		(if (then
			i32.const 544 local.get $reserve_count call $write_six_digits
			local.get $reserve_count call $decimal_digits local.set $reserve_digits
			i32.const 25 i32.const 550 local.get $reserve_digits i32.sub local.get $reserve_digits
			global.get $state_width_address i64.load i64.const 124000000 i64.sub call $to_host
			f32.const 54 f32.const 18 i32.const 0xffcf5cff i32.const 1 call $text drop))))
		call $draw_power_hud
		global.get $state_lifecycle_address i32.load local.set $index
		local.get $index i32.eqz
		(if (then
			global.get $state_invulnerability_address i32.load i32.eqz global.get $state_tick_address i32.load i32.const 8 i32.and i32.eqz i32.or
			(if (then i32.const 1 global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load global.get $state_ship_dx_address i64.load global.get $state_ship_dy_address i64.load i64.const 1000000 i32.const -1 call $thrust_is_presented call $draw_ship)))
			(else local.get $index i32.const 2 i32.eq
				(if (then
					call $load_flags global.get $flag_gate i32.and i32.eqz
					(if (then i32.const 1 global.get $state_ship_x_address i64.load global.get $state_ship_y_address i64.load global.get $state_ship_dx_address i64.load global.get $state_ship_dy_address i64.load i64.const 1000000 i32.const 0x777f8c99 i32.const 0 call $draw_ship))))))
		i32.const 0 local.set $index
		(block $bullets_done (loop $bullets
			local.get $index global.get $player_bullet_capacity i32.ge_u br_if $bullets_done
			local.get $index call $bullet_address local.set $address
			local.get $address i32.load (if (then
				local.get $index call $bullet_draw_key local.get $address i32.const 8 i32.add i64.load call $to_host local.get $address i32.const 16 i32.add i64.load call $to_host
				f32.const 2 f32.const 0 i32.const 0x58ff72ff i32.const 1 call $circle drop))
			local.get $index i32.const 1 i32.add local.set $index br $bullets))
		i32.const 0 local.set $index
		(block $enemy_bullets_done (loop $enemy_bullets
			local.get $index i32.const 8 i32.ge_u br_if $enemy_bullets_done
			local.get $index call $enemy_bullet_address local.set $address
			local.get $address i32.load (if (then
				i32.const 960 local.get $index i32.add
				local.get $address i32.const 8 i32.add i64.load call $to_host
				local.get $address i32.const 16 i32.add i64.load call $to_host
				f32.const 3 f32.const 0 i32.const 0xff5c73ff i32.const 1 call $circle drop))
			local.get $index i32.const 1 i32.add local.set $index br $enemy_bullets))
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
		call $draw_hazardous_blast
		global.get $state_lifecycle_address i32.load i32.const 1 i32.eq
		(if (then i32.const 32 i32.const 184 i32.const 14 global.get $state_width_address i64.load i64.const 2 i64.div_s call $to_host global.get $state_height_address i64.load i64.const 2 i64.div_s call $to_host f32.const 26 i32.const 0xff8a2bff i32.const 1 call $text drop))
		call $draw_blossom_available
		call $load_flags global.get $flag_blossom_active i32.and
		(if (then i32.const 36 i32.const 472 i32.const 14 global.get $state_width_address i64.load i64.const 2 i64.div_s call $to_host global.get $state_height_address i64.load i64.const 666667 call $fixed_mul call $to_host f32.const 36 i32.const 0xff5cf4ff i32.const 1 call $text drop))
		call $load_flags global.get $flag_paused i32.and
		(if (then i32.const 33 i32.const 100 i32.const 6 global.get $state_width_address i64.load i64.const 2 i64.div_s call $to_host global.get $state_height_address i64.load i64.const 2 i64.div_s call $to_host f32.const 36 i32.const 0xffcf5cff i32.const 1 call $text drop))
		call $load_flags global.get $flag_help_visible i32.and
		(if (then call $draw_help_overlay))
		global.get $state_lifecycle_address i32.load i32.const 3 i32.eq
		(if (then i32.const 34 i32.const 108 i32.const 9 global.get $state_width_address i64.load i64.const 2 i64.div_s call $to_host global.get $state_height_address i64.load i64.const 2 i64.div_s call $to_host f32.const 40 i32.const 0xff5c73ff i32.const 1 call $text drop))
		call $publish_gate_ui
		call $frame_end drop i32.const 0)
)
