import { capitalize, deepLerp, SignalContext, useLogger, } from '@motion-canvas/core';
import { makeSignalExtensions } from '../utils/makeSignalExtensions';
import { addInitializer, initialize } from './initializers';
const PROPERTIES = Symbol.for('@motion-canvas/2d/decorators/properties');
export function getPropertyMeta(object, key) {
    return object[PROPERTIES]?.[key] ?? null;
}
export function getPropertyMetaOrCreate(object, key) {
    let lookup;
    if (!object[PROPERTIES]) {
        object[PROPERTIES] = lookup = {};
    }
    else if (object[PROPERTIES] &&
        !Object.prototype.hasOwnProperty.call(object, PROPERTIES)) {
        object[PROPERTIES] = lookup = Object.fromEntries(Object.entries(object[PROPERTIES]).map(([key, meta]) => [key, { ...meta }]));
    }
    else {
        lookup = object[PROPERTIES];
    }
    lookup[key] ?? (lookup[key] = {
        cloneable: true,
        inspectable: true,
        compoundEntries: [],
    });
    return lookup[key];
}
export function getPropertiesOf(value) {
    if (value && typeof value === 'object') {
        return value[PROPERTIES] ?? {};
    }
    return {};
}
export function initializeSignals(instance, props) {
    initialize(instance);
    for (const [key, meta] of Object.entries(getPropertiesOf(instance))) {
        const signal = instance[key];
        signal.reset();
        if (props[key] !== undefined) {
            signal(props[key]);
        }
        if (meta.compoundEntries !== undefined) {
            for (const [key, property] of meta.compoundEntries) {
                if (property in props) {
                    signal[key](props[property]);
                }
            }
        }
    }
}
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
export function signal() {
    return (target, key) => {
        // FIXME property metadata is not inherited
        // Consider retrieving it inside the initializer using the instance and not
        // the class.
        const meta = getPropertyMetaOrCreate(target, key);
        addInitializer(target, (instance) => {
            let initial = meta.default;
            const defaultMethod = instance[`getDefault${capitalize(key)}`];
            if (defaultMethod) {
                initial = () => defaultMethod.call(instance, meta.default);
            }
            const signal = new SignalContext(initial, meta.interpolationFunction ?? deepLerp, instance, meta.parser?.bind(instance), makeSignalExtensions(meta, instance, key));
            instance[key] = signal.toSignal();
        });
    };
}
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
export function initial(value) {
    return (target, key) => {
        const meta = getPropertyMeta(target, key);
        if (!meta) {
            useLogger().error(`Missing property decorator for "${key.toString()}"`);
            return;
        }
        meta.default = value;
    };
}
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
export function interpolation(value) {
    return (target, key) => {
        const meta = getPropertyMeta(target, key);
        if (!meta) {
            useLogger().error(`Missing property decorator for "${key.toString()}"`);
            return;
        }
        meta.interpolationFunction = value;
    };
}
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
export function parser(value) {
    return (target, key) => {
        const meta = getPropertyMeta(target, key);
        if (!meta) {
            useLogger().error(`Missing property decorator for "${key.toString()}"`);
            return;
        }
        meta.parser = value;
    };
}
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
export function wrapper(value) {
    return (target, key) => {
        const meta = getPropertyMeta(target, key);
        if (!meta) {
            useLogger().error(`Missing property decorator for "${key.toString()}"`);
            return;
        }
        meta.parser = raw => new value(raw);
        if ('lerp' in value) {
            meta.interpolationFunction ?? (meta.interpolationFunction = value.lerp);
        }
    };
}
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
export function cloneable(value = true) {
    return (target, key) => {
        const meta = getPropertyMeta(target, key);
        if (!meta) {
            useLogger().error(`Missing property decorator for "${key.toString()}"`);
            return;
        }
        meta.cloneable = value;
    };
}
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
export function inspectable(value = true) {
    return (target, key) => {
        const meta = getPropertyMeta(target, key);
        if (!meta) {
            useLogger().error(`Missing property decorator for "${key.toString()}"`);
            return;
        }
        meta.inspectable = value;
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2lnbmFsLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9kZWNvcmF0b3JzL3NpZ25hbC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQ0wsVUFBVSxFQUNWLFFBQVEsRUFFUixhQUFhLEVBR2IsU0FBUyxHQUNWLE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUFDLG9CQUFvQixFQUFDLE1BQU0sK0JBQStCLENBQUM7QUFDbkUsT0FBTyxFQUFDLGNBQWMsRUFBRSxVQUFVLEVBQUMsTUFBTSxnQkFBZ0IsQ0FBQztBQXFCMUQsTUFBTSxVQUFVLEdBQUcsTUFBTSxDQUFDLEdBQUcsQ0FBQyx5Q0FBeUMsQ0FBQyxDQUFDO0FBRXpFLE1BQU0sVUFBVSxlQUFlLENBQzdCLE1BQVcsRUFDWCxHQUFvQjtJQUVwQixPQUFPLE1BQU0sQ0FBQyxVQUFVLENBQUMsRUFBRSxDQUFDLEdBQUcsQ0FBQyxJQUFJLElBQUksQ0FBQztBQUMzQyxDQUFDO0FBRUQsTUFBTSxVQUFVLHVCQUF1QixDQUNyQyxNQUFXLEVBQ1gsR0FBb0I7SUFFcEIsSUFBSSxNQUFvRCxDQUFDO0lBQ3pELElBQUksQ0FBQyxNQUFNLENBQUMsVUFBVSxDQUFDLEVBQUU7UUFDdkIsTUFBTSxDQUFDLFVBQVUsQ0FBQyxHQUFHLE1BQU0sR0FBRyxFQUFFLENBQUM7S0FDbEM7U0FBTSxJQUNMLE1BQU0sQ0FBQyxVQUFVLENBQUM7UUFDbEIsQ0FBQyxNQUFNLENBQUMsU0FBUyxDQUFDLGNBQWMsQ0FBQyxJQUFJLENBQUMsTUFBTSxFQUFFLFVBQVUsQ0FBQyxFQUN6RDtRQUNBLE1BQU0sQ0FBQyxVQUFVLENBQUMsR0FBRyxNQUFNLEdBQUcsTUFBTSxDQUFDLFdBQVcsQ0FDOUMsTUFBTSxDQUFDLE9BQU8sQ0FDa0MsTUFBTSxDQUFDLFVBQVUsQ0FBQyxDQUNqRSxDQUFDLEdBQUcsQ0FBQyxDQUFDLENBQUMsR0FBRyxFQUFFLElBQUksQ0FBQyxFQUFFLEVB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