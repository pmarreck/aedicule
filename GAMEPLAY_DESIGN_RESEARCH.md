# Vibesteroids Gameplay Design Research

## Purpose

This document separates gameplay-design research and recommendations from
`VIBESTEROIDS_BEHAVIOR_SPEC.md`, which records what the original Peter Marreck
implementation actually does. The question here is different: why do
*Asteroids* and its descendants remain compelling, and which additions would
strengthen Vibesteroids without burying its unusually clean core?

“Addictive” is used in the colloquial arcade sense of immediately replayable,
not as a goal of manufacturing compulsive behavior through dark patterns. The
target is intrinsic enjoyment: readable cause and effect, mastery, meaningful
choices, and honest challenge.

## Evidence base

The strongest historical evidence is designer Ed Logg's account of an early
field test. A player lost three ships in about twenty seconds and immediately
paid to retry. Logg's interpretation was not that the player had been randomly
rewarded, but that the player attributed each death to their own decisions and
believed they could improve. In the same interview, Logg explains that asteroid
splitting was added to create strategy, the saucer was added to prevent passive
play, the unfair immediate saucer shot was delayed, and the high-score table
preserved both score and identity. He also describes the desirable design space
as fixed rules plus enough randomness for players to form distinct strategies.
([Ed Logg interview](https://flylib.com/books/en/4.479.1.34/1/))

Atari's manuals expose the resulting decision system: large rocks split into
medium rocks and then small rocks; small targets are worth more; only a bounded
number of shots may coexist; later waves are harder; hyperspace trades escape
for risk; and the player is explicitly taught to prioritize proximity, speed,
and threat. The score and reserve ships remain continuously legible, with a
bonus ship tied to a known threshold.
([Atari Asteroids manual](https://www.atariage.com/manual_html_page.php?SoftwareID=2117))

Modern motivation research gives useful vocabulary without replacing the
historical evidence. Video-game enjoyment is consistently associated with
competence, autonomy, and relatedness; challenge/skill balance and perceived
control contribute to continued play, while “game feel” research emphasizes
clear, amplified feedback for meaningful events.
([motivational model](https://www.selfdeterminationtheory.org/SDT/documents/2010_PrzybylskiRigbyRyan_ROGP.pdf),
[challenge and continued play](https://pmc.ncbi.nlm.nih.gov/articles/PMC8943660/),
[game-feel survey](https://arxiv.org/abs/2011.09201))

## Why the original loop works

### 1. Death feels attributable and therefore reparable

The controls are small in number, deterministic, and physically coherent.
Momentum makes mistakes persist, but it also makes skill visible. A player can
usually name the failed decision—over-thrusting, turning too late, firing into a
bad split, or ignoring a nearer threat—and can imagine a better next attempt.

This property is more important than making the game gentle. A hard death that
feels earned invites another try; a moderate death caused by hidden rules,
latency, off-screen fire, or inconsistent collision does not.

### 2. Every shot changes the tactical geometry

Shooting a large asteroid does not simply remove health. It turns one slow,
large obstacle into two smaller, faster problems. The action is locally
satisfying and globally dangerous. Because the smaller targets are also worth
more, score and survival pull in related but nonidentical directions.

The player therefore manages a population, not a collection of hit points.
Firing is an irreversible tactical commitment, and the bounded bullet pool
prevents indiscriminate shooting from being free.

### 3. The ship is expressive before any content is added

Rotate, thrust, coast, counter-thrust, and wrap create a large state space from
four primary actions. Momentum permits recognizable personal styles: cautious
stationary rotation, sweeping orbits, aggressive close pursuit, and emergency
reversals. The rules remain learnable while the trajectories do not repeat.

This is the core “toy” test: maneuvering and shooting should feel good on an
empty field. New enemies cannot repair a ship that is unpleasant to steer.

### 4. Pressure alternates between self-created chaos and external interruption

Asteroid splitting lets the player create danger; the saucer prevents a stable
or exploitative position from becoming permanent. The original's large saucer
is noisy and inaccurate, while the small saucer becomes increasingly precise.
That produces an intelligible escalation from environmental management to
active pursuit.

The first saucer shot delay is a particularly valuable fairness lesson. Threat
can be severe after it has been announced; invisible or unavoidable punishment
breaks the “my fault, I can improve” loop.

### 5. Difficulty rises through legible dimensions

More asteroids, faster fragments, more accurate enemies, and tighter firing
windows all reuse rules the player already understands. The player improves at
the same vocabulary the game uses to become harder. This supports mastery
better than adding an unrelated rule every wave.

Vibesteroids already moves in this direction by increasing bullet speed and
asteroid component caps with level and by making fragments inherit motion plus
seeded, increasingly wild impulses.

### 6. Feedback is immediate and economical

The vector image keeps trajectories, silhouettes, and collision risks clear.
Distinct sounds identify firing, thrust, asteroid destruction, ship death,
extra lives, and Death Blossom without requiring the player to read. A split,
score increase, particle burst, and sound all agree about the same event.

Feedback should amplify information rather than obscure it. Long hit-stop,
camera shake, particle fog, or loud compression would be actively harmful in a
game whose challenge depends on reading every moving object.

### 7. Sessions restart quickly but identity persists

The interaction cost from death to another meaningful attempt is tiny. At the
same time, high score and initials make improvement visible and socially
legible. Logg compared the high-score table's identity function to graffiti:
the run ends, but evidence that the player was there remains.

## What Blasteroids added—and what is worth borrowing

Atari's *Blasteroids* operator manual describes three instantly switchable ship
forms: the fast Speeder, high-firepower Fighter, and armored Warrior. It adds
distinct enemies, temporary powers, energy crystals, selectable starting
difficulties, two-player cooperation/competition, and a galaxy boss. Crucially,
the forms have explicit advantages and disadvantages rather than forming a
linear upgrade ladder.
([Blasteroids operator manual](https://manualzz.com/doc/9180084/atari-blasteroids-arcade-game-operators-manual))

The most transferable idea is **situational transformation**. It deepens the
same navigation/combat decision rather than pausing play for an inventory:

- Speeder: smallest collision profile and best acceleration, weakest weapon;
- Fighter: current Vibesteroids baseline;
- Warrior: slower rotation/acceleration, wider shot or spread, limited armor.

This would be a better post-POC experiment than permanent numerical upgrades.
The player chooses a tool continuously, competence remains decisive, and every
form can stay useful.

Energy, crystals, power-ups, sector choice, enemies, bosses, and multiplayer
are potentially good later layers, but together they form a different-sized
game. Importing the entire Blasteroids feature list before validating the core
would make tuning harder and weaken Mecha Aedicule's purpose as a clear WAT
application demonstration.

## Recommended design direction

### Preserve these invariants

1. Controls, physics, and collision remain deterministic for a given seed and
   input stream.
2. A death is explainable from visible state; hostile fire receives a fair
   warning and cannot originate as an unavoidable off-screen surprise.
3. Thrust, turning, firing, collision, and restart respond immediately.
4. Visual and audio effects never conceal collision-relevant geometry.
5. Difficulty changes use displayed or inferable rules, not secret mid-run
   assistance or sabotage.
6. Restart reaches controllable play in one action and well under one second.
7. Kid Mode and accessibility options do not silently change score comparability;
   the HUD should indicate the active ruleset.

### Priority 0: perfect the core feel

- Run the deterministic simulation at 120 ticks per second while expressing
  all rates per second and proving 60/120 equivalence within documented
  fixed-decimal tolerances.
- Tune acceleration, 0.995 drag, rotation, bullet speed, and fire cadence as a
  coupled system. Judge them in play, not as isolated constants.
- Restore exact, recognizable sound identities from the original implementation
  and keep synth declarations guest-owned.
- Add a persistent local high-score table with seed, level, score, ruleset, and
  optional initials. Death Blossom and Kid Mode runs should be identifiable.
- Make every death restartable immediately without losing the last score.

### Priority 1: complete the classic pressure system

- Add large and small saucers with distinct behavior, audible arrival, a fair
  first-shot delay, and accuracy/rate escalation.
- Award more for smaller/faster threats and add a clearly announced extra-life
  threshold.
- Preserve fragment inheritance plus level-scaled random impulse; cap only the
  dimensions needed to prevent impossible openings.
- Add a short wave transition that provides relief without stopping momentum
  long enough to break concentration.

### Priority 2: deepen expression without adding a campaign

- Prototype the three situational ship forms from Blasteroids behind a mode
  flag. Do not add permanent upgrades yet.
- Consider a close-call bonus based on measurable risk: destroying a rock
  within a small ship-radius envelope or surviving a near miss. It must never
  incentivize visually ambiguous collision behavior.
- Consider a score multiplier that grows through accurate, timely destruction
  and decays through inactivity, rather than through randomized reward drops.
- Give Death Blossom an explicit opportunity cost—scarcity already provides
  one—and preserve its semi-secret delight rather than turning it into a routine
  cooldown button.

### Priority 3: only after repeated playtests

- Enemy archetypes with one readable behavior each.
- Optional sector choices that advertise their dominant hazard.
- Temporary powers with strong silhouettes and time remaining visible.
- A boss that exercises learned steering and splitting skills rather than
  introducing an unrelated damage-sponge phase.
- Cooperative play and Blasteroids-style docking, which would be delightful but
  materially expands input, camera, networking/local-device, and balance scope.

## Experiments that can falsify the recommendations

Use deterministic seeds and retain replays as input streams rather than adding
opaque analytics. For each playtest, record:

- time to first death and cause;
- whether Peter can explain the death before reviewing a replay;
- restart latency and whether another run begins voluntarily;
- highest wave, score, and effective actions per minute;
- shots fired, hit rate, dangerous splits, and deaths while firing;
- time spent thrusting/coasting and frequency of successful counter-thrust;
- Death Blossom activation timing and survival afterward; and
- subjective ratings for control, fairness, readability, sound, and “one more
  run” desire.

Useful comparisons are small and reversible:

1. 60 Hz versus 120 Hz with equivalent motion;
2. drag 0.995 versus nearby values;
3. current fragment wildness versus one slightly lower/higher curve;
4. saucer absent versus large-saucer-only;
5. plain scoring versus size/risk-weighted scoring; and
6. baseline ship versus opt-in three-form transformation.

Do not compare several new mechanics at once. If a change improves score but
reduces perceived fairness or death attribution, it has likely optimized the
wrong outcome.

## Scope conclusion

The strongest next game feature is not more content. It is a fully legible,
high-frequency version of the existing movement/splitting loop, followed by the
classic saucer pressure system and persistent high-score identity. Once that is
excellent, Blasteroids' situational ship transformation is the most promising
way to deepen autonomy without diluting Asteroids' mastery curve.

That order also makes Vibesteroids a better Mecha Aedicule demonstration: live
WAT reload, deterministic fixed-decimal state, synthesized audio, menus, input,
rendering, persistence, and WAST behavior tests are all exercised by meaningful
gameplay rather than by unrelated feature accumulation.
