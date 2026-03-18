import type { Scene } from './Scene';
export declare class Variables {
    private readonly scene;
    private signals;
    private variables;
    constructor(scene: Scene);
    /**
     * Get variable signal if exists or create signal if not
     *
     * @param name - The name of the variable.
     * @param initial - The initial value of the variable. It will be used if the
     *                  variable was not configured from the outside.
     */
    get<T>(name: string, initial: T): () => T;
    /**
     * Update all signals with new project variable values.
     */
    updateSignals(variables: Record<string, unknown>): void;
    /**
     * Reset all stored signals.
     */
    handleReset: () => void;
}
//# sourceMappingURL=Variables.d.ts.map