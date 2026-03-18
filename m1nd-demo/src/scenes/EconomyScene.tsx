import { useEffect, useState, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { COLORS, GLYPHS } from '../lib/colors';

/**
 * Scene 5: THE ECONOMY
 * Split screen. Left: grep token counter RACING. Right: m1nd counter FROZEN.
 * The contrast IS the story.
 */

function AnimatedCost({ target, speed, color, label }: {
  target: number;
  speed: number;
  color: string;
  label: string;
}) {
  const [value, setValue] = useState(0);
  const rafRef = useRef<number>(0);
  const startRef = useRef<number>(0);

  useEffect(() => {
    startRef.current = performance.now();
    const animate = (now: number) => {
      const elapsed = now - startRef.current;
      const progress = Math.min(elapsed / (speed * 1000), 1);
      // Ease-out cubic
      const eased = 1 - Math.pow(1 - progress, 3);
      setValue(Math.floor(eased * target));
      if (progress < 1) {
        rafRef.current = requestAnimationFrame(animate);
      }
    };
    rafRef.current = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(rafRef.current);
  }, [target, speed]);

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      gap: 8,
      padding: '24px 36px',
      background: `${color}08`,
      border: `1px solid ${color}25`,
      borderRadius: 12,
      minWidth: 240,
    }}>
      <div style={{
        fontFamily: '"JetBrains Mono", monospace',
        fontSize: 48,
        fontWeight: 700,
        color,
        letterSpacing: 2,
        fontVariantNumeric: 'tabular-nums',
      }}>
        ${(value / 100).toFixed(2)}
      </div>
      <div style={{
        fontSize: 11,
        color: COLORS.textMuted,
        textTransform: 'uppercase',
        letterSpacing: 3,
        fontFamily: 'monospace',
      }}>
        {label}
      </div>
    </div>
  );
}

function FrozenCounter() {
  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      gap: 8,
      padding: '24px 36px',
      background: `${COLORS.D}08`,
      border: `1px solid ${COLORS.D}25`,
      borderRadius: 12,
      minWidth: 240,
      position: 'relative',
    }}>
      <div style={{
        fontFamily: '"JetBrains Mono", monospace',
        fontSize: 48,
        fontWeight: 700,
        color: COLORS.D,
        letterSpacing: 2,
      }}>
        $0.00
      </div>
      <div style={{
        fontSize: 10,
        color: COLORS.textMuted,
        textTransform: 'uppercase',
        letterSpacing: 2,
        fontFamily: 'monospace',
        textAlign: 'center',
      }}>
        $0.00 per query
      </div>
      <div style={{
        fontSize: 11,
        color: COLORS.textMuted,
        textTransform: 'uppercase',
        letterSpacing: 3,
        fontFamily: 'monospace',
      }}>
        0 LLM tokens for navigation
      </div>
      <div style={{
        fontSize: 10,
        color: COLORS.textDim,
        fontFamily: 'monospace',
        textAlign: 'center',
        marginTop: 4,
        letterSpacing: 0.5,
        lineHeight: 1.5,
      }}>
        ~200 context tokens vs ~3,200 without m1nd
      </div>
      {/* Frozen badge */}
      <motion.div
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.5, type: 'spring', stiffness: 300 }}
        style={{
          position: 'absolute',
          top: -10,
          right: -10,
          background: COLORS.D,
          color: COLORS.bg,
          fontSize: 10,
          fontWeight: 700,
          fontFamily: 'monospace',
          letterSpacing: 2,
          padding: '3px 10px',
          borderRadius: 4,
        }}
      >
        FROZEN
      </motion.div>
    </div>
  );
}

export function EconomyScene() {
  const [phase, setPhase] = useState(0);
  // Phase 0: scene enters
  // Phase 1: counters appear (0.5s)
  // Phase 2: statement appears (3.5s)
  // Phase 3: "5 hours" quote (4s) -- moved from 5s to give 1s visibility

  useEffect(() => {
    const timers = [
      setTimeout(() => setPhase(1), 500),
      setTimeout(() => setPhase(2), 3500),
      setTimeout(() => setPhase(3), 4000),
    ];
    return () => timers.forEach(clearTimeout);
  }, []);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.5 }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 0,
        height: '100%',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '40px 48px',
      }}
    >
      {/* Scene label */}
      <motion.div
        initial={{ x: -20, opacity: 0 }}
        animate={{ x: 0, opacity: 1 }}
        style={{
          fontSize: 11,
          color: COLORS.D,
          letterSpacing: 3,
          fontFamily: 'monospace',
          marginBottom: 12,
          alignSelf: 'flex-start',
        }}
      >
        {GLYPHS.save} SCENE 5 -- THE ECONOMY
      </motion.div>

      {/* Split screen: two counters */}
      <AnimatePresence>
        {phase >= 1 && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            style={{
              display: 'flex',
              gap: 0,
              alignItems: 'stretch',
              width: '100%',
              maxWidth: 700,
              marginBottom: 32,
            }}
          >
            {/* LEFT: grep racing */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 16 }}>
              <div style={{
                fontSize: 12,
                color: COLORS.error,
                fontFamily: 'monospace',
                letterSpacing: 3,
                textTransform: 'uppercase',
              }}>
                grep + LLM
              </div>
              <AnimatedCost
                target={723}
                speed={3}
                color={COLORS.error}
                label="token cost (grep)"
              />
              {/* Token count underneath */}
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 1 }}
                style={{
                  fontSize: 11,
                  color: COLORS.textDim,
                  fontFamily: 'monospace',
                }}
              >
                ~193,000 tokens consumed
              </motion.div>
            </div>

            {/* Divider */}
            <div style={{
              width: 1,
              background: `${COLORS.border}`,
              margin: '0 32px',
              alignSelf: 'stretch',
            }} />

            {/* RIGHT: m1nd frozen */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 16 }}>
              <div style={{
                fontSize: 12,
                color: COLORS.D,
                fontFamily: 'monospace',
                letterSpacing: 3,
                textTransform: 'uppercase',
              }}>
                m1nd
              </div>
              <FrozenCounter />
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 1.5 }}
                style={{
                  fontSize: 11,
                  color: COLORS.textDim,
                  fontFamily: 'monospace',
                }}
              >
                graph queries -- no AI calls needed
              </motion.div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Statement line */}
      <AnimatePresence>
        {phase >= 2 && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.5 }}
            style={{
              textAlign: 'center',
              fontFamily: '"JetBrains Mono", monospace',
              marginBottom: 16,
            }}
          >
            <div style={{ fontSize: 22, color: COLORS.text, fontWeight: 600, marginBottom: 8 }}>
              <span style={{ color: COLORS.one }}>946</span> tokens saved per query.{' '}
              <span style={{ color: COLORS.textMuted }}>every query.</span>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* "5 hours" quote */}
      <AnimatePresence>
        {phase >= 3 && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.6 }}
            style={{
              textAlign: 'center',
              fontFamily: '"JetBrains Mono", monospace',
              padding: '16px 28px',
              background: `${COLORS.D}08`,
              border: `1px solid ${COLORS.D}20`,
              borderRadius: 10,
              maxWidth: 540,
            }}
          >
            <div style={{ fontSize: 20, color: COLORS.textMuted, lineHeight: 1.8, fontWeight: 600 }}>
              5 hours. 100+ agents. plan usage:{' '}
              <span style={{ color: COLORS.D, fontWeight: 800, fontSize: 22 }}>didn't move.</span>
            </div>
            <div style={{
              fontSize: 11,
              color: COLORS.textDim,
              marginTop: 8,
              letterSpacing: 1,
            }}>
              {GLYPHS.statement} measured from live m1nd.savings -- one session, 105 queries
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
