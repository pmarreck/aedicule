# Vibesteroids Source-Derived Behavioral Specification

## 1. Purpose and evidence standard

This document specifies the algorithms and observable behavior of Peter
Marreck's original Vibesteroids implementation so that its personality and
gameplay can be carried into the `gpui-wasm` WAT demonstrator.

This is explicitly **not** a clean-room specification. Original symbol names,
data layouts, algorithms, constants, implementation details, and quirks are
included intentionally.

The primary reference is:

- `/home/pmarreck/Code/vibesteroids/docs/index.html` at repository `HEAD`
  `a58e7b9`, especially the production symbols between
  `BEGIN_PURE_JS_TEST_SUITE_DO_NOT_REMOVE_OR_ALTER_THIS` and the embedded test
  suite;
- `docs/index.html`'s embedded unit and performance tests;
- `test/vibesteroids_test`, the Node/jsdom extraction runner;
- `README.md`, for user-facing intent;
- `vibesteroids`, the POSIX browser launcher; and
- `flake.nix` and `https_server.py`, for development/platform behavior.

References use `path::symbol`. Unless a passage is explicitly labeled
**Inference**, **Intent**, **Quirk**, or **Dead/legacy**, it describes behavior
directly present in the source.

The source repository had unrelated pre-existing changes to agent-instruction
symlinks and `.dirtree-state`; no game source was changed during this analysis.

## 2. Product shape and architecture

### 2.1 Distribution

Vibesteroids is a single-page browser game. All production HTML, CSS,
JavaScript, canvas drawing, generated sound, mobile controls, and tests live in
`docs/index.html`. `src/vibesteroids.html` is a symlink to that file.

The executable `vibesteroids` is a POSIX shell launcher. It resolves its own
directory, then opens `src/vibesteroids.html` with `xdg-open` when available or
macOS `open` otherwise. It prints an error and exits 1 if neither exists.

There is no application server in normal play. `flake.nix` supplies optional
HTTPS development servers because mobile motion/audio APIs may require a secure
context. The primary dev server uses BrowserSync; `https_server.py` is a
no-cache HTTPS fallback.

### 2.2 Runtime layers

The runtime is organized conceptually as:

1. environment and URL-mode detection;
2. seeded pseudorandom-number helpers;
3. sound adapters (`RealSoundInterface` and `MockSoundInterface`);
4. global viewport/scaling configuration;
5. one global `GameState` object;
6. DOM, keyboard, touch, and motion input adapters;
7. mostly state-in/state-out simulation functions;
8. a Canvas 2D plus DOM renderer; and
9. a `requestAnimationFrame` loop.

The advertised architecture is functional and immutable, but production makes
a deliberate performance compromise. `updateGameState` shallow-copies the
outer state and ship, while arrays and the mutable RNG object are shared and
sometimes modified in place. The code explains this as avoiding mobile garbage
collector pressure at high levels. See
`docs/index.html::updateGameState`.

### 2.3 Execution modes and URL parameters

`docs/index.html` recognizes:

- no query: play normally;
- `?test`: run embedded tests instead of the game;
- `?test=unit`, `?test=perf`, or a comma-separated suite list: filter tests;
- `?seed=N`: seed gameplay or tests deterministically; and
- `?level=N`: start/restart at that level.

Shift-T (`e.key === "T"`) navigates to the same pathname with `?test`, losing
the current game. The invisible upper-right 12.5-vw square does the same.

There is no validation for `level`. A nonnumeric value can produce `NaN`,
zero asteroid spawns, and a permanently nonprogressing game. A nonnumeric seed
is coerced to zero by the PRNG's bitwise operations.

On mobile user agents, the page dynamically loads Eruda from jsDelivr unless
test mode is active. Consequently, normal mobile play is not strictly
network-independent even though the game itself is otherwise self-contained.

## 3. Coordinate system, viewport scaling, and resizing

Source: `docs/index.html::calculateScaleFactor`,
`updateGameDimensions`, `scale`.

The reference coordinate system is 1024×768. Its diagonal is exactly 1280:

```text
BASE_WIDTH    = 1024
BASE_HEIGHT   = 768
BASE_DIAGONAL = sqrt(1024² + 768²) = 1280
```

The live game dimensions are the browser's full `innerWidth` and
`innerHeight`. The scalar visual scale is:

```text
SCALE_FACTOR =
  sqrt(viewport_width² + viewport_height²) / 1280
```

This preserves neither fixed horizontal nor fixed vertical scale; it scales
against diagonal size.

The following base quantities are cached after multiplying by
`SCALE_FACTOR`:

| Quantity | Base value |
| --- | ---: |
| ship collision radius | 10 |
| bullet drawing radius | 2 |
| particle drawing radius | 2 |
| bullet muzzle offset | 20 |
| title font | 60 |
| subtitle font | 20 |
| common line widths | 2, 4, 6, 8 |
| common shadow blurs | 5, 8, 10, 20 |
| maximum asteroid radius | 50 |
| respawn safe radius | 96 |
| desperation-bomb radius | 80 |
| Death Blossom/game-over-size font | 36 |
| subtitle horizontal offset | 200 |

`SCALED_BULLET_BUFFER` is half the scaled maximum asteroid radius: 25 at
reference scale.

On initial sizing and every changed window size:

- the canvas backing width and height become the viewport dimensions;
- CSS variable `--game-width` is set;
- exactly 100 one-pixel stars are regenerated from fixed seed 42; and
- if there was a previous nonzero viewport, every ship, asteroid, bullet,
  particle, and debris position is multiplied independently by
  `new_width/old_width` and `new_height/old_height`.

Velocities are not rescaled on resize.

**Quirk:** asteroid radii and polygon points are getters that use the current
scale, while `ship.radius` is a stored number. Resizing updates the asteroid
collision geometry immediately but leaves the ship collision radius and
level-dependent scaled acceleration/bullet-speed caches stale until a later
level transition calls `updateLevelDependentConstants`.

**Quirk:** movement functions multiply velocity by `SCALE_FACTOR`, while ship
acceleration and bullet speed have already been scaled. Some motion therefore
receives scale in both velocity construction and integration at non-reference
sizes.

## 4. Randomness and determinism

Source: `docs/index.html::mulberry32Step`, `createSeededRNG`,
`createRNGState`.

The game uses Mulberry32 with a 32-bit state:

```text
state = state + 0x6d2b79f5
t = imul(state xor (state >>> 15), state | 1)
t = t xor (t + imul(t xor (t >>> 7), t | 61))
value = unsigned(t xor (t >>> 14)) / 2^32
```

`createRNGState(seed)` exposes `{ seed, state, next() }`, but `state` in the
returned object is only the initial public property. The closure-local state is
what advances; the public `state` property is not updated.

Without `?seed`, one page-load seed is selected with
`floor(Math.random() * 1_000_000)`. Every `restartGame` reuses that same seed,
so restarts within one page session replay the same RNG stream and initial
asteroid field.

Game RNG drives:

- asteroid shapes, velocities, rotations, and spawn locations;
- asteroid splitting offsets and child velocities;
- particle directions, speeds, and lifetimes;
- debris directions, speeds, and spins; and
- the visible thrust-flame length.

Stars use a separate fixed seed of 42.

Generated explosion audio uses ambient `Math.random`, not the game RNG.

**Quirk:** `drawShip` calls `state.rng.next()` while rendering a thrust flame.
Rendering therefore advances gameplay RNG state. The future simulation depends
on how often the game was rendered while thrusting, so seeded behavior is not
strictly independent of refresh rate or extra renders.

## 5. State model

Source: `docs/index.html::createInitialGameState`.

The top-level state contains:

### 5.1 Ship

```text
x, y             position
vx, vy           velocity
angle            radians; -π/2 initially (up)
radius           scaled collision radius
thrusting        render/audio flag
```

Initial position is viewport center and velocity is zero.

### 5.2 Entity arrays

- `asteroids`
- `bullets`
- `particles`
- `shipDebris`

There is no production asteroid or bullet capacity. Ordinary explosion
particles are capped at 150, but ship-explosion particles bypass that cap.

### 5.3 Progress and lifecycle

```text
score                       0
lives                       3
nextExtraLifeScore          20,000
level                       ?level or 1
gameOver                    false
paused                      false
shipExploding               false
shipExplosionStartTime      0 ms
waitingForRespawn           false
waitingForRespawnStartTime  0 ms
lastFireTime                0 ms
frameNumber                 0
```

`lives` includes the currently active ship. The HUD deliberately displays
`max(0, lives - 1)` rocket emoji as reserve ships.

### 5.4 Input/mode latches

```text
escapeKeyWasPressed
manualPauseToggleRequested
autoFire                  added by restartGame; false
kidMode
```

### 5.5 Title splash

Active runtime fields are:

```text
titleSplashVisible
frameTitleFirstShownAt     null or starting frame
```

`titleSplashFrameCount` is retained but not advanced by production behavior.

### 5.6 Death Blossom

```text
deathBlossomActive
deathBlossomStartTime
deathBlossomRotations
deathBlossomTotalAngleRotated
deathBlossomAvailable
```

It starts available and represents one use for the current ship life.

### 5.7 Mobile capability state

```text
isMobileDevice
motionPermissionRequested
motionPermissionGranted
userHasInteracted
deathBlossomButtonFallbackEnabled
```

### 5.8 Injected sound and RNG

`rng` is a mutable `createRNGState` object. `sound` is a
`RealSoundInterface` in normal play and normally a `MockSoundInterface` in
tests.

## 6. Initialization, restart, and game loop

Source: `docs/index.html::restartGame`, `gameLoop`.

Normal startup executes:

1. `restartGame()`;
2. `requestAnimationFrame(gameLoop)`.

`restartGame`:

- hides the game-over overlay;
- creates a fresh state with real sound, a visible title splash beginning at
  frame 0, and auto-fire off;
- spawns `5 + (level - 1)` asteroids; and
- writes the level to the HUD.

The game loop computes:

```text
deltaTime = (requestAnimationFrame_timestamp - lastTime) / 1000
```

It then calls `updateGameState`, calls `render`, and schedules another frame.
There is no fixed-step accumulator and no maximum-delta clamp.

**Quirk:** the first frame measures from `lastTime = 0`, not from restart, so a
slow page startup can yield a large first update.

**Quirk:** `restartGame` does not call
`updateLevelDependentConstants`. Restarting after reaching a higher level can
leave high-level acceleration, rotation, and bullet speed in effect during the
new first wave. The initial game does not exhibit this because constants were
initialized earlier during page setup.

## 7. Input and control behavior

### 7.1 Desktop keyboard

Source: the `keydown`/`keyup` listeners and
`docs/index.html::updateGameState`.

| Input | Behavior |
| --- | --- |
| Left arrow | rotate counterclockwise |
| Right arrow | rotate clockwise |
| Up arrow | thrust |
| Space | fire while held, rate-limited |
| F or Shift-F | toggle continuous auto-fire |
| Escape | edge-trigger pause/resume |
| Shift-K (`e.key === "K"`) | toggle Kid Mode |
| Shift-B (`e.key === "B"`) | activate Death Blossom when eligible |
| Shift-T (`e.key === "T"`) | reload into test mode |

ArrowDown scrolling is prevented but it has no game action.

`keysPressed` records key state. Escape uses `escapeKeyWasPressed` so holding it
does not repeatedly toggle. F and K are handled directly in `keydown` and do
not suppress key-repeat toggling; holding either can toggle repeatedly under
browser auto-repeat.

Using arrows, space, F, or B adds `body.playing`, hiding the cursor. A new
three-second timer is installed on each such keydown; the cursor returns after
three seconds without another listed game key.

### 7.2 Click/touch overlays

- Invisible top-left square, 12.5% of viewport width on each side: toggle Kid
  Mode.
- Invisible top-right square: enter test mode.
- The top-center `🔆` Death Blossom availability indicator: toggle pause; it
  does **not** activate Death Blossom.
- Clicking/touching the pause overlay toggles back to play.
- The game-over overlay contains `Play Again`, which calls `restartGame`.

Click-plus-touch duplicate pause events are suppressed when less than 100 ms
apart.

### 7.3 Pause precedence

At the beginning of `updateGameState`:

1. a newly pressed Escape toggles pause and returns immediately;
2. a manual pause request toggles pause and returns immediately;
3. a paused state returns without simulation, while still tracking Escape
   release.

The render loop continues while paused, so the overlay remains responsive.

**Quirk:** `render` logs `Game paused - showing pause screen` every rendered
frame while paused.

### 7.4 Focus

The browser version has no blur/visibility handler. Held controls can remain
latched if a key-up is lost while changing focus. This differs from
`gpui-wasm`, whose generic focus-loss event clears held controls.

## 8. Mobile controls, shake detection, and permission flow

Source: mobile listener block around `handleDeviceMotion`,
`showMobileTutorial`, and the canvas touch handlers.

Mobile is detected solely from a user-agent regular expression. On detection:

- normal auto-fire is disabled;
- a tutorial pauses the game 100 ms after load;
- touch controls are registered; and
- motion-permission/fallback-button logic becomes available.

### 8.1 Touch zones

For the first touch:

- top center: `40% ≤ x ≤ 60%` and `y ≤ 70` requests pause/resume;
- left 20%: begin counterclockwise rotary control;
- right 20%: begin clockwise rotary control;
- middle 60%, excluding top center: thrust while held.

An edge touch also fires continuously because `isRotaryTouching` participates
in the normal firing predicate.

Dragging an edge maps vertical displacement linearly:

```text
TOUCH_ANGLE_RANGE = 4π

left edge:
  angle = angle_at_touch_start - (delta_y / game_height) * 4π

right edge:
  angle = angle_at_touch_start + (delta_y / game_height) * 4π
```

A full-height drag therefore makes two rotations. Death Blossom suppresses
touch rotation. Touch end/cancel clears all touch latches. Every touch handler
uses `try/catch`; errors are logged and latches are reset where possible.

### 8.2 Motion activation

Shake magnitude uses acceleration including gravity:

```text
magnitude = sqrt(x² + y² + z²)
trigger when magnitude > 25
cooldown > 2000 ms
```

Activation additionally requires available/not-active Death Blossom, a live
ship, no respawn wait, and no game over.

After the first canvas interaction:

1. audio is resumed and a zero-gain oscillator is attempted to unlock iOS
   sound;
2. after five seconds, an enable-shake prompt appears if permission has not
   been requested;
3. the prompt auto-hides after ten more seconds; and
4. if permission is still absent, a visible Death Blossom fallback button is
   enabled.

iOS `DeviceMotionEvent.requestPermission()` is called directly from the prompt
click. Permission denial/error enables the fallback. Other platforms attach a
`devicemotion` listener directly.

The fallback Death Blossom button has both touch and click activation handlers.
It also has a second click listener that treats two clicks within 300 ms as a
request to retry motion permission.

**Dead/legacy:** `requestMotionPermission()` contains a complete alternative
permission flow but has no caller in the current source.

No haptics or `navigator.vibrate` calls exist.

## 9. Simulation timing and update order

Source: `docs/index.html::updateGameState`.

After pause/game-over early returns, each animation update performs:

1. Death Blossom rotation/end check;
2. outer-state/ship shallow copy and `frameNumber++`;
3. ship-explosion timeout and transition to game over or respawn wait;
4. ship controls, ship physics, and firing;
5. bullet movement/lifetime filtering;
6. particle movement/decay;
7. ship-debris movement/decay;
8. asteroid movement;
9. bullet-versus-asteroid collisions;
10. ship-versus-asteroid collision;
11. empty-wave level progression and respawn;
12. title-splash lifetime update; and
13. safe-respawn, shrinking-zone, and desperation-bomb logic.

Collisions are endpoint tests after movement, not swept-volume tests.

The following quantities are time-based through `deltaTime`:

- position integration;
- keyboard rotation;
- thrust acceleration; and
- asteroid/debris angular motion.

The following are frame-based:

- ship drag (×0.99 per update);
- particle drag (×0.98 per update);
- debris linear drag (×0.99) and angular drag (×0.98);
- particle/debris lives decrementing by 1; and
- title splash's 240-frame lifetime.

The game therefore changes feel and effect duration with display refresh rate.

## 10. Level-dependent difficulty

Source: `docs/index.html::updateLevelDependentConstants`,
`calculateFireDelay`, `limitVelocityByLevel`.

For level `L < 20`:

```text
ship acceleration =
  300 + (500 - 300) * (L - 1) / 19

desktop rotation speed =
  5 + (10 - 5) * (L - 1) / 19 radians/second

bullet speed =
  337.5 + (765 - 337.5) * (L - 1) / 19

fire delay =
  250 - (250 - 125) * (L - 1) / 19 milliseconds

maximum absolute asteroid velocity component =
  30 * (2 + 0.25 * (L - 1))
  = 60 + 7.5 * (L - 1)
```

At level 20 and later:

- acceleration is 500;
- desktop rotation is 10 radians/second;
- bullet speed is 765;
- fire delay is 125 ms; and
- asteroid velocity is not capped.

Acceleration and bullet speed are multiplied by viewport scale when cached.
Mobile rotation always uses the base speed of 5 because touch sets angle
directly.

Death Blossom's intended bullet speed is always the base 450, scaled for the
viewport.

Each cleared wave increments level and spawns:

```text
5 + (new_level - 1)
```

Thus level 1 starts with 5 asteroids, level 2 with 6, and so on.

## 11. Ship physics and drawing

Source: `docs/index.html::updateGameState`, `drawShip`.

### 11.1 Rotation

Keyboard rotation is:

```text
left:  angle -= CURRENT_ROTATION_SPEED * deltaTime
right: angle += CURRENT_ROTATION_SPEED * deltaTime
```

Angles are never normalized.

### 11.2 Thrust

While Up or middle-screen mobile touch is held:

```text
acceleration_step = CURRENT_SHIP_ACCELERATION * deltaTime
vx += cos(angle) * acceleration_step
vy += sin(angle) * acceleration_step
```

The ship moves by:

```text
x += vx * SCALE_FACTOR * deltaTime
y += vy * SCALE_FACTOR * deltaTime
vx *= 0.99
vy *= 0.99
```

It wraps to the opposite exact edge whenever its center passes 0 or the
viewport width/height. Collision radius is not included in wrapping.

Thrust is disabled during Death Blossom and respawn waiting. During respawn
waiting, left/right steering remains enabled, but position is forced to center
and velocity to zero.

Each thrusting update has a 10% `mathObj.random()` chance to request a thrust
sound. Production passes ambient `Math`, not the seeded game RNG.

### 11.3 Vector appearance

The ship is a white outlined asymmetric four-point polygon:

```text
nose      ( 15,  0)
rear-left (-10, -8)
notch     ( -5,  0)
rear-right(-10,  8)
```

It has a filled white radius-3 center dot.

When thrusting, an orange three-point flame runs from `(-5,-3)` to a random
point between `(-25,0)` and `(-10,0)` and back to `(-5,3)`.

During respawn waiting the renderer applies alpha 0.3. It attempts to set gray
stroke first, but `drawShip` resets stroke to white, so the actual effect is a
dim white ship rather than gray.

## 12. Bullets and firing

Source: `docs/index.html::createBullet`, `fireBullet`, `updateBullets`.

A bullet starts 20 scaled units along the ship's facing direction:

```text
x = ship.x + cos(angle) * muzzle_offset
y = ship.y + sin(angle) * muzzle_offset
vx = ship.vx + cos(angle) * current_bullet_speed
vy = ship.vy + sin(angle) * current_bullet_speed
distanceTraveled = 0
```

Normal firing occurs when Space, auto-fire, or a mobile edge touch is active
and:

```text
currentTime - lastFireTime > level_fire_delay
```

There is no bullet-count limit.

Bullets:

- move by velocity × scale × delta time;
- accumulate Euclidean distance moved before wrapping;
- wrap only after their center passes a buffer of half the largest asteroid
  radius (25 scaled units); and
- disappear after traveling half the current viewport diagonal.

They render as filled green radius-2 circles.

**Quirk:** Death Blossom temporarily writes a selected speed into
`state.BULLET_SPEED`, but `createBullet` ignores that property and always reads
global `CURRENT_BULLET_SPEED`. Death Blossom therefore uses the current
level-dependent bullet speed in actual play, not its intended constant
base-speed cache.

## 13. Asteroid generation, shape, movement, and splitting

### 13.1 Initial asteroid construction

Source: `docs/index.html::createAsteroid`.

Each asteroid receives:

```text
baseRadius = 20 + random * 30       // [20, 50)
segments   = 8 + floor(random * 4) // 8..11
angle      = 0
angularVelocity = (random - 0.5) * 2 // [-1, 1)
```

For every base segment:

```text
baseAngle      = i / segments * 2π
angleJitter    = (random - 0.5) * 0.15
radiusVariation= baseRadius * (0.6 + random * 0.5)
point          = polar(baseAngle + angleJitter, radiusVariation)
```

Zero or one additional point is inserted. Its radius is
`baseRadius * [0.5, 0.8)`, so despite being called a “spike” it is commonly an
inward notch relative to neighboring points.

Total point count is 8 through 12.

Initial velocity components are independently sampled from `[-50,50)`, capped
for the level, then forced to absolute magnitude at least 15. An exact zero
gets a seeded random sign.

`radius` and `points` are dynamic getters over `baseRadius`, `basePoints`, and
the current viewport scale.

### 13.2 Wave spawning

Source: `docs/index.html::spawnAsteroids`.

The game creates eight axis-aligned rectangles around a square exclusion zone
centered on the current ship position. The exclusion half-size is:

```text
scaled safe radius 96 + scaled maximum asteroid radius 50
```

Invalid or out-of-bounds regions are discarded. Each asteroid chooses a valid
region uniformly, then chooses a uniform point within it.

Regions are not weighted by area, so smaller rectangles receive the same
selection probability as larger ones.

If no valid region exists, a fallback point is selected on a circle two safe
radii from viewport center. That point is not clamped before normal wrapping.

### 13.3 Motion and rendering

Asteroids move by velocity × scale × delta time and rotate by angular velocity
× delta time. Their centers wrap at fixed unscaled margins of -50 and
width/height +50.

The renderer draws a gray width-2 closed polygon. It also draws chords from
every third point to the point three positions later, producing internal rock
detail.

### 13.4 Splitting

Source: `docs/index.html::handleAsteroidCollision`.

Every asteroid collision creates an ordinary particle explosion. An asteroid
splits iff:

```text
baseRadius > ASTEROID_SPLIT_THRESHOLD
ASTEROID_SPLIT_THRESHOLD = 20
```

It produces exactly two children. Each child:

- starts within ±20 unscaled units on x and y of the parent;
- has `baseRadius = parent.baseRadius * 0.6`;
- starts from a newly randomized polygon, then rescales all points to the exact
  child radius;
- inherits parent velocity plus a seeded random perturbation;
- is level-capped and minimum-speed constrained; and
- receives angular velocity in `[-2,2)`.

The perturbation uses:

```text
baseSpeed     = 120
sizeMultiplier= 40 / child.baseRadius
velocityBoost = baseSpeed * sizeMultiplier
component delta in [-velocityBoost/2, velocityBoost/2)
```

Because initial asteroids are in `[20,50)` and the comparison is strict, nearly
every initial asteroid splits at least once. Parents above 33⅓ create children
still above 20, permitting another split generation.

At levels 20+, child velocities are uncapped and can become very large.

## 14. Collision rules

Source: `docs/index.html::distance`, `updateGameState`,
`handleAsteroidCollision`.

All collision tests use ordinary Euclidean center distance and strict `<`.
They do not account for toroidal adjacency across a wrapped screen edge.

There are:

- bullet–asteroid collisions;
- ship–asteroid collisions; and
- no asteroid–asteroid, bullet–ship, or debris collisions.

### 14.1 Bullet versus asteroid

Collision radius:

```text
distance(bullet, asteroid) < asteroid.radius + 5
```

The drawn bullet radius is 2, but collision adds a hard-coded 5.

Before splitting/destruction, bullet momentum is transferred:

```text
momentumTransfer = 0.12
sizeFactor       = 30 / (asteroid.baseRadius or asteroid.radius)
transferAmount   = momentumTransfer * sizeFactor

new asteroid velocity =
  level_limit(old asteroid velocity + bullet velocity * transferAmount)
```

The momentum-adjusted asteroid becomes the parent of split children. On hit:

- asteroid-explosion sound plays;
- bullet is removed;
- children are appended;
- original asteroid is removed;
- score and possible extra life are processed unless Kid Mode is active.

The loops run backward. One bullet can resolve only one asteroid in a frame.

### 14.2 Ship versus asteroid

Collision radius:

```text
distance(ship, asteroid) < ship.radius + asteroid.radius
```

Only one ship collision is processed per frame. The collision:

- begins a two-second ship explosion;
- creates four debris pieces and 40 particles;
- destroys/splits the asteroid exactly like a bullet hit;
- plays ship and asteroid explosion sounds together;
- decrements lives unless Kid Mode is active; and
- cancels active Death Blossom.

It awards no asteroid points.

Bullet collision processing happens before ship collision processing. Newly
created child asteroids are therefore eligible to hit the ship later in the
same frame.

**Quirk:** the third fixed asteroid is checked twice in the source's combined
ship-hit expression in the current WAT demo, but the original JavaScript loops
normally over the array once. This duplicate applies only to the current
rudimentary WAT, not original Vibesteroids.

### 14.3 No continuous collision

Fast bullets and asteroids can tunnel because collision is checked only at
post-update positions. Wrap transitions are also not checked along their path.

## 15. Score, lives, extra ships, levels, and game over

### 15.1 Scoring

Source: collision scoring inside
`docs/index.html::updateGameState`.

Only bullet hits outside Kid Mode score:

| Asteroid at hit | Points |
| --- | ---: |
| `baseRadius > 20` (will split) | 80 |
| `baseRadius <= 20` (will not split) | 120 |

Ship collisions and the desperation bomb award zero.

### 15.2 Extra lives

Source: `docs/index.html::checkExtraLife`.

The first threshold is 20,000 and thresholds advance by another 20,000. If a
single scoring event crosses the next threshold:

- `lives++`;
- `nextExtraLifeScore += 20_000`; and
- the extra-life chime plays.

Only one threshold can be awarded per call. Normal maximum point increments
are too small to skip several thresholds.

**Quirk:** `checkExtraLife` plays the chime, then its collision caller detects
the increased life and plays `extraLife()` again. Production therefore
requests the five-note sequence twice for one award.

### 15.3 Wave progression

When the asteroid array reaches zero and game over is false, the same update:

- increments level;
- recalculates level-dependent constants; and
- spawns the next wave.

This can occur while the ship is exploding or waiting to respawn.

### 15.4 Lives and game over

The active ship counts as one life. Collision on the final life sets lives to
zero but does not immediately set game over. The ship explosion remains
visible for more than 2000 ms. On the first later update satisfying:

```text
currentTime - shipExplosionStartTime > 2000
```

the debris is cleared and:

- lives ≤ 0: `gameOver = true`, final score is written to the DOM, and the
  game-over overlay appears;
- otherwise: enter safe-respawn waiting.

The game-over state stops simulation, but Escape/manual pause handling occurs
before the game-over early return.

## 16. Safe respawn

Source: `docs/index.html::isSpawnAreaClear` and the final block of
`updateGameState`.

After a nonfinal ship explosion:

- ship is displayed dimly at viewport center;
- x/y remain exactly centered;
- vx/vy remain zero;
- thrust and firing are disabled;
- keyboard/touch steering can preserve a chosen respawn angle;
- asteroids, old bullets, particles, and debris continue updating.

Spawn is clear only if every asteroid satisfies:

```text
distance(center, asteroid) >= safeRadius + asteroid.radius
```

The base safe radius is 96 scaled units. After more than five seconds of
waiting it becomes 48.

After more than ten seconds, a desperation bomb:

- removes every asteroid whose edge intersects an 80-scaled-unit center bomb;
- creates an explosion at each removed asteroid;
- awards no score and does not split those asteroids;
- plays one asteroid-explosion sound; and
- creates another explosion at spawn center.

Once clear:

- waiting ends;
- Death Blossom becomes available for the new ship; and
- the title splash begins again at the current frame.

The ship retains its current angle.

If the area is already clear at the update that ends the two-second explosion,
the waiting state can begin and end within that same update.

## 17. Kid Mode

Source: keyboard/corner input, collision logic, and
`docs/index.html::render`.

Kid Mode:

- suppresses all bullet-hit score;
- suppresses life decrement on ship collision; and
- hides the entire HUD group, including score, level, and reserve ships.

It does **not** prevent:

- ship collision;
- the ship explosion;
- debris and particle effects;
- the respawn wait;
- Death Blossom cancellation; or
- ordinary asteroid splitting/destruction.

Because lives do not fall, Kid Mode cannot reach game over through collisions.
Toggling Kid Mode has no confirmation or persistent setting.

## 18. Death Blossom

Source: `docs/index.html::activateDeathBlossom`,
`updateDeathBlossomState`, `shouldDeathBlossomFire`.

Death Blossom starts available once per active ship. Eligible activation:

- sets active true;
- records current time;
- zeroes the completed-rotation counter;
- marks availability false immediately; and
- plays a three-whoop siren.

`activateDeathBlossom` itself also refuses activation while paused. Desktop,
shake, and fallback-button adapters add further game-over/explosion/respawn
guards.

While active:

- manual keyboard and touch rotation is disabled;
- thrust is disabled;
- ship angle increases at 7.2 radians/second;
- total rotated angle accumulates without normalization; and
- bullets auto-fire without a key.

It ends after:

```text
12 * 2π radians
```

At 7.2 rad/s that is approximately 10.472 seconds.

The firing interval is:

```text
BASE_FIRE_DELAY / 10 = 25 ms
```

Only one bullet can be emitted per animation update, so effective rate is also
bounded by frame rate. At 60 Hz, the 25-ms threshold normally allows about one
shot every two frames rather than exactly 40 shots/second.

Ship death cancels an active blossom but does not restore availability. A
successful respawn restores availability.

The canvas displays a glowing `DEATH BLOSSOM!` message at two-thirds viewport
height while active. The top-center `🔆` availability indicator is hidden as
soon as activation consumes availability.

## 19. Particle and ship-debris systems

### 19.1 Ordinary explosions

Source: `docs/index.html::createExplosion`, `updateParticles`.

An ordinary asteroid explosion tries to add 20 particles without exceeding
150 total. Each particle:

```text
position = explosion point
angle    = random * 2π
speed    = 50 + random * 150       // [50, 200)
life     = 30 + floor(random * 20) // 30..49 frames
velocity = polar(angle, speed)
```

Per update:

```text
position += velocity * SCALE_FACTOR * deltaTime
velocity *= 0.98
life -= 1
remove when life <= 0
```

Particles do not wrap.

Rendering is a radius-2 filled circle:

```text
rgba(255, 150 - life, 0, life / 40)
```

Initial alpha can exceed 1 for life 41–49 and is browser-clamped.

### 19.2 Ship debris

Source: `docs/index.html::createShipDebris`, `updateShipDebris`,
`drawShipDebris`.

A ship explosion creates four triangular/polyline pieces corresponding to
nose, left wing, right wing, and center. Their nominal masses are 0.8, 1.2,
1.2, and 0.6, but mass is not used after creation.

Each piece:

- inherits ship velocity;
- adds random radial speed in `[80,200)`;
- starts at a random angle;
- has angular velocity in `[-4,4)`;
- lives 120 frames; and
- wraps at fixed margins ±50.

Linear velocity is multiplied by 0.99 and angular velocity by 0.98 per frame.
Stroke alpha is `life / 120`.

The explosion also adds 40 particles with inherited ship velocity, radial speed
`[60,260)`, and life 40–69 frames. These particles are added without enforcing
the ordinary 150-particle limit.

**Quirk:** debris polygon coordinates are not viewport-scaled when rendered,
although the intact ship geometry is scaled.

## 20. Rendering and HUD

Source: `docs/index.html::render` and draw helpers.

### 20.1 Canvas draw order

Each rendered frame:

1. fills the canvas `#111`;
2. draws the neon title splash;
3. draws the Death Blossom message;
4. occasionally updates hidden debug DOM;
5. draws 100 white stars;
6. draws the ship, unless exploding or game over;
7. draws green bullets;
8. draws gray outlined asteroids;
9. draws orange fading particles; and
10. draws fading ship debris.

Because stars follow the title, star pixels can appear over title glyphs.

### 20.2 HUD

The absolute-positioned HUD displays:

- top-left score;
- top-left second row level;
- top-right reserve ships as repeated `🚀`;
- top-center `🔆` while Death Blossom is available.

Kid Mode hides the whole score/level/lives HUD but not the `🔆`.

### 20.3 Title splash

On initial game and each respawn, `VIBESTEROIDS` appears centered horizontally
at one-third viewport height, with `by Peter Marreck` below and offset left.

Its runtime alpha uses a hard-coded 60/120/60-frame sequence:

- 60-frame fade in;
- 120 frames steady;
- 60-frame fade out.

At its first frame alpha is forced to approximately 0.017. The title uses
magenta outer glow, cyan middle glow, white core, cyan subtitle glow, and five
faint green scan lines.

**Dead/legacy:** configurable helper functions
`getTitleSplashTotalFrames` and `calculateTitleSplashAlpha` support arbitrary
frame rates and steady durations and are heavily tested, but runtime
`drawNeonTitle` uses `calculateTitleSplashAlphaFromFrames`, whose 60/120/60
values are fixed.

**Quirk:** `drawNeonTitle` reads global `GameState` instead of the state passed
to `render`, weakening renderer purity.

### 20.4 Pause and game-over overlays

Pause is a centered translucent black overlay. It shows device-specific
instructions and a controls list. On mobile it reminds the player to disable
silent-ring mode for audio.

Game over is a centered overlay showing final score and `Play Again`.

There are no normal-play settings, preferences, high-score, or level-select
menus.

## 21. Audio

Source: `docs/index.html::RealSoundInterface`.

All sound is synthesized with the Web Audio API; there are no audio assets.
Failure to create `AudioContext` disables sound silently.

| Event | Synthesis |
| --- | --- |
| shot | default oscillator, 800→200 Hz exponential sweep over 0.1 s; gain 0.3→0.01 |
| thrust | sawtooth 60 Hz through 200-Hz low-pass; gain 0.1→0.01 over 0.05 s |
| asteroid explosion | 0.5-s white-noise buffer with exponential decay, 120-Hz peaking bass boost, low-pass 1500→80 Hz, gain 0.5→0.01 |
| ship explosion | 1.2-s white plus brownish noise, initial sinusoidal impulse, 300-Hz band-pass, low-pass 3000→100 Hz, gain 1→0.001 |
| Death Blossom | three 0.3-s 400→800→400-Hz whoops, starting 0.4 s apart |
| extra life | five sawtooth bell-like chimes at C5 times `[1, 1.125, 1.25, 1.5, 2]`, starting 0.15 s apart |

iOS suspended audio is resumed on first canvas touch. No master-volume or mute
setting exists.

`MockSoundInterface` records method names and `Date.now()` timestamps and
provides count/filter helpers for tests.

## 22. Persistence and settings

There is no:

- `localStorage` or `sessionStorage`;
- high-score persistence;
- saved options;
- save game;
- replay file;
- controller configuration; or
- network gameplay.

Only URL parameters influence initial level, seed, and test mode. A page reload
resets all state.

## 23. Platform-specific and development behavior

- Browser Canvas 2D and Web Audio are the production platform APIs.
- Linux and macOS launch through the POSIX `vibesteroids` script.
- Mobile behavior is selected by user-agent regex, not capability probing.
- iOS motion permission is requested only from user gestures.
- A secure HTTPS dev server is provided for mobile API testing.
- Desktop has no native app menu.
- There is no gamepad support.
- There is no mouse aiming or pointer steering.
- There is no vibration/haptic feedback.
- Window resizing preserves relative entity positions but not exact physical
  trajectories because velocity/scaling caches are inconsistent.

## 24. Source quirks and apparent dead code

These should be consciously preserved or consciously corrected; they should
not enter the WAT port accidentally.

1. Rendering the thrust flame consumes gameplay RNG.
2. State is only shallowly immutable; arrays and RNG are shared/mutated.
3. Drag, particles, debris, and title timing are refresh-rate dependent.
4. No delta clamp exists after stalls or initial startup.
5. Position integration applies viewport scale in addition to already-scaled
   acceleration/bullet speed.
6. Resize leaves ship collision radius and level-dependent scaled caches stale.
7. Restart leaves the previous level's cached acceleration/rotation/bullet
   speed until a wave clear.
8. Death Blossom's intended base bullet speed override is ineffective.
9. An extra life requests its chime twice.
10. Pause logs to the console every render frame.
11. Collisions are neither swept nor wrap-aware.
12. The “spike” inserted into asteroid polygons is often an inward notch.
13. `requestMotionPermission` is defined but unused.
14. `debugInfoVisible` is defined but unused; the former debug corner now
    toggles Kid Mode.
15. `titleSplashFrameCount` and configurable alpha helpers are legacy relative
    to the active frame-origin implementation.
16. The renderer is not fully pure: it reads globals, mutates DOM, consumes RNG,
    and logs.
17. The `deepCopyGameState` test helper recreates RNG from the original seed,
    not its advanced closure state.
18. Eruda adds an external mobile runtime dependency.
19. Initial `level` and `seed` URL values are not validated.
20. There is no browser focus-loss handling for stuck keys.

## 25. Porting policy for `gpui-wasm`

The port should distinguish three categories:

### 25.1 Preserve because they define Vibesteroids

- Peter's title/authorship;
- the angular ship and neon/vector aesthetic;
- five-asteroid opening wave and one-more-per-level progression;
- jagged rotating asteroids;
- 60%-radius two-child splitting;
- 80/120 scoring;
- three lives with reserve-ship display;
- level-scaled ship, weapon, and asteroid difficulty;
- screen wrapping;
- explosion particles and ship debris;
- safe respawn with shrinking exclusion and desperation bomb;
- Kid Mode;
- Death Blossom;
- seeded replayability;
- semantic generated sound; and
- pause, restart, and game-over flows.

### 25.2 Correct deliberately in the WAT version

The WAT version should:

- use the frontplane's fixed simulation ticks;
- make drag and effect lifetimes tick-deterministic;
- keep rendering pure and RNG-free;
- include all gameplay randomness in snapshotted memory;
- apply viewport scale exactly once;
- update all size/speed caches on viewport and level changes;
- make Death Blossom use its specified bullet speed;
- emit one extra-life cue;
- clear held controls on focus loss;
- bound entity arrays explicitly; and
- validate seed, viewport, and state invariants.

These corrections improve reproducibility and the frontplane experiment without
changing the recognizable game.

### 25.3 Decide explicitly before changing

The following are observable and merit a named choice:

- Euclidean versus toroidal seam collision;
- endpoint versus swept collision;
- exact original 0.99/0.98 frame drag versus a 60-Hz-equivalent fixed-tick
  coefficient;
- whether Kid Mode still shows explosions/respawn waiting;
- whether initial asteroids always split;
- whether mobile/touch/shake parity belongs in a desktop GPUI proof of concept;
- finite bullet/entity capacities required by the host; and
- whether restart repeats the same seed or obtains a new host-provided seed.

## Appendix A. Recommended implementation increments for the WAT demo

The current `plugins/vibesteroids.wat` proves the architecture but has only
three circular asteroids, one bullet slot, no visible numeric HUD, no splitting,
no particles, and immediate life reset. The following increments turn it into
an impressive Vibesteroids demo without adding application-specific Rust.

### Increment 1 — Expand deterministic state and show a real HUD

**Dependencies**

- existing snapshot/reload ABI;
- existing line, circle, and text commands;
- a schema bump because the memory layout changes.

**Implementation**

- Store a full Mulberry32 state in snapshotted memory.
- Define fixed-capacity arrays in WAT memory, initially:
  - 32 asteroids;
  - 64 bullets;
  - 150 particles;
  - 4 ship-debris pieces.
- Add authoritative `score`, `lives`, `level`, mode flags, timers, and
  next-extra-life threshold.
- Write a small signed/unsigned integer-to-decimal formatter in WAT so score,
  level, and lives can be passed to `host.v0.text`.
- Render score and level at top-left and reserve-ship glyphs or small vector
  ships at top-right.
- Initialize five asteroids, not three.

**Acceptance criteria**

- Seed plus viewport produces byte-identical snapshots and frame commands.
- Initial report visibly says score 0, level 1, and two reserve ships.
- Five active asteroid records exist at level 1.
- Snapshot restore and compatible hot reload preserve the HUD and RNG stream.
- Capacity overflow is deterministic and tested rather than corrupting memory.

### Increment 2 — Replace circles with recognizable jagged asteroids

**Dependencies**

- Increment 1 entity records;
- existing line commands, or generic path commands if enabled.

**Implementation**

- Give each asteroid 8–12 precomputed local polygon points at spawn time.
- Because core Wasm has no sine/cosine import, use one of:
  - a compact precomputed unit-circle table in WAT memory; or
  - fixed polygon templates plus per-asteroid scale/rotation using stored
    direction vectors.
- Rotate local points during rendering and emit a closed line loop plus sparse
  interior chords.
- Keep visual shape data in the snapshot so reload and headless SVG are exact.

**Acceptance criteria**

- No active asteroid renders as a circle.
- Two seeds produce visibly and bytewise different asteroid outlines.
- Replaying one seed produces identical outlines.
- Rotation changes rendered vertices without changing collision radius.
- Headless SVG contains closed jagged outlines and internal detail strokes.

### Increment 3 — Multi-bullet firing, collisions, splitting, and score

**Dependencies**

- entity arrays and jagged asteroid records.

**Implementation**

- Replace the single bullet slot with a bounded pool/ring or free-slot scan.
- Preserve muzzle offset, inherited ship velocity, rate limiting, wrapping, and
  travel-distance expiry.
- Implement endpoint circle collision first, matching the original.
- Transfer bullet momentum by the original size-sensitive formula.
- Split `baseRadius > 20` into exactly two children of 60% radius.
- Award 80 for a splitting parent and 120 for a terminal asteroid.
- Emit asteroid-explosion audio.

**Acceptance criteria**

- Holding Fire can create more than one simultaneous bullet.
- A deterministic large-asteroid hit removes one bullet, awards 80, and creates
  exactly two 60%-radius children.
- A deterministic terminal-child hit awards 120 and creates no child.
- Child velocities are minimum-speed constrained and deterministic.
- Clearing all descendants advances level and creates six level-2 parents.
- Independent host-side tests inspect snapshot records rather than trusting
  rendered circles alone.

### Increment 4 — Ship death, particles, debris, respawn, and game over

**Dependencies**

- particle/debris pools;
- complete collision rules;
- tick-based timers.

**Implementation**

- Add ship–asteroid collision and a 120-tick explosion at 60 simulation Hz.
- Render four recognizable ship fragments with rotation/fade.
- Add capped orange explosion particles with drag/fade.
- Keep asteroids and existing bullets moving during respawn wait.
- Display a dim center ship; permit steering but not thrust/fire.
- Use safe radius 96, halve after 300 ticks, and bomb after 600 ticks.
- Show a centered game-over message only after final explosion completion.
- Make New Game/menu restart reset the full WAT state.

**Acceptance criteria**

- First collision reduces lives from 3 to 2 and visibly explodes the ship.
- The intact ship is absent during explosion.
- Blocked center prevents respawn; steering changes future angle.
- Radius shrinks at exactly five seconds in fixed ticks.
- Desperation bomb removes only in-range asteroids without scoring.
- Final-life game over waits for explosion completion.
- New Game restores level 1, score 0, three lives, and five asteroids.

### Increment 5 — Original difficulty progression

**Dependencies**

- correct level transitions and entity spawning.

**Implementation**

- Port the exact level-1-through-20 formulas for acceleration, rotation,
  bullet speed, fire delay, and asteroid component caps.
- Recalculate from authoritative level and viewport values rather than mutable
  global caches.
- Keep mobile-specific rotation out until a mobile adapter exists.

**Acceptance criteria**

- Table-driven tests verify levels 1, 2, 19, 20, and 21.
- Level 1 and 20 match 300/500 acceleration, 5/10 rotation, 337.5/765 bullet
  speed, and 250/125-ms-equivalent tick delay.
- Level 20+ asteroid velocity is uncapped by the level limiter.
- Restart cannot retain prior-level constants.

### Increment 6 — Kid Mode and Death Blossom

**Dependencies**

- general key IDs or menu actions for K/B;
- multi-bullet pool;
- mode/timer state.

**Implementation**

- Extend the generic frontplane key map with ordinary physical K, B, and F key
  IDs; keep the Rust mapping generic rather than naming game actions.
- Kid Mode suppresses score/life loss and hides the ordinary HUD while retaining
  explosions and respawn flow.
- Death Blossom consumes one availability per ship, rotates at 7.2 rad/s,
  lasts 12 rotations, disables thrust/manual rotation, and auto-fires at the
  tick equivalent of 25 ms.
- Add the glowing `DEATH BLOSSOM!` overlay and availability indicator.
- Restore availability only on successful respawn/new game.

**Acceptance criteria**

- Kid Mode collision leaves lives and score unchanged but produces explosion
  feedback.
- Death Blossom cannot activate paused, dead, respawning, already active, or
  unavailable.
- It rotates exactly 24π radians before ending.
- It fires without Fire input and creates a visible radial bullet stream.
- Death cancels it; respawn restores availability.
- Snapshot/reload in the middle of Death Blossom resumes the exact remaining
  rotation and firing schedule.

### Increment 7 — Audio and presentation polish

**Dependencies**

- semantic audio event support already present;
- additional generic sound IDs only if the host lacks them.

**Implementation**

- Keep host audio synthesis generic and event-driven.
- Add distinct events for shot, thrust, asteroid explosion, ship explosion,
  siren, and extra life.
- Rate-limit thrust audio so one held key does not exhaust host event budgets.
- Recreate the neon title, Peter byline, stars, pause message, Death Blossom
  message, and game-over hierarchy.
- Add a short title splash at new game and respawn.

**Acceptance criteria**

- Every semantic event is observable in headless audio-event tests.
- One extra life emits one cue, not the JavaScript's accidental duplicate.
- Render-command budgets remain below configured limits at maximum supported
  entities.
- A saved SVG frame is immediately recognizable as Asteroids/Vibesteroids,
  with a readable score and jagged rocks.

### Increment 8 — Optional pointer/mobile parity

This is not required to make the desktop proof of concept impressive.

If pursued:

- implement left/right 20% rotary pointer zones and middle thrust through the
  existing generic pointer ABI;
- add a top-center pause target;
- treat device-motion as a new generic capability/event only when a real
  platform adapter exists; and
- do not add a Vibesteroids-specific “shake” host call.

### Recommended impressive milestone

For the immediate demo, complete Increments 1–4 and the visual portion of
Increment 7. That produces:

- five genuinely jagged moving asteroids;
- a clear score/level/lives HUD;
- multiple bullets;
- two-generation asteroid splitting;
- escalating waves;
- ship death, particles, debris, respawn, and game over;
- recognizable title/authorship; and
- audible firing/explosions.

Kid Mode and Death Blossom are excellent second-wave demonstrations because
they stress live state evolution and generic input/audio capabilities, but they
should not delay replacing the current three-circle scene.

## Appendix B. High-value test vectors

1. **Seed replay:** same seed, viewport, and input ticks produce identical
   snapshots, frame commands, and semantic audio events.
2. **Split boundary:** base radius 20 does not split; the next representable
   value above 20 does.
3. **Split geometry:** every child radius equals parent ×0.6 and every child is
   smaller.
4. **Score classifier:** a set containing splitting, terminal, ship-hit, Kid
   Mode, and bomb cases yields `[80,120,0,0,0]`.
5. **Fire delay table:** levels 1, 2, 19, 20, and 21.
6. **Entity pool saturation:** full bullet/particle/asteroid pools neither trap
   nor overwrite adjacent state.
7. **Collision ordering:** bullet hit and ship proximity in one tick has an
   explicitly chosen result.
8. **Respawn thresholds:** 299/300 and 599/600 fixed ticks.
9. **Death Blossom boundary:** one tick before and exactly at 24π accumulated
   rotation.
10. **Focus loss:** every held gameplay bit clears.
11. **Reload:** compatible WAT edit preserves an active split wave, particles,
    score, RNG, and Death Blossom timers.
12. **Schema change:** incompatible layout restarts rather than partially
    restoring.
13. **Renderer purity:** calling render twice without ticking produces
    identical commands and does not alter the state snapshot.
14. **Viewport change:** all authoritative scale-dependent values update once,
    with no stale collision radius or double scaling.
