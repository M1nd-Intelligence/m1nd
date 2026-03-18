/**
 * A simple semaphore implementation with a capacity of 1.
 *
 * @internal
 */
export declare class Semaphore {
    private resolveCurrent;
    private current;
    acquire(): Promise<void>;
    release(): void;
}
//# sourceMappingURL=Semaphore.d.ts.map