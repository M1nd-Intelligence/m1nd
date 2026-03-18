import { ThreadGenerator } from './ThreadGenerator';
/**
 * Pause the current generator until all listed tasks are finished.
 *
 * @example
 * ```ts
 * const task = yield generatorFunction();
 *
 * // do something concurrently
 *
 * yield* join(task);
 * ```
 *
 * @param tasks - A list of tasks to join.
 */
export declare function join(...tasks: ThreadGenerator[]): ThreadGenerator;
/**
 * Pause the current generator until listed tasks are finished.
 *
 * @example
 * ```ts
 * const taskA = yield generatorFunctionA();
 * const taskB = yield generatorFunctionB();
 *
 * // do something concurrently
 *
 * // await any of the tasks
 * yield* join(false, taskA, taskB);
 * ```
 *
 * @param all - Whether we should wait for all tasks or for at least one.
 * @param tasks - A list of tasks to join.
 */
export declare function join(all: boolean, ...tasks: ThreadGenerator[]): ThreadGenerator;
//# sourceMappingURL=join.d.ts.map