import { motion } from 'framer-motion';
import { COLORS } from '../lib/colors';

interface ComparisonItem {
  label: string;
  value?: string;
  calls?: number;
  tokens?: number;
}

interface ComparisonRowProps {
  before: ComparisonItem;
  after: ComparisonItem;
  delay?: number;
}

export function ComparisonRow({ before, after, delay = 0 }: ComparisonRowProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay, duration: 0.4 }}
      style={{
        display: 'grid',
        gridTemplateColumns: '1fr auto 1fr',
        gap: 16,
        alignItems: 'center',
        padding: '12px 0',
        borderBottom: `1px solid ${COLORS.border}`,
        fontFamily: '"JetBrains Mono", monospace',
      }}
    >
      {/* Before */}
      <div style={{
        background: `${COLORS.error}15`,
        border: `1px solid ${COLORS.error}40`,
        borderRadius: 8,
        padding: '12px 16px',
      }}>
        <div style={{ fontSize: 11, color: COLORS.error, letterSpacing: 2, marginBottom: 6 }}>BEFORE</div>
        <div style={{ fontSize: 13, color: COLORS.text, marginBottom: 4 }}>{before.label}</div>
        {before.calls !== undefined && (
          <div style={{ fontSize: 11, color: COLORS.textMuted }}>
            {before.calls} tool calls · {before.tokens?.toLocaleString()} tokens
          </div>
        )}
      </div>

      {/* Arrow */}
      <div style={{ color: COLORS.N, fontSize: 20 }}>⍐</div>

      {/* After */}
      <div style={{
        background: `${COLORS.D}15`,
        border: `1px solid ${COLORS.D}40`,
        borderRadius: 8,
        padding: '12px 16px',
      }}>
        <div style={{ fontSize: 11, color: COLORS.D, letterSpacing: 2, marginBottom: 6 }}>AFTER</div>
        <div style={{ fontSize: 13, color: COLORS.text, marginBottom: 4 }}>{after.label}</div>
        {after.calls !== undefined && (
          <div style={{ fontSize: 11, color: COLORS.textMuted }}>
            {after.calls} tool {after.calls === 1 ? 'call' : 'calls'} · {after.tokens?.toLocaleString()} tokens
          </div>
        )}
      </div>
    </motion.div>
  );
}
