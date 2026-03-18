import { motion } from 'framer-motion';
import { COLORS } from '../lib/colors';

interface GlyphRevealProps {
  glyphs: string[];
  colors?: string[];
  size?: number;
  gap?: number;
  staggerDelay?: number;
}

export function GlyphReveal({
  glyphs,
  colors = [COLORS.M, COLORS.one, COLORS.N, COLORS.D, COLORS.M],
  size = 72,
  gap = 24,
  staggerDelay = 0.15,
}: GlyphRevealProps) {
  return (
    <div style={{ display: 'flex', gap, alignItems: 'center', justifyContent: 'center' }}>
      {glyphs.map((glyph, i) => (
        <motion.span
          key={i}
          initial={{ opacity: 0, scale: 0.3, filter: 'blur(12px)' }}
          animate={{ opacity: 1, scale: 1, filter: 'blur(0px)' }}
          transition={{
            delay: i * staggerDelay,
            duration: 0.5,
            type: 'spring',
            stiffness: 200,
          }}
          style={{
            fontSize: size,
            color: colors[i % colors.length],
            fontFamily: 'monospace',
            textShadow: `0 0 30px ${colors[i % colors.length]}80`,
            display: 'inline-block',
          }}
        >
          {glyph}
        </motion.span>
      ))}
    </div>
  );
}
