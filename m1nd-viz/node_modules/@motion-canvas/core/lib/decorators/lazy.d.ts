/**
 * Create a lazy decorator.
 *
 * @remarks
 * A property marked as lazy will not be initialized until it's requested for
 * the first time. Lazy properties are read-only.
 *
 * Must be used for any static properties that require the DOM API to be
 * initialized.
 *
 * @param factory - A function that returns the value of this property.
 */
export declare function lazy(factory: () => unknown): PropertyDecorator;
//# sourceMappingURL=lazy.d.ts.map