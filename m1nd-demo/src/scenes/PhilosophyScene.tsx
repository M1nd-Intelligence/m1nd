import { useEffect, useState, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { COLORS } from '../lib/colors';

/**
 * Scene 8: THE PHILOSOPHY
 * Fade to near-black. Text appears line by line.
 * "you don't need to understand how it works."
 * "your agent does."
 * pause 500ms <- SACRED
 * "by Max Elias Kleinschmidt"
 * "AI should amplify, never replace."
 * "m1nd shortens the distance."
 * github.com/maxkle1nz/m1nd
 *
 * This must feel like the end of a film -- NOT a product page.
 */

interface TypedLineProps {
  text: string;
  color: string;
  fontSize: number;
  fontWeight?: number;
  delay: number;
  speed?: number; // ms per char
  letterSpacing?: number;
}

function TypedLine({ text, color, fontSize, fontWeight = 400, delay, speed = 30, letterSpacing = 0 }: TypedLineProps) {
  const [displayed, setDisplayed] = useState('');
  const [started, setStarted] = useState(false);
  const charRef = useRef(0);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef(0);

  // Check prefers-reduced-motion
  const prefersReducedMotion = typeof window !== 'undefined' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  useEffect(() => {
    const startTimer = setTimeout(() => setStarted(true), delay);
    return () => clearTimeout(startTimer);
  }, [delay]);

  // rAF-based typing (replaces setInterval for frame accuracy)
  useEffect(() => {
    if (!started || prefersReducedMotion) return;
    charRef.current = 0;

    const animate = (timestamp: number) => {
      if (!lastTimeRef.current) lastTimeRef.current = timestamp;
      const elapsed = timestamp - lastTimeRef.current;

      if (elapsed >= speed) {
        lastTimeRef.current = timestamp;
        charRef.current++;
        setDisplayed(text.slice(0, charRef.current));
        if (charRef.current >= text.length) return;
      }
      rafRef.current = requestAnimationFrame(animate);
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      lastTimeRef.current = 0;
    };
  }, [started, text, speed, prefersReducedMotion]);

  if (!started && !prefersReducedMotion) return <div style={{ height: fontSize * 1.4, minHeight: fontSize * 1.4 }} />;

  return (
    <div style={{
      fontFamily: '"JetBrains Mono", monospace',
      fontSize,
      fontWeight,
      color,
      letterSpacing,
      lineHeight: 1.4,
      minHeight: fontSize * 1.4,
    }}>
      {prefersReducedMotion ? text : displayed}
      {!prefersReducedMotion && started && displayed.length < text.length && (
        <span style={{
          display: 'inline-block',
          width: 2,
          height: fontSize * 0.8,
          background: color,
          marginLeft: 2,
          animation: 'cursor-blink 1.06s step-end infinite',
          verticalAlign: 'text-bottom',
          opacity: 0.7,
        }} />
      )}
    </div>
  );
}

export function PhilosophyScene() {
  const [phase, setPhase] = useState(0);
  // Phase 0: near-black void (0ms)
  // Phase 1: "you don't need to understand how it works." (800ms)
  // Phase 2: "your agent does." (2805ms) -- 800 + 1505ms typing (43*35ms) + 500ms sacred pause
  // Phase 3: author + philosophy (5500ms)
  // Phase 4: github link (7500ms)

  useEffect(() => {
    const timers = [
      setTimeout(() => setPhase(1), 800),
      setTimeout(() => setPhase(2), 2805),  // 800 + 1505 typing + 500ms sacred pause
      setTimeout(() => setPhase(3), 5500),
      setTimeout(() => setPhase(4), 7500),
    ];
    return () => timers.forEach(clearTimeout);
  }, []);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 1.0 }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 0,
        height: '100%',
        alignItems: 'center',
        justifyContent: 'center',
        textAlign: 'center',
        padding: '40px 60px',
        // Near-black -- darker than the rest of the demo
        background: 'linear-gradient(180deg, #050810 0%, #080C14 100%)',
        position: 'relative',
      }}
    >
      {/* Main tagline */}
      <div style={{ maxWidth: 600, marginBottom: 24 }}>
        <AnimatePresence>
          {phase >= 1 && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.3 }}
            >
              <TypedLine
                text="you don't need to understand how it works."
                color={COLORS.text}
                fontSize={28}
                fontWeight={600}
                delay={0}
                speed={35}
                letterSpacing={-0.5}
              />
            </motion.div>
          )}
        </AnimatePresence>

        {/* 500ms SACRED pause is built into the phase timing above */}

        <AnimatePresence>
          {phase >= 2 && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.3 }}
              style={{ marginTop: 16 }}
            >
              <TypedLine
                text="your agent does."
                color={COLORS.M}
                fontSize={28}
                fontWeight={700}
                delay={0}
                speed={40}
                letterSpacing={-0.5}
              />
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Author + philosophy */}
      <AnimatePresence>
        {phase >= 3 && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            style={{
              display: 'flex',
              flexDirection: 'column',
              gap: 12,
              maxWidth: 480,
              marginBottom: 24,
            }}
          >
            <div style={{
              fontSize: 16,
              color: COLORS.text,
              fontFamily: '"JetBrains Mono", monospace',
              letterSpacing: 2,
              opacity: 0.7,
            }}>
              by Max Elias Kleinschmidt
            </div>

            <div style={{
              width: 40,
              height: 1,
              background: COLORS.border,
              alignSelf: 'center',
              margin: '4px 0',
            }} />

            <div style={{
              fontSize: 14,
              color: COLORS.text,
              fontFamily: '"JetBrains Mono", monospace',
              lineHeight: 2,
              opacity: 0.8,
            }}>
              AI should amplify, never replace.
            </div>

            <div style={{
              fontSize: 14,
              color: COLORS.text,
              fontFamily: '"JetBrains Mono", monospace',
              lineHeight: 2,
              opacity: 0.8,
            }}>
              m1nd shortens the distance.
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* GitHub link */}
      <AnimatePresence>
        {phase >= 4 && (
          <motion.a
            href="https://github.com/maxkle1nz/m1nd"
            target="_blank"
            rel="noopener noreferrer"
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            style={{
              fontSize: 13,
              color: COLORS.M,
              fontFamily: '"JetBrains Mono", monospace',
              letterSpacing: 2,
              textDecoration: 'none',
              padding: '10px 24px',
              border: `1px solid ${COLORS.M}30`,
              borderRadius: 8,
              background: `${COLORS.M}08`,
              cursor: 'pointer',
              transition: 'border-color 0.3s, background 0.3s',
            }}
            whileHover={{
              borderColor: `${COLORS.M}60`,
              background: `${COLORS.M}15`,
            }}
          >
            github.com/maxkle1nz/m1nd
          </motion.a>
        )}
      </AnimatePresence>

      {/* Accessibility */}
      <h2 style={{
        position: 'absolute',
        width: 1,
        height: 1,
        overflow: 'hidden',
        clip: 'rect(0, 0, 0, 0)',
      }}>
        You don't need to understand how it works. Your agent does.
      </h2>
    </motion.div>
  );
}
