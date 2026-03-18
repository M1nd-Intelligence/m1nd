import { motion, AnimatePresence } from 'framer-motion';
import { useEffect, useState, useRef } from 'react';
import { COLORS, GLYPHS } from '../lib/colors';

/**
 * SCENE 4: THE KILLER FEATURE (Layer 3)
 *
 * apply_batch. Show 3 file panels side by side. Code changing simultaneously
 * in all 3. "3 files. atomic. 165ms." This is the jaw-drop moment.
 *
 * Emotion: REVELATION
 * The moment they realize m1nd doesn't just search -- it EDITS.
 */

interface FilePanel {
  name: string;
  operation: 'insert' | 'replace' | 'delete';
  beforeLines: string[];
  afterLines: string[];
  changeLineIdx: number; // which line gets highlighted as changed
}

const FILES: FilePanel[] = [
  {
    name: 'auth/handler.py',
    operation: 'insert',
    beforeLines: [
      'def handleAuth(request):',
      '    token = request.headers["auth"]',
      '    user = validate(token)',
      '    return user',
    ],
    afterLines: [
      'def handleAuth(request):',
      '    rate_limit_check(request)',
      '    token = request.headers["auth"]',
      '    user = validate(token)',
      '    return user',
    ],
    changeLineIdx: 1,
  },
  {
    name: 'middleware/cors.py',
    operation: 'replace',
    beforeLines: [
      'from auth import handleAuth',
      '',
      'def cors_middleware(req):',
      '    pass_through()',
      '    return next(req)',
    ],
    afterLines: [
      'from auth import handleAuth',
      '',
      'def cors_middleware(req):',
      '    validate(ctx)',
      '    return next(req)',
    ],
    changeLineIdx: 3,
  },
  {
    name: 'tests/test_auth.py',
    operation: 'insert',
    beforeLines: [
      'def test_auth_basic():',
      '    assert handleAuth(mock)',
      '',
      '# TODO: rate limit test',
    ],
    afterLines: [
      'def test_auth_basic():',
      '    assert handleAuth(mock)',
      '',
      'def test_rate_limit():',
      '    req = mock_request()',
      '    assert rate_limited(req)',
    ],
    changeLineIdx: 3,
  },
];

const COMMAND_LINES = [
  'm1nd.apply_batch([',
  '  {"file": "auth/handler.py",',
  '   "operation": "insert",',
  '   "content": "rate_limit_check(request)"},',
  '  {"file": "middleware/cors.py",',
  '   "operation": "replace",',
  '   "old": "pass_through()", "new": "validate(ctx)"},',
  '  {"file": "tests/test_auth.py",',
  '   "operation": "insert",',
  '   "content": "def test_rate_limit(): ..."}',
  '])',
];

export function KillerScene() {
  const [phase, setPhase] = useState<'typing' | 'executing' | 'file0' | 'file1' | 'file2' | 'complete'>('typing');
  const [commandChars, setCommandChars] = useState(0);
  const [fileComplete, setFileComplete] = useState<boolean[]>([false, false, false]);
  const [showGraphUpdate, setShowGraphUpdate] = useState(false);
  const [cursorVisible, setCursorVisible] = useState(true);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef(0);

  const fullCommand = COMMAND_LINES.join('\n');

  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Cursor blink
  useEffect(() => {
    const interval = setInterval(() => setCursorVisible(v => !v), 530);
    return () => clearInterval(interval);
  }, []);

  // Phase 1: Type the command (fast, 8ms/char)
  useEffect(() => {
    if (phase !== 'typing' || prefersReducedMotion) {
      if (prefersReducedMotion) {
        setCommandChars(fullCommand.length);
        setFileComplete([true, true, true]);
        setShowGraphUpdate(true);
        setPhase('complete');
      }
      return;
    }

    const speed = 8;
    const animate = (timestamp: number) => {
      if (!lastTimeRef.current) lastTimeRef.current = timestamp;
      const elapsed = timestamp - lastTimeRef.current;

      if (elapsed >= speed) {
        lastTimeRef.current = timestamp;
        setCommandChars(prev => {
          const next = prev + 1;
          if (next >= fullCommand.length) {
            setTimeout(() => setPhase('executing'), 400);
            return fullCommand.length;
          }
          return next;
        });
      }
      if (commandChars < fullCommand.length) {
        rafRef.current = requestAnimationFrame(animate);
      }
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      lastTimeRef.current = 0;
    };
  }, [phase, commandChars, fullCommand.length, prefersReducedMotion]);

  // Phase 2: Execute -- cascade file completions
  useEffect(() => {
    if (phase !== 'executing') return;
    const timers: ReturnType<typeof setTimeout>[] = [];

    timers.push(setTimeout(() => {
      setPhase('file0');
      setFileComplete([true, false, false]);
    }, 200));

    timers.push(setTimeout(() => {
      setPhase('file1');
      setFileComplete([true, true, false]);
    }, 400));

    timers.push(setTimeout(() => {
      setPhase('file2');
      setFileComplete([true, true, true]);
    }, 600));

    timers.push(setTimeout(() => {
      setShowGraphUpdate(true);
      setPhase('complete');
    }, 1200));

    return () => timers.forEach(clearTimeout);
  }, [phase]);

  const isExecuting = phase === 'executing' || phase === 'file0' || phase === 'file1' || phase === 'file2' || phase === 'complete';

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 20,
        padding: '28px 48px',
        height: '100%',
      }}
    >
      {/* Header */}
      <div>
        <motion.div
          initial={{ x: -20, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          style={{ fontSize: 11, color: COLORS.N, letterSpacing: 3, fontFamily: 'monospace', marginBottom: 6 }}
        >
          {GLYPHS.convergence} SCENE 4 -- THE KILLER FEATURE
        </motion.div>
        <motion.h2
          initial={{ y: 10, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.2 }}
          style={{ fontSize: 24, color: COLORS.text, fontWeight: 700, fontFamily: '"JetBrains Mono", monospace' }}
        >
          3 files. one shot. 165ms.<br />
          <span style={{ color: COLORS.N }}>this is the jaw-drop moment.</span>
        </motion.h2>
      </div>

      {/* Command terminal -- compact */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.3 }}
        style={{
          background: 'rgba(0, 0, 0, 0.7)',
          border: `1px solid ${COLORS.N}20`,
          borderRadius: 8,
          padding: '12px 16px',
          fontFamily: '"JetBrains Mono", monospace',
          fontSize: 11,
          lineHeight: 1.5,
          maxHeight: 170,
          overflowY: 'auto',
          boxShadow: `0 0 30px ${COLORS.N}08`,
        }}
      >
        <div style={{ display: 'flex', gap: 6, marginBottom: 8 }}>
          {['#FF5F56', '#FFBD2E', '#27C93F'].map((c, i) => (
            <div key={i} style={{ width: 8, height: 8, borderRadius: '50%', background: c, opacity: 0.6 }} />
          ))}
        </div>
        <pre style={{ margin: 0, color: COLORS.D, whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
          {fullCommand.slice(0, commandChars)}
          {phase === 'typing' && (
            <span style={{
              display: 'inline-block', width: 7, height: 13,
              background: COLORS.D, opacity: cursorVisible ? 1 : 0,
            }} />
          )}
        </pre>
      </motion.div>

      {/* Three file panels side by side */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(3, 1fr)',
        gap: 16,
        flex: 1,
        minHeight: 180,
      }}>
        {FILES.map((file, fileIdx) => {
          const isComplete = fileComplete[fileIdx];
          const isChanging = phase === `file${fileIdx}`;
          const showAfter = isComplete;
          const lines = showAfter ? file.afterLines : file.beforeLines;

          return (
            <motion.div
              key={file.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{
                opacity: isExecuting ? 1 : 0.3,
                y: 0,
                boxShadow: isComplete
                  ? `0 0 24px ${COLORS.D}20`
                  : isChanging
                  ? `0 0 28px ${COLORS.N}30`
                  : '0 0 0px transparent',
              }}
              transition={{ delay: 0.5 + fileIdx * 0.1, boxShadow: { duration: 0.4 } }}
              style={{
                background: 'rgba(0, 0, 0, 0.6)',
                border: `1px solid ${isComplete ? `${COLORS.D}50` : isChanging ? `${COLORS.N}70` : COLORS.border}`,
                borderRadius: 8,
                overflow: 'visible',
                display: 'flex',
                flexDirection: 'column',
                position: 'relative',
                transition: 'border-color 0.3s ease',
              }}
            >
              {/* File header */}
              <div style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '8px 12px',
                borderBottom: `1px solid ${COLORS.border}`,
                background: 'rgba(0, 0, 0, 0.4)',
              }}>
                <div style={{
                  width: 8, height: 8, borderRadius: '50%',
                  background: isComplete ? COLORS.D : (isChanging ? COLORS.N : COLORS.textDim),
                  transition: 'background 0.3s',
                  boxShadow: isComplete ? `0 0 8px ${COLORS.D}80` : 'none',
                }} />
                <div style={{ fontSize: 11, color: COLORS.textMuted, fontFamily: 'monospace', flex: 1 }}>
                  {file.name}
                </div>
                <div style={{
                  fontSize: 9,
                  padding: '2px 6px',
                  borderRadius: 3,
                  background: isComplete
                    ? `${COLORS.D}20`
                    : `${COLORS.textDim}20`,
                  color: isComplete ? COLORS.D : COLORS.textDim,
                  fontFamily: 'monospace',
                  letterSpacing: 1,
                }}>
                  {file.operation.toUpperCase()}
                </div>
              </div>

              {/* Code content */}
              <div style={{
                padding: '10px 12px',
                fontFamily: '"JetBrains Mono", monospace',
                fontSize: 11,
                lineHeight: 1.6,
                flex: 1,
                overflowY: 'auto',
                minHeight: 0,
              }}>
                {lines.map((line, lineIdx) => {
                  const isChanged = showAfter && lineIdx >= file.changeLineIdx && lineIdx < file.changeLineIdx + (file.afterLines.length - file.beforeLines.length + 1);

                  return (
                    <motion.div
                      key={`${file.name}-${lineIdx}-${showAfter ? 'after' : 'before'}`}
                      initial={isChanged ? { opacity: 0, x: -4, backgroundColor: `${COLORS.D}30` } : false}
                      animate={isChanged ? { opacity: 1, x: 0, backgroundColor: `${COLORS.D}14` } : {}}
                      transition={{ duration: 0.4, delay: isChanged ? 0.05 * (lineIdx - file.changeLineIdx) : 0 }}
                      style={{
                        display: 'flex',
                        gap: 8,
                        padding: '1px 4px',
                        background: isChanged ? `${COLORS.D}14` : 'transparent',
                        borderLeft: isChanged ? `2px solid ${COLORS.D}` : '2px solid transparent',
                      }}
                    >
                      <span style={{ color: COLORS.textDim, minWidth: 16, textAlign: 'right', fontSize: 10, opacity: 0.5 }}>
                        {lineIdx + 1}
                      </span>
                      <span style={{
                        color: isChanged ? COLORS.D : COLORS.text,
                        fontWeight: isChanged ? 600 : 400,
                      }}>
                        {line || '\u00A0'}
                      </span>
                    </motion.div>
                  );
                })}
              </div>

              {/* Checkmark overlay */}
              <AnimatePresence>
                {isComplete && (
                  <motion.div
                    initial={{ opacity: 0, scale: 0.5 }}
                    animate={{ opacity: 1, scale: 1 }}
                    transition={{ type: 'spring', stiffness: 300, damping: 15 }}
                    style={{
                      position: 'absolute',
                      bottom: 8,
                      right: 8,
                      width: 28,
                      height: 28,
                      borderRadius: '50%',
                      background: `${COLORS.D}20`,
                      border: `1px solid ${COLORS.D}60`,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      fontSize: 14,
                      color: COLORS.D,
                    }}
                  >
                    {'\u2713'}
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.div>
          );
        })}
      </div>

      {/* Graph updated notification */}
      <AnimatePresence>
        {showGraphUpdate && (
          <motion.div
            initial={{ opacity: 0, y: 10, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            transition={{ type: 'spring', stiffness: 200 }}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 24,
              padding: '12px 24px',
              background: `${COLORS.D}08`,
              border: `1px solid ${COLORS.D}25`,
              borderRadius: 10,
              fontFamily: 'monospace',
              fontSize: 12,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span style={{ color: COLORS.D, fontSize: 16 }}>{GLYPHS.convergence}</span>
              <span style={{ color: COLORS.D, fontWeight: 700 }}>graph updated</span>
            </div>
            <span style={{ color: COLORS.textDim }}>|</span>
            <span style={{ color: COLORS.textMuted }}>3 files written</span>
            <span style={{ color: COLORS.textDim }}>|</span>
            <span style={{ color: COLORS.textMuted }}>4 new edges</span>
            <span style={{ color: COLORS.textDim }}>|</span>
            <span style={{ color: COLORS.D }}>0 LLM tokens</span>
            <span style={{ color: COLORS.textDim }}>|</span>
            <span style={{ color: COLORS.one }}>165ms</span>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Bottom tagline */}
      {showGraphUpdate && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.5 }}
          style={{
            textAlign: 'center',
            fontFamily: '"JetBrains Mono", monospace',
            fontSize: 13,
            color: COLORS.N,
            letterSpacing: 1,
          }}
        >
          {GLYPHS.structure} write code + update the graph = one single call.
        </motion.div>
      )}
    </motion.div>
  );
}
