# Motion Canvas API Reference — m1nd-viz

Verified against installed packages in `node_modules/@motion-canvas/`. Version in use: check `package.json`.

---

## 1. IMPORTS

```typescript
// Components
import {
  makeScene2D,
  Camera, Circle, Line, Rect, Node, Txt, TxtLeaf,
  Layout, Shape, Curve, Spline, Path, Code,
  Bezier, CubicBezier, QuadBezier, Polygon, Ray, Grid, Img, Video,
} from '@motion-canvas/2d';

// Core utilities
import {
  // Refs
  createRef, createRefArray, makeRef,
  // Signals
  createSignal,
  // Flow
  all, any, chain, delay, sequence, loop, loopFor, loopUntil,
  waitFor, waitUntil, run, every, spawn,
  // Tweening
  tween,
  spring, makeSpring,
  BeatSpring, PlopSpring, BounceSpring, SwingSpring, JumpSpring, StrikeSpring, SmoothSpring,
  // Easing
  linear, cos, sin,
  easeInSine, easeOutSine, easeInOutSine,
  easeInQuad, easeOutQuad, easeInOutQuad,
  easeInCubic, easeOutCubic, easeInOutCubic,
  easeInQuart, easeOutQuart, easeInOutQuart,
  easeInQuint, easeOutQuint, easeInOutQuint,
  easeInExpo, easeOutExpo, easeInOutExpo,
  easeInCirc, easeOutCirc, easeInOutCirc,
  easeInBack, easeOutBack, easeInOutBack,
  easeInBounce, easeOutBounce, easeInOutBounce,
  easeInElastic, easeOutElastic, easeInOutElastic,
  createEaseInBack, createEaseOutBack, createEaseInOutBack,
  createEaseInElastic, createEaseOutElastic, createEaseInOutElastic,
  createEaseInBounce, createEaseOutBounce, createEaseInOutBounce,
  // Interpolation
  textLerp, deepLerp, map, remap, clamp, clampRemap,
  // Types
  Vector2, Color,
} from '@motion-canvas/core';
```

---

## 2. COMPONENTS — Key Props

All components inherit down the chain: **Node → Layout → Shape → Curve → Circle/Rect/Line/Spline**

### Node (base for everything)
```typescript
interface NodeProps {
  ref?: Reference<any>;
  x?: number;               // position x
  y?: number;               // position y
  position?: [number, number] | Vector2;
  rotation?: number;        // degrees
  scaleX?: number;
  scaleY?: number;
  scale?: number | [number, number];
  skewX?: number;
  skewY?: number;
  zIndex?: number;
  opacity?: number;         // 0..1
  filters?: Filter[];
  shadowColor?: string;
  shadowBlur?: number;
  shadowOffsetX?: number;
  shadowOffsetY?: number;
  cache?: boolean;
  cachePadding?: number | [number, number] | [number, number, number, number];
  composite?: boolean;
  compositeOperation?: GlobalCompositeOperation;  // e.g. 'lighter', 'screen'
  shaders?: PossibleShaderConfig;  // experimental GLSL shaders
}
```

Node methods (all non-animated, instant):
- `node.add(child)` — append child
- `node.insert(child, index)` — insert at index
- `node.remove()` — remove from parent
- `node.move(by)`, `moveUp()`, `moveDown()`, `moveToTop()`, `moveToBottom()`, `moveTo(index)`
- `node.reparent(newParent)` — change parent, keep world position
- `node.removeChildren()`
- `node.save()` / `node.restore()` / `yield* node.restore(duration)` — state stack
- `node.clone(overrides?)` / `node.snapshotClone()` / `node.reactiveClone()`
- `node.findAll(predicate)` / `node.findFirst(predicate)` / `node.findLast(predicate)`
- `node.localToWorld()` / `node.worldToLocal()` — DOMMatrix transforms

### Layout (extends Node) — adds Flexbox + sizing
```typescript
// Additional props beyond Node:
width?: number | string;   // '50%' or pixels
height?: number | string;
size?: [width, height];
layout?: boolean | null;   // enable flexbox layout
direction?: 'row' | 'column';
justifyContent?: FlexContent;
alignItems?: FlexItems;
gap?: number | [number, number];
padding?: number | [t, r, b, l];
margin?: number | [t, r, b, l];
fontFamily?: string;
fontSize?: number;
fontWeight?: number;
fontStyle?: string;
lineHeight?: number | string;
letterSpacing?: number;
textWrap?: boolean | 'wrap' | 'pre';
textAlign?: CanvasTextAlign;
offset?: [number, number];   // pivot offset, -1..1 ([-1,-1] = top-left)
clip?: boolean;
// Position shortcuts (set node so that edge lands at given position):
middle?: [number, number];
top?, bottom?, left?, right?;
topLeft?, topRight?, bottomLeft?, bottomRight?;
```

### Shape (extends Layout) — adds fill, stroke
```typescript
fill?: string | null;        // '#00E5A0', 'rgba(0,229,160,0.5)', null
stroke?: string | null;
strokeFirst?: boolean;       // draw stroke before fill
lineWidth?: number;
lineJoin?: 'miter' | 'bevel' | 'round';
lineCap?: 'butt' | 'round' | 'square';
lineDash?: number[];         // e.g. [6, 4] for dashes
lineDashOffset?: number;
antialiased?: boolean;
```

Shape method:
- `yield* shape.ripple(duration?)` — built-in ripple pulse effect

### Curve (extends Shape) — adds path trimming + arrows
```typescript
// Animate drawing a curve on/off screen:
start?: number;      // 0..1 clip from start (default 0)
end?: number;        // 0..1 clip from end (default 1)
startOffset?: number;  // fixed pixel offset from start
endOffset?: number;
startArrow?: boolean;
endArrow?: boolean;
arrowSize?: number;
closed?: boolean;    // connect start to end
```

KEY TRICK: animate `end` from 0 to 1 to draw the line on screen:
```typescript
<Line ref={lineRef} points={...} stroke="#fff" lineWidth={2} end={0} />
yield* lineRef().end(1, 0.5, easeInOutCubic);
```

### Circle
```typescript
// Extends Curve. Use width+height for ellipse, or size for circle.
// size={160} = diameter 160px circle
// startAngle/endAngle for arcs and sectors:
startAngle?: number;   // degrees, default 0
endAngle?: number;     // degrees, default 360
counterclockwise?: boolean;
closed?: boolean;      // true = pie/sector, false = arc
```

### Rect
```typescript
radius?: number | [number, number] | [number, number, number, number];  // corner rounding
smoothCorners?: boolean;   // Figma-style squircle corners
cornerSharpness?: number;  // 0..1, default 0.6
// Also inherits all Shape + Curve props (start/end for animated drawing)
```

### Line (polyline / polygon)
```typescript
points?: [number, number][];  // or SignalValue array for reactive points
radius?: number;              // corner rounding at vertices
// All Curve props: start, end, startArrow, endArrow, closed, lineDash
```

Tween points:
```typescript
yield* line().points([[0,0],[100,50],[200,0]], 1, easeInOutCubic);
```

### Spline (smooth bezier through points)
```typescript
points?: [number, number][];
smoothness?: number;   // 0..1, default 0.4
// Can also use <Knot> children for explicit handles
```

### Txt
```typescript
text?: string;
// All Layout props: fontFamily, fontSize, fontWeight, fontStyle, fill
// Children can be string or mixed Txt/TxtLeaf nodes
```

Animate text change:
```typescript
yield* txt().text('new text', 0.5, linear, textLerp);
```

Static helpers:
- `Txt.b(props)` — bold text
- `Txt.i(props)` — italic text

### Code (syntax-highlighted code block)
```typescript
code?: string;          // the code string
highlighter?: CodeHighlighter | null;
selection?: CodeRange | CodeRange[];  // highlight ranges
```

Animate code change (morphing diff):
```typescript
yield* code().code('new code string', 1);
yield* code().code.append('\nnew line', 1);
```

### Camera
```typescript
zoom?: number;   // default 1
scene?: Node;    // the scene node to render
```

Camera methods:
```typescript
yield* camera().zoom(2, 1, easeInOutCubic);          // zoom in
yield* camera().rotation(45, 1);                      // rotate
yield* camera().centerOn([100, 50], 1);               // pan to position
yield* camera().centerOn(someNodeRef(), 1);           // pan to node
yield* camera().reset(1);                             // back to default
yield* camera().followCurve(splineRef(), 2);          // dolly along path
yield* camera().followCurveWithRotation(spline, 2);   // orient to path
```

Camera setup pattern:
```typescript
const cam = createRef<Camera>();
view.add(
  <Camera ref={cam}>
    <Node ref={sceneRoot} /> {/* put your whole scene here */}
  </Camera>
);
```

### Img
```typescript
src?: string;    // URL or imported asset
```

### Layout (flexbox container)
Use `layout={true}` on any Layout/Shape/Rect to enable flex children:
```typescript
<Rect layout direction="row" gap={20} padding={[10, 20]}>
  <Txt>A</Txt>
  <Txt>B</Txt>
</Rect>
```

---

## 3. ANIMATION METHODS

Every animatable signal on a node can be called as:
```typescript
// Instant set:
node.fill('#ff0000');
// Tween: signal(targetValue, duration, timingFn?, interpolationFn?)
yield* node.fill('#ff0000', 1, easeOutCubic);
// Chain: .to() continues from where last tween ends
yield* node.fill('#ff0000', 1).to('#00ff00', 1);
// Reverse: .back() reverses the previous tween
yield* node.fill('#ff0000', 1).back(1);
// Wait: .wait() adds pause in chain
yield* node.fill('#ff0000', 1).wait(0.5).to('#00ff00', 1);
```

### tween() — raw value tween
```typescript
yield* tween(1.5, (value, time) => {
  // value is 0..1 (progress), time is elapsed seconds
  node.x(easeOutExpo(value) * 500);
});
```

### spring() — physics-based animation
```typescript
import { spring, BeatSpring, SmoothSpring } from '@motion-canvas/core';

yield* spring(
  BeatSpring,   // Spring config: { mass, stiffness, damping, initialVelocity? }
  0,            // from value
  100,          // to value
  0.001,        // settleTolerance (optional)
  (value, time) => {
    node.x(value);
  },
);
```

Available spring presets:
- `BeatSpring` — snappy beat
- `PlopSpring` — overshoots then settles
- `BounceSpring` — multiple bounces
- `SwingSpring` — pendulum
- `JumpSpring` — quick pop
- `StrikeSpring` — sharp impact
- `SmoothSpring` — gentle settle

Custom spring:
```typescript
const mySpring = makeSpring(mass, stiffness, damping, initialVelocity?);
```

---

## 4. FLOW CONTROL

### all() — concurrent, waits for longest
```typescript
yield* all(
  circle().scale(2, 1),
  circle().fill('#ff0000', 1),
  txt().opacity(1, 0.5),
);
```

### chain() — sequential, waits for each before next
```typescript
yield* chain(
  circle().scale(2, 0.5),
  txt().opacity(1, 0.4),
  circle().fill('#ff0000', 0.3),
);
```

### sequence() — staggered start, overlapping
```typescript
// Start each 0.05s after previous, don't wait for each to finish
yield* sequence(
  0.05,
  ...nodes.map(n => n.opacity(1, 0.3)),
);
```

### delay() — offset start within all()
```typescript
yield* all(
  rect().opacity(1, 2),
  delay(0.5, txt().opacity(1, 1)),  // starts 0.5s into the all()
);
```

### loop() — infinite loop (use with spawn)
```typescript
// Infinite loop must be spawned, not yielded*
spawn(loop(() => circle().scale(1.1, 0.5).to(1.0, 0.5)));
// Loop N times:
yield* loop(3, i => circle().fill(colors[i], 0.5));
```

### loopFor() — loop for duration
```typescript
yield* loopFor(3, () => node.x(-10, 0.1).to(10, 0.1));
```

### waitFor() — pause execution
```typescript
yield* waitFor(0.5);   // wait 500ms
```

### waitUntil() — wait for named event (timeline marker)
```typescript
yield* waitUntil('myEvent');
```

### spawn() — fire-and-forget parallel task
```typescript
spawn(function* () {
  yield* loop(() => particleRef().scale(1.2, 0.3).to(1.0, 0.3));
});
```

### run() — wrap generator for passing to flow functions
```typescript
yield* all(
  run(function* () {
    yield* waitFor(0.5);
    yield* txt().opacity(1, 0.3);
  }),
  circle().scale(2, 1),
);
```

---

## 5. CAMERA ZOOM INTO NODE CLUSTER

```typescript
const cam = createRef<Camera>();
const cluster = createRef<Node>();

view.add(
  <Camera ref={cam}>
    <Node ref={cluster}>
      {/* nodes here */}
    </Node>
  </Camera>
);

// Zoom into cluster
yield* all(
  cam().zoom(3, 1.5, easeInOutCubic),
  cam().centerOn(cluster(), 1.5, easeInOutCubic),
);

// Zoom back out
yield* cam().reset(1, easeInOutCubic);
```

---

## 6. TYPING TEXT (terminal style)

Motion Canvas has `textLerp` interpolation but no built-in typing effect.
The most reliable approach uses a signal + tween:

```typescript
const txt = createRef<Txt>();
view.add(<Txt ref={txt} text="" fill="#00E5A0" fontFamily="monospace" />);

const fullText = '> activating spreading attention...';

// Approach 1: tween with textLerp (morphs characters, not pure typing)
yield* txt().text(fullText, fullText.length * 0.05, linear, textLerp);

// Approach 2: manual character-by-character (pure typing effect)
yield* tween(fullText.length * 0.04, value => {
  const chars = Math.floor(value * fullText.length);
  txt().text(fullText.slice(0, chars));
});

// Approach 3: signal-driven (cleanest)
const progress = createSignal(0);
txt().text(() => fullText.slice(0, Math.floor(progress() * fullText.length)));
yield* progress(1, fullText.length * 0.04, linear);
```

---

## 7. NODE PULSING WITH GLOW

```typescript
const node = createRef<Circle>();
view.add(
  <Circle
    ref={node}
    size={20}
    fill="#00E5A0"
    shadowColor="#00E5A0"
    shadowBlur={0}
  />
);

// Single pulse burst
yield* all(
  node().shadowBlur(40, 0.15, easeOutExpo),
  node().scale(1.4, 0.15, easeOutExpo),
);
yield* all(
  node().shadowBlur(10, 0.4, easeInOutCubic),
  node().scale(1.0, 0.4, easeInOutCubic),
);

// Continuous breathing glow (use spawn for background)
spawn(loop(() =>
  chain(
    all(
      node().shadowBlur(30, 0.8, easeInOutSine),
      node().scale(1.1, 0.8, easeInOutSine),
    ),
    all(
      node().shadowBlur(8, 0.8, easeInOutSine),
      node().scale(1.0, 0.8, easeInOutSine),
    ),
  )
));

// Signal-driven glow (for reactive control)
const glowStrength = createSignal(0);
view.add(
  <Circle
    size={20}
    fill="#00E5A0"
    shadowColor="#00E5A0"
    shadowBlur={() => glowStrength()}
  />
);
yield* glowStrength(40, 0.3, easeOutExpo);
```

---

## 8. EDGE DRAWING WITH TRAIL

```typescript
// Basic edge draw-on (start=0, animate end 0→1)
const edge = createRef<Line>();
view.add(
  <Line
    ref={edge}
    points={[[0, 0], [300, 100]]}
    stroke="#00E5A0"
    lineWidth={2}
    end={0}
    opacity={0}
  />
);
yield* all(
  edge().opacity(1, 0.1),
  edge().end(1, 0.5, easeInOutCubic),
);

// Trail effect: animate both start and end forward
// This makes a "snake" that travels along the path
const trailLine = createRef<Line>();
view.add(
  <Line ref={trailLine} points={path} stroke="#00B4D8" lineWidth={3}
    start={0} end={0} />
);
// Draw line head forward
yield* trailLine().end(1, 0.6, easeInOutCubic);
// Then erase tail (creating a traveling trail)
yield* trailLine().start(1, 0.6, easeInCubic);

// Dashed ghost edge
<Line
  points={[[x1, y1], [x2, y2]]}
  stroke="#00B4D8"
  lineWidth={1}
  lineDash={[6, 4]}
  opacity={0.4}
  end={0}
/>
yield* ghostLine().end(1, 0.4, easeInOutCubic);
```

---

## 9. NUMBER SLAMMING ONTO SCREEN

```typescript
const counter = createRef<Txt>();
view.add(
  <Txt
    ref={counter}
    text="0"
    fontSize={72}
    fontWeight={800}
    fill="#00E5A0"
    shadowColor="#00E5A0"
    shadowBlur={0}
    scale={0}
    opacity={0}
  />
);

// Slam in with spring
yield* spring(
  PlopSpring,
  0, 1,
  (value) => {
    counter().scale(value);
    counter().opacity(Math.min(1, value * 2));
  }
);

// Count up animation
const targetNum = 9767;
yield* tween(1.5, value => {
  const current = Math.floor(easeOutExpo(value) * targetNum);
  counter().text(current.toLocaleString());
});

// Sequence: count + slam
counter().text('0');
counter().scale(1);
counter().opacity(1);
yield* tween(1.2, value => {
  counter().text(Math.floor(easeOutExpo(value) * 9767).toString());
});
// Then slam to final with shadow
yield* all(
  counter().shadowBlur(40, 0.1),
  counter().scale(1.15, 0.1, easeOutExpo),
);
yield* all(
  counter().shadowBlur(15, 0.3),
  counter().scale(1.0, 0.3, easeInOutCubic),
);
```

---

## 10. FADE TRANSITIONS BETWEEN ACTS

```typescript
// Option A: Fade out / fade in with waitFor
yield* graphContainer().opacity(0, 0.4, easeInCubic);
// swap content here (instant, no animation)
resetGraph();
yield* graphContainer().opacity(1, 0.3, easeOutCubic);

// Option B: Cross-fade two containers
yield* all(
  oldContainer().opacity(0, 0.5, easeInOutCubic),
  newContainer().opacity(1, 0.5, easeInOutCubic),
);

// Option C: Scale + fade (dramatic)
yield* all(
  container().opacity(0, 0.3, easeInCubic),
  container().scale(0.9, 0.3, easeInCubic),
);
container().scale(1.1);
yield* all(
  container().opacity(1, 0.3, easeOutCubic),
  container().scale(1.0, 0.3, easeOutExpo),
);
```

---

## 11. SIGNALS — REACTIVE VALUES

```typescript
// Simple signal
const glow = createSignal(0);
<Circle shadowBlur={() => glow()} />  // reactive binding

// Set instantly
glow(30);
// Animate
yield* glow(60, 0.5, easeOutExpo);

// Computed signal (reactive derived value)
const isActive = createSignal(false);
const glowColor = () => isActive() ? '#00E5A0' : '#4A5568';
<Circle fill={() => glowColor()} />
```

---

## 12. REFS

```typescript
// Single ref
const circle = createRef<Circle>();
<Circle ref={circle} />
circle().fill('#ff0000');    // call () to get instance
yield* circle().scale(2, 1);

// Ref array — collects multiple nodes via same ref
const circles = createRefArray<Circle>();
view.add(nodes.map(n => <Circle ref={circles} x={n.x} y={n.y} />));
// Access by index
circles[0].fill('#ff0000');
// Animate all
yield* all(...circles.map(c => c.opacity(1, 0.3)));
```

---

## 13. EASING FUNCTIONS — COMPLETE LIST

All are `(value: number) => number` where value is 0..1.

```typescript
// Basic
linear, sin, cos

// Sine
easeInSine, easeOutSine, easeInOutSine

// Quad (gentle)
easeInQuad, easeOutQuad, easeInOutQuad

// Cubic (standard)
easeInCubic, easeOutCubic, easeInOutCubic

// Quart (stronger)
easeInQuart, easeOutQuart, easeInOutQuart

// Quint (strongest polynomial)
easeInQuint, easeOutQuint, easeInOutQuint

// Expo (dramatic)
easeInExpo, easeOutExpo, easeInOutExpo

// Circ (circular)
easeInCirc, easeOutCirc, easeInOutCirc

// Back (overshoots)
easeInBack, easeOutBack, easeInOutBack
// Custom overshoot: createEaseOutBack(s = 1.70158)

// Bounce
easeInBounce, easeOutBounce, easeInOutBounce

// Elastic
easeInElastic, easeOutElastic, easeInOutElastic
```

Best picks for m1nd animations:
- Node pop-in: `easeOutExpo` or `PlopSpring`
- Edge draw: `easeInOutCubic`
- Glow pulse: `easeInOutSine` (smooth breathing)
- Number slam: `easeOutExpo` (fast in, slow land)
- Text fade: `easeOutCubic` in / `easeInCubic` out
- Camera zoom: `easeInOutCubic`
- Scale slam: `easeOutBack` (slight overshoot)

---

## 14. COLOR HELPERS

Uses `chroma-js` under the hood. Colors accept:
- Hex strings: `'#00E5A0'`, `'#00E5A080'` (with alpha)
- CSS: `'rgba(0,229,160,0.5)'`, `'hsl(160,100%,45%)'`
- Named: `'hotpink'`, `'lightseagreen'`
- Numbers: `0x00E5A0`

```typescript
import { Color } from '@motion-canvas/core';

const c = new Color('#00E5A0');
c.alpha(0.5).hex();           // '#00E5A080'
Color.lerp('#ff0000', '#00ff00', 0.5);  // midpoint color
```

---

## 15. PARTICLE / TRAIL EFFECTS (workarounds)

Motion Canvas has no built-in particle system. Patterns that work:

### Particle burst (pre-instantiate N circles)
```typescript
const particles: ReturnType<typeof createRef<Circle>>[] = [];
const N = 20;
for (let i = 0; i < N; i++) {
  const p = createRef<Circle>();
  particles.push(p);
  container().add(
    <Circle ref={p} size={4} fill="#00E5A0" opacity={0} x={cx} y={cy} />
  );
}

// Emit burst
yield* all(
  ...particles.map((p, i) => {
    const angle = (i / N) * Math.PI * 2;
    const dist = 60 + Math.random() * 40;
    return all(
      p().opacity(1, 0.05),
      p().x(cx + Math.cos(angle) * dist, 0.6, easeOutExpo),
      p().y(cy + Math.sin(angle) * dist, 0.6, easeOutExpo),
      p().opacity(0, 0.6, easeInQuad),
    );
  })
);
```

### Trailing activation line (head + tail)
```typescript
// Animate end forward, then start forward (leaving no line behind)
yield* activationLine().end(1, 0.4, easeInOutCubic);
yield* activationLine().start(1, 0.4, easeInCubic);
```

### Glow halo ring
```typescript
// Animate a large-radius ring from opacity 0, scale 1 to opacity 0, scale 2
const halo = createRef<Circle>();
container().add(
  <Circle ref={halo} size={60} stroke="#00E5A0" lineWidth={2}
    opacity={0} fill={null} />
);

yield* all(
  halo().opacity(0.8, 0.05),
  halo().size(60, 0),
);
yield* all(
  halo().opacity(0, 0.5),
  halo().size(140, 0.5, easeOutExpo),
);
```

---

## 16. NOTES FROM brain.tsx (V1 — what works)

- `createSignal` per node property works well for batch-animating many nodes without ref arrays.
- `sequence(delay, ...tasks)` is the right tool for staggered node appearance.
- `ghostEdgeContainer().children() as Line[]` — cast children array to get refs after dynamic add.
- Setting signal values instantly (no duration) works for "reset" steps between acts: `signal(value)` not `yield* signal(value, 0)`.
- `view.fill(BG)` sets the scene background color — call on the view object directly.
- `graphContainer().opacity(0)` / `.opacity(1, dur)` is the cleanest way to hide/show a whole subtree.
- `lineDash={[6, 4]}` on Line creates dashed appearance without any animation overhead.
- `shadowColor` + `shadowBlur` is the glow technique (no separate blur pass needed).
- Signals initialized with `createSignal(initialValue)` can be passed as reactive props: `fill={() => signal()}`.
