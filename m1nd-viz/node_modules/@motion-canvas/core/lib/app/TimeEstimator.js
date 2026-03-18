import { ValueDispatcher } from '../events';
import { clamp } from '../tweening';
/**
 * Calculates the estimated time remaining until a process is finished.
 */
export class TimeEstimator {
    constructor() {
        this.completion = new ValueDispatcher(0);
        this.startTimestamp = 0;
        this.lastUpdateTimestamp = 0;
        this.nextCompletion = 0;
    }
    get onCompletionChanged() {
        return this.completion.subscribable;
    }
    /**
     * Get the current time estimate.
     *
     * @param timestamp - The timestamp to calculate the estimate against.
     *                    Defaults to `performance.now()`.
     */
    estimate(timestamp = performance.now()) {
        const elapsed = timestamp - this.startTimestamp;
        const completion = this.completion.current;
        let eta = Infinity;
        if (completion >= 1) {
            eta = 0;
        }
        else if (completion > 0) {
            const lastUpdateDuration = this.lastUpdateTimestamp - this.startTimestamp;
            eta = lastUpdateDuration / completion - elapsed;
            eta = Math.max(0, eta);
        }
        else if (this.nextCompletion > 0) {
            eta = elapsed / this.nextCompletion - elapsed;
        }
        return { completion, elapsed, eta };
    }
    /**
     * Update the completion percentage.
     *
     * @param completion - The completion percentage ranging from `0` to `1`.
     * @param timestamp - A timestamp at which the process was updated.
     *                    Defaults to `performance.now()`.
     */
    update(completion, timestamp = performance.now()) {
        this.completion.current = clamp(0, 1, completion);
        this.lastUpdateTimestamp = timestamp;
    }
    /**
     * Reset the estimator.
     *
     * @param nextCompletion - If known, the completion percentage of the next
     *                         update.
     * @param timestamp - A timestamp at which the process started.
     *                    Defaults to `performance.now()`.
     */
    reset(nextCompletion = 0, timestamp = performance.now()) {
        this.startTimestamp = timestamp;
        this.lastUpdateTimestamp = timestamp;
        this.completion.current = 0;
        this.nextCompletion = nextCompletion;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiVGltZUVzdGltYXRvci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9hcHAvVGltZUVzdGltYXRvci50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUMsZUFBZSxFQUFDLE1BQU0sV0FBVyxDQUFDO0FBQzFDLE9BQU8sRUFBQyxLQUFLLEVBQUMsTUFBTSxhQUFhLENBQUM7QUFvQmxDOztHQUVHO0FBQ0gsTUFBTSxPQUFPLGFBQWE7SUFBMUI7UUFJbUIsZUFBVSxHQUFHLElBQUksZUFBZSxDQUFDLENBQUMsQ0FBQyxDQUFDO1FBQzdDLG1CQUFjLEdBQUcsQ0FBQyxDQUFDO1FBQ25CLHdCQUFtQixHQUFHLENBQUMsQ0FBQztRQUN4QixtQkFBYyxHQUFHLENBQUMsQ0FBQztJQW9EN0IsQ0FBQztJQTFEQyxJQUFXLG1CQUFtQjtRQUM1QixPQUFPLElBQUksQ0FBQyxVQUFVLENBQUMsWUFBWSxDQUFDO0lBQ3RDLENBQUM7SUFNRDs7Ozs7T0FLRztJQUNJLFFBQVEsQ0FBQyxTQUFTLEdBQUcsV0FBVyxDQUFDLEdBQUcsRUFBRTtRQUMzQyxNQUFNLE9BQU8sR0FBRyxTQUFTLEdBQUcsSUFBSSxDQUFDLGNBQWMsQ0FBQztRQUNoRCxNQUFNLFVBQVUsR0FBRyxJQUFJLENBQUMsVUFBVSxDQUFDLE9BQU8sQ0FBQztRQUUzQyxJQUFJLEdBQUcsR0FBRyxRQUFRLENBQUM7UUFDbkIsSUFBSSxVQUFVLElBQUksQ0FBQyxFQUFFO1lBQ25CLEdBQUcsR0FBRyxDQUFDLENBQUM7U0FDVDthQUFNLElBQUksVUFBVSxHQUFHLENBQUMsRUFBRTtZQUN6QixNQUFNLGtCQUFrQixHQUFHLElBQUksQ0FBQyxtQkFBbUIsR0FBRyxJQUFJLENBQUMsY0FBYyxDQUFDO1lBQzFFLEdBQUcsR0FBRyxrQkFBa0IsR0FBRyxVQUFVLEdBQUcsT0FBTyxDQUFDO1lBQ2hELEdBQUcsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLENBQUMsRUFBRSxHQUFHLENBQUMsQ0FBQztTQUN4QjthQUFNLElBQUksSUFBSSxDQUFDLGNBQWMsR0FBRyxDQUFDLEVBQUU7WUFDbEMsR0FBRyxHQUFHLE9BQU8sR0FBRyxJQUFJLENBQUMsY0FBYyxHQUFHLE9BQU8sQ0FBQztTQUMvQztRQUVELE9BQU8sRUFBQyxVQUFVLEVBQUUsT0FBTyxFQUFFLEdBQUcsRUFBQyxDQUFDO0lBQ3BDLENBQUM7SUFFRDs7Ozs7O09BTUc7SUFDSSxNQUFNLENBQUMsVUFBa0IsRUFBRSxTQUFTLEdBQUcsV0FBVyxDQUFDLEdBQUcsRUFBRTtRQUM3RCxJQUFJLENBQUMsVUFBVSxDQUFDLE9BQU8sR0FBRyxLQUFLLENBQUMsQ0FBQyxFQUFFLENBQUMsRUFBRSxVQUFVLENBQUMsQ0FBQztRQUNsRCxJQUFJLENBQUMsbUJBQW1CLEdBQUcsU0FBUyxDQUFDO0lBQ3ZDLENBQUM7SUFFRDs7Ozs7OztPQU9HO0lBQ0ksS0FBSyxDQUFDLGNBQWMsR0FBRyxDQUFDLEVBQUUsU0FBUyxHQUFHLFdBQVcsQ0FBQyxHQUFHLEVBQUU7UUFDNUQsSUFBSSxDQUFDLGNBQWMsR0FBRyxTQUFTLENBQUM7UUFDaEMsSUFBSSxDQUFDLG1CQUFtQixHQUFHLFNBQVMsQ0FBQztRQUNyQyxJQUFJLENBQUMsVUFBVSxDQUFDLE9BQU8sR0FBRyxDQUFDLENBQUM7UUFDNUIsSUFBSSxDQUFDLGNBQWMsR0FBRyxjQUFjLENBQUM7SUFDdkMsQ0FBQztDQUNGIn0=