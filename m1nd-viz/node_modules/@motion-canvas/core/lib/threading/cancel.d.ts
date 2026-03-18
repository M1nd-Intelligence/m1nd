import { ThreadGenerator } from './ThreadGenerator';
/**
 * Cancel all listed tasks.
 *
 * Example:
 * ```ts
 * const task = yield generatorFunction();
 *
 * // do something concurrently
 *
 * yield* cancel(task);
 * ```
 *
 * @param tasks - A list of tasks to cancel.
 */
export declare function cancel(...tasks: ThreadGenerator[]): void;
//# sourceMappingURL=cancel.d.ts.map