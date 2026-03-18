import { useEffect, useState, useRef } from 'react';
import { COLORS } from '../lib/colors';

interface TerminalLine {
  text: string;
  color?: string;
  delay?: number; // extra delay before this line starts
}

interface TerminalBlockProps {
  lines: (string | TerminalLine)[];
  typingSpeed?: number; // ms per character
  startDelay?: number;
  showPrompt?: boolean;
  onComplete?: () => void;
  scrolling?: boolean; // auto-scroll output for grep-style
  glowColor?: string; // glow effect on the terminal border
}

function normalizeLine(line: string | TerminalLine): TerminalLine {
  if (typeof line === 'string') return { text: line };
  return line;
}

export function TerminalBlock({
  lines: rawLines,
  typingSpeed = 15,
  startDelay = 0,
  showPrompt = true,
  onComplete,
  scrolling = false,
  glowColor,
}: TerminalBlockProps) {
  const lines = rawLines.map(normalizeLine);
  const [displayedLines, setDisplayedLines] = useState<string[]>([]);
  const [currentLine, setCurrentLine] = useState(-1);
  const [charIdx, setCharIdx] = useState(0);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef(0);
  const onCompleteRef = useRef(onComplete);
  const scrollRef = useRef<HTMLDivElement>(null);
  onCompleteRef.current = onComplete;

  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Start with delay
  useEffect(() => {
    if (prefersReducedMotion) {
      setDisplayedLines(lines.map(l => l.text));
      setCurrentLine(lines.length);
      return;
    }
    const timer = setTimeout(() => setCurrentLine(0), startDelay);
    return () => clearTimeout(timer);
  }, [startDelay, prefersReducedMotion]);

  // rAF animation loop
  useEffect(() => {
    if (currentLine < 0 || currentLine >= lines.length || prefersReducedMotion) {
      if (currentLine >= lines.length && !prefersReducedMotion) {
        onCompleteRef.current?.();
      }
      return;
    }

    const line = lines[currentLine];
    const lineDelay = line.delay || 0;

    const animate = (timestamp: number) => {
      if (!lastTimeRef.current) {
        lastTimeRef.current = timestamp + lineDelay; // add line-specific delay
      }
      const elapsed = timestamp - lastTimeRef.current;

      if (elapsed < 0) {
        // Still in line delay
        rafRef.current = requestAnimationFrame(animate);
        return;
      }

      if (charIdx >= line.text.length) {
        // Line complete
        setDisplayedLines(prev => {
          const next = [...prev];
          next[currentLine] = line.text;
          return next;
        });
        lastTimeRef.current = 0;
        setCharIdx(0);
        setCurrentLine(prev => prev + 1);
        return;
      }

      if (elapsed >= typingSpeed) {
        lastTimeRef.current = timestamp;
        const newIdx = charIdx + 1;
        setCharIdx(newIdx);
        setDisplayedLines(prev => {
          const next = [...prev];
          next[currentLine] = line.text.slice(0, newIdx);
          return next;
        });
      }

      rafRef.current = requestAnimationFrame(animate);
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      lastTimeRef.current = 0;
    };
  }, [currentLine, charIdx, lines, typingSpeed, prefersReducedMotion]);

  // Auto-scroll
  useEffect(() => {
    if (scrolling && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [displayedLines, scrolling]);

  const isComment = (text: string) => text.trim().startsWith('#') || text.trim().startsWith('//');
  const isCommand = (text: string) => text.trim().startsWith('$') || text.trim().startsWith('>');
  const isJson = (text: string) => /^\s*[{"\[\]}]/.test(text);
  const isResult = (text: string) => text.trim().startsWith('./') || text.trim().startsWith('...');
  const isOutput = (text: string) => text.trim().startsWith('{') || text.trim().startsWith('"');

  const getLineColor = (line: TerminalLine, text: string) => {
    if (line.color) return line.color;
    if (isComment(text)) return COLORS.textDim;
    if (isCommand(text)) return COLORS.D;
    if (isResult(text)) return COLORS.error;
    if (isJson(text) || isOutput(text)) return COLORS.M;
    return COLORS.text;
  };

  const isCurrentlyTyping = currentLine >= 0 && currentLine < lines.length;

  return (
    <div style={{
      background: 'rgba(0, 0, 0, 0.7)',
      border: `1px solid ${glowColor ? `${glowColor}40` : COLORS.border}`,
      borderRadius: 8,
      padding: '14px 18px',
      fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
      fontSize: 13,
      lineHeight: 1.7,
      minHeight: 80,
      position: 'relative',
      overflow: 'hidden',
      boxShadow: glowColor ? `0 0 30px ${glowColor}15, inset 0 0 30px ${glowColor}05` : undefined,
    }}>
      {/* Terminal header dots */}
      <div style={{ display: 'flex', gap: 6, marginBottom: 12 }}>
        {['#FF5F56', '#FFBD2E', '#27C93F'].map((c, i) => (
          <div key={i} style={{ width: 10, height: 10, borderRadius: '50%', background: c, opacity: 0.7 }} />
        ))}
        <div style={{ flex: 1 }} />
        <div style={{ fontSize: 10, color: COLORS.textDim, fontFamily: 'monospace' }}>
          terminal
        </div>
      </div>

      <div
        ref={scrollRef}
        style={{
          maxHeight: scrolling ? 300 : undefined,
          overflowY: scrolling ? 'auto' : undefined,
        }}
      >
        {lines.map((line, i) => {
          const displayed = i < displayedLines.length ? (displayedLines[i] ?? '') : '';
          const isActive = i === currentLine;
          const isCompleted = i < currentLine;

          return (
            <div key={i} style={{ display: 'flex', alignItems: 'center', minHeight: 22 }}>
              {showPrompt && isCommand(line.text) && (
                <span style={{ color: COLORS.D, marginRight: 8, userSelect: 'none', opacity: 0.7 }}>
                </span>
              )}
              <span style={{ color: getLineColor(line, displayed || line.text) }}>
                {(isCompleted || isActive) ? displayed : ''}
                {isActive && isCurrentlyTyping && (
                  <span className="terminal-cursor" style={{
                    display: 'inline-block',
                    width: 8,
                    height: 15,
                    background: COLORS.M,
                    marginLeft: 1,
                    verticalAlign: 'text-bottom',
                    animation: 'cursor-blink 1.06s step-end infinite',
                  }} />
                )}
              </span>
            </div>
          );
        })}
      </div>

    </div>
  );
}
