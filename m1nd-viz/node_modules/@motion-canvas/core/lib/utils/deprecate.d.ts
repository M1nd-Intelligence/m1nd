/**
 * Mark the given function as deprecated.
 *
 * @param fn - The function to deprecate.
 * @param message - The log message.
 * @param remarks - The optional log remarks.
 */
export declare function deprecate<TArgs extends any[], TReturn>(fn: (...args: TArgs) => TReturn, message: string, remarks?: string): (...args: TArgs) => TReturn;
//# sourceMappingURL=deprecate.d.ts.map