import { CompoundSignalContext } from '@motion-canvas/core';
/**
 * Create a compound property decorator.
 *
 * @remarks
 * This decorator turns a given property into a signal consisting of one or more
 * nested signals.
 *
 * @example
 * ```ts
 * class Example {
 *   \@compound({x: 'scaleX', y: 'scaleY'})
 *   public declare readonly scale: Signal<Vector2, this>;
 *
 *   public setScale() {
 *     this.scale({x: 7, y: 3});
 *     // same as:
 *     this.scale.x(7).scale.y(3);
 *   }
 * }
 * ```
 *
 * @param entries - A record mapping the property in the compound object to the
 *                  corresponding property on the owner node.
 */
export declare function compound<TSetterValue, TValue extends TSetterValue, TKeys extends keyof TValue = keyof TValue, TOwner = void>(entries: Record<string, string>, klass?: typeof CompoundSignalContext<TSetterValue, TValue, TKeys, TOwner>): PropertyDecorator;
//# sourceMappingURL=compound.d.ts.map