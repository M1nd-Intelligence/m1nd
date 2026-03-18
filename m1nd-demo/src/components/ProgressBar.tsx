import { COLORS } from '../lib/colors';
import { SCENES } from '../lib/scenes';

interface ProgressBarProps {
  currentScene: number;
  onSelect: (index: number) => void;
}

export function ProgressBar({ currentScene, onSelect }: ProgressBarProps) {
  return (
    <div style={{
      display: 'flex',
      gap: 4,
      alignItems: 'center',
      padding: '0 24px 8px',
    }}>
      {SCENES.map((scene, i) => (
        <button
          key={i}
          onClick={() => onSelect(i)}
          title={scene.title}
          aria-label={`Go to scene ${i + 1}: ${scene.title}`}
          style={{
            flex: i === currentScene ? 3 : 1,
            height: 4,
            background: i < currentScene
              ? COLORS.D
              : i === currentScene
              ? scene.color
              : COLORS.textDim,
            border: 'none',
            borderRadius: 2,
            cursor: 'pointer',
            transition: 'flex 0.3s ease, background 0.3s ease',
            padding: 0,
            opacity: i > currentScene ? 0.4 : 1,
          }}
        />
      ))}
    </div>
  );
}
