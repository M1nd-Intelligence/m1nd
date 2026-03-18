import type { Scene, SceneDescriptionReload, Slide } from '../scenes';
export declare enum PlaybackState {
    Playing = 0,
    Rendering = 1,
    Paused = 2,
    Presenting = 3
}
/**
 * A general class for managing a sequence of scenes.
 *
 * @remarks
 * This class provides primitive operations that can be executed on a scene
 * sequence, such as {@link progress} or {@link seek}.
 *
 * @internal
 */
export declare class PlaybackManager {
    /**
     * Triggered when the active scene changes.
     *
     * @eventProperty
     */
    get onSceneChanged(): import("../events").SubscribableValueEvent<Scene<unknown>>;
    /**
     * Triggered when the scenes get recalculated.
     *
     * @remarks
     * This event indicates that the timing of at least one scene has changed.
     *
     * @eventProperty
     */
    get onScenesRecalculated(): import("../events").SubscribableValueEvent<Scene<unknown>[]>;
    frame: number;
    speed: number;
    fps: number;
    duration: number;
    finished: boolean;
    slides: Slide[];
    previousScene: Scene | null;
    state: PlaybackState;
    get currentScene(): Scene;
    set currentScene(scene: Scene);
    private currentSceneReference;
    private scenes;
    setup(scenes: Scene[]): void;
    progress(): Promise<boolean>;
    seek(frame: number): Promise<boolean>;
    goBack(): Promise<void>;
    goForward(): Promise<void>;
    goTo(slideId: string): Promise<void>;
    private seekSlide;
    reset(): Promise<void>;
    reload(description?: SceneDescriptionReload<never>): void;
    recalculate(): Promise<void>;
    private next;
    private findBestScene;
    private getNextScene;
}
//# sourceMappingURL=PlaybackManager.d.ts.map