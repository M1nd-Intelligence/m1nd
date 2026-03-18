import { motion } from 'framer-motion';
import { useEffect, useState, useRef } from 'react';
import { COLORS, GLYPHS } from '../lib/colors';
import { TokenCounter } from '../components/TokenCounter';

/**
 * SCENE 1: THE PROBLEM (Layer 1 — THE FLASHLIGHT)
 *
 * Hacker terminal. grep commands appearing. Token counter RACING up in red.
 * $0.04 burning. Frustration builds. Dark, urgent.
 *
 * Emotion: FRUSTRATION
 * Verified numbers: 847 results, 12K tokens, $0.04
 */

const GREP_RESULTS = [
  { text: '$ grep -r "handleAuth" .', isCommand: true },
  { text: '' },
  { text: '  ./auth/handler.py:14:  def handleAuth(request):', delay: 200 },
  { text: '  ./auth/handler.py:47:  # handleAuth edge case', delay: 100 },
  { text: '  ./middleware/cors.py:3: from auth import handleAuth', delay: 80 },
  { text: '  ./tests/test_auth.py:8: handleAuth(mock_req)', delay: 80 },
  { text: '  ./routes/api.py:22:    handleAuth(ctx)', delay: 60 },
  { text: '  ./docs/README.md:44:   `handleAuth` accepts...', delay: 60 },
  { text: '  ./utils/validators.py:12: if handleAuth(r):', delay: 50 },
  { text: '  ./config/routes.yaml:7:  handler: handleAuth', delay: 50 },
  { text: '  ./middleware/session.py:31: await handleAuth(req)', delay: 40 },
  { text: '  ./auth/oauth.py:55:  return handleAuth(token)', delay: 40 },
  { text: '  ./tests/test_cors.py:19: mock_handleAuth()', delay: 30 },
  { text: '  ./auth/jwt.py:8: from .handler import handleAuth', delay: 30 },
  { text: '  ./routes/v2/api.py:14:    handleAuth(ctx, v2=True)', delay: 30 },
  { text: '  ./middleware/ratelimit.py:22: handleAuth(throttled_req)', delay: 20 },
  { text: '  ... (847 results)', delay: 300, isSummary: true },
];

export function ProblemScene() {
  const [visibleLines, setVisibleLines] = useState<number>(0);
  const [showOverlay, setShowOverlay] = useState(false);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef(0);
  const lineTimers = useRef<ReturnType<typeof setTimeout>[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);

  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Cascade grep results with accelerating speed (frustration builds)
  useEffect(() => {
    if (prefersReducedMotion) {
      setVisibleLines(GREP_RESULTS.length);
      setShowOverlay(true);
      return;
    }

    let totalDelay = 400; // initial delay
    GREP_RESULTS.forEach((line, i) => {
      const lineDelay = line.delay || 120;
      totalDelay += lineDelay;
      const timer = setTimeout(() => {
        setVisibleLines(i + 1);
      }, totalDelay);
      lineTimers.current.push(timer);
    });

    // Show overlay text after grep completes
    const overlayTimer = setTimeout(() => setShowOverlay(true), totalDelay + 600);
    lineTimers.current.push(overlayTimer);

    return () => {
      lineTimers.current.forEach(clearTimeout);
      lineTimers.current = [];
    };
  }, [prefersReducedMotion]);

  // Auto-scroll the terminal output
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [visibleLines]);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 24,
        padding: '32px 48px',
        height: '100%',
        position: 'relative',
      }}
    >
      {/* Scene header */}
      <div>
        <motion.div
          initial={{ x: -20, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          transition={{ delay: 0.1 }}
          style={{
            fontSize: 11,
            color: COLORS.error,
            letterSpacing: 3,
            fontFamily: 'monospace',
            marginBottom: 8,
          }}
        >
          {GLYPHS.statement} SCENE 1 — THE PROBLEM
        </motion.div>
        <motion.h2
          initial={{ y: 10, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.2 }}
          style={{
            fontSize: 28,
            color: COLORS.text,
            fontWeight: 700,
            lineHeight: 1.3,
            fontFamily: '"JetBrains Mono", monospace',
          }}
        >
          your agent uses grep.<br />
          <span style={{ color: COLORS.error }}>
            that's like using a flashlight in a library.
          </span>
        </motion.h2>
      </div>

      {/* Main content: terminal + counters */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '1.5fr 1fr',
        gap: 24,
        flex: 1,
        minHeight: 0,
      }}>
        {/* Terminal with grep output */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.3 }}
        >
          <div style={{
            background: 'rgba(0, 0, 0, 0.7)',
            border: `1px solid ${COLORS.error}25`,
            borderRadius: 8,
            padding: '14px 18px',
            fontFamily: '"JetBrains Mono", "Fira Code", monospace',
            fontSize: 12,
            lineHeight: 1.6,
            height: '100%',
            display: 'flex',
            flexDirection: 'column',
            boxShadow: `0 0 40px ${COLORS.error}08, inset 0 0 40px ${COLORS.error}03`,
          }}>
            {/* Terminal dots */}
            <div style={{ display: 'flex', gap: 6, marginBottom: 10, flexShrink: 0 }}>
              {['#FF5F56', '#FFBD2E', '#27C93F'].map((c, i) => (
                <div key={i} style={{ width: 9, height: 9, borderRadius: '50%', background: c, opacity: 0.7 }} />
              ))}
              <div style={{ flex: 1 }} />
              <div style={{ fontSize: 9, color: COLORS.textDim }}>bash</div>
            </div>

            {/* Scrolling output */}
            <div
              ref={scrollRef}
              style={{
                flex: 1,
                overflowY: 'auto',
                overflowX: 'hidden',
              }}
            >
              {GREP_RESULTS.slice(0, visibleLines).map((line, i) => (
                <div
                  key={i}
                  style={{
                    color: line.isCommand
                      ? COLORS.D
                      : line.isSummary
                      ? COLORS.textMuted
                      : COLORS.error,
                    opacity: line.isCommand ? 1 : (line.isSummary ? 0.7 : 0.85),
                    whiteSpace: 'nowrap',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    minHeight: line.text === '' ? 12 : undefined,
                  }}
                >
                  {line.text}
                </div>
              ))}

              {/* Blinking cursor at end */}
              {visibleLines > 0 && visibleLines < GREP_RESULTS.length && (
                <span style={{
                  display: 'inline-block',
                  width: 7,
                  height: 13,
                  background: COLORS.error,
                  animation: 'cursor-blink 1.06s step-end infinite',
                }} />
              )}
            </div>
          </div>
        </motion.div>

        {/* Right side: token counters + cost */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16, justifyContent: 'center' }}>
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.8 }}
          >
            <TokenCounter
              label="tokens burned"
              targetValue={12000}
              color={COLORS.error}
              duration={3500}
              startDelay={500}
              showCost
              costValue="$0.04"
            />
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 1.2 }}
          >
            <TokenCounter
              label="tool calls"
              targetValue={12}
              color={COLORS.error}
              duration={2500}
              startDelay={800}
            />
          </motion.div>

          {/* The frustration callout */}
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 2.5 }}
            style={{
              fontSize: 12,
              color: COLORS.textMuted,
              fontFamily: 'monospace',
              lineHeight: 1.8,
              background: `${COLORS.error}08`,
              border: `1px solid ${COLORS.error}20`,
              borderRadius: 8,
              padding: '12px 16px',
            }}
          >
            <div style={{ color: COLORS.error, marginBottom: 4 }}>
              {GLYPHS.statement} 847 results. Zero ranking.
            </div>
            <div>{GLYPHS.statement} Every result costs tokens to process.</div>
            <div>{GLYPHS.statement} Agent doing archaeology, not engineering.</div>
          </motion.div>
        </div>
      </div>

      {/* Bottom overlay text — the hook */}
      {showOverlay && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          style={{
            textAlign: 'center',
            fontFamily: '"JetBrains Mono", monospace',
            fontSize: 14,
            color: COLORS.M,
            letterSpacing: 1,
            padding: '8px 0',
          }}
        >
          <span style={{ opacity: 0.6 }}>{GLYPHS.statement}</span>{' '}
          your agent spends most of its budget{' '}
          <span style={{ color: COLORS.error }}>being lost.</span>
        </motion.div>
      )}

    </motion.div>
  );
}
