/**
 * Describes the state of a scene.
 */
export declare enum SceneState {
    /**
     * The scene has just been created/reset.
     */
    Initial = 0,
    /**
     * The scene has finished transitioning in.
     *
     * @remarks
     * Informs the Project that the previous scene is no longer necessary and can
     * be disposed of.
     */
    AfterTransitionIn = 1,
    /**
     * The scene is ready to transition out.
     *
     * @remarks
     * Informs the project that the next scene can begin.
     * The {@link Scene.next} method will still be invoked until the next scene
     * enters {@link AfterTransitionIn}.
     */
    CanTransitionOut = 2,
    /**
     * The scene has finished.
     *
     * @remarks
     * Invoking {@link Scene.next} won't have any effect.
     */
    Finished = 3
}
//# sourceMappingURL=SceneState.d.ts.map