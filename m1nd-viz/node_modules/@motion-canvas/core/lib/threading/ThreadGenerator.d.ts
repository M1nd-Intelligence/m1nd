import { Thread } from './Thread';
export interface Promisable<T> {
    toPromise(): Promise<T>;
}
export declare function isPromisable(value: any): value is Promisable<any>;
/**
 * The main generator type produced by all generator functions in Motion Canvas.
 *
 * @example
 * Yielded values can be used to control the flow of animation:
 *
 * Progress to the next frame:
 * ```ts
 * yield;
 * ```
 *
 * Run another generator synchronously:
 * ```ts
 * yield* generatorFunction();
 * ```
 *
 * Run another generator concurrently:
 * ```ts
 * const task = yield generatorFunction();
 * ```
 *
 * Await a Promise:
 * ```ts
 * const result = yield asyncFunction();
 * ```
 */
export type ThreadGenerator = Generator<ThreadGenerator | Promise<any> | Promisable<any> | void, void, Thread | any>;
/**
 * Check if the given value is a {@link ThreadGenerator}.
 *
 * @param value - A possible thread {@link ThreadGenerator}.
 */
export declare function isThreadGenerator(value: unknown): value is ThreadGenerator;
//# sourceMappingURL=ThreadGenerator.d.ts.map