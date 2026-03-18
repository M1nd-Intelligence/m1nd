import { useEffect, useState, useRef } from 'react';
import { COLORS, GLYPHS } from '../lib/colors';

interface TokenCounterProps {
  label: string;
  targetValue: number;
  color: string;
  duration?: number; // ms to reach target
  startDelay?: number;
  frozen?: boolean;
  showCost?: boolean;
  costValue?: string;
  size?: 'normal' | 'large';
}

export function TokenCounter({
  label,
  targetValue,
  color,
  duration = 3000,
  startDelay = 0,
  frozen = false,
  showCost = false,
  costValue = '$0.00',
  size = 'normal',
}: TokenCounterProps) {
  const [displayed, setDisplayed] = useState(0);
  const [started, setStarted] = useState(false);
  const rafRef = useRef<number>(0);
  const startTimeRef = useRef(0);

  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Start delay
  useEffect(() => {
    if (frozen) return;
    if (prefersReducedMotion) {
      setDisplayed(targetValue);
      return;
    }
    const timer = setTimeout(() => setStarted(true), startDelay);
    return () => clearTimeout(timer);
  }, [startDelay, frozen, prefersReducedMotion, targetValue]);

  // rAF counter animation
  useEffect(() => {
    if (!started || frozen || prefersReducedMotion) return;

    const animate = (timestamp: number) => {
      if (!startTimeRef.current) startTimeRef.current = timestamp;
      const elapsed = timestamp - startTimeRef.current;
      const progress = Math.min(elapsed / duration, 1);
      // Ease-out for dramatic deceleration at the end
      const eased = 1 - Math.pow(1 - progress, 3);
      setDisplayed(Math.round(eased * targetValue));

      if (progress < 1) {
        rafRef.current = requestAnimationFrame(animate);
      }
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      startTimeRef.current = 0;
    };
  }, [started, targetValue, duration, frozen, prefersReducedMotion]);

  const formatted = displayed.toLocaleString();
  const isLarge = size === 'large';

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      gap: 6,
      padding: isLarge ? '20px 28px' : '14px 20px',
      background: 'rgba(0, 0, 0, 0.5)',
      border: `1px solid ${color}30`,
      borderRadius: 10,
      position: 'relative',
      overflow: 'hidden',
    }}>
      {/* Animated glow background when counting */}
      {!frozen && started && displayed < targetValue && (
        <div style={{
          position: 'absolute',
          inset: 0,
          background: `radial-gradient(ellipse at center, ${color}08 0%, transparent 70%)`,
          animation: 'pulse-glow 1s ease-in-out infinite',
        }} />
      )}

      <div style={{
        fontSize: 11,
        color: COLORS.textMuted,
        textTransform: 'uppercase',
        letterSpacing: 2,
        fontFamily: 'monospace',
        position: 'relative',
      }}>
        {label}
      </div>

      <div style={{
        display: 'flex',
        alignItems: 'baseline',
        gap: 12,
        position: 'relative',
      }}>
        <div style={{
          fontFamily: '"JetBrains Mono", monospace',
          fontSize: isLarge ? 36 : 28,
          fontWeight: 'bold',
          color: frozen ? COLORS.D : color,
          letterSpacing: 2,
          transition: 'color 0.5s',
          textShadow: frozen
            ? `0 0 20px ${COLORS.D}40`
            : (started && displayed > 0 ? `0 0 20px ${color}40` : 'none'),
        }}>
          {frozen ? (
            <span style={{ color: COLORS.D }}>
              {GLYPHS.statement} 0
            </span>
          ) : formatted}
        </div>

        {showCost && (
          <div style={{
            fontSize: isLarge ? 16 : 13,
            color: frozen ? COLORS.D : color,
            fontFamily: 'monospace',
            opacity: 0.8,
          }}>
            {costValue}
          </div>
        )}
      </div>

      {/* Progress bar showing burn rate */}
      {!frozen && (
        <div style={{
          height: 3,
          background: `${color}20`,
          borderRadius: 2,
          overflow: 'hidden',
          position: 'relative',
        }}>
          <div style={{
            height: '100%',
            width: `${(displayed / targetValue) * 100}%`,
            background: color,
            borderRadius: 2,
            transition: 'width 0.1s linear',
            boxShadow: `0 0 8px ${color}80`,
          }} />
        </div>
      )}

      {frozen && (
        <div style={{
          fontSize: 10,
          color: COLORS.D,
          fontFamily: 'monospace',
          letterSpacing: 1,
          opacity: 0.8,
        }}>
          0 LLM tokens for navigation
        </div>
      )}

      <style>{`
        @keyframes pulse-glow {
          0%, 100% { opacity: 0.5; }
          50% { opacity: 1; }
        }
      `}</style>
    </div>
  );
}
