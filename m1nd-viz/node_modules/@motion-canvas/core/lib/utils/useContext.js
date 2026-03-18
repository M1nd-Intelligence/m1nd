import { useScene } from './useScene';
/**
 * Provide a function to access the Context2D before the scene is rendered.
 *
 * @param callback - The function that will be provided the context before render.
 */
export function useContext(callback) {
    return useScene().lifecycleEvents.onBeginRender.subscribe(callback);
}
/**
 * Provide a function to access the Context2D after the scene is rendered.
 *
 * @param callback - The function that will be provided the context after render.
 */
export function useContextAfter(callback) {
    return useScene().lifecycleEvents.onFinishRender.subscribe(callback);
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidXNlQ29udGV4dC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy91dGlscy91c2VDb250ZXh0LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBQyxRQUFRLEVBQUMsTUFBTSxZQUFZLENBQUM7QUFFcEM7Ozs7R0FJRztBQUNILE1BQU0sVUFBVSxVQUFVLENBQ3hCLFFBQWlEO0lBRWpELE9BQU8sUUFBUSxFQUFFLENBQUMsZUFBZSxDQUFDLGFBQWEsQ0FBQyxTQUFTLENBQUMsUUFBUSxDQUFDLENBQUM7QUFDdEUsQ0FBQztBQUVEOzs7O0dBSUc7QUFDSCxNQUFNLFVBQVUsZUFBZSxDQUM3QixRQUFpRDtJQUVqRCxPQUFPLFFBQVEsRUFBRSxDQUFDLGVBQWUsQ0FBQyxjQUFjLENBQUMsU0FBUyxDQUFDLFFBQVEsQ0FBQyxDQUFDO0FBQ3ZFLENBQUMifQ==