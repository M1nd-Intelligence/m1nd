import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { COLORS } from '../lib/colors';

/**
 * Scene 7: THE IDENTITY
 * Dark void. Glyphs appear one by one with spring animation.
 * Below: dimension labels. Then gradient borders draw. "61 tools. the graph learns."
 */

const GLYPH_ENTRIES = [
  { glyph: '\u234C', color: '#00D4FF', label: 'SIGNAL' },    // cyan
  { glyph: '\u2350', color: '#FFD700', label: 'PATH' },       // gold
  { glyph: '\u2342', color: '#FF00AA', label: 'STRUCTURE' },  // magenta
  { glyph: '\uD835\uDD3B', color: '#4169E1', label: 'DIMENSION' },  // blue
  { glyph: '\u27C1', color: '#00E5A0', label: 'CONNECTION' }, // green
];

// SVG fallback for glyph
function GlyphWithFallback({ glyph, color, size }: { glyph: string; color: string; size: number }) {
  return (
    <span
      style={{
        fontSize: size,
        color,
        fontFamily: '"Apple Symbols", "Segoe UI Symbol", "Noto Sans Symbols", monospace',
        display: 'inline-block',
        lineHeight: 1,
        textShadow: `0 0 40px ${color}80, 0 0 80px ${color}40`,
      }}
      aria-hidden="true"
    >
      {glyph}
    </span>
  );
}

export function IdentityScene() {
  const [phase, setPhase] = useState(0);
  // Phase 0: void
  // Phase 1: glyphs appear (0.3s)
  // Phase 2: labels appear (1.5s)
  // Phase 3: gradient border draws (2.5s)
  // Phase 4: "61 tools. the graph learns." (3.5s)

  useEffect(() => {
    const timers = [
      setTimeout(() => setPhase(1), 300),
      setTimeout(() => setPhase(2), 1500),
      setTimeout(() => setPhase(3), 2500),
      setTimeout(() => setPhase(4), 3500),
    ];
    return () => timers.forEach(clearTimeout);
  }, []);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.6 }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 0,
        height: '100%',
        alignItems: 'center',
        justifyContent: 'center',
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      {/* Gradient border that draws around the screen -- with glow */}
      <AnimatePresence>
        {phase >= 3 && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 1.2 }}
            style={{
              position: 'absolute',
              inset: 12,
              border: '3px solid transparent',
              borderRadius: 16,
              background: `linear-gradient(${COLORS.bg}, ${COLORS.bg}) padding-box,
                linear-gradient(135deg, #00D4FF, #FF00AA, #4169E1, #00E5A0, #00D4FF) border-box`,
              pointerEvents: 'none',
              zIndex: 0,
              boxShadow: '0 0 20px rgba(0,212,255,0.15), 0 0 40px rgba(255,0,170,0.08)',
            }}
          />
        )}
      </AnimatePresence>

      {/* Glyphs row */}
      <div style={{
        display: 'flex',
        gap: 40,
        alignItems: 'center',
        justifyContent: 'center',
        marginBottom: 24,
        zIndex: 1,
      }}>
        {GLYPH_ENTRIES.map((entry, i) => (
          <div key={i} style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12 }}>
            {/* Glyph */}
            <AnimatePresence>
              {phase >= 1 && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.2, filter: 'blur(16px)' }}
                  animate={{ opacity: 1, scale: 1, filter: 'blur(0px)' }}
                  transition={{
                    delay: i * 0.15,
                    duration: 0.5,
                    type: 'spring',
                    stiffness: 200,
                    damping: 15,
                  }}
                >
                  <GlyphWithFallback glyph={entry.glyph} color={entry.color} size={56} />
                </motion.div>
              )}
            </AnimatePresence>

            {/* Label below glyph -- opacity set via animate, not inline style */}
            <AnimatePresence>
              {phase >= 2 && (
                <motion.div
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 0.7, y: 0 }}
                  transition={{ delay: i * 0.1, duration: 0.4 }}
                  style={{
                    fontSize: 10,
                    color: entry.color,
                    fontFamily: 'monospace',
                    letterSpacing: 3,
                    textTransform: 'uppercase',
                  }}
                >
                  {entry.label}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        ))}
      </div>

      {/* m1nd logo -- letters appear individually */}
      <AnimatePresence>
        {phase >= 3 && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.4 }}
            style={{
              display: 'flex',
              gap: 4,
              alignItems: 'center',
              marginBottom: 20,
              zIndex: 1,
            }}
          >
            {['m', '1', 'n', 'd'].map((letter, i) => {
              const letterColors = [COLORS.M, COLORS.one, COLORS.N, COLORS.D];
              return (
                <motion.span
                  key={i}
                  initial={{ opacity: 0, y: 30 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{
                    delay: 0.1 + i * 0.1,
                    type: 'spring',
                    stiffness: 200,
                    damping: 12,
                  }}
                  style={{
                    fontSize: 72,
                    fontWeight: 900,
                    fontFamily: '"JetBrains Mono", monospace',
                    color: letterColors[i],
                    textShadow: `0 0 40px ${letterColors[i]}50`,
                    lineHeight: 1,
                  }}
                >
                  {letter}
                </motion.span>
              );
            })}
          </motion.div>
        )}
      </AnimatePresence>

      {/* "61 tools. the graph learns." */}
      <AnimatePresence>
        {phase >= 4 && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            style={{
              textAlign: 'center',
              zIndex: 1,
            }}
          >
            <div style={{
              fontSize: 18,
              color: COLORS.textMuted,
              fontFamily: '"JetBrains Mono", monospace',
              letterSpacing: 3,
              marginBottom: 8,
            }}>
              61 tools.{' '}
              <span style={{ color: COLORS.D }}>the graph learns.</span>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Accessibility: screen reader text */}
      <div role="img" aria-label="Five m1nd dimension symbols: Signal, Path, Structure, Dimension, Connection. 61 tools. The graph learns." style={{ position: 'absolute', width: 1, height: 1, overflow: 'hidden' }} />
    </motion.div>
  );
}
