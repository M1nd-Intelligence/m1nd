import { motion, AnimatePresence } from 'framer-motion';
import { useEffect, useState } from 'react';
import { COLORS, GLYPHS } from '../lib/colors';
import { TokenCounter } from '../components/TokenCounter';

/**
 * SCENE 3: THE BRAIN (Layer 2 part 2)
 *
 * Graph visualization. Nodes appearing. Signal propagating in 4D waves
 * (cyan/gold/magenta/blue matching M1ND). XLR noise cancellation (red noise fading).
 * Results ranking on the side. "31ms. 8 results. 0 LLM tokens for navigation."
 *
 * Emotion: AWE
 * Verified: 7 nodes in 31ms, measured on 335-file codebase with 9,767 nodes
 */

interface GraphNode {
  id: string;
  label: string;
  x: number; // percentage
  y: number; // percentage
  activation: number;
  dimensions: { M: number; one: number; N: number; D: number };
  type: string;
}

const NODES: GraphNode[] = [
  { id: 'auth', label: 'auth_handler.py', x: 50, y: 18, activation: 0.94, dimensions: { M: 0.97, one: 0.54, N: 0.80, D: 0.88 }, type: 'Module' },
  { id: 'mid', label: 'middleware.py', x: 22, y: 50, activation: 0.71, dimensions: { M: 0.82, one: 0.61, N: 0.45, D: 0.73 }, type: 'Module' },
  { id: 'ses', label: 'session.py', x: 78, y: 50, activation: 0.63, dimensions: { M: 0.75, one: 0.48, N: 0.39, D: 0.81 }, type: 'Module' },
  { id: 'cors', label: 'cors.py', x: 15, y: 80, activation: 0.46, dimensions: { M: 0.55, one: 0.32, N: 0.28, D: 0.62 }, type: 'Module' },
  { id: 'test', label: 'test_auth.py', x: 85, y: 80, activation: 0.41, dimensions: { M: 0.48, one: 0.71, N: 0.22, D: 0.57 }, type: 'Test' },
  { id: 'api', label: 'routes/api.py', x: 50, y: 82, activation: 0.38, dimensions: { M: 0.65, one: 0.29, N: 0.35, D: 0.44 }, type: 'Module' },
  { id: 'jwt', label: 'jwt.py', x: 42, y: 55, activation: 0.33, dimensions: { M: 0.42, one: 0.38, N: 0.51, D: 0.69 }, type: 'Module' },
];

interface GraphEdge {
  from: string;
  to: string;
  type: 'calls' | 'imports' | 'ghost';
}

const EDGES: GraphEdge[] = [
  { from: 'auth', to: 'mid', type: 'calls' },
  { from: 'auth', to: 'ses', type: 'calls' },
  { from: 'auth', to: 'jwt', type: 'imports' },
  { from: 'mid', to: 'cors', type: 'imports' },
  { from: 'mid', to: 'ses', type: 'ghost' }, // undocumented connection
  { from: 'ses', to: 'test', type: 'calls' },
  { from: 'auth', to: 'api', type: 'imports' },
  { from: 'jwt', to: 'api', type: 'calls' },
];

export function BrainScene() {
  const [phase, setPhase] = useState<'empty' | 'signal' | 'propagate' | 'ghost' | 'complete'>('empty');
  const [activeNodes, setActiveNodes] = useState<Set<string>>(new Set());
  const [activeEdges, setActiveEdges] = useState<Set<number>>(new Set());
  const [ghostRevealed, setGhostRevealed] = useState(false);
  const [showStats, setShowStats] = useState(false);
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);

  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Animation cascade
  useEffect(() => {
    if (prefersReducedMotion) {
      setActiveNodes(new Set(NODES.map(n => n.id)));
      setActiveEdges(new Set(EDGES.map((_, i) => i)));
      setGhostRevealed(true);
      setShowStats(true);
      setPhase('complete');
      return;
    }

    const timers: ReturnType<typeof setTimeout>[] = [];

    // Wave 1: Center node appears
    timers.push(setTimeout(() => {
      setPhase('signal');
      setActiveNodes(new Set(['auth']));
    }, 400));

    // Wave 2: Direct connections
    timers.push(setTimeout(() => {
      setPhase('propagate');
      setActiveNodes(new Set(['auth', 'mid', 'ses', 'jwt']));
      setActiveEdges(new Set([0, 1, 2])); // auth->mid, auth->ses, auth->jwt
    }, 1000));

    // Wave 3: Secondary connections
    timers.push(setTimeout(() => {
      setActiveNodes(new Set(['auth', 'mid', 'ses', 'jwt', 'cors', 'test', 'api']));
      setActiveEdges(new Set([0, 1, 2, 3, 5, 6, 7])); // all except ghost
    }, 1800));

    // Wave 4: Ghost edge reveal (dramatic)
    timers.push(setTimeout(() => {
      setPhase('ghost');
      setGhostRevealed(true);
      setActiveEdges(new Set([0, 1, 2, 3, 4, 5, 6, 7])); // now including ghost
    }, 2800));

    // Stats appear
    timers.push(setTimeout(() => {
      setShowStats(true);
      setPhase('complete');
    }, 3500));

    return () => timers.forEach(clearTimeout);
  }, [prefersReducedMotion]);

  const getNode = (id: string) => NODES.find(n => n.id === id)!;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 16,
        padding: '32px 48px',
        height: '100%',
      }}
    >
      {/* Header */}
      <div>
        <motion.div
          initial={{ x: -20, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          style={{ fontSize: 11, color: COLORS.one, letterSpacing: 3, fontFamily: 'monospace', marginBottom: 8 }}
        >
          {GLYPHS.structure} SCENE 3 -- THE BRAIN
        </motion.div>
        <motion.h2
          initial={{ y: 10, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.2 }}
          style={{ fontSize: 24, color: COLORS.text, fontWeight: 700, fontFamily: '"JetBrains Mono", monospace' }}
        >
          signal propagates through 4 dimensions.<br />
          <span style={{ color: COLORS.one }}>31ms. 7 nodes. 0 LLM tokens for navigation.</span>
        </motion.h2>
      </div>

      {/* Main: Graph + Results panel */}
      <div style={{ display: 'grid', gridTemplateColumns: '1.2fr 1fr', gap: 24, flex: 1, minHeight: 0 }}>
        {/* SVG Graph */}
        <div style={{
          position: 'relative',
          background: 'rgba(0, 0, 0, 0.5)',
          borderRadius: 12,
          border: `1px solid ${COLORS.border}`,
          overflow: 'hidden',
        }}>
          {/* Radial glow from center on activation */}
          {phase !== 'empty' && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 0.4 }}
              transition={{ duration: 1 }}
              style={{
                position: 'absolute',
                inset: 0,
                background: `radial-gradient(ellipse at 50% 18%, ${COLORS.M}15 0%, transparent 60%)`,
                pointerEvents: 'none',
              }}
            />
          )}

          <svg width="100%" height="100%" viewBox="0 0 100 100" preserveAspectRatio="xMidYMid meet">
            {/* Edges */}
            {EDGES.map((edge, i) => {
              const from = getNode(edge.from);
              const to = getNode(edge.to);
              const isActive = activeEdges.has(i);
              const isGhost = edge.type === 'ghost';

              if (!isActive) return null;

              return (
                <g key={`edge-${i}`}>
                  <line
                    x1={from.x} y1={from.y}
                    x2={to.x} y2={to.y}
                    stroke={isGhost ? '#FF00FF' : COLORS.M}
                    strokeWidth={isGhost ? 0.35 : 0.3}
                    strokeOpacity={isGhost ? 0.8 : 0.6}
                    strokeDasharray={isGhost ? '1.5 1' : undefined}
                    style={{
                      transition: 'all 0.5s ease',
                      filter: isGhost ? 'drop-shadow(0 0 2px #FF00FF)' : undefined,
                    }}
                  />
                  {/* Pulse traveling along edge */}
                  {isActive && !isGhost && (
                    <circle r="0.8" fill={COLORS.M} opacity={0.8}>
                      <animateMotion
                        dur="1.5s"
                        repeatCount="indefinite"
                        path={`M${from.x},${from.y} L${to.x},${to.y}`}
                      />
                    </circle>
                  )}
                </g>
              );
            })}

            {/* Ghost edge label */}
            {ghostRevealed && (
              <g>
                <rect x="27" y="47" width="26" height="5" rx="1" fill="rgba(0,0,0,0.7)" stroke="#FF00FF" strokeWidth="0.3" strokeOpacity="0.6" />
                <text x="40" y="50.5" textAnchor="middle" fill="#FF00FF" fontSize="3.2" fontFamily="monospace" opacity="0.9">
                  undocumented connection
                </text>
              </g>
            )}

            {/* Nodes */}
            {NODES.map(node => {
              const isActive = activeNodes.has(node.id);
              const isHovered = hoveredNode === node.id;
              const nodeColor = isActive
                ? (node.activation > 0.7 ? COLORS.M : node.activation > 0.5 ? COLORS.one : COLORS.D)
                : COLORS.textDim;

              return (
                <g
                  key={node.id}
                  style={{ cursor: 'pointer' }}
                  onMouseEnter={() => setHoveredNode(node.id)}
                  onMouseLeave={() => setHoveredNode(null)}
                >
                  {/* Activation ring */}
                  {isActive && (
                    <circle
                      cx={node.x} cy={node.y}
                      r={4 + node.activation * 2}
                      fill="none"
                      stroke={nodeColor}
                      strokeWidth={0.2}
                      opacity={0.2}
                    >
                      <animate
                        attributeName="r"
                        values={`${3 + node.activation * 2};${5 + node.activation * 2};${3 + node.activation * 2}`}
                        dur="2s"
                        repeatCount="indefinite"
                      />
                      <animate
                        attributeName="opacity"
                        values="0.3;0.1;0.3"
                        dur="2s"
                        repeatCount="indefinite"
                      />
                    </circle>
                  )}

                  {/* Node circle */}
                  <circle
                    cx={node.x} cy={node.y}
                    r={isActive ? (isHovered ? 3.5 : 2.8) : 1.5}
                    fill={isActive ? nodeColor : COLORS.textDim}
                    style={{
                      transition: 'all 0.4s ease',
                      filter: isActive ? `drop-shadow(0 0 4px ${nodeColor})` : 'none',
                    }}
                  />

                  {/* Node label */}
                  <text
                    x={node.x}
                    y={node.y + (node.y < 30 ? -5 : 6)}
                    textAnchor="middle"
                    fill={isActive ? nodeColor : COLORS.textDim}
                    fontSize="2.8"
                    fontFamily="monospace"
                    fontWeight={isActive ? 'bold' : 'normal'}
                    style={{ transition: 'fill 0.4s ease' }}
                  >
                    {node.label}
                  </text>

                  {/* Activation score */}
                  {isActive && (
                    <text
                      x={node.x}
                      y={node.y + (node.y < 30 ? -8 : 9)}
                      textAnchor="middle"
                      fill={nodeColor}
                      fontSize="2.2"
                      fontFamily="monospace"
                      opacity={0.8}
                    >
                      {node.activation.toFixed(2)}
                    </text>
                  )}

                  {/* Dimension breakdown on hover -- using brand colors */}
                  {isHovered && isActive && (
                    <g>
                      <rect
                        x={node.x - 14} y={node.y + 11}
                        width="28" height="12"
                        rx="1.5"
                        fill="rgba(0,0,0,0.85)"
                        stroke={nodeColor}
                        strokeWidth="0.3"
                      />
                      {[
                        { key: 'M', val: node.dimensions.M, color: COLORS.M },
                        { key: '1', val: node.dimensions.one, color: COLORS.one },
                        { key: 'N', val: node.dimensions.N, color: COLORS.N },
                        { key: 'D', val: node.dimensions.D, color: COLORS.D },
                      ].map((d, j) => (
                        <g key={d.key}>
                          <text
                            x={node.x - 12 + j * 7} y={node.y + 16}
                            fill={d.color} fontSize="2" fontFamily="monospace" fontWeight="bold"
                          >
                            {d.key}
                          </text>
                          <text
                            x={node.x - 12 + j * 7} y={node.y + 20}
                            fill={d.color} fontSize="1.8" fontFamily="monospace"
                          >
                            {d.val.toFixed(2)}
                          </text>
                        </g>
                      ))}
                    </g>
                  )}
                </g>
              );
            })}
          </svg>
        </div>

        {/* Right side: Results ranking + stats */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {/* Results list */}
          <div style={{
            fontSize: 11,
            color: COLORS.textMuted,
            letterSpacing: 2,
            fontFamily: 'monospace',
          }}>
            ACTIVATED NODES -- ranked by relevance
          </div>

          <div style={{ flex: 1, overflowY: 'auto' }}>
            {[...NODES]
              .sort((a, b) => b.activation - a.activation)
              .map((node, i) => {
                const isActive = activeNodes.has(node.id);
                return (
                  <motion.div
                    key={node.id}
                    initial={{ opacity: 0, x: 20 }}
                    animate={{ opacity: isActive ? 1 : 0.2, x: 0 }}
                    transition={{ delay: 0.3 + i * 0.08, duration: 0.3 }}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 10,
                      fontFamily: 'monospace',
                      fontSize: 12,
                      padding: '6px 0',
                      borderBottom: `1px solid ${COLORS.border}`,
                    }}
                  >
                    <div style={{
                      width: 8, height: 8, borderRadius: '50%',
                      background: isActive
                        ? (node.activation > 0.7 ? COLORS.M : node.activation > 0.5 ? COLORS.one : COLORS.D)
                        : COLORS.textDim,
                      flexShrink: 0,
                      boxShadow: isActive ? `0 0 6px ${COLORS.M}80` : 'none',
                    }} />
                    <div style={{ flex: 1, color: COLORS.text }}>{node.label}</div>
                    <div style={{ fontSize: 9, color: COLORS.textDim, width: 40 }}>{node.type}</div>
                    <div style={{
                      color: node.activation > 0.7 ? COLORS.M : node.activation > 0.5 ? COLORS.one : COLORS.D,
                      fontWeight: 700,
                      minWidth: 36,
                      textAlign: 'right',
                    }}>
                      {node.activation.toFixed(2)}
                    </div>
                    <div style={{
                      width: 50, height: 4, background: `${COLORS.bgCard}`, borderRadius: 2, overflow: 'hidden',
                    }}>
                      <motion.div
                        initial={{ width: 0 }}
                        animate={{ width: isActive ? `${node.activation * 100}%` : 0 }}
                        transition={{ delay: 0.5 + i * 0.08, duration: 0.5 }}
                        style={{
                          height: '100%',
                          background: node.activation > 0.7 ? COLORS.M : node.activation > 0.5 ? COLORS.one : COLORS.D,
                          borderRadius: 2,
                        }}
                      />
                    </div>
                  </motion.div>
                );
              })}
          </div>

          {/* Ghost edge alert */}
          <AnimatePresence>
            {ghostRevealed && (
              <motion.div
                initial={{ opacity: 0, scale: 0.95, y: 10 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                transition={{ type: 'spring', stiffness: 200 }}
                style={{
                  background: 'rgba(255, 0, 255, 0.08)',
                  border: '1px solid rgba(255, 0, 255, 0.3)',
                  borderRadius: 8,
                  padding: '10px 14px',
                  fontFamily: 'monospace',
                  fontSize: 11,
                }}
              >
                <div style={{ color: '#FF00FF', fontWeight: 700, marginBottom: 4 }}>
                  {GLYPHS.structure} GHOST EDGE DETECTED
                </div>
                <div style={{ color: COLORS.textMuted }}>
                  middleware.py {'\u2192'} session.py
                </div>
                <div style={{ color: COLORS.textDim, fontSize: 10 }}>
                  undocumented runtime dependency found automatically
                </div>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Token counter: frozen at zero */}
          {showStats && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.3 }}
            >
              <TokenCounter
                label="LLM tokens"
                targetValue={0}
                color={COLORS.D}
                frozen
              />
            </motion.div>
          )}
        </div>
      </div>
    </motion.div>
  );
}
