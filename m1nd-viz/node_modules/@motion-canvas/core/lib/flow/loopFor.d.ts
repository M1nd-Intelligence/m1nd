import { ThreadGenerator } from '../threading';
import { LoopCallback } from './loop';
/**
 * Run a generator in a loop for the given amount of time.
 *
 * @remarks
 * Generators are executed completely before the next iteration starts.
 * An iteration is allowed to finish even when the time is up. This means that
 * the actual duration of the loop may be longer than the given duration.
 *
 * @example
 * ```ts
 * yield* loopFor(
 *   3,
 *   () => circle().position.x(-10, 0.1).to(10, 0.1)
 * );
 * ```
 *
 * @param seconds - The duration in seconds.
 * @param factory - A function creating the generator to run. Because generators
 *                  can't be reset, a new generator is created on each
 *                  iteration.
 */
export declare function loopFor(seconds: number, factory: LoopCallback): ThreadGenerator;
//# sourceMappingURL=loopFor.d.ts.map