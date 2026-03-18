import { ThreadGenerator } from '../threading';
/**
 * Run all tasks concurrently and wait for any of them to finish.
 *
 * @example
 * ```ts
 * // current time: 0s
 * yield* any(
 *   rect.fill('#ff0000', 2),
 *   rect.opacity(1, 1),
 * );
 * // current time: 1s
 * ```
 *
 * @param tasks - A list of tasks to run.
 */
export declare function any(...tasks: ThreadGenerator[]): ThreadGenerator;
//# sourceMappingURL=any.d.ts.map