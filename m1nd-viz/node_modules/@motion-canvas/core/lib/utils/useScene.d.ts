import type { Scene } from '../scenes';
/**
 * Get a reference to the current scene.
 */
export declare function useScene(): Scene;
export declare function startScene(scene: Scene): void;
export declare function endScene(scene: Scene): void;
export declare function useLogger(): Console | import("..").Logger;
/**
 * Mark the current scene as ready to transition out.
 *
 * @remarks
 * Usually used together with transitions. When a scene is marked as finished,
 * the transition will start but the scene generator will continue running.
 */
export declare function finishScene(): void;
//# sourceMappingURL=useScene.d.ts.map