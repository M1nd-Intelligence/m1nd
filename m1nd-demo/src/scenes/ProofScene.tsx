import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { COLORS, GLYPHS } from '../lib/colors';

/**
 * Scene 6: THE PROOF
 * Comparison table building row by row with slam animation.
 * Each row slams. Green vs red. "invisible bugs: 8 vs 0" hits HARDEST.
 */

interface ProofRow {
  label: string;
  m1nd: string;
  grepLlm: string;
  m1ndColor: string;
  grepColor: string;
  isKiller?: boolean; // the "invisible bugs" row -- hits hardest
}

const PROOF_DATA: ProofRow[] = [
  {
    label: 'queries',
    m1nd: '46',
    grepLlm: '~210',
    m1ndColor: COLORS.D,
    grepColor: COLORS.error,
  },
  {
    label: 'time',
    m1nd: '3.1s',
    grepLlm: '~35 min',
    m1ndColor: COLORS.D,
    grepColor: COLORS.error,
  },
  {
    label: 'tokens',
    m1nd: '0',
    grepLlm: '~193,000',
    m1ndColor: COLORS.D,
    grepColor: COLORS.error,
  },
  {
    label: 'cost',
    m1nd: '$0.00',
    grepLlm: '~$7.23',
    m1ndColor: COLORS.D,
    grepColor: COLORS.error,
  },
  {
    label: 'bugs found',
    m1nd: '39',
    grepLlm: '~23',
    m1ndColor: COLORS.D,
    grepColor: COLORS.one,
  },
  {
    label: 'invisible bugs',
    m1nd: '8',
    grepLlm: '0',
    m1ndColor: COLORS.M,
    grepColor: COLORS.error,
    isKiller: true,
  },
];

function SlamRow({ row, visible }: { row: ProofRow; index: number; visible: boolean }) {
  if (!visible) return null;

  return (
    <motion.div
      initial={{ opacity: 0, x: -40, scale: 0.95 }}
      animate={{ opacity: 1, x: 0, scale: 1 }}
      transition={{
        duration: 0.3,
        type: 'spring',
        stiffness: 400,
        damping: 25,
      }}
      style={{
        display: 'grid',
        gridTemplateColumns: '180px 1fr 1fr',
        gap: 0,
        alignItems: 'center',
        borderBottom: row.isKiller ? 'none' : `1px solid ${COLORS.border}`,
        background: row.isKiller ? `${COLORS.M}10` : 'transparent',
        borderRadius: row.isKiller ? 8 : 0,
        border: row.isKiller ? `1px solid ${COLORS.M}40` : undefined,
        overflow: 'hidden',
      }}
    >
      {/* Label */}
      <div style={{
        padding: '14px 20px',
        fontFamily: '"JetBrains Mono", monospace',
        fontSize: row.isKiller ? 14 : 13,
        color: row.isKiller ? COLORS.M : COLORS.textMuted,
        fontWeight: row.isKiller ? 700 : 400,
        letterSpacing: 1,
      }}>
        {row.label}
      </div>

      {/* m1nd value */}
      <div style={{
        padding: '14px 20px',
        fontFamily: '"JetBrains Mono", monospace',
        fontSize: row.isKiller ? 22 : 16,
        fontWeight: 700,
        color: row.m1ndColor,
        textAlign: 'center',
        textShadow: row.isKiller ? `0 0 20px ${row.m1ndColor}60` : 'none',
      }}>
        {row.m1nd}
      </div>

      {/* grep+LLM value */}
      <div style={{
        padding: '14px 20px',
        fontFamily: '"JetBrains Mono", monospace',
        fontSize: row.isKiller ? 22 : 16,
        fontWeight: row.isKiller ? 700 : 400,
        color: row.grepColor,
        textAlign: 'center',
        opacity: row.isKiller ? 0.6 : 0.8,
      }}>
        {row.grepLlm}
      </div>
    </motion.div>
  );
}

export function ProofScene() {
  const [visibleRows, setVisibleRows] = useState(0);

  useEffect(() => {
    const timers: ReturnType<typeof setTimeout>[] = [];
    PROOF_DATA.forEach((_, i) => {
      // Each row slams in with stagger. Last row (killer) gets extra delay.
      const delay = i < PROOF_DATA.length - 1
        ? 600 + i * 650
        : 600 + (i - 1) * 650 + 800; // extra pause before "invisible bugs"
      timers.push(setTimeout(() => setVisibleRows(i + 1), delay));
    });
    return () => timers.forEach(clearTimeout);
  }, []);

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
        padding: '32px 60px',
        height: '100%',
        justifyContent: 'center',
      }}
    >
      {/* Scene label */}
      <motion.div
        initial={{ x: -20, opacity: 0 }}
        animate={{ x: 0, opacity: 1 }}
        style={{
          fontSize: 11,
          color: COLORS.M,
          letterSpacing: 3,
          fontFamily: 'monospace',
          marginBottom: 4,
          maxWidth: 640,
          alignSelf: 'center',
          width: '100%',
        }}
      >
        {GLYPHS.structure} SCENE 6 -- THE PROOF
      </motion.div>

      {/* Table container */}
      <div style={{
        background: `${COLORS.bgCard}`,
        border: `1px solid ${COLORS.border}`,
        borderRadius: 12,
        overflow: 'hidden',
        maxWidth: 640,
        width: '100%',
        alignSelf: 'center',
      }}>
        {/* Header */}
        <div style={{
          display: 'grid',
          gridTemplateColumns: '180px 1fr 1fr',
          borderBottom: `1px solid ${COLORS.border}`,
          background: `${COLORS.bgSurface}`,
        }}>
          <div style={{
            padding: '12px 20px',
            fontFamily: 'monospace',
            fontSize: 11,
            color: COLORS.textDim,
            letterSpacing: 2,
            textTransform: 'uppercase',
          }} />
          <div style={{
            padding: '12px 20px',
            fontFamily: 'monospace',
            fontSize: 12,
            color: COLORS.D,
            letterSpacing: 2,
            textAlign: 'center',
            fontWeight: 700,
          }}>
            m1nd
          </div>
          <div style={{
            padding: '12px 20px',
            fontFamily: 'monospace',
            fontSize: 12,
            color: COLORS.error,
            letterSpacing: 2,
            textAlign: 'center',
            fontWeight: 700,
            opacity: 0.7,
          }}>
            grep+LLM
          </div>
        </div>

        {/* Rows -- slam in one by one */}
        <div>
          {PROOF_DATA.map((row, i) => (
            <SlamRow key={row.label} row={row} index={i} visible={i < visibleRows} />
          ))}
        </div>
      </div>

      {/* Killer row emphasis -- appears after all rows */}
      <AnimatePresence>
        {visibleRows >= PROOF_DATA.length && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.5 }}
            style={{
              textAlign: 'center',
              fontFamily: '"JetBrains Mono", monospace',
              fontSize: 13,
              color: COLORS.M,
              letterSpacing: 1,
              maxWidth: 640,
              alignSelf: 'center',
            }}
          >
            {GLYPHS.statement} 8 bugs that exist in your code right now that grep will never find.
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
