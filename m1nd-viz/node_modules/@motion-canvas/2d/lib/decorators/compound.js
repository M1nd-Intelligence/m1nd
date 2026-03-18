import { CompoundSignalContext, SignalContext, deepLerp, map, modify, useLogger, } from '@motion-canvas/core';
import { makeSignalExtensions } from '../utils/makeSignalExtensions';
import { addInitializer } from './initializers';
import { getPropertyMetaOrCreate } from './signal';
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
export function compound(entries, klass = CompoundSignalContext) {
    return (target, key) => {
        const meta = getPropertyMetaOrCreate(target, key);
        meta.compound = true;
        meta.compoundEntries = Object.entries(entries);
        addInitializer(target, (instance) => {
            if (!meta.parser) {
                useLogger().error(`Missing parser decorator for "${key.toString()}"`);
                return;
            }
            const initial = meta.default;
            const parser = meta.parser.bind(instance);
            const signalContext = new klass(meta.compoundEntries.map(([key, property]) => {
                const signal = new SignalContext(modify(initial, value => parser(value)[key]), map, instance, undefined, makeSignalExtensions(undefined, instance, property)).toSignal();
                return [key, signal];
            }), parser, initial, meta.interpolationFunction ?? deepLerp, instance, makeSignalExtensions(meta, instance, key));
            instance[key] = signalContext.toSignal();
        });
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY29tcG91bmQuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2RlY29yYXRvcnMvY29tcG91bmQudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUNMLHFCQUFxQixFQUNyQixhQUFhLEVBQ2IsUUFBUSxFQUNSLEdBQUcsRUFDSCxNQUFNLEVBQ04sU0FBUyxHQUNWLE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUFDLG9CQUFvQixFQUFDLE1BQU0sK0JBQStCLENBQUM7QUFDbkUsT0FBTyxFQUFDLGNBQWMsRUFBQyxNQUFNLGdCQUFnQixDQUFDO0FBQzlDLE9BQU8sRUFBQyx1QkFBdUIsRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUVqRDs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7R0F1Qkc7QUFDSCxNQUFNLFVBQVUsUUFBUSxDQU10QixPQUErQixFQUMvQixRQUtJLHFCQUFxQjtJQUV6QixPQUFPLENBQUMsTUFBTSxFQUFFLEdBQUcsRUFBRSxFQUFFO1FBQ3JCLE1BQU0sSUFBSSxHQUFHLHVCQUF1QixDQUFNLE1BQU0sRUFBRSxHQUFHLENBQUMsQ0FBQztRQUN2RCxJQUFJLENBQUMsUUFBUSxHQUFHLElBQUksQ0FBQztRQUNyQixJQUFJLENBQUMsZUFBZSxHQUFHLE1BQU0sQ0FBQyxPQUFPLENBQUMsT0FBTyxDQUFDLENBQUM7UUFFL0MsY0FBYyxDQUFDLE1BQU0sRUFBRSxDQUFDLFFBQWEsRUFBRSxFQUFFO1lBQ3ZDLElBQUksQ0FBQyxJQUFJLENBQUMsTUFBTSxFQUFFO2dCQUNoQixTQUFTLEVBQUUsQ0FBQyxLQUFLLENBQUMsaUNBQWlDLEdBQUcsQ0FBQyxRQUFRLEVBQUUsR0FBRyxDQUFDLENBQUM7Z0JBQ3RFLE9BQU87YUFDUjtZQUVELE1BQU0sT0FBTyxHQUFHLElBQUksQ0FBQyxPQUFPLENBQUM7WUFDN0IsTUFBTSxNQUFNLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsUUFBUSxDQUFDLENBQUM7WUFDMUMsTUFBTSxhQUFhLEdBQUcsSUFBSSxLQUFLLENBQzdCLElBQUksQ0FBQyxlQUFlLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQyxHQUFHLEVBQUUsUUFBUSxDQUFDLEVBQUUsRUFBRTtnQkFDM0MsTUFBTSxNQUFNLEdBQUcsSUFBSSxhQUFhLENBQzlCLE1BQU0sQ0FBQyxPQUFPLEVBQUUsS0FBSyxDQUFDLEVBQUUsQ0FBQyxNQUFNLENBQUMsS0FBSyxDQUFDLENBQUMsR0FBRyxDQUFDLENBQUMsRUFDdkMsR0FBRyxFQUNSLFFBQVEsRUFDUixTQUFTLEVBQ1Qsb0JBQW9CLENBQUMsU0FBUyxFQUFFLFFBQVEsRUFBRSxRQUFRLENBQUMsQ0FDcEQsQ0FBQyxRQUFRLEVBQUUsQ0FBQztnQkFDYixPQUFPLENBQUMsR0FBWSxFQUFFLE1BQU0sQ0FBQyxDQUFDO1lBQ2hDLENBQUMsQ0FBQyxFQUNGLE1BQU0sRUFDTixPQUFPLEVBQ1AsSUFBSSxDQUFDLHFCQUFxQixJQUFJLFFBQVEsRUFDdEMsUUFBUSxFQUNSLG9CQUFvQixDQUFDLElBQUksRUFBRSxRQUFRLEVBQVUsR0FBRyxDQUFDLENBQ2xELENBQUM7WUFFRixRQUFRLENBQUMsR0FBRyxDQUFDLEdBQUcsYUFBYSxDQUFDLFFBQVEsRUFBRSxDQUFDO1FBQzNDLENBQUMsQ0FBQyxDQUFDO0lBQ0wsQ0FBQyxDQUFDO0FBQ0osQ0FBQyJ9