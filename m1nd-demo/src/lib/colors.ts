// m1nd identity color palette
// Each letter M-1-N-D has its own color
export const COLORS = {
  bg: '#080C14',
  bgSurface: '#0F1420',
  bgCard: '#141B2D',

  // Identity letters
  M: '#00D4FF',    // cyan-blue — M
  one: '#FFD700',  // gold      — 1
  N: '#FF00AA',    // magenta   — N
  D: '#00FF88',    // green     — D

  // Semantic colors
  text: '#E8EDF5',
  textMuted: '#6B7FA3',
  textDim: '#3A4A6B',

  // Signal colors
  error: '#FF4444',
  success: '#00FF88',
  warning: '#FFD700',
  info: '#00D4FF',

  // Gradient stops
  gradientStart: '#00D4FF',
  gradientMid: '#FF00AA',
  gradientEnd: '#00FF88',

  // Border
  border: 'rgba(0, 212, 255, 0.2)',
  borderHover: 'rgba(0, 212, 255, 0.6)',
} as const;

export const GRADIENT_BORDER = `linear-gradient(135deg, ${COLORS.M}, ${COLORS.N}, ${COLORS.M}, ${COLORS.D})`;

// Glyph set — m1nd identity symbols
export const GLYPHS = {
  statement: '⍌',
  transition: '⍐',
  structure: '⍂',
  delta: '𝔻',
  convergence: '⟁',
  // Additional
  node: '◈',
  edge: '─',
  activate: '⚡',
  save: '◉',
} as const;
