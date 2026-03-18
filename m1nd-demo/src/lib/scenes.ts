import { COLORS } from './colors';
import type { SceneConfig } from './types';

export const SCENES: SceneConfig[] = [
  {
    id: 'problem',
    title: 'The Token Burn',
    subtitle: 'grep burns tokens. Every. Single. Query.',
    durationMs: 5000,
    color: COLORS.error,
  },
  {
    id: 'command',
    title: 'm1nd.activate',
    subtitle: 'One query. The graph answers.',
    durationMs: 5000,
    color: COLORS.M,
  },
  {
    id: 'brain',
    title: 'Graph Activates',
    subtitle: '4 dimensions. Spreading activation. 31ms.',
    durationMs: 6000,
    color: COLORS.one,
  },
  {
    id: 'killer',
    title: 'surgical_context + apply_batch',
    subtitle: '2 calls. 3 files. Zero partial state.',
    durationMs: 6000,
    color: COLORS.N,
  },
  {
    id: 'economy',
    title: 'Token Counter: Frozen',
    subtitle: '99,300 tokens saved. $4.40 back in your pocket.',
    durationMs: 5000,
    color: COLORS.D,
  },
  {
    id: 'proof',
    title: 'Before vs After',
    subtitle: 'grep loop: 12 calls → m1nd: 2 calls.',
    durationMs: 7000,
    color: COLORS.M,
  },
  {
    id: 'identity',
    title: '⍌⍐⍂𝔻⟁',
    subtitle: 'The nervous system reveals itself.',
    durationMs: 5000,
    color: COLORS.N,
  },
  {
    id: 'philosophy',
    title: 'You don\'t need to understand it.',
    subtitle: 'Your agent does.',
    durationMs: 10000,
    color: COLORS.D,
  },
];

export const TOTAL_DURATION_MS = SCENES.reduce((sum, s) => sum + s.durationMs, 0);
