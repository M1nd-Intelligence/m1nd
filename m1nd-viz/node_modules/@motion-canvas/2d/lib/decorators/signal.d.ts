import { InterpolationFunction, TimingFunction } from '@motion-canvas/core';
export interface PropertyMetadata<T> {
    default?: T;
    interpolationFunction?: InterpolationFunction<T>;
    parser?: (value: any) => T;
    getter?: () => T;
    setter?: (value: any) => void;
    tweener?: (value: T, duration: number, timingFunction: TimingFunction, interpolationFunction: InterpolationFunction<T>) => void;
    cloneable?: boolean;
    inspectable?: boolean;
    compoundParent?: string;
    compound?: boolean;
    compoundEntries: [string, string][];
}
export declare function getPropertyMeta<T>(object: any, key: string | symbol): PropertyMetadata<T> | null;
export declare function getPropertyMetaOrCreate<T>(object: any, key: string | symbol): PropertyMetadata<T>;
export declare function getPropertiesOf(value: any): Record<string, PropertyMetadata<any>>;
export declare function initializeSignals(instance: any, props: Record<string, any>): void;
/**
 * Create a signal decorator.
 *
 * @remarks
 * This decorator turns the given property into a signal.
 *
 * The class using this decorator can implement the following methods:
 * - `get[PropertyName]` - A property getter.
 * - `get[PropertyName]` - A property setter.
 * - `tween[PropertyName]` - A tween provider.
 *
 * @example
 * ```ts
 * class Example {
 *   \@property()
 *   public declare length: Signal<number, this>;
 * }
 * ```
 */
export declare function signal<T>(): PropertyDecorator;
/**
 * Create an initial signal value decorator.
 *
 * @remarks
 * This decorator specifies the initial value of a property.
 *
 * Must be specified before the {@link signal} decorator.
 *
 * @example
 * ```ts
 * class Example {
 *   \@initial(1)
 *   \@property()
 *   public declare length: Signal<number, this>;
 * }
 * ```
 *
 * @param value - The initial value of the property.
 */
export declare function initial<T>(value: T): PropertyDecorator;
/**
 * Create a signal interpolation function decorator.
 *
 * @remarks
 * This decorator specifies the interpolation function of a property.
 * The interpolation function is used when tweening between different values.
 *
 * Must be specified before the {@link signal} decorator.
 *
 * @example
 * ```ts
 * class Example {
 *   \@interpolation(textLerp)
 *   \@property()
 *   public declare text: Signal<string, this>;
 * }
 * ```
 *
 * @param value - The interpolation function for the property.
 */
export declare function interpolation<T>(value: InterpolationFunction<T>): PropertyDecorator;
/**
 * Create a signal parser decorator.
 *
 * @remarks
 * This decorator specifies the parser of a property.
 * Instead of returning the raw value, its passed as the first parameter to the
 * parser and the resulting value is returned.
 *
 * If the wrapper class has a method called `lerp` it will be set as the
 * default interpolation function for the property.
 *
 * Must be specified before the {@link signal} decorator.
 *
 * @example
 * ```ts
 * class Example {
 *   \@wrapper(Vector2)
 *   \@property()
 *   public declare offset: Signal<Vector2, this>;
 * }
 * ```
 *
 * @param value - The wrapper class for the property.
 */
export declare function parser<T>(value: (value: any) => T): PropertyDecorator;
/**
 * Create a signal wrapper decorator.
 *
 * @remarks
 * This is a shortcut decorator for setting both the {@link parser} and
 * {@link interpolation}.
 *
 * The interpolation function will be set only if the wrapper class has a method
 * called `lerp`, which will be used as said function.
 *
 * Must be specified before the {@link signal} decorator.
 *
 * @example
 * ```ts
 * class Example {
 *   \@wrapper(Vector2)
 *   \@property()
 *   public declare offset: Signal<Vector2, this>;
 *
 *   // same as:
 *   \@parser(value => new Vector2(value))
 *   \@interpolation(Vector2.lerp)
 *   \@property()
 *   public declare offset: Signal<Vector2, this>;
 * }
 * ```
 *
 * @param value - The wrapper class for the property.
 */
export declare function wrapper<T>(value: (new (value: any) => T) & {
    lerp?: InterpolationFunction<T>;
}): PropertyDecorator;
/**
 * Create a cloneable property decorator.
 *
 * @remarks
 * This decorator specifies whether the property should be copied over when
 * cloning the node.
 *
 * By default, any property is cloneable.
 *
 * Must be specified before the {@link signal} decorator.
 *
 * @example
 * ```ts
 * class Example {
 *   \@clone(false)
 *   \@property()
 *   public declare length: Signal<number, this>;
 * }
 * ```
 *
 * @param value - Whether the property should be cloneable.
 */
export declare function cloneable<T>(value?: boolean): PropertyDecorator;
/**
 * Create an inspectable property decorator.
 *
 * @remarks
 * This decorator specifies whether the property should be visible in the
 * inspector.
 *
 * By default, any property is inspectable.
 *
 * Must be specified before the {@link signal} decorator.
 *
 * @example
 * ```ts
 * class Example {
 *   \@inspectable(false)
 *   \@property()
 *   public declare hiddenLength: Signal<number, this>;
 * }
 * ```
 *
 * @param value - Whether the property should be inspectable.
 */
export declare function inspectable<T>(value?: boolean): PropertyDecorator;
//# sourceMappingURL=signal.d.ts.map