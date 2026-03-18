import { ThreadGenerator } from '../threading';
/**
 * Run the given generator or callback after a specific amount of time.
 *
 * @example
 * ```ts
 * yield* delay(1, rect.fill('#ff0000', 2));
 * ```
 *
 * Note that the same animation can be written as:
 * ```ts
 * yield* waitFor(1),
 * yield* rect.fill('#ff0000', 2),
 * ```
 *
 * The reason `delay` exists is to make it easier to pass it to other flow
 * functions. For example:
 * ```ts
 * yield* all(
 *   rect.opacity(1, 3),
 *   delay(1, rect.fill('#ff0000', 2));
 * );
 * ```
 *
 * @param time - The delay in seconds
 * @param task - The task or callback to run after the delay.
 */
export declare function delay(time: number, task: ThreadGenerator | Callback): ThreadGenerator;
//# sourceMappingURL=delay.d.ts.map