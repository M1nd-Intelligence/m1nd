/**
 * Create a predicate that checks if the given object is an instance of the
 * given class.
 *
 * @param klass - The class to check against.
 */
export declare function is<T>(klass: new (...args: any[]) => T): (object: any) => object is T;
//# sourceMappingURL=is.d.ts.map