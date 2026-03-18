import {makeScene2D, Camera, Circle, Line, Txt, Rect, Node} from '@motion-canvas/2d';
import {
  all,
  chain,
  createRef,
  createSignal,
  delay,
  easeInCubic,
  easeInOutCubic,
  easeInOutQuad,
  easeInOutSine,
  easeOutCubic,
  easeOutExpo,
  linear,
  loop,
  loopFor,
  sequence,
  spawn,
  run,
  waitFor,
  createRefArray,
  tween,
  spring,
  PlopSpring,
  Vector2,
} from '@motion-canvas/core';

// =============================================================================
// THEME — v2 Cinema Palette
// =============================================================================
const VOID          = '#060B14';
const TERMINAL      = '#00FF88';
const SIGNAL        = '#00E5A0';
const SEMANTIC      = '#00B4D8';
const TEMPORAL      = '#F59E0B';
const CAUSAL        = '#EF4444';
const GHOST         = '#6366F1';
const BONE          = '#E2E8F0';
const ASH           = '#64748B';
const GRAPHITE      = '#1E293B';
const COST_RED      = '#FF2D55';

const MONO = 'JetBrains Mono, Fira Code, monospace';
const SANS = 'Inter, system-ui, sans-serif';

// =============================================================================
// GRAPH DATA — procedural brain-like topology
// =============================================================================
interface GNode { x: number; y: number; layer: number; id: number }
interface GEdge { from: number; to: number; dimension?: 'structural' | 'semantic' | 'temporal' | 'causal' }

function seededRandom(seed: number) {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

function generateGraph(
  nodeCount: number,
  edgeDensity: number,
  seed: number,
): { nodes: GNode[]; edges: GEdge[] } {
  const rng = seededRandom(seed);
  const nodes: GNode[] = [];
  const edges: GEdge[] = [];

  // Concentric elliptical shells — brain-like organic shape
  const layers = 5;
  const nodesPerLayer = Math.ceil(nodeCount / layers);

  for (let layer = 0; layer < layers; layer++) {
    const count = layer === layers - 1
      ? nodeCount - nodes.length
      : nodesPerLayer;
    const radiusX = 100 + layer * 130;
    const radiusY = 70 + layer * 95;

    for (let i = 0; i < count; i++) {
      const angle = (i / count) * Math.PI * 2 + rng() * 0.6;
      const jitterX = (rng() - 0.5) * 55;
      const jitterY = (rng() - 0.5) * 38;
      nodes.push({
        x: Math.cos(angle) * radiusX + jitterX,
        y: Math.sin(angle) * radiusY + jitterY,
        layer,
        id: nodes.length,
      });
    }
  }

  // Edges: proximity + layer adjacency
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const dx = nodes[i].x - nodes[j].x;
      const dy = nodes[i].y - nodes[j].y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const layerDiff = Math.abs(nodes[i].layer - nodes[j].layer);
      const threshold = edgeDensity / (1 + dist / 80) / (1 + layerDiff * 2);
      if (rng() < threshold) {
        edges.push({ from: i, to: j });
      }
    }
  }

  return { nodes, edges };
}

const GRAPH = generateGraph(48, 0.35, 42);

// Adjacency map for BFS
function buildAdjacency(graph: { nodes: GNode[]; edges: GEdge[] }): Map<number, number[]> {
  const adj = new Map<number, number[]>();
  for (const n of graph.nodes) adj.set(n.id, []);
  for (const e of graph.edges) {
    adj.get(e.from)!.push(e.to);
    adj.get(e.to)!.push(e.from);
  }
  return adj;
}

const ADJ = buildAdjacency(GRAPH);

function bfsLayers(source: number, maxDepth: number): number[][] {
  const visited = new Set<number>([source]);
  const layers: number[][] = [[source]];
  for (let d = 0; d < maxDepth; d++) {
    const frontier: number[] = [];
    for (const n of layers[d]) {
      for (const nb of ADJ.get(n) ?? []) {
        if (!visited.has(nb)) {
          visited.add(nb);
          frontier.push(nb);
        }
      }
    }
    if (frontier.length === 0) break;
    layers.push(frontier);
  }
  return layers;
}

// =============================================================================
// HELPERS
// =============================================================================

/** Type text character-by-character into a Txt ref */
function* typeText(
  ref: ReturnType<typeof createRef<Txt>>,
  text: string,
  charDelay = 0.04,
) {
  for (let i = 0; i <= text.length; i++) {
    ref().text(text.slice(0, i));
    yield* waitFor(charDelay);
  }
}

// =============================================================================
// MAIN SCENE
// =============================================================================
export default makeScene2D(function* (view) {
  view.fill(VOID);

  // -------------------------------------------------------------------------
  // SHARED CONTAINERS
  // -------------------------------------------------------------------------
  const cam = createRef<Camera>();
  const graphContainer = createRef<Node>();
  const uiContainer = createRef<Node>();
  const terminalContainer = createRef<Node>();

  view.add(
    <Camera ref={cam} zoom={1}>
      <Node ref={graphContainer} opacity={0} />
    </Camera>,
  );
  view.add(<Node ref={uiContainer} />);
  view.add(<Node ref={terminalContainer} />);

  // -------------------------------------------------------------------------
  // NODE + EDGE SIGNALS (pre-created for graph scenes)
  // -------------------------------------------------------------------------
  const nodeSignals = GRAPH.nodes.map(() => ({
    fillOpacity: createSignal(0),
    fillColor: createSignal(ASH),
    radius: createSignal(2.5),
    glowBlur: createSignal(0),
  }));

  const edgeSignals = GRAPH.edges.map(() => ({
    progress: createSignal(0),
    strokeColor: createSignal(GRAPHITE),
    strokeWidth: createSignal(0.5),
    strokeOpacity: createSignal(0),
  }));

  // Create node circles
  for (let i = 0; i < GRAPH.nodes.length; i++) {
    const n = GRAPH.nodes[i];
    graphContainer().add(
      <Circle
        x={n.x}
        y={n.y}
        width={() => nodeSignals[i].radius() * 2}
        height={() => nodeSignals[i].radius() * 2}
        fill={() => nodeSignals[i].fillColor()}
        opacity={() => nodeSignals[i].fillOpacity()}
        shadowColor={() => nodeSignals[i].fillColor()}
        shadowBlur={() => nodeSignals[i].glowBlur()}
      />,
    );
  }

  // Create edge lines
  for (let i = 0; i < GRAPH.edges.length; i++) {
    const e = GRAPH.edges[i];
    const fromN = GRAPH.nodes[e.from];
    const toN = GRAPH.nodes[e.to];
    graphContainer().add(
      <Line
        points={[[fromN.x, fromN.y], [toN.x, toN.y]]}
        stroke={() => edgeSignals[i].strokeColor()}
        lineWidth={() => edgeSignals[i].strokeWidth()}
        opacity={() => edgeSignals[i].strokeOpacity()}
        end={() => edgeSignals[i].progress()}
      />,
    );
  }

  /** Reset all graph visuals to dormant state */
  function resetGraph() {
    for (const s of nodeSignals) {
      s.fillColor(ASH);
      s.radius(2.5);
      s.glowBlur(0);
      s.fillOpacity(0);
    }
    for (const s of edgeSignals) {
      s.strokeColor(GRAPHITE);
      s.strokeWidth(0.5);
      s.strokeOpacity(0);
      s.progress(0);
    }
  }

  // =========================================================================
  // SCENE 1: COLD OPEN — "The Familiar" (0:00 - 0:03, 90 frames)
  // =========================================================================

  // --- Frame 0-15 (0.0-0.5s): Pure black. Nothing. ---
  yield* waitFor(0.5);

  // --- Frame 15-30 (0.5-1.0s): Terminal cursor appears and blinks ---
  const cursor = createRef<Rect>();
  const cmdText = createRef<Txt>();
  const promptText = createRef<Txt>();

  terminalContainer().add(
    <>
      <Txt
        ref={promptText}
        text="$ "
        fontFamily={MONO}
        fontSize={20}
        fontWeight={400}
        fill={ASH}
        x={-380}
        y={0}
        opacity={0}
      />
      <Txt
        ref={cmdText}
        text=""
        fontFamily={MONO}
        fontSize={20}
        fontWeight={400}
        fill={TERMINAL}
        x={-364}
        y={0}
        opacity={1}
        textAlign={'left'}
      />
      <Rect
        ref={cursor}
        width={2}
        height={20}
        fill={TERMINAL}
        x={-360}
        y={0}
        opacity={0}
      />
    </>,
  );

  // Cursor appears
  yield* cursor().opacity(1, 0.05, linear);
  // Blink: on 0.3s, off 0.2s, on
  yield* waitFor(0.3);
  yield* cursor().opacity(0, 0.02, linear);
  yield* waitFor(0.2);
  yield* cursor().opacity(1, 0.02, linear);

  // --- Frame 30-75 (1.0-2.5s): Type the grep command ---
  promptText().opacity(1);
  const grepCmd = 'grep -rn "authentication" ./backend';

  // Type character by character, cursor moves with text
  for (let i = 0; i <= grepCmd.length; i++) {
    cmdText().text(grepCmd.slice(0, i));
    // Move cursor to end of typed text
    // Each char ~10px wide in 20px monospace
    cursor().x(-360 + i * 10.2);
    yield* waitFor(0.04);
  }

  // --- Frame 75-90 (2.5-3.0s): Pause. Cursor stops blinking. Anticipation. ---
  yield* waitFor(0.3);
  // Cursor holds steady (already visible)
  yield* waitFor(0.2);

  // =========================================================================
  // SCENE 2: THE COST — "What You Don't See" (0:03 - 0:08, 150 frames)
  // =========================================================================

  // --- Frame 90-105 (3.0-3.5s): Enter pressed, grep output scrolls ---
  cursor().opacity(0);

  // Grep output lines — appear stacked below the command
  const grepLines = [
    'backend/auth.py:42:    def authenticate(self, ...',
    'backend/middleware.py:18:    # authentication check',
    'backend/session.py:91:    authentication_required = True',
    'backend/jwt_handler.py:7:    """JWT authentication module"""',
    'backend/oauth.py:23:    class AuthenticationProvider:',
    'backend/permissions.py:55:    if not authentication_valid:',
    'backend/api/users.py:112:    @requires_authentication',
    'backend/api/admin.py:8:    # authentication bypass for...',
    'backend/tests/test_auth.py:31:    def test_authentication...',
    'backend/config.py:67:    AUTHENTICATION_BACKEND = ...',
    'backend/utils/tokens.py:14:    # re-authentication flow',
    'backend/cache.py:29:    authentication_cache_ttl = 300',
  ];

  const grepOutputRefs: ReturnType<typeof createRef<Txt>>[] = [];
  for (let i = 0; i < grepLines.length; i++) {
    const ref = createRef<Txt>();
    grepOutputRefs.push(ref);
    terminalContainer().add(
      <Txt
        ref={ref}
        text={grepLines[i]}
        fontFamily={MONO}
        fontSize={15}
        fontWeight={400}
        fill={ASH}
        x={-380}
        y={30 + i * 22}
        opacity={0}
        textAlign={'left'}
      />,
    );
  }

  // Lines appear rapidly, 60ms intervals. Last 4 fade at bottom edge
  for (let i = 0; i < grepOutputRefs.length; i++) {
    const targetOpacity = i >= 8 ? 0.4 - (i - 8) * 0.08 : 0.85;
    grepOutputRefs[i]().opacity(targetOpacity);
    yield* waitFor(0.06);
  }

  // --- Frame 105-135 (3.5-4.5s): Three counters fade in at top ---
  const tokenCountSignal = createSignal(0);
  const clockSignal = createSignal(0);
  const costSignal = createSignal(0);

  const tokenLabel = createRef<Txt>();
  const tokenValue = createRef<Txt>();
  const clockLabel = createRef<Txt>();
  const clockValue = createRef<Txt>();
  const costLabel = createRef<Txt>();
  const costValue = createRef<Txt>();

  uiContainer().add(
    <>
      {/* Token counter */}
      <Txt
        ref={tokenLabel}
        text="tokens burned"
        fontFamily={SANS}
        fontSize={13}
        fontWeight={400}
        fill={ASH}
        x={-400}
        y={-370}
        opacity={0}
      />
      <Txt
        ref={tokenValue}
        text={() => Math.floor(tokenCountSignal()).toLocaleString()}
        fontFamily={SANS}
        fontSize={42}
        fontWeight={800}
        fill={COST_RED}
        x={-400}
        y={-330}
        opacity={0}
        shadowColor={COST_RED}
        shadowBlur={0}
      />
      {/* Clock */}
      <Txt
        ref={clockLabel}
        text="wall clock"
        fontFamily={SANS}
        fontSize={13}
        fontWeight={400}
        fill={ASH}
        x={0}
        y={-370}
        opacity={0}
      />
      <Txt
        ref={clockValue}
        text={() => clockSignal().toFixed(1) + 's'}
        fontFamily={SANS}
        fontSize={42}
        fontWeight={800}
        fill={TEMPORAL}
        x={0}
        y={-330}
        opacity={0}
      />
      {/* Cost */}
      <Txt
        ref={costLabel}
        text="API cost"
        fontFamily={SANS}
        fontSize={13}
        fontWeight={400}
        fill={ASH}
        x={400}
        y={-370}
        opacity={0}
      />
      <Txt
        ref={costValue}
        text={() => '$' + costSignal().toFixed(3)}
        fontFamily={SANS}
        fontSize={42}
        fontWeight={800}
        fill={COST_RED}
        x={400}
        y={-330}
        opacity={0}
        shadowColor={COST_RED}
        shadowBlur={0}
      />
    </>,
  );

  // Fade labels and values in simultaneously
  yield* all(
    tokenLabel().opacity(1, 0.3, easeOutCubic),
    tokenValue().opacity(1, 0.3, easeOutCubic),
    clockLabel().opacity(1, 0.3, easeOutCubic),
    clockValue().opacity(1, 0.3, easeOutCubic),
    costLabel().opacity(1, 0.3, easeOutCubic),
    costValue().opacity(1, 0.3, easeOutCubic),
  );

  // Count up all three simultaneously — SLAM easing
  yield* all(
    tokenCountSignal(47000, 0.8, easeOutExpo),
    clockSignal(3.2, 0.8, linear),
    costSignal(0.041, 0.8, easeOutExpo),
  );

  // --- Frame 135-165 (4.5-5.5s): Rhetorical question typed ---
  const rhetoricalRef = createRef<Txt>();
  terminalContainer().add(
    <Txt
      ref={rhetoricalRef}
      text=""
      fontFamily={MONO}
      fontSize={15}
      fontWeight={400}
      fill={ASH}
      x={-380}
      y={300}
      opacity={1}
      textAlign={'left'}
    />,
  );

  yield* waitFor(0.2);
  const rhetoricalText = '$ grep found 12 matches. But what did it miss?';
  yield* typeText(rhetoricalRef, rhetoricalText, 0.035);
  yield* waitFor(0.5);

  // --- Frame 165-210 (5.5-7.0s): Four sins of grep ---
  const sins = [
    { main: '  no blast radius', comment: '  -- what else breaks?' },
    { main: '  no structural holes', comment: '  -- what\'s missing?' },
    { main: '  no co-change prediction', comment: '  -- what else will change?' },
    { main: '  no learning', comment: '  -- it\'s as dumb next time' },
  ];

  const sinRefs: ReturnType<typeof createRef<Node>>[] = [];
  for (let i = 0; i < sins.length; i++) {
    const sinContainer = createRef<Node>();
    const sinMain = createRef<Txt>();
    const sinComment = createRef<Txt>();
    sinRefs.push(sinContainer);

    terminalContainer().add(
      <Node ref={sinContainer} opacity={0} y={340 + i * 28}>
        <Txt
          ref={sinMain}
          text={sins[i].main}
          fontFamily={SANS}
          fontSize={16}
          fontWeight={400}
          fill={ASH}
          x={-300}
          textAlign={'left'}
        />
        <Txt
          ref={sinComment}
          text={sins[i].comment}
          fontFamily={SANS}
          fontSize={16}
          fontWeight={400}
          fill={CAUSAL}
          x={-80}
          textAlign={'left'}
        />
      </Node>,
    );
  }

  // Staggered appearance: 0.3s each, 0.2s between
  for (const sinRef of sinRefs) {
    yield* sinRef().opacity(1, 0.3, easeOutCubic);
    yield* waitFor(0.2);
  }

  // --- Frame 210-240 (7.0-8.0s): Everything fades to black ---
  yield* waitFor(0.4);
  yield* all(
    terminalContainer().opacity(0, 0.6, easeInCubic),
    tokenLabel().opacity(0, 0.6, easeInCubic),
    tokenValue().opacity(0, 0.6, easeInCubic),
    clockLabel().opacity(0, 0.6, easeInCubic),
    clockValue().opacity(0, 0.6, easeInCubic),
    costLabel().opacity(0, 0.6, easeInCubic),
    costValue().opacity(0, 0.6, easeInCubic),
  );

  // Clean up terminal elements (remove children, reset)
  terminalContainer().removeChildren();
  terminalContainer().opacity(1);

  // =========================================================================
  // SCENE 3: THE COMMAND — "The Alternative" (0:08 - 0:10, 60 frames)
  // =========================================================================

  // --- Frame 240-255 (8.0-8.5s): Black void. Cursor reappears. ---
  yield* waitFor(0.5);

  const cursor2 = createRef<Rect>();
  const prompt2 = createRef<Txt>();
  const cmdText2 = createRef<Txt>();

  terminalContainer().add(
    <>
      <Txt
        ref={prompt2}
        text="$ "
        fontFamily={MONO}
        fontSize={20}
        fontWeight={400}
        fill={ASH}
        x={-380}
        y={0}
        opacity={0}
      />
      <Txt
        ref={cmdText2}
        text=""
        fontFamily={MONO}
        fontSize={20}
        fontWeight={400}
        fill={SIGNAL}
        x={-364}
        y={0}
        opacity={1}
        textAlign={'left'}
      />
      <Rect
        ref={cursor2}
        width={2}
        height={20}
        fill={SIGNAL}
        x={-360}
        y={0}
        opacity={0}
      />
    </>,
  );

  // Cursor blink
  yield* cursor2().opacity(1, 0.05, linear);
  yield* waitFor(0.2);
  yield* cursor2().opacity(0, 0.02, linear);
  yield* waitFor(0.15);
  yield* cursor2().opacity(1, 0.02, linear);

  // --- Frame 255-285 (8.5-9.5s): Type m1nd.activate ---
  prompt2().opacity(1);
  const m1ndCmd = 'm1nd.activate("authentication")';

  // Type with two styles: "m1nd" in bold signal, rest normal
  // We use a single Txt and style via the whole string — the signal green is already set
  for (let i = 0; i <= m1ndCmd.length; i++) {
    cmdText2().text(m1ndCmd.slice(0, i));
    // Bold the "m1nd" portion via fontWeight (first 4 chars)
    if (i <= 4) {
      cmdText2().fontWeight(700);
    } else {
      cmdText2().fontWeight(400);
    }
    cursor2().x(-360 + i * 10.2);
    yield* waitFor(0.04);
  }

  // --- Frame 285-300 (9.5-10.0s): Hold, then cursor disappears (Enter pressed) ---
  yield* waitFor(0.3);
  // Instant cursor off — Enter pressed
  cursor2().opacity(0);

  // The command begins to fade and drift upward into scene 4
  // Don't yield — this runs concurrently with scene 4 setup
  const scene3FadePromise = all(
    cmdText2().opacity(0.15, 1.0, easeInCubic),
    cmdText2().y(-480, 1.0, easeInOutCubic),
    prompt2().opacity(0.15, 1.0, easeInCubic),
    prompt2().y(-480, 1.0, easeInOutCubic),
  );

  // =========================================================================
  // SCENE 4: THE BRAIN WAKES — "Activation" (0:10 - 0:15, 150 frames)
  // =========================================================================

  // Camera starts zoomed out
  cam().zoom(0.6);
  graphContainer().opacity(0);

  // BFS activation layers from center-ish source node
  const sourceIdx = 2;
  const activationLayers = bfsLayers(sourceIdx, 4);

  // Classify waves — pick from BFS layers
  const wave1 = activationLayers[1] ? activationLayers[1].slice(0, 5) : [];
  const wave2 = activationLayers[2] ? activationLayers[2].slice(0, 8) : [];
  const wave3 = activationLayers[3] ? activationLayers[3].slice(0, 12) : [];
  // Remaining nodes = dormant (all nodes not in any wave)
  const activatedSet = new Set<number>([sourceIdx, ...wave1, ...wave2, ...wave3]);
  const dormantNodes = GRAPH.nodes.filter(n => !activatedSet.has(n.id)).map(n => n.id);

  // --- Frame 300-330 (10.0-11.0s): Command fades to ghost. Nodes appear in waves. ---

  // Start fading the graph container in
  yield* all(
    graphContainer().opacity(1, 0.5, easeOutCubic),
    run(function* () { yield* scene3FadePromise; }),
    // Camera zoom begins — the only camera move in the animation
    cam().zoom(1.0, 4.0, easeInOutCubic),
  );

  // Source node appears first — the query node "authentication"
  yield* all(
    nodeSignals[sourceIdx].fillOpacity(1, 0.15, easeOutExpo),
    nodeSignals[sourceIdx].fillColor(SIGNAL, 0.15),
    nodeSignals[sourceIdx].radius(8, 0.2, easeOutExpo),
    nodeSignals[sourceIdx].glowBlur(25, 0.2, easeOutExpo),
  );

  yield* waitFor(0.15);

  // Wave 1: Direct structural connections (green) — 5 nodes, first ring
  if (wave1.length > 0) {
    yield* sequence(
      0.03,
      ...wave1.map(id => all(
        nodeSignals[id].fillOpacity(1, 0.15, easeOutExpo),
        nodeSignals[id].fillColor(SIGNAL, 0.15),
        nodeSignals[id].radius(5, 0.15, easeOutExpo),
        nodeSignals[id].glowBlur(12, 0.15, easeOutExpo),
      )),
    );
  }

  yield* waitFor(0.1);

  // Wave 2: Semantic connections (blue) — 8 nodes, second ring
  if (wave2.length > 0) {
    yield* sequence(
      0.03,
      ...wave2.map(id => all(
        nodeSignals[id].fillOpacity(0.9, 0.15, easeOutExpo),
        nodeSignals[id].fillColor(SEMANTIC, 0.15),
        nodeSignals[id].radius(4, 0.15, easeOutExpo),
        nodeSignals[id].glowBlur(8, 0.15, easeOutExpo),
      )),
    );
  }

  yield* waitFor(0.1);

  // Wave 3: Temporal connections (amber) — 12 nodes, third ring
  if (wave3.length > 0) {
    yield* sequence(
      0.03,
      ...wave3.map(id => all(
        nodeSignals[id].fillOpacity(0.8, 0.15, easeOutExpo),
        nodeSignals[id].fillColor(TEMPORAL, 0.15),
        nodeSignals[id].radius(3.5, 0.15, easeOutExpo),
        nodeSignals[id].glowBlur(6, 0.15, easeOutExpo),
      )),
    );
  }

  yield* waitFor(0.1);

  // Dormant nodes: ~20 remaining, ash colored, establishing the full graph context
  if (dormantNodes.length > 0) {
    yield* sequence(
      0.015,
      ...dormantNodes.map(id => all(
        nodeSignals[id].fillOpacity(0.4, 0.1),
        nodeSignals[id].fillColor(ASH, 0.1),
        nodeSignals[id].radius(2.5, 0.1),
      )),
    );
  }

  // --- Frame 330-360 (11.0-12.0s): Edges draw in order of activation ---

  // Categorize edges by the activation wave of their endpoints
  const nodeWaveMap = new Map<number, number>(); // node id -> wave number
  nodeWaveMap.set(sourceIdx, 0);
  wave1.forEach(id => nodeWaveMap.set(id, 1));
  wave2.forEach(id => nodeWaveMap.set(id, 2));
  wave3.forEach(id => nodeWaveMap.set(id, 3));

  // Sort edges: source-connected first, then semantic, temporal, dormant
  const edgesByPriority: { idx: number; wave: number }[] = [];
  for (let i = 0; i < GRAPH.edges.length; i++) {
    const e = GRAPH.edges[i];
    const fromWave = nodeWaveMap.get(e.from) ?? 99;
    const toWave = nodeWaveMap.get(e.to) ?? 99;
    const minWave = Math.min(fromWave, toWave);
    edgesByPriority.push({ idx: i, wave: minWave });
  }
  edgesByPriority.sort((a, b) => a.wave - b.wave);

  // Draw edges with dimension-appropriate colors
  yield* sequence(
    0.02,
    ...edgesByPriority.map(({ idx, wave }) => {
      let color = GRAPHITE;
      let width = 0.5;
      let opacity = 0.3;

      if (wave === 0) {
        color = SIGNAL; width = 2; opacity = 0.7;
      } else if (wave === 1) {
        color = SIGNAL; width = 2; opacity = 0.7;
      } else if (wave === 2) {
        color = SEMANTIC; width = 1.5; opacity = 0.6;
      } else if (wave === 3) {
        color = TEMPORAL; width = 1; opacity = 0.5;
      }

      return all(
        edgeSignals[idx].strokeColor(color, 0.15),
        edgeSignals[idx].strokeWidth(width, 0.15),
        edgeSignals[idx].strokeOpacity(opacity, 0.15),
        edgeSignals[idx].progress(1, 0.25, easeInOutCubic),
      );
    }),
  );

  // --- Frame 360-390 (12.0-13.0s): Ghost edges appear (structural holes) ---
  const ghostEdgeContainer = createRef<Node>();
  graphContainer().add(<Node ref={ghostEdgeContainer} opacity={0} />);

  const existingEdgeSet = new Set(
    GRAPH.edges.map(e => `${Math.min(e.from, e.to)}-${Math.max(e.from, e.to)}`),
  );

  const ghostLabels = [
    'missing: rate_limiter',
    'gap: audit_log',
    'missing: session_store',
    'gap: token_refresh',
    'missing: rbac_check',
  ];

  const activatedArr = [...activatedSet];
  let ghostCount = 0;
  const ghostLineRefs: ReturnType<typeof createRef<Line>>[] = [];
  const ghostLabelRefs: ReturnType<typeof createRef<Txt>>[] = [];

  for (let a = 0; a < activatedArr.length && ghostCount < 5; a++) {
    for (let b = a + 1; b < activatedArr.length && ghostCount < 5; b++) {
      const id1 = activatedArr[a];
      const id2 = activatedArr[b];
      const key = `${Math.min(id1, id2)}-${Math.max(id1, id2)}`;
      if (!existingEdgeSet.has(key)) {
        const n1 = GRAPH.nodes[id1];
        const n2 = GRAPH.nodes[id2];
        const dx = n1.x - n2.x;
        const dy = n1.y - n2.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist > 80 && dist < 300) {
          const gRef = createRef<Line>();
          const gLabel = createRef<Txt>();
          ghostLineRefs.push(gRef);
          ghostLabelRefs.push(gLabel);

          const midX = (n1.x + n2.x) / 2;
          const midY = (n1.y + n2.y) / 2;

          ghostEdgeContainer().add(
            <>
              <Line
                ref={gRef}
                points={[[n1.x, n1.y], [n2.x, n2.y]]}
                stroke={GHOST}
                lineWidth={1.5}
                lineDash={[8, 4]}
                opacity={0.3}
                end={0}
              />
              <Txt
                ref={gLabel}
                text={ghostLabels[ghostCount] ?? 'missing: unknown'}
                fontFamily={SANS}
                fontSize={13}
                fontWeight={400}
                fill={GHOST}
                x={midX}
                y={midY - 12}
                opacity={0}
              />
            </>,
          );
          ghostCount++;
        }
      }
    }
  }

  // Reveal ghost edges with labels
  yield* ghostEdgeContainer().opacity(1, 0.2);

  if (ghostLineRefs.length > 0) {
    yield* sequence(
      0.12,
      ...ghostLineRefs.map((gRef, i) =>
        all(
          gRef().end(1, 0.4, easeInOutCubic),
          delay(0.2, ghostLabelRefs[i]().opacity(0.8, 0.3, easeOutCubic)),
        ),
      ),
    );
  }

  // Ghost edges pulse (breathing) — spawn as background animation
  if (ghostLineRefs.length > 0) {
    spawn(function* () {
      yield* loopFor(2.5, function* () {
        yield* all(
          ...ghostLineRefs.map(gRef =>
            gRef().opacity(0.6, 0.75, easeInOutQuad),
          ),
        );
        yield* all(
          ...ghostLineRefs.map(gRef =>
            gRef().opacity(0.3, 0.75, easeInOutQuad),
          ),
        );
      });
    });
  }

  // --- Frame 390-420 (13.0-14.0s): Result badge at bottom ---
  const resultBadge = createRef<Node>();
  const resultPrefix = createRef<Txt>();
  const result31ms = createRef<Txt>();
  const resultDash1 = createRef<Txt>();
  const result8 = createRef<Txt>();
  const resultMiddle = createRef<Txt>();
  const result3 = createRef<Txt>();
  const resultSuffix = createRef<Txt>();

  uiContainer().add(
    <Node ref={resultBadge} y={430} opacity={0}>
      <Txt
        ref={resultPrefix}
        text="activate: "
        fontFamily={SANS}
        fontSize={16}
        fontWeight={600}
        fill={BONE}
        x={-240}
        textAlign={'left'}
      />
      <Txt
        ref={result31ms}
        text="31ms"
        fontFamily={SANS}
        fontSize={18}
        fontWeight={700}
        fill={SIGNAL}
        x={-168}
        textAlign={'left'}
        shadowColor={SIGNAL}
        shadowBlur={0}
        scale={1.3}
        opacity={0}
      />
      <Txt
        ref={resultDash1}
        text=" -- "
        fontFamily={SANS}
        fontSize={16}
        fontWeight={600}
        fill={BONE}
        x={-115}
        textAlign={'left'}
      />
      <Txt
        ref={result8}
        text="8"
        fontFamily={SANS}
        fontSize={18}
        fontWeight={700}
        fill={SIGNAL}
        x={-75}
        textAlign={'left'}
        shadowColor={SIGNAL}
        shadowBlur={0}
        scale={1.3}
        opacity={0}
      />
      <Txt
        ref={resultMiddle}
        text=" results -- "
        fontFamily={SANS}
        fontSize={16}
        fontWeight={600}
        fill={BONE}
        x={-10}
        textAlign={'left'}
      />
      <Txt
        ref={result3}
        text="3"
        fontFamily={SANS}
        fontSize={18}
        fontWeight={700}
        fill={SIGNAL}
        x={90}
        textAlign={'left'}
        shadowColor={SIGNAL}
        shadowBlur={0}
        scale={1.3}
        opacity={0}
      />
      <Txt
        ref={resultSuffix}
        text=" structural holes detected"
        fontFamily={SANS}
        fontSize={16}
        fontWeight={600}
        fill={BONE}
        x={200}
        textAlign={'left'}
      />
    </Node>,
  );

  yield* resultBadge().opacity(1, 0.3, easeOutCubic);

  // SLAM the green numbers in
  yield* all(
    all(
      result31ms().opacity(1, 0.15),
      result31ms().scale(1, 0.4, easeOutExpo),
      result31ms().shadowBlur(12, 0.2, easeOutExpo),
    ),
    delay(0.1, all(
      result8().opacity(1, 0.15),
      result8().scale(1, 0.4, easeOutExpo),
      result8().shadowBlur(12, 0.2, easeOutExpo),
    )),
    delay(0.2, all(
      result3().opacity(1, 0.15),
      result3().scale(1, 0.4, easeOutExpo),
      result3().shadowBlur(12, 0.2, easeOutExpo),
    )),
  );

  // Settle the glow
  yield* all(
    result31ms().shadowBlur(4, 0.3, easeInOutCubic),
    result8().shadowBlur(4, 0.3, easeInOutCubic),
    result3().shadowBlur(4, 0.3, easeInOutCubic),
  );

  // --- Frame 420-450 (14.0-15.0s): Hold. Graph breathes. ---

  // Spawn breathing animation on activated nodes
  const breatheNodeIds = [sourceIdx, ...wave1, ...wave2.slice(0, 4)];
  spawn(function* () {
    yield* loopFor(2.0, function* () {
      yield* all(
        ...breatheNodeIds.map(id =>
          nodeSignals[id].glowBlur(
            (nodeSignals[id].glowBlur() as number) + 3,
            0.75,
            easeInOutSine,
          ),
        ),
      );
      yield* all(
        ...breatheNodeIds.map(id =>
          nodeSignals[id].glowBlur(
            Math.max(0, (nodeSignals[id].glowBlur() as number) - 3),
            0.75,
            easeInOutSine,
          ),
        ),
      );
    });
  });

  // Hold for 1 second — let viewer absorb
  yield* waitFor(1.0);

  // =========================================================================
  // END OF SCENES 1-4
  // =========================================================================
  // Scenes 5-9 will be appended below by the other agent.
  // Current state on screen:
  //   - Graph fully materialized with 48 nodes, edges drawn
  //   - Ghost edges pulsing in purple
  //   - Result badge at bottom: "activate: 31ms -- 8 results -- 3 structural holes detected"
  //   - Camera at zoom 1.0 (finished zooming in)
  //   - Activated nodes breathing gently
  //   - Terminal command ghosted at top (opacity 0.15, y -480)
  //
  // Available references for continuation:
  //   - cam (Camera), graphContainer, uiContainer, terminalContainer
  //   - nodeSignals[], edgeSignals[] — per-node/edge reactive state
  //   - ghostEdgeContainer — contains ghost edge Lines + labels
  //   - resultBadge — the bottom result badge Node
  //   - GRAPH, ADJ, bfsLayers() — graph data + helpers
  //   - resetGraph() — resets all node/edge visuals to dormant
  //   - typeText() — types text char-by-char into a Txt ref
  //   - Theme colors: VOID, TERMINAL, SIGNAL, SEMANTIC, TEMPORAL, CAUSAL, GHOST, BONE, ASH, GRAPHITE, COST_RED
  //   - Fonts: MONO, SANS
  // =========================================================================

  // SCENE 5: XLR CANCELLATION will start here
  // First, clean up scene 4 elements
  yield* all(
    resultBadge().opacity(0, 0.3, easeInCubic),
    ghostEdgeContainer().opacity(0, 0.3, easeInCubic),
    ...ghostLabelRefs.map(l => l().opacity(0, 0.3, easeInCubic)),
  );

  // Dim all nodes to ASH, fade edges
  yield* all(
    ...GRAPH.nodes.map((_, i) =>
      all(
        nodeSignals[i].fillColor(ASH, 0.3),
        nodeSignals[i].glowBlur(0, 0.3),
        nodeSignals[i].radius(2.5, 0.3),
        nodeSignals[i].fillOpacity(0.4, 0.3),
      ),
    ),
    ...GRAPH.edges.map((_, i) =>
      all(
        edgeSignals[i].strokeColor(GRAPHITE, 0.3),
        edgeSignals[i].strokeWidth(0.5, 0.3),
        edgeSignals[i].strokeOpacity(0.2, 0.3),
      ),
    ),
  );

  // Clean up terminal ghost text
  terminalContainer().removeChildren();

  // -------------------------------------------------------------------------
  // SCENE 5: XLR CANCELLATION — "The Secret Weapon" (0:15 - 0:18)
  // -------------------------------------------------------------------------

  // XLR title
  const xlrTitle = createRef<Txt>();
  const xlrSubtitle = createRef<Txt>();

  uiContainer().add(
    <>
      <Txt
        ref={xlrTitle}
        text="XLR NOISE CANCELLATION"
        fontFamily={SANS}
        fontSize={28}
        fontWeight={700}
        fill={BONE}
        y={-420}
        opacity={0}
      />
      <Txt
        ref={xlrSubtitle}
        text="borrowed from audio engineering"
        fontFamily={SANS}
        fontSize={16}
        fontWeight={400}
        fill={ASH}
        y={-385}
        opacity={0}
      />
    </>,
  );

  yield* all(
    xlrTitle().opacity(1, 0.4, easeOutCubic),
    delay(0.15, xlrSubtitle().opacity(1, 0.3, easeOutCubic)),
  );

  // Pick two convergent paths for XLR demo
  // Path A: upper-left nodes converging to center
  // Path B: lower-left nodes converging to same center
  const pathAIds = [0, 1, 5, 10].filter(i => i < GRAPH.nodes.length);
  const pathBIds = [3, 4, 8, 10].filter(i => i < GRAPH.nodes.length);
  const mergeNodeId = pathAIds[pathAIds.length - 1];

  // XLR overlay container
  const xlrContainer = createRef<Node>();
  graphContainer().add(<Node ref={xlrContainer} opacity={0} />);

  // Path A line
  const pathAPoints: [number, number][] = pathAIds.map(i => [GRAPH.nodes[i].x, GRAPH.nodes[i].y]);
  const pathARef = createRef<Line>();
  xlrContainer().add(
    <Line
      ref={pathARef}
      points={pathAPoints}
      stroke={SIGNAL}
      lineWidth={3}
      end={0}
      opacity={0.9}
    />,
  );

  // Path B line
  const pathBPoints: [number, number][] = pathBIds.map(i => [GRAPH.nodes[i].x, GRAPH.nodes[i].y]);
  const pathBRef = createRef<Line>();
  xlrContainer().add(
    <Line
      ref={pathBRef}
      points={pathBPoints}
      stroke={SIGNAL}
      lineWidth={3}
      end={0}
      opacity={0.9}
    />,
  );

  // + and - labels at path starts
  const plusLabel = createRef<Txt>();
  const minusLabel = createRef<Txt>();
  xlrContainer().add(
    <>
      <Txt
        ref={plusLabel}
        text="+"
        fontFamily={SANS}
        fontSize={18}
        fontWeight={700}
        fill={SIGNAL}
        x={GRAPH.nodes[pathAIds[0]].x - 20}
        y={GRAPH.nodes[pathAIds[0]].y - 15}
        opacity={0}
      />
      <Txt
        ref={minusLabel}
        text="-"
        fontFamily={SANS}
        fontSize={18}
        fontWeight={700}
        fill={SIGNAL}
        x={GRAPH.nodes[pathBIds[0]].x - 20}
        y={GRAPH.nodes[pathBIds[0]].y + 15}
        opacity={0}
      />
    </>,
  );

  yield* xlrContainer().opacity(1, 0.2);

  // Light up path nodes
  yield* all(
    ...pathAIds.map(id => all(
      nodeSignals[id].fillColor(SIGNAL, 0.2),
      nodeSignals[id].fillOpacity(1, 0.2),
      nodeSignals[id].radius(5, 0.2),
      nodeSignals[id].glowBlur(10, 0.2),
    )),
    ...pathBIds.map(id => all(
      nodeSignals[id].fillColor(SIGNAL, 0.2),
      nodeSignals[id].fillOpacity(1, 0.2),
      nodeSignals[id].radius(5, 0.2),
      nodeSignals[id].glowBlur(10, 0.2),
    )),
  );

  // Draw both paths simultaneously with + and - labels
  yield* all(
    pathARef().end(1, 0.6, easeInOutCubic),
    pathBRef().end(1, 0.6, easeInOutCubic),
    plusLabel().opacity(1, 0.3, easeOutCubic),
    minusLabel().opacity(1, 0.3, easeOutCubic),
  );

  // Noise injection — intermediate nodes turn red, paths shift color
  const noiseA = pathAIds.slice(1, -1);
  const noiseB = pathBIds.slice(1, -1);

  // Create noise particles along both paths
  const noiseParticles: ReturnType<typeof createRef<Circle>>[] = [];
  const allNoiseNodes = [...noiseA, ...noiseB];
  for (const id of allNoiseNodes) {
    for (let p = 0; p < 3; p++) {
      const pRef = createRef<Circle>();
      noiseParticles.push(pRef);
      xlrContainer().add(
        <Circle
          ref={pRef}
          x={GRAPH.nodes[id].x + (Math.random() - 0.5) * 30}
          y={GRAPH.nodes[id].y + (Math.random() - 0.5) * 30}
          width={6}
          height={6}
          fill={CAUSAL}
          shadowColor={CAUSAL}
          shadowBlur={8}
          opacity={0}
        />,
      );
    }
  }

  yield* all(
    ...noiseA.map(id => nodeSignals[id].fillColor(CAUSAL, 0.4)),
    ...noiseB.map(id => nodeSignals[id].fillColor(CAUSAL, 0.4)),
    pathARef().stroke(CAUSAL, 0.4),
    pathBRef().stroke(CAUSAL, 0.4),
    ...noiseParticles.map(pRef =>
      pRef().opacity(0.4 + Math.random() * 0.4, 0.3),
    ),
  );

  yield* waitFor(0.3);

  // Noise cancellation at merge node
  // 1. Particles drift toward merge node
  const mergeX = GRAPH.nodes[mergeNodeId].x;
  const mergeY = GRAPH.nodes[mergeNodeId].y;

  yield* all(
    ...noiseParticles.map(pRef =>
      all(
        pRef().x(mergeX, 0.3, easeInOutCubic),
        pRef().y(mergeY, 0.3, easeInOutCubic),
      ),
    ),
  );

  // 2. Particles annihilate — shrink to 0, flash white
  yield* all(
    ...noiseParticles.map(pRef =>
      all(
        pRef().width(0, 0.15, easeInCubic),
        pRef().height(0, 0.15, easeInCubic),
        pRef().opacity(0, 0.15),
        pRef().fill('#FFFFFF', 0.05),
      ),
    ),
  );

  // 3. Merge node pulses strong green
  yield* all(
    nodeSignals[mergeNodeId].fillColor(SIGNAL, 0.2),
    nodeSignals[mergeNodeId].radius(10, 0.2, easeOutExpo),
    nodeSignals[mergeNodeId].glowBlur(35, 0.2, easeOutExpo),
  );

  // 4. Paths restore to clean green
  yield* all(
    pathARef().stroke(SIGNAL, 0.3),
    pathBRef().stroke(SIGNAL, 0.3),
    ...noiseA.map(id => all(
      nodeSignals[id].fillColor(ASH, 0.4),
      nodeSignals[id].glowBlur(0, 0.4),
    )),
    ...noiseB.map(id => all(
      nodeSignals[id].fillColor(ASH, 0.4),
      nodeSignals[id].glowBlur(0, 0.4),
    )),
  );

  // 5. "signal survives" label at merge node
  const signalSurvives = createRef<Txt>();
  xlrContainer().add(
    <Txt
      ref={signalSurvives}
      text="signal survives"
      fontFamily={SANS}
      fontSize={13}
      fontWeight={600}
      fill={SIGNAL}
      x={mergeX}
      y={mergeY - 20}
      opacity={0}
    />,
  );
  yield* signalSurvives().opacity(1, 0.3, easeOutCubic);

  yield* waitFor(0.5);

  // Clean up scene 5
  yield* all(
    xlrTitle().opacity(0, 0.3, easeInCubic),
    xlrSubtitle().opacity(0, 0.3, easeInCubic),
    xlrContainer().opacity(0, 0.3, easeInCubic),
  );

  // Reset graph for next scene
  yield* all(
    ...GRAPH.nodes.map((_, i) =>
      all(
        nodeSignals[i].fillColor(ASH, 0.2),
        nodeSignals[i].glowBlur(0, 0.2),
        nodeSignals[i].radius(2.5, 0.2),
        nodeSignals[i].fillOpacity(0.4, 0.2),
      ),
    ),
  );

  // =========================================================================
  // HANDOFF POINT — Scenes 6-9 continue from here
  // =========================================================================
  // State: graph dimmed, all overlays removed, camera at zoom 1.0
  // The other agent appends scenes 6-9 below this line.
  yield* waitFor(0.1);
});
