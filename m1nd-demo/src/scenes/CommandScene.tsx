import { motion, AnimatePresence } from 'framer-motion';
import { useEffect, useState, useRef } from 'react';
import { COLORS, GLYPHS } from '../lib/colors';

/**
 * SCENE 2: THE COMMAND (Layer 2 part 1)
 *
 * Screen clears. Single green cursor. "m1nd.activate" typed slowly.
 * Enter pressed. Flash of light. Transition to the brain.
 *
 * Emotion: CURIOSITY -> AWE
 * The moment of "wait, what?"
 */

const COMMAND = 'm1nd.activate("authentication flow")';
const RESULT_LINE = '\u27C1 7 nodes activated in 31ms';

export function CommandScene() {
  const [phase, setPhase] = useState<'empty' | 'typing' | 'enter' | 'flash' | 'result' | 'dimensions'>('empty');
  const [typedChars, setTypedChars] = useState(0);
  const [cursorVisible, setCursorVisible] = useState(true);
  const [resultChars, setResultChars] = useState(0);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef(0);

  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Phase sequencing
  useEffect(() => {
    if (prefersReducedMotion) {
      setPhase('dimensions');
      setTypedChars(COMMAND.length);
      setResultChars(RESULT_LINE.length);
      return;
    }

    const timers: ReturnType<typeof setTimeout>[] = [];
    // 0ms: empty screen with cursor
    // 800ms: start typing command
    timers.push(setTimeout(() => setPhase('typing'), 800));
    return () => timers.forEach(clearTimeout);
  }, [prefersReducedMotion]);

  // Cursor blink
  useEffect(() => {
    const interval = setInterval(() => setCursorVisible(v => !v), 530);
    return () => clearInterval(interval);
  }, []);

  // rAF typing for command — SLOW (30ms/char for dramatic effect)
  useEffect(() => {
    if (phase !== 'typing' || prefersReducedMotion) return;

    const speed = 30; // ms per char — slow, deliberate

    const animate = (timestamp: number) => {
      if (!lastTimeRef.current) lastTimeRef.current = timestamp;
      const elapsed = timestamp - lastTimeRef.current;

      if (elapsed >= speed) {
        lastTimeRef.current = timestamp;
        setTypedChars(prev => {
          const next = prev + 1;
          if (next >= COMMAND.length) {
            // Command complete — trigger enter
            setTimeout(() => setPhase('enter'), 400);
            return COMMAND.length;
          }
          return next;
        });
      }
      if (typedChars < COMMAND.length) {
        rafRef.current = requestAnimationFrame(animate);
      }
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      lastTimeRef.current = 0;
    };
  }, [phase, typedChars, prefersReducedMotion]);

  // Enter -> flash -> result
  useEffect(() => {
    if (phase !== 'enter') return;
    const t1 = setTimeout(() => setPhase('flash'), 200);
    const t2 = setTimeout(() => setPhase('result'), 500);
    return () => { clearTimeout(t1); clearTimeout(t2); };
  }, [phase]);

  // rAF typing for result line — fast (8ms/char)
  useEffect(() => {
    if (phase !== 'result' || prefersReducedMotion) return;

    const speed = 8;
    let localLastTime = 0;

    const animate = (timestamp: number) => {
      if (!localLastTime) localLastTime = timestamp;
      const elapsed = timestamp - localLastTime;

      if (elapsed >= speed) {
        localLastTime = timestamp;
        setResultChars(prev => {
          const next = prev + 1;
          if (next >= RESULT_LINE.length) {
            setTimeout(() => setPhase('dimensions'), 500);
            return RESULT_LINE.length;
          }
          return next;
        });
      }
      if (resultChars < RESULT_LINE.length) {
        rafRef.current = requestAnimationFrame(animate);
      }
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, [phase, resultChars, prefersReducedMotion]);

  const DIMENSIONS = [
    { letter: 'M', label: 'Structural', value: '0.97', color: COLORS.M, desc: 'how your code is connected' },
    { letter: '1', label: 'Temporal', value: '0.54', color: COLORS.one, desc: 'what usually changes together' },
    { letter: 'N', label: 'Causal', value: '0.80', color: COLORS.N, desc: 'what might break if you touch this' },
    { letter: 'D', label: 'Semantic', value: '0.88', color: COLORS.D, desc: 'what your code means, not just what it says' },
  ];

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      {/* Flash effect — NO exit prop to prevent double-exit blank screen */}
      <AnimatePresence>
        {phase === 'flash' && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 0.6 }}
            transition={{ duration: 0.15 }}
            style={{
              position: 'absolute',
              inset: 0,
              background: `radial-gradient(ellipse at center, ${COLORS.M}80, transparent 70%)`,
              zIndex: 10,
              pointerEvents: 'none',
            }}
          />
        )}
      </AnimatePresence>

      {/* Scene header */}
      <div style={{ padding: '32px 48px 0' }}>
        <motion.div
          initial={{ x: -20, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          transition={{ delay: 0.1 }}
          style={{
            fontSize: 11,
            color: COLORS.M,
            letterSpacing: 3,
            fontFamily: 'monospace',
            marginBottom: 8,
          }}
        >
          {GLYPHS.transition} SCENE 2 — THE COMMAND
        </motion.div>
      </div>

      {/* The terminal — large, centered, cinematic */}
      <div style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '0 48px 32px',
        gap: 32,
      }}>
        {/* Big terminal box */}
        <div style={{
          width: '100%',
          maxWidth: 800,
          background: 'rgba(0, 0, 0, 0.8)',
          border: `1px solid ${COLORS.M}25`,
          borderRadius: 10,
          padding: '20px 24px',
          fontFamily: '"JetBrains Mono", "Fira Code", monospace',
          fontSize: 16,
          lineHeight: 2,
          boxShadow: `0 0 60px ${COLORS.M}08`,
          minHeight: 140,
        }}>
          {/* Terminal dots */}
          <div style={{ display: 'flex', gap: 6, marginBottom: 16 }}>
            {['#FF5F56', '#FFBD2E', '#27C93F'].map((c, i) => (
              <div key={i} style={{ width: 10, height: 10, borderRadius: '50%', background: c, opacity: 0.7 }} />
            ))}
          </div>

          {/* Command line */}
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <span style={{ color: COLORS.D, marginRight: 10, opacity: 0.7 }}>{GLYPHS.statement}</span>
            <span style={{ color: COLORS.D }}>
              {COMMAND.slice(0, typedChars)}
            </span>
            {(phase === 'empty' || phase === 'typing') && (
              <span style={{
                display: 'inline-block',
                width: 10,
                height: 18,
                background: COLORS.D,
                marginLeft: 2,
                verticalAlign: 'text-bottom',
                opacity: cursorVisible ? 1 : 0,
              }} />
            )}
          </div>

          {/* Result line */}
          {(phase === 'result' || phase === 'dimensions') && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              style={{ marginTop: 8 }}
            >
              <span style={{ color: COLORS.D, marginRight: 10, opacity: 0.4 }}> </span>
              <span style={{ color: '#00FF88' }}>
                {RESULT_LINE.slice(0, resultChars)}
              </span>
              {phase === 'result' && resultChars < RESULT_LINE.length && (
                <span style={{
                  display: 'inline-block',
                  width: 10,
                  height: 18,
                  background: '#00FF88',
                  marginLeft: 2,
                  opacity: cursorVisible ? 1 : 0,
                }} />
              )}
            </motion.div>
          )}
        </div>

        {/* 4 dimension cards appearing after result */}
        {phase === 'dimensions' && (
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(4, 1fr)',
            gap: 16,
            width: '100%',
            maxWidth: 800,
          }}>
            {DIMENSIONS.map((dim, i) => (
              <motion.div
                key={dim.letter}
                initial={{ opacity: 0, y: 20, scale: 0.95 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                transition={{ delay: 0.15 * i, type: 'spring', stiffness: 200, damping: 20 }}
                style={{
                  background: `${dim.color}10`,
                  border: `1px solid ${dim.color}30`,
                  borderRadius: 8,
                  padding: '14px 16px',
                  fontFamily: 'monospace',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'baseline', gap: 8, marginBottom: 6 }}>
                  <span style={{
                    fontSize: 24,
                    fontWeight: 900,
                    color: dim.color,
                    textShadow: `0 0 15px ${dim.color}40`,
                  }}>
                    {dim.letter}
                  </span>
                  <span style={{ fontSize: 11, color: dim.color, letterSpacing: 1 }}>
                    {dim.label}
                  </span>
                </div>
                <div style={{
                  fontSize: 20,
                  fontWeight: 700,
                  color: dim.color,
                  marginBottom: 4,
                }}>
                  {dim.value}
                </div>
                <div style={{ fontSize: 10, color: COLORS.textMuted, lineHeight: 1.4 }}>
                  {dim.desc}
                </div>
                {/* Dimension bar */}
                <div style={{
                  height: 3,
                  background: `${dim.color}15`,
                  borderRadius: 2,
                  marginTop: 8,
                  overflow: 'hidden',
                }}>
                  <motion.div
                    initial={{ width: 0 }}
                    animate={{ width: `${parseFloat(dim.value) * 100}%` }}
                    transition={{ delay: 0.3 + i * 0.1, duration: 0.6 }}
                    style={{
                      height: '100%',
                      background: dim.color,
                      borderRadius: 2,
                      boxShadow: `0 0 6px ${dim.color}60`,
                    }}
                  />
                </div>
              </motion.div>
            ))}
          </div>
        )}

        {/* Overlay text */}
        {phase === 'dimensions' && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.8 }}
            style={{
              textAlign: 'center',
              fontFamily: '"JetBrains Mono", monospace',
              fontSize: 15,
              letterSpacing: 1,
            }}
          >
            <span style={{ color: COLORS.M }}>
              it doesn't read your code.
            </span>
            <br />
            <span style={{ color: COLORS.M }}>
              it{' '}
              <span style={{ fontWeight: 900, textTransform: 'uppercase' }}>thinks</span>{' '}
              about your code.
            </span>
          </motion.div>
        )}
      </div>
    </motion.div>
  );
}
