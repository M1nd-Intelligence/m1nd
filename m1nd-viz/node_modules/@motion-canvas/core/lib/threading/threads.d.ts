import { Thread } from './Thread';
import { ThreadGenerator } from './ThreadGenerator';
/**
 * Check if the given value is a [Promise][promise].
 *
 * @param value - A possible [Promise][promise].
 *
 * [promise]: https://developer.mozilla.org/en-US/docs/web/javascript/reference/global_objects/promise
 */
export declare function isPromise(value: any): value is Promise<any>;
/**
 * A generator function or a normal function that returns a generator.
 */
export interface ThreadsFactory {
    (): ThreadGenerator;
}
export interface ThreadsCallback {
    (root: Thread): void;
}
/**
 * Create a context in which generators can be run concurrently.
 *
 * @remarks
 * From the perspective of the external generator, `threads` is executed
 * synchronously. By default, each scene generator is wrapped in its own
 * `threads` generator.
 *
 * @example
 * ```ts
 * // first
 *
 * yield* threads(function* () {
 *   const task = yield generatorFunction();
 *   // second
 * }); // <- `task` will be terminated here because the scope
 *     //    of this `threads` generator has ended
 *
 * // third
 * ```
 *
 * @param factory - A function that returns the generator to run.
 * @param callback - Called whenever threads are created, canceled or finished.
 *                   Used for debugging purposes.
 */
export declare function threads(factory: ThreadsFactory, callback?: ThreadsCallback): ThreadGenerator;
//# sourceMappingURL=threads.d.ts.map