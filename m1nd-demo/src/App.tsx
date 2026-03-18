import { useState, useEffect } from 'react';
import { COLORS } from './lib/colors';
import { SCENES } from './lib/scenes';
import { ProgressBar } from './components/ProgressBar';
import {
  ProblemScene,
  CommandScene,
  BrainScene,
  KillerScene,
  EconomyScene,
  ProofScene,
  IdentityScene,
  PhilosophyScene,
} from './scenes';

const SCENE_COMPONENTS = [
  ProblemScene,
  CommandScene,
  BrainScene,
  KillerScene,
  EconomyScene,
  ProofScene,
  IdentityScene,
  PhilosophyScene,
];

export default function App() {
  const [currentScene, setCurrentScene] = useState(0);

  const goToScene = (index: number) => {
    setCurrentScene(Math.max(0, Math.min(SCENES.length - 1, index)));
  };

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight' || e.key === 'ArrowDown') goToScene(currentScene + 1);
      if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') goToScene(currentScene - 1);
      if (e.key === ' ') {
        e.preventDefault();
        goToScene(currentScene + 1);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [currentScene]);

  const CurrentScene = SCENE_COMPONENTS[currentScene];

  return (
    <div style={{
      width: '100vw',
      height: '100vh',
      background: COLORS.bg,
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden',
    }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        padding: '12px 24px',
        fontFamily: 'monospace',
        fontSize: 12,
        color: '#8090A8',
      }}>
        <div>
          <span style={{ color: COLORS.M }}>m</span>
          <span style={{ color: COLORS.one }}>1</span>
          <span style={{ color: COLORS.N }}>n</span>
          <span style={{ color: COLORS.D }}>d</span>
          <span style={{ marginLeft: 12 }}>
            {currentScene + 1}/{SCENES.length} — {SCENES[currentScene]?.title?.toUpperCase()}
          </span>
        </div>
        <div>[←→] navigate · [SPACE] advance</div>
      </div>

      {/* Progress */}
      <ProgressBar currentScene={currentScene} onSelect={goToScene} />

      {/* Scene */}
      <div style={{ flex: 1, overflow: 'hidden' }}>
        <CurrentScene key={currentScene} />
      </div>
    </div>
  );
}
