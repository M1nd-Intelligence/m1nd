/**
 * Describes the state of a scene.
 */
export var SceneState;
(function (SceneState) {
    /**
     * The scene has just been created/reset.
     */
    SceneState[SceneState["Initial"] = 0] = "Initial";
    /**
     * The scene has finished transitioning in.
     *
     * @remarks
     * Informs the Project that the previous scene is no longer necessary and can
     * be disposed of.
     */
    SceneState[SceneState["AfterTransitionIn"] = 1] = "AfterTransitionIn";
    /**
     * The scene is ready to transition out.
     *
     * @remarks
     * Informs the project that the next scene can begin.
     * The {@link Scene.next} method will still be invoked until the next scene
     * enters {@link AfterTransitionIn}.
     */
    SceneState[SceneState["CanTransitionOut"] = 2] = "CanTransitionOut";
    /**
     * The scene has finished.
     *
     * @remarks
     * Invoking {@link Scene.next} won't have any effect.
     */
    SceneState[SceneState["Finished"] = 3] = "Finished";
})(SceneState || (SceneState = {}));
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiU2NlbmVTdGF0ZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zY2VuZXMvU2NlbmVTdGF0ZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQTs7R0FFRztBQUNILE1BQU0sQ0FBTixJQUFZLFVBZ0NYO0FBaENELFdBQVksVUFBVTtJQUNwQjs7T0FFRztJQUNILGlEQUFPLENBQUE7SUFFUDs7Ozs7O09BTUc7SUFDSCxxRUFBaUIsQ0FBQTtJQUVqQjs7Ozs7OztPQU9HO0lBQ0gsbUVBQWdCLENBQUE7SUFFaEI7Ozs7O09BS0c7SUFDSCxtREFBUSxDQUFBO0FBQ1YsQ0FBQyxFQWhDVyxVQUFVLEtBQVYsVUFBVSxRQWdDckIifQ==