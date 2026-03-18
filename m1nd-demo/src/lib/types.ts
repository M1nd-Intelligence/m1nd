export type SceneId =
  | 'problem'
  | 'command'
  | 'brain'
  | 'killer'
  | 'economy'
  | 'proof'
  | 'identity'
  | 'philosophy';

export interface SceneConfig {
  id: SceneId;
  title: string;
  subtitle: string;
  durationMs: number;
  color: string;
}
