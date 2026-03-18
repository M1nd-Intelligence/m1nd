import {makeScene2D, Circle, Line, Txt, Rect, Node} from '@motion-canvas/2d';
import {
  all,
  chain,
  createRef,
  createSignal,
  delay,
  easeInOutCubic,
  easeInOutQuad,
  easeOutCubic,
  easeOutExpo,
  easeInCubic,
  easeOutBack,
  linear,
  loop,
  sequence,
  waitFor,
  createRefArray,
  tween,
  spring,
  BeatSpring,
  PlopSpring,
  spawn,
  run,
} from '@motion-canvas/core';

// ---------------------------------------------------------------------------
// CONSTANTS (shared palette with brain.tsx)
// ---------------------------------------------------------------------------
const BG          = '#060B14';
const PRIMARY     = '#00E5A0';
const ACCENT      = '#00B4D8';
const ERROR       = '#EF4444';
const WARN        = '#F59E0B';
const DIM         = '#64748B';
const DIM_EDGE    = '#1E293B';
const WHITE       = '#E2E8F0';
const BONE        = '#E2E8F0';
const ASH         = '#64748B';
const GHOST       = '#6366F1';
const COST_RED    = '#FF2D55';
const GRAPHITE    = '#1E293B';
const CARD_BG     = '#0F172A';
const FONT        = 'Inter, system-ui, sans-serif';
const MONO        = 'JetBrains Mono, Fira Code, monospace';

// Dimension colours
const COL_STRUCTURAL = PRIMARY;
const COL_SEMANTIC   = ACCENT;
const COL_TEMPORAL   = WARN;

// ---------------------------------------------------------------------------
// GRAPH DATA (same generator as brain.tsx for visual continuity)
// ---------------------------------------------------------------------------
interface GNode { x: number; y: number; layer: number; id: number }
interface GEdge { from: number; to: number }

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

  const layers = 5;
  const nodesPerLayer = Math.ceil(nodeCount / layers);

  for (let layer = 0; layer < layers; layer++) {
    const count = layer === layers - 1
      ? nodeCount - nodes.length
      : nodesPerLayer;
    const radiusX = 120 + layer * 140;
    const radiusY = 80 + layer * 100;

    for (let i = 0; i < count; i++) {
      const angle = (i / count) * Math.PI * 2 + rng() * 0.6;
      const jitterX = (rng() - 0.5) * 60;
      const jitterY = (rng() - 0.5) * 40;
      nodes.push({
        x: Math.cos(angle) * radiusX + jitterX,
        y: Math.sin(angle) * radiusY + jitterY,
        layer,
        id: nodes.length,
      });
    }
  }

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

function adjacencyMap(graph: { nodes: GNode[]; edges: GEdge[] }): Map<number, number[]> {
  const adj = new Map<number, number[]>();
  for (const n of graph.nodes) adj.set(n.id, []);
  for (const e of graph.edges) {
    adj.get(e.from)!.push(e.to);
    adj.get(e.to)!.push(e.from);
  }
  return adj;
}

const ADJ = adjacencyMap(GRAPH);

// ---------------------------------------------------------------------------
// SCENE
// ---------------------------------------------------------------------------
export default makeScene2D(function* (view) {
  view.fill(BG);

  // =======================================================================
  // CONTAINERS
  // =======================================================================
  const graphContainer = createRef<Node>();
  const uiContainer    = createRef<Node>();

  view.add(<Node ref={graphContainer} opacity={0} />);
  view.add(<Node ref={uiContainer} />);

  // =======================================================================
  // GRAPH NODES + EDGES (dimmed backdrop)
  // =======================================================================
  const nodeSignals = GRAPH.nodes.map(() => ({
    fillOpacity: createSignal(0.35),
    fillColor:   createSignal(DIM),
    radius:      createSignal(2.5),
    glowBlur:    createSignal(0),
  }));

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

  const edgeSignals = GRAPH.edges.map(() => ({
    progress:      createSignal(1),
    strokeColor:   createSignal(DIM_EDGE),
    strokeWidth:   createSignal(0.5),
    strokeOpacity: createSignal(0.25),
  }));

  for (let i = 0; i < GRAPH.edges.length; i++) {
    const e = GRAPH.edges[i];
    const fromN = GRAPH.nodes[e.from];
    const toN   = GRAPH.nodes[e.to];
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

  // Show background graph at low opacity
  graphContainer().opacity(0.15);

  // Reset all graph visuals to dim state
  function resetGraph() {
    for (const s of nodeSignals) {
      s.fillColor(DIM);
      s.radius(2.5);
      s.glowBlur(0);
      s.fillOpacity(0.35);
    }
    for (const s of edgeSignals) {
      s.strokeColor(DIM_EDGE);
      s.strokeWidth(0.5);
      s.strokeOpacity(0.25);
    }
  }

  // =========================================================================
  // SCENE 5: HYPOTHESIZE (0:21 - 0:25, ~4s)
  // =========================================================================

  // Brief darkness
  yield* waitFor(0.3);

  // Fade graph up slightly for context
  yield* graphContainer().opacity(0.15, 0.3, easeOutCubic);

  // Title
  const hypTitle = createRef<Txt>();
  const hypSubtitle = createRef<Txt>();
  uiContainer().add(
    <>
      <Txt
        ref={hypTitle}
        text="HYPOTHESIZE"
        fontSize={28}
        fontFamily={FONT}
        fontWeight={700}
        fill={BONE}
        y={-440}
        opacity={0}
      />
      <Txt
        ref={hypSubtitle}
        text={'is authentication connected to rate_limiter?'}
        fontSize={16}
        fontFamily={FONT}
        fontWeight={400}
        fontStyle={'italic'}
        fill={ASH}
        y={-400}
        opacity={0}
      />
    </>,
  );

  yield* chain(
    hypTitle().opacity(1, 0.4, easeOutCubic),
    hypSubtitle().opacity(1, 0.3, easeOutCubic),
  );

  // Source and target nodes
  const hypSource = 0;
  const hypTarget = GRAPH.nodes.length - 1;

  // Light up source (green) and target (blue)
  yield* all(
    graphContainer().opacity(0.25, 0.3),
    nodeSignals[hypSource].fillColor(PRIMARY, 0.3),
    nodeSignals[hypSource].radius(8, 0.3, easeOutExpo),
    nodeSignals[hypSource].glowBlur(25, 0.3),
    nodeSignals[hypSource].fillOpacity(1, 0.3),
    nodeSignals[hypTarget].fillColor(ACCENT, 0.3),
    nodeSignals[hypTarget].radius(8, 0.3, easeOutExpo),
    nodeSignals[hypTarget].glowBlur(25, 0.3),
    nodeSignals[hypTarget].fillOpacity(1, 0.3),
  );

  // Exploration paths
  const explorerContainer = createRef<Node>();
  graphContainer().add(<Node ref={explorerContainer} opacity={1} />);

  const rng = seededRandom(99);
  const explorationPaths: [number, number][][] = [];
  for (let p = 0; p < 12; p++) {
    const path: [number, number][] = [[GRAPH.nodes[hypSource].x, GRAPH.nodes[hypSource].y]];
    let current = hypSource;
    const steps = 3 + Math.floor(rng() * 3);
    for (let s = 0; s < steps; s++) {
      const neighbors = ADJ.get(current) ?? [];
      if (neighbors.length === 0) break;
      const next = neighbors[Math.floor(rng() * neighbors.length)];
      path.push([GRAPH.nodes[next].x, GRAPH.nodes[next].y]);
      current = next;
    }
    // First 3 paths reach the target
    if (p < 3) {
      path.push([GRAPH.nodes[hypTarget].x, GRAPH.nodes[hypTarget].y]);
    }
    explorationPaths.push(path);
  }

  // Create path lines
  const pathRefs: ReturnType<typeof createRef<Line>>[] = [];
  const winnerColors = [PRIMARY, ACCENT, WARN];
  for (let p = 0; p < explorationPaths.length; p++) {
    const pRef = createRef<Line>();
    pathRefs.push(pRef);
    const reachesTarget = p < 3;
    explorerContainer().add(
      <Line
        ref={pRef}
        points={explorationPaths[p]}
        stroke={reachesTarget ? ASH : ASH}
        lineWidth={1}
        opacity={0}
        end={0}
        lineDash={[4, 3]}
      />,
    );
  }

  // All 12 paths fan out simultaneously
  yield* sequence(
    0.05,
    ...pathRefs.map(pRef =>
      all(
        pRef().opacity(0.4, 0.1),
        pRef().end(1, 0.4, easeInOutCubic),
      ),
    ),
  );

  yield* waitFor(0.2);

  // Dead-end paths fade red then disappear
  yield* all(
    ...pathRefs.slice(3).map(pRef =>
      chain(
        pRef().stroke(ERROR, 0.2),
        all(
          pRef().opacity(0, 0.4, easeInCubic),
        ),
      ),
    ),
  );

  // 3 winning paths glow and thicken with dimension colors
  yield* all(
    ...pathRefs.slice(0, 3).map((pRef, i) =>
      all(
        pRef().stroke(winnerColors[i], 0.3),
        pRef().lineWidth(2.5, 0.3),
        pRef().opacity(0.9, 0.3),
        pRef().lineDash([], 0),
      ),
    ),
  );

  // Light up nodes along winning paths softly
  yield* waitFor(0.2);

  // Verdict card slams on screen
  const verdictCard = createRef<Rect>();
  const verdictText = createRef<Txt>();
  const verdictConf = createRef<Txt>();
  const verdictStat = createRef<Txt>();

  uiContainer().add(
    <Rect
      ref={verdictCard}
      width={420}
      height={80}
      radius={12}
      fill={CARD_BG}
      stroke={GRAPHITE}
      lineWidth={1}
      y={380}
      opacity={0}
      scale={0.8}
    >
      <Node y={-12}>
        <Txt
          ref={verdictText}
          text="likely_true"
          fontSize={20}
          fontFamily={FONT}
          fontWeight={700}
          fill={PRIMARY}
          x={-60}
        />
        <Txt
          text=" — "
          fontSize={20}
          fontFamily={FONT}
          fontWeight={400}
          fill={ASH}
          x={20}
        />
        <Txt
          ref={verdictConf}
          text="87%"
          fontSize={28}
          fontFamily={FONT}
          fontWeight={800}
          fill={PRIMARY}
          x={80}
          shadowColor={PRIMARY}
          shadowBlur={0}
        />
        <Txt
          text=" confidence"
          fontSize={16}
          fontFamily={FONT}
          fontWeight={400}
          fill={ASH}
          x={160}
        />
      </Node>
      <Txt
        ref={verdictStat}
        text="25,015 paths explored in 58ms"
        fontSize={14}
        fontFamily={FONT}
        fontWeight={400}
        fill={ASH}
        y={20}
      />
    </Rect>,
  );

  // SLAM the verdict card in with BeatSpring
  yield* spring(
    BeatSpring,
    0, 1, 0.001,
    (value) => {
      verdictCard().scale(0.8 + value * 0.2);
      verdictCard().opacity(Math.min(1, value * 1.5));
    },
  );

  // Glow on the 87% number
  yield* verdictConf().shadowBlur(20, 0.4, easeOutExpo);

  yield* waitFor(0.6);

  // Clean up scene 5
  yield* all(
    hypTitle().opacity(0, 0.3, easeInCubic),
    hypSubtitle().opacity(0, 0.3, easeInCubic),
    verdictCard().opacity(0, 0.3, easeInCubic),
    explorerContainer().opacity(0, 0.3, easeInCubic),
  );
  resetGraph();

  // =========================================================================
  // SCENE 6: THE INVISIBLE (0:25 - 0:29, ~4s)
  // =========================================================================

  yield* waitFor(0.3);

  // Dim graph stays as backdrop
  yield* graphContainer().opacity(0.08, 0.3);

  // Bug icons appear around the graph
  const bugContainer = createRef<Node>();
  uiContainer().add(<Node ref={bugContainer} opacity={0} />);

  const bugLabels = [
    'TOCTOU race condition',
    'shutdown flag unchecked',
    'orphan storm cascade',
    'concurrent materialize',
    'dead websocket fire-and-forget',
    'duplicate storm spawn',
    'restart race condition',
    'command injection',
  ];

  // Position bugs around the graph in a rough ring
  const bugPositions: [number, number][] = [
    [-420, -220], [420, -220],
    [-400, -80],  [400, -80],
    [-420,  80],  [420,  80],
    [-400, 220],  [400, 220],
  ];

  const bugRefs: ReturnType<typeof createRef<Node>>[] = [];
  const bugIconRefs: ReturnType<typeof createRef<Circle>>[] = [];
  const bugTxtRefs: ReturnType<typeof createRef<Txt>>[] = [];

  for (let i = 0; i < bugLabels.length; i++) {
    const bRef = createRef<Node>();
    const iRef = createRef<Circle>();
    const tRef = createRef<Txt>();
    bugRefs.push(bRef);
    bugIconRefs.push(iRef);
    bugTxtRefs.push(tRef);

    const isLeft = bugPositions[i][0] < 0;

    bugContainer().add(
      <Node ref={bRef} x={bugPositions[i][0]} y={bugPositions[i][1]} opacity={0}>
        <Circle
          ref={iRef}
          size={12}
          fill={ERROR}
          shadowColor={ERROR}
          shadowBlur={8}
          x={isLeft ? -10 : 10}
        />
        <Txt
          ref={tRef}
          text={bugLabels[i]}
          fontSize={13}
          fontFamily={MONO}
          fontWeight={400}
          fill={BONE}
          letterSpacing={0.3}
          x={isLeft ? 15 : -15}
          textAlign={isLeft ? 'left' : 'right'}
        />
      </Node>,
    );
  }

  yield* bugContainer().opacity(1, 0.2);

  // Bugs appear one by one with a SLAM stagger
  yield* sequence(
    0.3,
    ...bugRefs.map((bRef, _i) =>
      all(
        bRef().opacity(1, 0.25, easeOutExpo),
        bRef().y(bRef().y() - 6, 0),
        bRef().y(bRef().y() + 6, 0.25, easeOutCubic),
      ),
    ),
  );

  yield* waitFor(0.3);

  // Bottom text: the kill line
  const invisibleText = createRef<Txt>();
  uiContainer().add(
    <Txt
      ref={invisibleText}
      text="8 bugs. no keyword. no string. just structure."
      fontSize={18}
      fontFamily={FONT}
      fontWeight={600}
      fill={BONE}
      y={380}
      opacity={0}
      letterSpacing={0.5}
    />,
  );

  yield* invisibleText().opacity(1, 0.4, easeOutCubic);
  yield* waitFor(0.8);

  // Clean up scene 6
  yield* all(
    bugContainer().opacity(0, 0.3, easeInCubic),
    invisibleText().opacity(0, 0.3, easeInCubic),
  );

  // =========================================================================
  // SCENE 7: THE COMPARISON (0:29 - 0:32, ~3s)
  // =========================================================================

  yield* waitFor(0.3);

  // Full dark backdrop
  yield* graphContainer().opacity(0.04, 0.2);

  const tableContainer = createRef<Node>();
  uiContainer().add(<Node ref={tableContainer} opacity={0} />);

  // Table header row
  const headerM1nd = createRef<Txt>();
  const headerGrep = createRef<Txt>();
  const headerSep  = createRef<Line>();

  // Comparison rows data (per storyboard mission brief)
  const compRows = [
    { metric: 'time:',      m1nd: '1.9s',     grep: '~35 min',     m1ndColor: PRIMARY, grepColor: ERROR   },
    { metric: 'tokens:',    m1nd: '0',         grep: '~193,000',    m1ndColor: PRIMARY, grepColor: ERROR   },
    { metric: 'cost:',      m1nd: '$0.00',     grep: '~$7.23',      m1ndColor: PRIMARY, grepColor: ERROR   },
    { metric: 'bugs:',      m1nd: '39',        grep: '~23',         m1ndColor: PRIMARY, grepColor: WARN    },
    { metric: 'invisible:', m1nd: '8',         grep: '0',           m1ndColor: PRIMARY, grepColor: ERROR   },
  ];

  const ROW_H = 52;
  const TABLE_TOP = -130;
  const COL_METRIC = -280;
  const COL_M1ND   = 50;
  const COL_GREP   = 280;

  // Headers
  tableContainer().add(
    <>
      <Txt
        text="m1nd"
        fontSize={22}
        fontFamily={FONT}
        fontWeight={700}
        fill={PRIMARY}
        x={COL_M1ND}
        y={TABLE_TOP - 40}
        shadowColor={PRIMARY}
        shadowBlur={10}
      />
      <Txt
        text="LLM + grep"
        fontSize={22}
        fontFamily={FONT}
        fontWeight={700}
        fill={ERROR}
        x={COL_GREP}
        y={TABLE_TOP - 40}
      />
      <Line
        ref={headerSep}
        points={[[COL_METRIC - 40, TABLE_TOP - 15], [COL_GREP + 140, TABLE_TOP - 15]]}
        stroke={GRAPHITE}
        lineWidth={1}
        end={0}
      />
    </>,
  );

  // Create rows (initially invisible)
  const rowRefs: {
    metricRef: ReturnType<typeof createRef<Txt>>;
    m1ndRef: ReturnType<typeof createRef<Txt>>;
    grepRef: ReturnType<typeof createRef<Txt>>;
  }[] = [];

  for (let i = 0; i < compRows.length; i++) {
    const row = compRows[i];
    const mRef = createRef<Txt>();
    const nRef = createRef<Txt>();
    const gRef = createRef<Txt>();
    rowRefs.push({ metricRef: mRef, m1ndRef: nRef, grepRef: gRef });

    const rowY = TABLE_TOP + i * ROW_H;
    const isLastRow = i === compRows.length - 1;
    const m1ndFontSize = isLastRow ? 32 : (row.m1nd === '0' ? 28 : 20);

    tableContainer().add(
      <>
        <Txt
          ref={mRef}
          text={row.metric}
          fontSize={16}
          fontFamily={MONO}
          fontWeight={400}
          fill={ASH}
          x={COL_METRIC}
          y={rowY}
          opacity={0}
        />
        <Txt
          ref={nRef}
          text={row.m1nd}
          fontSize={m1ndFontSize}
          fontFamily={FONT}
          fontWeight={700}
          fill={row.m1ndColor}
          x={COL_M1ND}
          y={rowY}
          opacity={0}
          shadowColor={row.m1ndColor}
          shadowBlur={0}
        />
        <Txt
          ref={gRef}
          text={row.grep}
          fontSize={18}
          fontFamily={FONT}
          fontWeight={700}
          fill={row.grepColor}
          x={COL_GREP}
          y={rowY}
          opacity={0}
        />
      </>,
    );
  }

  // Animate table entrance
  yield* tableContainer().opacity(1, 0.2);

  // Header separator draws
  yield* headerSep().end(1, 0.3, easeInOutCubic);

  // Rows slam in one by one
  for (let i = 0; i < compRows.length; i++) {
    const { metricRef, m1ndRef, grepRef } = rowRefs[i];
    const isLastRow = i === compRows.length - 1;

    // Metric name fades in
    yield* metricRef().opacity(1, 0.15, easeOutCubic);

    // M1nd value SLAMS in
    yield* all(
      m1ndRef().opacity(1, 0.15, easeOutExpo),
      m1ndRef().scale(1.2, 0),
      m1ndRef().scale(1.0, 0.2, easeOutExpo),
    );

    // Grep value appears 0.1s after (the micro-tension)
    yield* delay(0.1,
      all(
        grepRef().opacity(1, 0.15, easeOutExpo),
        grepRef().scale(1.1, 0),
        grepRef().scale(1.0, 0.15, easeOutCubic),
      ),
    );

    // Special treatment for the final row: harder slam + glow
    if (isLastRow) {
      yield* all(
        m1ndRef().shadowBlur(25, 0.2, easeOutExpo),
        m1ndRef().scale(1.15, 0.1, easeOutExpo),
      );
      yield* all(
        m1ndRef().shadowBlur(12, 0.3, easeInOutCubic),
        m1ndRef().scale(1.0, 0.3, easeInOutCubic),
      );
    }

    // Brief pause between rows (more for last row)
    yield* waitFor(isLastRow ? 0.15 : 0.08);
  }

  // Hold the table -- let it sink in
  yield* waitFor(1.0);

  // Clean up scene 7
  yield* tableContainer().opacity(0, 0.4, easeInCubic);

  // =========================================================================
  // SCENE 8: LEARN (0:32 - 0:34, ~2s)
  // =========================================================================

  yield* waitFor(0.3);

  // Bring graph back
  resetGraph();
  for (const s of nodeSignals) {
    s.fillOpacity(0.6);
    s.fillColor(DIM);
  }
  for (const s of edgeSignals) {
    s.strokeOpacity(0.4);
  }

  yield* graphContainer().opacity(0.7, 0.3, easeOutCubic);

  // Title
  const learnTitle = createRef<Txt>();
  const learnSub = createRef<Txt>();
  uiContainer().add(
    <>
      <Txt
        ref={learnTitle}
        text="LEARN"
        fontSize={28}
        fontFamily={FONT}
        fontWeight={700}
        fill={BONE}
        y={-440}
        opacity={0}
      />
      <Txt
        ref={learnSub}
        text="the graph learns. next query is smarter."
        fontSize={18}
        fontFamily={FONT}
        fontWeight={500}
        fill={BONE}
        y={420}
        opacity={0}
      />
    </>,
  );

  yield* learnTitle().opacity(1, 0.3, easeOutCubic);

  // LTP: select edges to strengthen (green glow intensifies)
  const ltpEdges = GRAPH.edges
    .map((_, i) => i)
    .filter((_, i) => i % 4 === 0)
    .slice(0, 12);

  // LTD: edges that thin and fade
  const ltdEdges = GRAPH.edges
    .map((_, i) => i)
    .filter((_, i) => i % 7 === 1)
    .slice(0, 8);

  // LTP edges thicken + glow green
  yield* sequence(
    0.05,
    ...ltpEdges.map(i => all(
      edgeSignals[i].strokeWidth(3.5, 0.35, easeOutCubic),
      edgeSignals[i].strokeColor(PRIMARY, 0.35),
      edgeSignals[i].strokeOpacity(0.8, 0.35),
    )),
  );

  // Nodes at LTP endpoints glow
  const ltpNodeSet = new Set<number>();
  for (const i of ltpEdges) {
    ltpNodeSet.add(GRAPH.edges[i].from);
    ltpNodeSet.add(GRAPH.edges[i].to);
  }
  yield* all(
    ...[...ltpNodeSet].map(id => all(
      nodeSignals[id].fillColor(PRIMARY, 0.3),
      nodeSignals[id].glowBlur(10, 0.3),
      nodeSignals[id].radius(5, 0.3),
      nodeSignals[id].fillOpacity(1, 0.3),
    )),
  );

  // LTD edges thin + fade
  yield* sequence(
    0.05,
    ...ltdEdges.map(i => all(
      edgeSignals[i].strokeColor(ERROR, 0.25),
      edgeSignals[i].strokeWidth(0.3, 0.4, easeInCubic),
      edgeSignals[i].strokeOpacity(0.1, 0.4),
    )),
  );

  // Show learn text
  yield* learnSub().opacity(1, 0.3, easeOutCubic);
  yield* waitFor(0.6);

  // Clean up scene 8
  yield* all(
    learnTitle().opacity(0, 0.3, easeInCubic),
    learnSub().opacity(0, 0.3, easeInCubic),
  );

  // =========================================================================
  // SCENE 9: FINALE (0:34 - 0:37, ~3s)
  // =========================================================================

  // Dim graph to silhouette
  for (const s of nodeSignals) {
    s.fillColor(DIM);
    s.glowBlur(0);
    s.radius(2.5);
    s.fillOpacity(0.15);
  }
  for (const s of edgeSignals) {
    s.strokeColor(DIM_EDGE);
    s.strokeWidth(0.5);
    s.strokeOpacity(0.1);
  }

  yield* graphContainer().opacity(0.04, 0.4, easeInCubic);

  yield* waitFor(0.3);

  // Logo
  const logoRef   = createRef<Txt>();
  const toolsRef  = createRef<Txt>();
  const urlRef    = createRef<Txt>();

  uiContainer().add(
    <>
      <Txt
        ref={logoRef}
        text="m1nd"
        fontSize={80}
        fontFamily={FONT}
        fontWeight={900}
        fill={PRIMARY}
        y={-40}
        opacity={0}
        scale={1.3}
        shadowColor={PRIMARY}
        shadowBlur={40}
        letterSpacing={-3}
      />
      <Txt
        ref={toolsRef}
        text="52 tools. zero tokens. pure rust."
        fontSize={20}
        fontFamily={FONT}
        fontWeight={500}
        fill={BONE}
        y={40}
        opacity={0}
      />
      <Txt
        ref={urlRef}
        text="github.com/maxkle1nz/m1nd"
        fontSize={16}
        fontFamily={FONT}
        fontWeight={400}
        fill={ASH}
        y={85}
        opacity={0}
      />
    </>,
  );

  // Logo SLAMS in
  yield* all(
    logoRef().opacity(1, 0.3, easeOutExpo),
    logoRef().scale(1.0, 0.4, easeOutExpo),
  );

  // Settle glow
  yield* logoRef().shadowBlur(25, 0.3, easeOutCubic);

  // Tagline fades in
  yield* toolsRef().opacity(1, 0.4, easeOutCubic);

  // URL fades in
  yield* urlRef().opacity(1, 0.3, easeOutCubic);

  // Background graph heartbeat: all nodes pulse once in unison
  yield* graphContainer().opacity(0.06, 0.3);

  // One unified pulse
  yield* all(
    ...GRAPH.nodes.map((_, i) => all(
      nodeSignals[i].fillColor(PRIMARY, 0.5),
      nodeSignals[i].glowBlur(8, 0.5, easeInOutQuad),
      nodeSignals[i].fillOpacity(0.3, 0.5),
    )),
  );

  yield* all(
    ...GRAPH.nodes.map((_, i) => all(
      nodeSignals[i].glowBlur(0, 0.8, easeInOutQuad),
      nodeSignals[i].fillOpacity(0.1, 0.8),
    )),
  );

  // Hold for the loop point
  yield* waitFor(1.5);
});
