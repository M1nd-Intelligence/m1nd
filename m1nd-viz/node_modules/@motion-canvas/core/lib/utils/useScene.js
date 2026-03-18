const SceneStack = [];
/**
 * Get a reference to the current scene.
 */
export function useScene() {
    const scene = SceneStack.at(-1);
    if (!scene) {
        throw new Error('The scene is not available in the current context.');
    }
    return scene;
}
export function startScene(scene) {
    SceneStack.push(scene);
}
export function endScene(scene) {
    if (SceneStack.pop() !== scene) {
        throw new Error('startScene/endScene were called out of order.');
    }
}
export function useLogger() {
    return SceneStack.at(-1)?.logger ?? console;
}
/**
 * Mark the current scene as ready to transition out.
 *
 * @remarks
 * Usually used together with transitions. When a scene is marked as finished,
 * the transition will start but the scene generator will continue running.
 */
export function finishScene() {
    useScene().enterCanTransitionOut();
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidXNlU2NlbmUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvdXRpbHMvdXNlU2NlbmUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBRUEsTUFBTSxVQUFVLEdBQVksRUFBRSxDQUFDO0FBRS9COztHQUVHO0FBQ0gsTUFBTSxVQUFVLFFBQVE7SUFDdEIsTUFBTSxLQUFLLEdBQUcsVUFBVSxDQUFDLEVBQUUsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDO0lBQ2hDLElBQUksQ0FBQyxLQUFLLEVBQUU7UUFDVixNQUFNLElBQUksS0FBSyxDQUFDLG9EQUFvRCxDQUFDLENBQUM7S0FDdkU7SUFDRCxPQUFPLEtBQUssQ0FBQztBQUNmLENBQUM7QUFFRCxNQUFNLFVBQVUsVUFBVSxDQUFDLEtBQVk7SUFDckMsVUFBVSxDQUFDLElBQUksQ0FBQyxLQUFLLENBQUMsQ0FBQztBQUN6QixDQUFDO0FBRUQsTUFBTSxVQUFVLFFBQVEsQ0FBQyxLQUFZO0lBQ25DLElBQUksVUFBVSxDQUFDLEdBQUcsRUFBRSxLQUFLLEtBQUssRUFBRTtRQUM5QixNQUFNLElBQUksS0FBSyxDQUFDLCtDQUErQyxDQUFDLENBQUM7S0FDbEU7QUFDSCxDQUFDO0FBRUQsTUFBTSxVQUFVLFNBQVM7SUFDdkIsT0FBTyxVQUFVLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQyxDQUFDLEVBQUUsTUFBTSxJQUFJLE9BQU8sQ0FBQztBQUM5QyxDQUFDO0FBRUQ7Ozs7OztHQU1HO0FBQ0gsTUFBTSxVQUFVLFdBQVc7SUFDekIsUUFBUSxFQUFFLENBQUMscUJBQXFCLEVBQUUsQ0FBQztBQUNyQyxDQUFDIn0=