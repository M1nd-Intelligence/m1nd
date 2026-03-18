/**
 * An estimate of the time remaining until the process is finished.
 */
export interface TimeEstimate {
    /**
     * The completion percentage ranging from `0` to `1`.
     */
    completion: number;
    /**
     * The time passed since the beginning of the process in milliseconds.
     */
    elapsed: number;
    /**
     * The estimated time remaining until the process is finished in milliseconds.
     */
    eta: number;
}
/**
 * Calculates the estimated time remaining until a process is finished.
 */
export declare class TimeEstimator {
    get onCompletionChanged(): import("../events").SubscribableValueEvent<number>;
    private readonly completion;
    private startTimestamp;
    private lastUpdateTimestamp;
    private nextCompletion;
    /**
     * Get the current time estimate.
     *
     * @param timestamp - The timestamp to calculate the estimate against.
     *                    Defaults to `performance.now()`.
     */
    estimate(timestamp?: number): TimeEstimate;
    /**
     * Update the completion percentage.
     *
     * @param completion - The completion percentage ranging from `0` to `1`.
     * @param timestamp - A timestamp at which the process was updated.
     *                    Defaults to `performance.now()`.
     */
    update(completion: number, timestamp?: number): void;
    /**
     * Reset the estimator.
     *
     * @param nextCompletion - If known, the completion percentage of the next
     *                         update.
     * @param timestamp - A timestamp at which the process started.
     *                    Defaults to `performance.now()`.
     */
    reset(nextCompletion?: number, timestamp?: number): void;
}
//# sourceMappingURL=TimeEstimator.d.ts.map