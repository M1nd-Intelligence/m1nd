import { useState, useEffect, useRef } from 'react';

interface TypewriterOptions {
  text: string;
  speed?: number; // ms per character
  startDelay?: number;
  onComplete?: () => void;
  enabled?: boolean;
}

interface TypewriterResult {
  displayedText: string;
  isTyping: boolean;
  isComplete: boolean;
  cursorVisible: boolean;
}

/**
 * rAF-based typewriter hook.
 * Uses requestAnimationFrame with elapsed-time tracking instead of setTimeout chains.
 * Respects prefers-reduced-motion by showing text instantly.
 */
export function useTypewriter({
  text,
  speed = 15,
  startDelay = 0,
  onComplete,
  enabled = true,
}: TypewriterOptions): TypewriterResult {
  const [charIndex, setCharIndex] = useState(0);
  const [started, setStarted] = useState(false);
  const [cursorVisible, setCursorVisible] = useState(true);
  const lastCharTime = useRef(0);
  const rafRef = useRef<number>(0);
  const onCompleteRef = useRef(onComplete);
  onCompleteRef.current = onComplete;

  // Check prefers-reduced-motion
  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Start delay
  useEffect(() => {
    if (!enabled) return;
    if (prefersReducedMotion) {
      setStarted(true);
      setCharIndex(text.length);
      return;
    }
    const timer = setTimeout(() => setStarted(true), startDelay);
    return () => clearTimeout(timer);
  }, [startDelay, enabled, prefersReducedMotion, text.length]);

  // rAF typing loop — charIndex removed from deps to prevent O(n^2) cancel/register
  useEffect(() => {
    if (!started || !enabled || prefersReducedMotion) return;
    if (charIndex >= text.length) {
      onCompleteRef.current?.();
      return;
    }

    const animate = (timestamp: number) => {
      if (!lastCharTime.current) lastCharTime.current = timestamp;
      const elapsed = timestamp - lastCharTime.current;

      if (elapsed >= speed) {
        lastCharTime.current = timestamp;
        setCharIndex(prev => Math.min(prev + 1, text.length));
      }
      rafRef.current = requestAnimationFrame(animate);
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      lastCharTime.current = 0;
    };
  }, [started, text, speed, enabled, prefersReducedMotion]);

  // Cursor blink (530ms per the contracts)
  useEffect(() => {
    const interval = setInterval(() => setCursorVisible(v => !v), 530);
    return () => clearInterval(interval);
  }, []);

  return {
    displayedText: text.slice(0, charIndex),
    isTyping: started && charIndex < text.length,
    isComplete: charIndex >= text.length,
    cursorVisible,
  };
}

/**
 * Multi-line typewriter -- types lines sequentially.
 */
export function useMultiLineTypewriter(
  lines: string[],
  speed = 15,
  startDelay = 0,
  lineGap = 80,
  enabled = true,
) {
  const [currentLine, setCurrentLine] = useState(0);
  const [completedLines, setCompletedLines] = useState<string[]>([]);
  const [activeLine, setActiveLine] = useState('');
  const [started, setStarted] = useState(false);
  const [allComplete, setAllComplete] = useState(false);
  const [cursorVisible, setCursorVisible] = useState(true);
  const lastCharTime = useRef(0);
  const rafRef = useRef<number>(0);
  const charIndex = useRef(0);
  const lineGapRef = useRef(false);

  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  useEffect(() => {
    if (!enabled) return;
    if (prefersReducedMotion) {
      setCompletedLines([...lines]);
      setAllComplete(true);
      setCurrentLine(lines.length);
      return;
    }
    const timer = setTimeout(() => setStarted(true), startDelay);
    return () => clearTimeout(timer);
  }, [startDelay, enabled, prefersReducedMotion, lines]);

  useEffect(() => {
    if (!started || !enabled || prefersReducedMotion || allComplete) return;
    if (currentLine >= lines.length) {
      setAllComplete(true);
      return;
    }

    const line = lines[currentLine];

    const animate = (timestamp: number) => {
      if (!lastCharTime.current) lastCharTime.current = timestamp;
      const elapsed = timestamp - lastCharTime.current;

      if (lineGapRef.current) {
        // Between lines gap
        if (elapsed >= lineGap) {
          lineGapRef.current = false;
          lastCharTime.current = timestamp;
          charIndex.current = 0;
          setActiveLine('');
          setCurrentLine(prev => prev + 1);
        }
        rafRef.current = requestAnimationFrame(animate);
        return;
      }

      if (charIndex.current >= line.length) {
        // Line complete
        setCompletedLines(prev => [...prev, line]);
        setActiveLine('');
        lineGapRef.current = true;
        lastCharTime.current = timestamp;
        rafRef.current = requestAnimationFrame(animate);
        return;
      }

      if (elapsed >= speed) {
        lastCharTime.current = timestamp;
        charIndex.current++;
        setActiveLine(line.slice(0, charIndex.current));
      }
      rafRef.current = requestAnimationFrame(animate);
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      lastCharTime.current = 0;
      charIndex.current = 0;
      lineGapRef.current = false;
    };
  }, [started, currentLine, lines, speed, lineGap, enabled, prefersReducedMotion, allComplete]);

  // Cursor blink
  useEffect(() => {
    const interval = setInterval(() => setCursorVisible(v => !v), 530);
    return () => clearInterval(interval);
  }, []);

  return {
    completedLines,
    activeLine,
    currentLineIndex: currentLine,
    isTyping: started && !allComplete,
    isComplete: allComplete,
    cursorVisible,
  };
}

/**
 * Animated counter -- counts from start to target using rAF.
 */
export function useAnimatedCounter(
  target: number,
  duration: number = 2000,
  startDelay: number = 0,
  enabled: boolean = true,
): number {
  const [value, setValue] = useState(0);
  const startTimeRef = useRef<number>(0);
  const rafRef = useRef<number>(0);

  const prefersReducedMotion = typeof window !== 'undefined'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  useEffect(() => {
    if (!enabled) return;
    if (prefersReducedMotion) {
      setValue(target);
      return;
    }

    const timer = setTimeout(() => {
      const animate = (timestamp: number) => {
        if (!startTimeRef.current) startTimeRef.current = timestamp;
        const elapsed = timestamp - startTimeRef.current;
        const progress = Math.min(elapsed / duration, 1);
        // Ease-out cubic for satisfying deceleration
        const eased = 1 - Math.pow(1 - progress, 3);
        setValue(Math.round(eased * target));

        if (progress < 1) {
          rafRef.current = requestAnimationFrame(animate);
        }
      };
      rafRef.current = requestAnimationFrame(animate);
    }, startDelay);

    return () => {
      clearTimeout(timer);
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      startTimeRef.current = 0;
    };
  }, [target, duration, startDelay, enabled, prefersReducedMotion]);

  return value;
}
