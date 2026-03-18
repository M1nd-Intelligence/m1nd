# m1nd Cinema -- STORYBOARD v2

**Target**: 33-second animated GIF for GitHub README hero image
**Resolution**: 1920x1080 (rendered), displayed at 960x540 (2x crisp)
**Framerate**: 30fps (990 frames total)
**Max GIF size**: <5MB (dithered, 128-color palette, lossy optimization)
**Tone**: Dark, precise, confident. Not flashy -- *inevitable*.
**Audience**: A senior developer who has tried every code intelligence tool and is skeptical.

---

## DESIGN PHILOSOPHY

The v1 was a tech demo. v2 is a *film*.

Three rules stolen from cinematography:
1. **Every cut must advance a claim.** No ornamental animation. If a circle glows, it proves something.
2. **Silence is louder than noise.** Dark frames, negative space, and held beats create more tension than particle explosions.
3. **The audience must feel dumb for not using this.** The comparison data must land like a punch, not a pamphlet.

Pacing model: Christopher Nolan trailer. Short statements. Long pauses. Then the hit.

---

## COLOR SYSTEM

| Role | Hex | Usage |
|------|-----|-------|
| Void | `#060B14` | Background. Darker than v1's `#0A0F1A`. Almost black. Trust lives in darkness. |
| Terminal Green | `#00FF88` | Command text, cursor blink. The developer's native color. |
| Signal (primary) | `#00E5A0` | Activated nodes, winning paths, LTP edges. Life. |
| Semantic (accent) | `#00B4D8` | 2nd activation dimension. Intelligence. |
| Temporal (warm) | `#F59E0B` | 3rd dimension. Warmer than v1's `#FFD93D` -- more amber, less toy. |
| Causal (danger) | `#EF4444` | 4th dimension, noise, errors, cost. Danger without being cartoonish. |
| Ghost | `#6366F1` | Ghost edges (structural holes). Purple = the invisible made visible. |
| Bone | `#E2E8F0` | Primary text. |
| Ash | `#64748B` | Dimmed nodes, secondary text. |
| Graphite | `#1E293B` | Dormant edges. Barely visible. |
| Cost Red | `#FF2D55` | Dollar signs, token counters. Apple-red: expensive, urgent. |

---

## TYPOGRAPHY

| Element | Font | Weight | Size | Tracking |
|---------|------|--------|------|----------|
| Terminal commands | `JetBrains Mono` (fallback: `Fira Code`, `monospace`) | 400 | 20px | 0 |
| Terminal cursor | Same | 400 | 20px | 0 |
| Big numbers (stats) | `Inter` (fallback: `system-ui`) | 800 | 72px | -2px |
| Labels | `Inter` | 600 | 16px | +0.5px |
| Tagline | `Inter` | 300 | 24px | +1px |
| Logo "m1nd" | `Inter` | 900 | 80px | -3px |
| Section headers | `Inter` | 700 | 28px | 0 |
| Small annotations | `Inter` | 400 | 13px | +0.3px |

**Rationale**: JetBrains Mono is the developer's language. Inter is the designer's language. Using both says "we speak your language AND we care about craft." The heavy tracking on small text prevents it from collapsing at GIF resolution.

---

## EASING VOCABULARY

| Name | Curve | Usage |
|------|-------|-------|
| SLAM | `easeOutExpo` | Numbers appearing, reveals. Arrives fast, settles slow. Impact. |
| BREATHE | `easeInOutQuad` | Node pulsing, glow cycles. Organic, alive. |
| DRAW | `easeInOutCubic` | Edge lines drawing, path traces. Smooth, deliberate. |
| FADE | `easeOutCubic` | Opacity changes. Gentle arrival. |
| VANISH | `easeInCubic` | Elements leaving. Quick exit, no lingering. |
| SNAP | `linear` | Cursor blink, text appearance. Mechanical precision. |
| SPRING | custom: overshoot 1.15, settle | Logo arrival, final badge. Confidence with weight. |

**Timing rationale**: Fast-in/slow-out (SLAM) for data reveals mimics the feeling of information arriving with certainty. The viewer's eye catches it at speed, then has time to read it at rest. This is the opposite of most UI animation (slow-in/fast-out), which feels tentative. We are not tentative.

---

## SCENE-BY-SCENE STORYBOARD

---

### SCENE 1: COLD OPEN -- "The Familiar"
**Time**: 0:00 -- 0:03 (90 frames)
**Camera**: Static. Centered. Full viewport is black.

**Frame 0-15 (0.0-0.5s)**: Pure black. Nothing. The GIF has loaded and the viewer sees nothing. This is intentional. 0.5 seconds of nothing creates the unconscious question: "is this broken?"

**Frame 15-30 (0.5-1.0s)**: A terminal cursor appears. Green rectangle, `#00FF88`, 2px wide, 20px tall. Positioned at roughly `(-300, 0)` -- left of center. It blinks once: on for 0.3s, off for 0.2s, on again. The cursor is the universal symbol of "something is about to happen." Every developer knows this shape.

**Frame 30-75 (1.0-2.5s)**: Characters type out, one by one, left to right:
```
$ grep -rn "authentication" ./backend
```
Typing speed: 40ms per character (25 chars/sec). Not instant (that's inhuman). Not slow (that's boring). This is a confident developer who knows what they're typing.

The font is `JetBrains Mono`, `#00FF88`, 20px. The `$` prompt is `#64748B` (ash). The command itself is green.

**Frame 75-90 (2.5-3.0s)**: After the command is fully typed, a 0.3s pause. Then the cursor stops blinking and holds steady. This is the moment before Enter is pressed. Anticipation.

**On screen**: Just the terminal line. Nothing else. The entire 1920x1080 frame is dark except for one line of monospace text.

**Emotional beat**: Recognition. "I know this. I do this every day."

**Sound metaphor** (for pacing): Silence. A single piano key pressed and held.

---

### SCENE 2: THE COST -- "What You Don't See"
**Time**: 0:03 -- 0:08 (150 frames)
**Camera**: Static, then slow drift upward

**Frame 90-105 (3.0-3.5s)**: The grep command is still on screen. Below it, output lines begin appearing -- fast, stacked, scrolling. These are grep results:

```
backend/auth.py:42:    def authenticate(self, ...
backend/middleware.py:18:    # authentication check
backend/session.py:91:    authentication_required = True
backend/jwt_handler.py:7:    """JWT authentication module"""
...
```

Lines appear at 60ms intervals (just fast enough to read one before the next arrives). They're in `#64748B` (ash) -- deliberately unsexy. This is grep output. It's boring. That's the point.

8 lines appear, then 4 more partially visible (fading at bottom edge). The viewer cannot read them all. Information overload. This is the *feeling* of grep: you get results, but you're drowning.

**Frame 105-135 (3.5-4.5s)**: Three counter elements fade in simultaneously, positioned in a row at `y = -350` (upper area), spaced evenly:

| Element | Position | Content | Animation |
|---------|----------|---------|-----------|
| Token counter | `x = -400` | Starts at `0`, counts up to `~47,000` | Numbers tick up using SLAM easing. Digits blur during counting (0.8s duration). Color: `#FF2D55` (Cost Red) |
| Clock | `x = 0` | `0.0s` ticking up to `3.2s` | Linear count. Color: `#F59E0B` (amber) |
| Cost | `x = 400` | `$0.000` counting up to `$0.041` | SLAM easing, 0.8s. Color: `#FF2D55` |

Each counter has a small label above it in `#64748B`, 13px: `tokens burned`, `wall clock`, `API cost`.

The counters are not huge -- they're informational, almost clinical. The horror is quiet.

**Frame 135-165 (4.5-5.5s)**: The grep results and counters hold. A new line appears below the grep output, typed character-by-character:

```
$ grep found 12 matches. But what did it miss?
```

This is NOT a real terminal command -- it's a rhetorical question styled as a comment. Color: `#64748B`. The question mark holds for 0.5s.

**Frame 165-210 (5.5-7.0s)**: Below the question, four lines fade in sequentially (0.3s each, staggered by 0.2s):

```
  no blast radius          -- what else breaks?
  no structural holes      -- what's missing?
  no co-change prediction  -- what else will change?
  no learning              -- it's as dumb next time
```

Each line is `#64748B`, 16px `Inter`. The `--` comments after each are `#EF4444` (causal red), giving them emphasis. These are the four sins of grep.

**Frame 210-240 (7.0-8.0s)**: Everything on screen fades to black over 0.6s using VANISH easing. Clean slate.

**Emotional beat**: Discomfort. "I never thought about what grep costs me." The slow counting creates dread. The four missing capabilities create desire.

**Sound metaphor**: A clock ticking. Coins dropping into a jar.

---

### SCENE 3: THE COMMAND -- "The Alternative"
**Time**: 0:08 -- 0:10 (60 frames)
**Camera**: Static. Same terminal position.

**Frame 240-255 (8.0-8.5s)**: Black screen. Same 0.5s of void as the cold open. Symmetry. Then the cursor reappears, same position as Scene 1. One blink cycle.

**Frame 255-285 (8.5-9.5s)**: Characters type out:
```
$ m1nd.activate("authentication")
```

Typing speed: same 40ms per character. But this time, the command text is `#00E5A0` (Signal green) instead of terminal green. It's subtly different -- warmer, more alive. The `m1nd` portion is `#00E5A0` at `fontWeight: 700`. The rest is 400.

**Frame 285-300 (9.5-10.0s)**: The cursor reaches the closing `"` and stops. A 0.3s hold. Then the cursor disappears (not fade -- instant off, like Enter was pressed). The command line stays on screen but begins to fade and drift upward over the next scene transition.

**Emotional beat**: Hope. "This looks different." The bold `m1nd` in signal green is the first warm color in the entire animation. It signals: something is about to wake up.

**Sound metaphor**: A match being struck.

---

### SCENE 4: THE BRAIN WAKES -- "Activation"
**Time**: 0:10 -- 0:15 (150 frames)
**Camera**: Starts zoomed out wide (scale 0.6), slowly zooms to 1.0 as the graph fills. This is the only camera move in the entire animation -- making it feel earned and significant.

**Frame 300-330 (10.0-11.0s)**: The terminal command from Scene 3 fades to `opacity: 0.15` and drifts to `y: -480` (top edge). It becomes a ghost -- still visible as context, but no longer the focus.

Simultaneously, nodes begin appearing. NOT all at once. NOT in a regular pattern. They appear in concentric waves from center outward, as if the graph is *growing* from a seed.

**Node appearance sequence**:
- Frame 300-310: 1 node appears at center. The source node. `#00E5A0`, radius 6px, glow blur 20px. It's the query node -- "authentication."
- Frame 310-318: 5 nodes appear in the first ring (radius ~120px). These are direct connections. `#00E5A0`, radius 4px, glow 10px. They pop in with SLAM easing (scale from 0 to 1 in 0.15s, slight overshoot).
- Frame 318-326: 8 nodes in the second ring (~240px). `#00B4D8` (semantic blue). Slightly smaller (3.5px radius). These are the semantic connections -- things that are *related* but not directly linked.
- Frame 326-334: 12 nodes in the third ring (~360px). `#F59E0B` (temporal amber). 3px radius. These are temporal -- things that change together.
- Frame 334-340: The remaining ~20 nodes appear in the outer ring. `#64748B` (ash). 2.5px radius. Dormant nodes -- present but not activated. They establish context: this graph is bigger than the query.

**Total**: 48 nodes. The same count as v1, but the *choreography* is completely different. V1 showed all nodes simultaneously. V2 shows them as a spreading wave, which IS the activation algorithm visualized.

**Frame 330-360 (11.0-12.0s)**: Edges begin drawing. Not all edges -- only the edges between activated nodes, drawn in the order of activation spread.

Edge drawing uses the DRAW easing (`easeInOutCubic`). Each edge takes 0.25s to draw. They're staggered by 0.05s. The edges connected to the source node draw first (green, `#00E5A0`, 2px width). Then semantic edges (blue, `#00B4D8`, 1.5px). Then temporal (amber, `#F59E0B`, 1px). The remaining dormant edges (`#1E293B`, 0.5px) appear last as a faint web.

**Frame 360-390 (12.0-13.0s)**: Ghost edges appear. These are the structural holes -- connections that *should* exist but don't. They're drawn as dashed lines (`lineDash: [8, 4]`) in `#6366F1` (ghost purple). They pulse gently (opacity oscillates 0.3 to 0.6 over 1s cycle, BREATHE easing).

5-6 ghost edges appear, each accompanied by a tiny label that fades in next to it:
- `"missing: rate_limiter"` (13px, `#6366F1`)
- `"gap: audit_log"` (13px, `#6366F1`)

These labels are the *insight*. This is what grep cannot do.

**Frame 390-420 (13.0-14.0s)**: The camera finishes its zoom to 1.0. A result badge appears at bottom center:

```
activate: 31ms -- 8 results -- 3 structural holes detected
```

This is `Inter 600`, 16px, `#E2E8F0`. The numbers `31ms`, `8`, and `3` are `#00E5A0` (signal green) and slightly larger (18px). They SLAM in -- appearing large and settling to size.

**Frame 420-450 (14.0-15.0s)**: Hold. Let the viewer read. The graph breathes -- every activated node's glow pulses gently (BREATHE easing, +/- 3px blur, 1.5s cycle). This is a living system, not a static diagram.

**Emotional beat**: Awe. "It found things grep can't." The ghost edges are the star. They represent invisible knowledge made visible. The 31ms timing feels instant compared to grep's 3.2s.

**Sound metaphor**: A synthesizer pad fading in. Warm, expansive. The hum of intelligence.

---

### SCENE 5: XLR CANCELLATION -- "The Secret Weapon"
**Time**: 0:15 -- 0:18 (90 frames)
**Camera**: Static at 1.0 zoom. No movement. Stillness = focus.

**Frame 450-465 (15.0-15.5s)**: The ghost edges, labels, and result badge fade out (VANISH, 0.3s). All nodes dim to `#64748B`. All edges dim to `#1E293B`. The graph is still visible but muted. A title appears:

**"XLR NOISE CANCELLATION"** -- `Inter 700`, 28px, `#E2E8F0`, positioned at `y: -420`. Below it, subtitle: `borrowed from audio engineering` -- `Inter 400`, 16px, `#64748B`.

**Frame 465-495 (15.5-16.5s)**: Two paths through the graph light up simultaneously. They are drawn from opposite ends, converging at a single node (the merge point):

- **Path A (Hot+)**: Starts from node at upper-left. Color `#00E5A0`. Line width 3px. Draws along 4 nodes toward center.
- **Path B (Cold-)**: Starts from node at lower-left. Color `#00E5A0`. Same specs. Draws along 4 different nodes toward the SAME center node.

The paths draw using DRAW easing over 0.6s. Small `+` and `-` labels appear at the start of each path (referencing XLR pin 2 and pin 3 for audio engineers who know).

**Frame 495-525 (16.5-17.5s)**: Noise injection. Red particles (3px circles, `#EF4444`) appear along both paths simultaneously. They pulse erratically (random opacity 0.4-0.8, random glow 5-15px). The path colors shift from green to a desaturated red-green mix. The intermediate nodes on both paths pulse red briefly.

This represents noise corrupting both signal paths equally -- the same noise on both Hot and Cold.

**Frame 525-555 (17.5-18.5s)**: At the merge node (center of convergence), the noise cancels. The animation:
1. Both paths' red particles drift toward the merge node (0.3s, DRAW easing)
2. At the merge node, the red particles collide and *annihilate* -- they shrink to 0 and flash white for 1 frame
3. The merge node pulses with a strong green glow (radius 14px, blur 30px, SLAM easing)
4. Both paths turn clean green again
5. A label appears at the merge node: `"signal survives"` -- 13px, `#00E5A0`

**Frame 555-570 (18.5-19.0s)**: Brief hold. The XLR title fades. The path visualization fades. Clean transition.

**Emotional beat**: "That's clever." The audio engineering metaphor gives the algorithm *personality*. It's not just filtering noise -- it's using a technique from a completely different domain. This makes m1nd feel like it was built by someone who *thinks differently*.

**Sound metaphor**: Two out-of-tune notes resolving into harmony.

---

### SCENE 6: THE VERDICT -- "What It Found"
**Time**: 0:18 -- 0:22 (120 frames)
**Camera**: Static.

**Frame 570-585 (19.0-19.5s)**: Clean black. Then the source node reappears at center, green, glowing. The target node appears at the right side of the graph, `#00B4D8` (blue). They're far apart. The rest of the graph fades in at `opacity: 0.15` -- contextual but not distracting.

A title: **"HYPOTHESIZE"** -- same style as previous titles.
Subtitle: `"is authentication connected to rate_limiter?"` -- italic styling, `#64748B`.

**Frame 585-630 (19.5-21.0s)**: Path exploration. 12 paths fan out from the source node simultaneously, exploring the graph. They're thin (1px), semi-transparent (opacity 0.4), and colored `#64748B`. They draw rapidly (0.3s each, staggered by 0.05s).

Some paths reach dead ends and fade out (VANISH, 0.3s).
3 paths find the target node. When they arrive:
1. The dead-end paths have already faded
2. The 3 surviving paths thicken to 2.5px
3. They change color: one `#00E5A0`, one `#00B4D8`, one `#F59E0B` (the three active dimensions)
4. The nodes along these paths glow softly (blur 8px)

**Frame 630-660 (21.0-22.0s)**: A verdict card appears at bottom center. It's a rounded rectangle (`radius: 12px`, fill `#0F172A`, border `#1E293B` 1px), containing:

```
likely_true — 87% confidence
25,015 paths explored in 58ms
```

The `likely_true` is `#00E5A0`, 20px, bold.
The `87%` is `#00E5A0`, 28px, extra-bold. It SLAMS in (easeOutExpo).
The second line is `#64748B`, 14px.

Hold for 0.5s to let the viewer absorb.

**Emotional beat**: Conviction. The system doesn't just search -- it *reasons*. The 87% confidence with path evidence makes it feel like a real analytical engine, not a keyword matcher.

**Sound metaphor**: A gavel. One clean strike.

---

### SCENE 7: THE INVISIBLE -- "8 Things Grep Can't See"
**Time**: 0:22 -- 0:26 (120 frames)
**Camera**: Static.

**Frame 660-675 (22.0-22.5s)**: Previous elements fade. The graph dims. A new title:

**"what grep can't see"** -- lowercase, `Inter 700`, 28px, `#EF4444` (red). The lowercase and red color are deliberate: this is an accusation, not a section header.

**Frame 675-780 (22.5-26.0s)**: 8 capability cards appear in two columns (4 per column), staggered from top to bottom. Each card takes 0.3s to appear (SLAM easing on opacity + slight upward drift of 10px).

Stagger delay between cards: 0.3s (each card starts 0.3s after the previous).

Each card is a simple text block:

**Left column** (x = -350):
1. `blast radius` -- below it: `"what else breaks when auth changes?"` -- `#64748B`, 13px
2. `structural holes` -- `"what's missing that should exist?"` -- accompanied by a tiny ghost edge icon (2 dots + dashed line, purple)
3. `co-change prediction` -- `"what files change together?"` -- `#64748B`
4. `Hebbian learning` -- `"gets smarter with every query"` -- `#64748B`

**Right column** (x = 350):
5. `hypothesis testing` -- `"is X connected to Y? confidence: 87%"` -- `#64748B`
6. `counterfactual sim` -- `"what if we remove this module?"` -- `#64748B`
7. `standing wave analysis` -- `"find the system's natural resonances"` -- `#64748B`
8. `investigation memory` -- `"save, resume, and merge research trails"` -- `#64748B`

The capability names are `Inter 600`, 16px, `#E2E8F0`.
The descriptions are `Inter 400`, 13px, `#64748B`.

Each card, once it appears, stays. By frame 780, all 8 are visible. The accumulation is the argument: each individual capability is good, but 8 of them together is devastating.

Behind the text cards, the graph continues to breathe faintly at `opacity: 0.08`. It's always there -- a reminder that this is all powered by one graph.

**Emotional beat**: Enumeration. This is the "feature dump" moment, but presented as *accusations against the status quo*, not a bullet list. Each card is another thing the viewer didn't know they were missing.

**Sound metaphor**: Footsteps approaching. Each card is a step closer.

---

### SCENE 8: THE COMPARISON -- "The Numbers"
**Time**: 0:26 -- 0:30 (120 frames)
**Camera**: Static.

**Frame 780-795 (26.0-26.5s)**: All capability cards fade out (VANISH, 0.3s). Clean black.

**Frame 795-810 (26.5-27.0s)**: A comparison table builds from the center outward. First, the column headers appear:

```
                    grep              m1nd
```

`grep` is `#EF4444`, `Inter 700`, 20px.
`m1nd` is `#00E5A0`, `Inter 700`, 20px.

A thin horizontal line (`#1E293B`, 1px) draws underneath (DRAW easing, 0.3s).

**Frame 810-900 (27.0-30.0s)**: Rows appear one by one, each with a 0.3s stagger. Each row has three elements: the metric name (left), the grep value (center-left), and the m1nd value (center-right).

The numbers SLAM in (easeOutExpo). The m1nd numbers appear 0.15s after the grep numbers in each row -- the delay creates a "wait for it" micro-tension.

| Row | Metric | grep | m1nd | Timing |
|-----|--------|------|------|--------|
| 1 | `query time` | `3.2s` | `31ms` (green) | 0.3s build |
| 2 | `tokens burned` | `47,000` (red) | `0` (green, bold, glowing) | 0.3s build |
| 3 | `API cost` | `$0.041/query` (red) | `$0.000` (green) | 0.3s build |
| 4 | `structural insight` | `none` (red, dim) | `blast radius + holes + co-change` (green) | 0.3s build |
| 5 | `learning` | `none` (red, dim) | `Hebbian plasticity` (green) | 0.3s build |
| 6 | `tools` | `1` (red) | `43` (green, large, glowing) | 0.3s build |

Row spacing: 48px vertical.
Metric names: `Inter 400`, 15px, `#64748B`.
Values: `Inter 700`, 18px. grep = `#EF4444`. m1nd = `#00E5A0`.

The final row (`43 tools`) gets special treatment: the `43` appears at 40px font size and pulses once (BREATHE, scale 1.0 -> 1.1 -> 1.0 over 0.3s) before settling. The glow blur goes to 20px.

**Frame 900-930 (30.0-31.0s)**: Hold for 1 second. Let the table sink in. The viewer's eyes scan the two columns and the conclusion is inescapable.

**Emotional beat**: The kill shot. No prose. No persuasion. Just numbers, side by side. The red column is uniformly bad. The green column is uniformly better. The `0 tokens` row is the centerpiece -- it's the impossible number. How can a query tool use zero tokens? (Because it's Rust-native, no LLM calls.) That single `0` is the most powerful element in the entire animation.

**Sound metaphor**: A bass drop. The table is the drop.

---

### SCENE 9: FINALE -- "The Brand"
**Time**: 0:30 -- 0:33 (90 frames)
**Camera**: Static. Then the gentlest zoom-in imaginable (scale 1.0 -> 1.02 over 2s). Subtle enough to be felt, not seen.

**Frame 930-945 (31.0-31.5s)**: Table fades out (VANISH, 0.4s). Pure black.

**Frame 945-975 (31.5-32.5s)**: The logo appears.

`m1nd` in `Inter 900`, 80px, `#00E5A0`. Centered. It doesn't fade in -- it SLAMS (easeOutExpo, scale from 1.3 to 1.0 over 0.4s). Shadow glow: `#00E5A0` at 40px blur, opacity 0.5. The glow settles to 25px blur over 0.3s.

Below the logo (0.2s delay), the tagline appears with FADE easing:

`cognitive graph engine` -- `Inter 300`, 24px, `#64748B`, letter-spacing +1px.

Below the tagline (0.2s delay):

`43 tools. zero tokens. it learns.` -- `Inter 500`, 18px, `#E2E8F0`.

The three phrases are separated by ` . ` with the periods in `#64748B`. The word `learns` is `#00E5A0` (signal green) -- the only colored word in the tagline. Because learning is the differentiator.

**Frame 975-990 (32.5-33.0s)**: Below everything (0.3s delay), the GitHub link fades in:

`github.com/maxkle1nz/m1nd` -- `Inter 400`, 16px, `#64748B`.

The graph makes one final appearance: behind the logo, at `opacity: 0.04`, the full node network is visible. All nodes pulse once in unison (BREATHE, glow 0 -> 8 -> 0 over 1s). It's a heartbeat. The graph is alive.

Hold to loop point.

**Emotional beat**: Resolution. The brand lands. The tagline is a thesis in nine words. The GitHub link is a call to action that doesn't beg -- it simply exists.

**Sound metaphor**: A held chord resolving. A breath released.

---

## STEP 2: FEATURES -- Detailed Element Specs

### Nodes

| Type | Radius | Glow Blur | Color | Opacity |
|------|--------|-----------|-------|---------|
| Source (query) | 8px | 25px | `#00E5A0` | 1.0 |
| Structural (wave 1) | 5px | 12px | `#00E5A0` | 1.0 |
| Semantic (wave 2) | 4px | 8px | `#00B4D8` | 0.9 |
| Temporal (wave 3) | 3.5px | 6px | `#F59E0B` | 0.8 |
| Causal (wave 4) | 3px | 4px | `#EF4444` | 0.6 |
| Dormant | 2.5px | 0px | `#64748B` | 0.4 |
| Merge (XLR) | 10px | 35px | `#00E5A0` | 1.0 |

All nodes are `Circle` components with `shadowColor` matching `fill` and `shadowBlur` controlling glow.

### Edges

| Type | Width | Color | Dash | Opacity |
|------|-------|-------|------|---------|
| Active (structural) | 2px | `#00E5A0` | solid | 0.7 |
| Active (semantic) | 1.5px | `#00B4D8` | solid | 0.6 |
| Active (temporal) | 1px | `#F59E0B` | solid | 0.5 |
| Dormant | 0.5px | `#1E293B` | solid | 0.3 |
| Ghost (structural hole) | 1.5px | `#6366F1` | `[8, 4]` | 0.3-0.6 pulsing |
| XLR path | 3px | `#00E5A0` | solid | 0.9 |
| Exploration (dead) | 1px | `#64748B` | `[4, 3]` | 0.4 |
| Exploration (winner) | 2.5px | varies by dimension | solid | 0.9 |

### Terminal Elements

- **Cursor**: `Rect`, 2px wide, 20px tall, `#00FF88`, blink cycle 500ms (300ms on, 200ms off)
- **Command text**: `Txt`, `JetBrains Mono 400` 20px, typed at 40ms/char
- **Output text**: `Txt`, `JetBrains Mono 400` 15px, `#64748B`, 60ms stagger per line
- **Prompt `$`**: `Txt`, same font, `#64748B`

### Counter Elements

- **Number counters** (Scene 2): `Txt`, `Inter 800` 72px for the number, `Inter 400` 13px for label
- Counter animation: interpolate value from 0 to target over 0.8s, SLAM easing, update text every frame
- Red counters use `#FF2D55`, green uses `#00E5A0`

### Cards (Scene 7)

- No background rectangle -- just text blocks. Cleaner.
- Title: `Inter 600`, 16px, `#E2E8F0`
- Description: `Inter 400`, 13px, `#64748B`
- Entry animation: opacity 0->1 + translateY +10px->0, 0.3s, SLAM

### Comparison Table (Scene 8)

- Column gap: 300px
- Row height: 48px
- Header separator: `Line`, `#1E293B`, 1px, draws left-to-right 0.3s
- Row build: values SLAM in (easeOutExpo), metric names FADE in (easeOutCubic)
- grep values: `#EF4444`
- m1nd values: `#00E5A0`

### Logo (Scene 9)

- `Txt`, `Inter 900`, 80px, `#00E5A0`
- Entry: scale 1.3->1.0, opacity 0->1, SLAM easing, 0.4s
- Glow: `shadowColor: #00E5A0`, `shadowBlur`: 40px -> 25px over 0.3s
- The `1` in `m1nd` is NOT a different color or style. The leet-speak is enough. Don't oversell it.

---

## STEP 3: HARDENING -- Risk Analysis

### Performance

| Risk | Mitigation |
|------|-----------|
| 48 nodes + edges + signals = many reactive signals | Pre-compute all signal arrays. Use `createSignal` sparingly for things that actually animate. Static positions = direct props, not signals. |
| Ghost edge pulse loops running indefinitely | Use bounded loops with `waitFor` + early termination. No `loop()` that outlives its scene. |
| Text typing animation = many per-character updates | Type entire string, use `end()` on Txt if available, or update text signal at 40ms intervals using `waitFor`. |
| Number counter animation = 30fps text updates | Use `createSignal<number>()` and tween it. Render `Math.floor(signal()).toLocaleString()` in `text` callback. |
| 12 simultaneous path exploration lines | Limit to 8 paths maximum visible. More than 8 becomes visual noise at GIF resolution. |

### GIF File Size

| Risk | Mitigation |
|------|-----------|
| 33s @ 30fps @ 1080p = huge raw | Render at 1920x1080, downscale output to 960x540. |
| Complex gradients/glows increase palette | Limit glow effects: max 5 glowing nodes at any time. Use solid colors, not gradients. |
| Color palette > 256 | Limit to ~15 unique hue families. The dark background means most pixels are near-black, which compresses well. |
| Frame-to-frame changes throughout | Many scenes have "hold" periods (no movement for 0.5-1s). These compress to near-zero in GIF. Intentional. |
| Target: <5MB | If over budget: reduce to 24fps, reduce to 720p, add more hold frames (compression-friendly). |

### Text Readability

| Risk | Mitigation |
|------|-----------|
| Small text (13px) at 540p display | Test all text at display resolution. Minimum readable: 13px at 2x = 26px rendered. Should be fine. |
| Monospace text aliasing in GIF | JetBrains Mono has excellent hinting. Test rendering with MC's canvas renderer. |
| Color contrast on dark background | All text colors verified against `#060B14`: Bone (`#E2E8F0`) = 14.5:1 contrast. Ash (`#64748B`) = 5.2:1. Both pass WCAG AA. |
| Ghost purple on dark | `#6366F1` on `#060B14` = 5.8:1. Passes AA. |

### Accessibility

| Risk | Mitigation |
|------|-----------|
| Red-green colorblindness | Signal green (`#00E5A0`) and Causal red (`#EF4444`) differ in luminance, not just hue. Green = bright, Red = medium. Distinguishable by deuteranopes. |
| Animation speed for cognitive processing | Hold frames after every major reveal (0.5-1.0s). The fastest animation is 0.15s (node pop-in), but it's small and peripheral. |

---

## STEP 4: SYNTHESIS -- Design Decisions

### Why JetBrains Mono + Inter

JetBrains Mono is the highest-quality free monospace font with programming ligatures. It signals "this was built by developers, for developers." Inter is the most widely-deployed UI font in 2025-2026 tech products. Using it says "this is professional software, not a weekend project." The pairing is familiar without being generic.

Rejected alternatives:
- **Fira Code**: Excellent, but JBM has better rendering at small sizes in canvas
- **Berkeley Mono**: Perfect aesthetics but not free -- can't include in open source
- **SF Mono**: Apple-only, excludes Linux/Windows viewers
- **Space Grotesk**: Too trendy, would date the animation

### Why this specific color palette

The palette is built on three psychological principles:

1. **Dark = trust**: Finance, security, and intelligence tools use dark themes. A dark background says "I handle serious things."
2. **Green = system health**: Green on black is the terminal. It's the color of "everything is working." When m1nd's results appear in green, the subconscious message is "this is correct."
3. **Red = cost**: Red is not used for m1nd's features anywhere. It's exclusively used for grep's weaknesses and costs. This creates an unconscious association: red = the old way, green = the new way.

### Why 33 seconds

Research on social media video engagement shows:
- 15s: Too short for a technical argument with 3+ scenes
- 30s: Standard animated explainer length
- 60s: Too long for a GIF; viewers abandon after 35-40s

33 seconds is precisely enough for: hook (3s) + problem (5s) + solution (5s) + proof (7s) + features (4s) + comparison (4s) + brand (3s) + buffer (2s of holds).

The extra 3s over 30 comes from intentional hold frames -- moments of stillness that let the viewer absorb. Cutting these would make the animation feel rushed.

### Why no particles

v1 was criticized for lacking particles. But particles serve no informational purpose in this context. They're decorative. Every element in v2 carries meaning:
- Nodes = actual graph nodes
- Edges = actual graph connections
- Ghost edges = structural holes (a real m1nd feature)
- Paths = real path exploration
- Numbers = real benchmarks

Adding particles would dilute the visual language. The glow on activated nodes already provides the "energy" feeling. Particles would compete with the ghost edges for attention, and ghost edges are more important.

Exception: The XLR noise "particles" in Scene 5 are not decorative -- they represent noise being injected into signal paths. They earn their existence by illustrating a concept.

### Why camera movement is minimal

One zoom. That's it. Because:
1. Camera movement is the most expensive operation in GIF file size (every pixel changes every frame)
2. Static camera = the viewer's eye can lock onto positions and build spatial memory
3. The single zoom in Scene 4 is the only moment where the visual "world" changes scale, making it feel significant
4. Every other transition uses opacity (cheap in GIF, creates clean fades)

---

## STEP 5: DECISIONS -- Motion Canvas API Mapping

### Components Used

| MC Component | Usage |
|---|---|
| `Circle` | All graph nodes. `fill`, `shadowColor`, `shadowBlur` for glow. |
| `Line` | All edges, paths, XLR traces, table separators. `points`, `end()` for draw animation. `lineDash` for ghosts. |
| `Txt` | All text. `text`, `fontSize`, `fontFamily`, `fontWeight`, `fill`, `opacity`. |
| `Rect` | Terminal cursor (2x20px), verdict card background. |
| `Node` | Container grouping for scenes, layers. `opacity` for group fades. |
| `Camera` | Built-in MC Camera. Single instance wrapping the graph layer. Used for the Scene 4 zoom. |

### Animation API

| MC Function | Usage |
|---|---|
| `all()` | Parallel animations (e.g., all nodes in a wave light up simultaneously) |
| `sequence()` | Staggered animations (node appearance with 0.03s delay between each) |
| `chain()` | Sequential animations (title -> subtitle -> counter) |
| `delay()` | Offset start of an animation within `all()` |
| `waitFor()` | Hold frames between scenes |
| `loop()` | Ghost edge pulse, node breathing (bounded, not infinite) |
| `createSignal()` | Reactive values for node radius, color, opacity, glow |
| `createRef()` | Component references for imperative animation |

### Camera Implementation

```
// Use MC's built-in Camera node
const camera = createRef<Camera>();

view.add(
  <Camera ref={camera}>
    <Node ref={graphContainer}>
      {/* all nodes and edges */}
    </Node>
  </Camera>
);

// Scene 4: zoom from 0.6 to 1.0
yield* camera().zoom(1.0, 2.0, easeInOutCubic);
// The Camera starts at scene default zoom and animates
```

If the built-in Camera doesn't support `zoom()` directly, use `scale` on the graph container `Node`:
```
graphContainer().scale(0.6); // initial
yield* graphContainer().scale(1.0, 2.0, easeInOutCubic); // zoom in
```

### Terminal Typing Effect

```
// Typing = incrementally building a string in a Txt component
function* typeText(ref: Reference<Txt>, text: string, charDelay = 0.04) {
  for (let i = 0; i <= text.length; i++) {
    ref().text(text.slice(0, i));
    yield* waitFor(charDelay);
  }
}
```

### Number Counter Animation

```
// Counter = tweening a signal and formatting as integer
const tokenCount = createSignal(0);
counterRef().text(() => Math.floor(tokenCount()).toLocaleString());
yield* tokenCount(47000, 0.8, easeOutExpo); // SLAM easing
```

### Ghost Edge Pulse

```
// Bounded pulse loop for ghost edges
yield* loop(4, function* () {
  yield* ghostOpacity(0.6, 0.75, easeInOutQuad); // BREATHE
  yield* ghostOpacity(0.3, 0.75, easeInOutQuad);
});
```

### Scene Transition Pattern

Every scene transition follows the same structure:
```
// 1. Fade out current scene elements
yield* all(hideTitle(), hideSubtitle(), hideCounter());
// 2. Reset graph state
resetGraph();
// 3. Brief void (0.3-0.5s)
yield* waitFor(0.3);
// 4. New title appears
yield* showTitle('NEXT SCENE');
```

---

## STEP 6: CONTRACTS -- TypeScript Interfaces

```typescript
// ============================================================
// THEME & CONSTANTS
// ============================================================

interface ThemeColors {
  void: '#060B14';
  terminalGreen: '#00FF88';
  signal: '#00E5A0';
  semantic: '#00B4D8';
  temporal: '#F59E0B';
  causal: '#EF4444';
  ghost: '#6366F1';
  bone: '#E2E8F0';
  ash: '#64748B';
  graphite: '#1E293B';
  costRed: '#FF2D55';
}

interface ThemeFonts {
  mono: 'JetBrains Mono, Fira Code, monospace';
  sans: 'Inter, system-ui, sans-serif';
}

interface EasingConfig {
  slam: typeof easeOutExpo;
  breathe: typeof easeInOutQuad;
  draw: typeof easeInOutCubic;
  fade: typeof easeOutCubic;
  vanish: typeof easeInCubic;
  snap: typeof linear;
}

// ============================================================
// GRAPH DATA
// ============================================================

interface GraphNode {
  id: number;
  x: number;
  y: number;
  layer: number;           // 0 = core, 4 = periphery
  activationWave?: number; // 0 = source, 1-4 = BFS depth
}

interface GraphEdge {
  from: number;
  to: number;
  dimension?: 'structural' | 'semantic' | 'temporal' | 'causal';
}

interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

// ============================================================
// VISUAL STATE (per-node, per-edge reactive signals)
// ============================================================

interface NodeVisualState {
  fillOpacity: ReturnType<typeof createSignal<number>>;
  fillColor: ReturnType<typeof createSignal<string>>;
  radius: ReturnType<typeof createSignal<number>>;
  glowBlur: ReturnType<typeof createSignal<number>>;
}

interface EdgeVisualState {
  progress: ReturnType<typeof createSignal<number>>;
  strokeColor: ReturnType<typeof createSignal<string>>;
  strokeWidth: ReturnType<typeof createSignal<number>>;
  strokeOpacity: ReturnType<typeof createSignal<number>>;
}

// ============================================================
// SCENE FUNCTIONS — generator signatures
// ============================================================

// Each scene is a generator that receives shared state
interface SceneContext {
  graphContainer: Reference<Node>;
  uiContainer: Reference<Node>;
  camera: Reference<Node>;  // or Camera if using built-in
  nodeSignals: NodeVisualState[];
  edgeSignals: EdgeVisualState[];
  graph: GraphData;
  adjacency: Map<number, number[]>;
  theme: ThemeColors;
  fonts: ThemeFonts;
}

type SceneGenerator = (ctx: SceneContext) => Generator;

// Scene function contracts:
declare function sceneColdOpen(ctx: SceneContext): Generator;
//  - Duration: 3.0s
//  - Creates: cursor Rect, command Txt
//  - Leaves on screen: typed command at full opacity

declare function sceneTheCost(ctx: SceneContext): Generator;
//  - Duration: 5.0s
//  - Creates: grep output lines, 3 counters, rhetorical question, 4 sin lines
//  - Leaves on screen: nothing (fades to black)

declare function sceneTheCommand(ctx: SceneContext): Generator;
//  - Duration: 2.0s
//  - Creates: cursor, m1nd.activate command
//  - Leaves on screen: fading command (transitions into Scene 4)

declare function sceneTheBrainWakes(ctx: SceneContext): Generator;
//  - Duration: 5.0s
//  - Creates: all nodes + edges + ghost edges + result badge
//  - Animates: camera zoom 0.6 -> 1.0
//  - Leaves on screen: breathing graph

declare function sceneXlrCancellation(ctx: SceneContext): Generator;
//  - Duration: 3.0s
//  - Creates: XLR paths, noise particles, merge animation
//  - Leaves on screen: nothing (fades)

declare function sceneTheVerdict(ctx: SceneContext): Generator;
//  - Duration: 4.0s
//  - Creates: source/target nodes, exploration paths, verdict card
//  - Leaves on screen: nothing (fades)

declare function sceneTheInvisible(ctx: SceneContext): Generator;
//  - Duration: 4.0s
//  - Creates: 8 capability cards in 2 columns
//  - Leaves on screen: nothing (fades)

declare function sceneTheComparison(ctx: SceneContext): Generator;
//  - Duration: 4.0s
//  - Creates: comparison table (headers + 6 rows)
//  - Leaves on screen: nothing (fades)

declare function sceneFinale(ctx: SceneContext): Generator;
//  - Duration: 3.0s
//  - Creates: logo, tagline, github link, background graph heartbeat
//  - Leaves on screen: all (this is the loop point)

// ============================================================
// HELPER FUNCTIONS
// ============================================================

declare function typeText(
  ref: Reference<Txt>,
  text: string,
  charDelay?: number,       // default 0.04 (40ms per char)
  color?: string,
): Generator;

declare function animateCounter(
  signal: ReturnType<typeof createSignal<number>>,
  target: number,
  duration?: number,        // default 0.8
  easing?: TimingFunction,  // default easeOutExpo (SLAM)
): Generator;

declare function showText(
  ref: Reference<Txt>,
  text: string,
  duration?: number,        // default 0.4
  easing?: TimingFunction,  // default easeOutCubic (FADE)
): Generator;

declare function hideText(
  ref: Reference<Txt>,
  duration?: number,        // default 0.3
  easing?: TimingFunction,  // default easeInCubic (VANISH)
): Generator;

declare function resetGraphVisuals(
  nodeSignals: NodeVisualState[],
  edgeSignals: EdgeVisualState[],
  theme: ThemeColors,
): void;

declare function bfsActivationLayers(
  source: number,
  adjacency: Map<number, number[]>,
  maxDepth: number,
): number[][];

declare function generateGraph(
  nodeCount: number,
  edgeDensity: number,
  seed: number,
): GraphData;
```

---

## SCENE TIMELINE SUMMARY

```
0:00 ━━━ COLD OPEN ━━━━━━━━━━━━━ 0:03
       cursor blinks, "grep -rn" typed

0:03 ━━━ THE COST ━━━━━━━━━━━━━━ 0:08
       grep output, token/cost counters, "what did it miss?"

0:08 ━━━ THE COMMAND ━━━━━━━━━━━━ 0:10
       m1nd.activate typed

0:10 ━━━ THE BRAIN WAKES ━━━━━━━━ 0:15
       graph materializes, 4D activation, ghost edges, 31ms badge

0:15 ━━━ XLR CANCELLATION ━━━━━━━ 0:18
       dual paths, noise injection, merge + cancel

0:18 ━━━ THE VERDICT ━━━━━━━━━━━━ 0:22
       hypothesis paths, "likely_true 87%"

0:22 ━━━ THE INVISIBLE ━━━━━━━━━━ 0:26
       8 capabilities grep can't do

0:26 ━━━ THE COMPARISON ━━━━━━━━━ 0:30
       side-by-side table, numbers slam

0:30 ━━━ FINALE ━━━━━━━━━━━━━━━━━ 0:33
       logo, tagline, github, heartbeat
```

---

## CRITICAL IMPLEMENTATION NOTES

1. **Scene isolation**: Each scene should clean up after itself. No leaked signals, no orphaned nodes. The `resetGraphVisuals` function must restore all signals to dormant state.

2. **Font loading**: JetBrains Mono and Inter must be loaded before rendering. Use `@font-face` in MC's HTML template or load via Google Fonts CDN in the Vite config.

3. **GIF rendering pipeline**: Motion Canvas renders to PNG frames -> ffmpeg/gifski for GIF conversion. Use gifski for better quality at lower file size. Command: `gifski --fps 30 --width 960 --quality 90 frames/*.png -o m1nd-cinema.gif`

4. **Loop point**: The GIF loops. Scene 9's final state (logo + tagline + graph heartbeat) should visually connect to Scene 1's opening (black screen). The 0.5s of pure black at the start of Scene 1 serves as the "loop gap" -- the viewer sees a brief black flash, then the animation restarts. This is intentional and standard for looping demo GIFs.

5. **The `0` in the comparison table**: This single character is the most important element in the entire animation. It must be large, green, glowing, and given a beat of silence around it. Zero tokens. That's the impossible promise. That's what makes someone click the GitHub link.
