/**
 * Provide a function to access the Context2D before the scene is rendered.
 *
 * @param callback - The function that will be provided the context before render.
 */
export declare function useContext(callback: (ctx: CanvasRenderingContext2D) => void): () => void;
/**
 * Provide a function to access the Context2D after the scene is rendered.
 *
 * @param callback - The function that will be provided the context after render.
 */
export declare function useContextAfter(callback: (ctx: CanvasRenderingContext2D) => void): () => void;
//# sourceMappingURL=useContext.d.ts.map