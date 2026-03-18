import { ThreadGenerator } from '../threading';
/**
 * Start all tasks one after another with a constant delay between.
 *
 * @remarks
 * The function doesn't wait until the previous task in the sequence has
 * finished. Once the delay has passed, the next task will start even if
 * the previous is still running.
 *
 * @example
 * ```ts
 * yield* sequence(
 *   0.1,
 *   ...rects.map(rect => rect.x(100, 1))
 * );
 * ```
 *
 * @param delay - The delay between each of the tasks.
 * @param tasks - A list of tasks to be run in a sequence.
 */
export declare function sequence(delay: number, ...tasks: ThreadGenerator[]): ThreadGenerator;
//# sourceMappingURL=sequence.d.ts.map