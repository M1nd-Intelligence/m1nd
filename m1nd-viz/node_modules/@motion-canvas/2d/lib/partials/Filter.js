import { createSignal, map, transformScalar, } from '@motion-canvas/core';
/**
 * Definitions of all possible CSS filters.
 *
 * @internal
 */
export const FILTERS = {
    invert: {
        name: 'invert',
    },
    sepia: {
        name: 'sepia',
    },
    grayscale: {
        name: 'grayscale',
    },
    brightness: {
        name: 'brightness',
        default: 1,
    },
    contrast: {
        name: 'contrast',
        default: 1,
    },
    saturate: {
        name: 'saturate',
        default: 1,
    },
    hue: {
        name: 'hue-rotate',
        unit: 'deg',
        scale: 1,
    },
    blur: {
        name: 'blur',
        transform: true,
        unit: 'px',
        scale: 1,
    },
};
export class Filter {
    get name() {
        return this.props.name;
    }
    get default() {
        return this.props.default;
    }
    constructor(props) {
        this.props = {
            name: 'invert',
            default: 0,
            unit: '%',
            scale: 100,
            transform: false,
            ...props,
            value: props.value ?? props.default ?? 0,
        };
        this.value = createSignal(this.props.value, map, this);
    }
    isActive() {
        return this.value() !== this.props.default;
    }
    serialize(matrix) {
        let value = this.value();
        if (this.props.transform) {
            value = transformScalar(value, matrix);
        }
        return `${this.props.name}(${value * this.props.scale}${this.props.unit})`;
    }
}
/**
 * Create an {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/invert | invert} filter.
 *
 * @param value - The value of the filter.
 */
export function invert(value) {
    return new Filter({ ...FILTERS.invert, value });
}
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/sepia | sepia} filter.
 *
 * @param value - The value of the filter.
 */
export function sepia(value) {
    return new Filter({ ...FILTERS.sepia, value });
}
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/grayscale | grayscale} filter.
 *
 * @param value - The value of the filter.
 */
export function grayscale(value) {
    return new Filter({ ...FILTERS.grayscale, value });
}
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/brightness | brightness} filter.
 *
 * @param value - The value of the filter.
 */
export function brightness(value) {
    return new Filter({ ...FILTERS.brightness, value });
}
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/contrast | contrast} filter.
 *
 * @param value - The value of the filter.
 */
export function contrast(value) {
    return new Filter({ ...FILTERS.contrast, value });
}
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/saturate | saturate} filter.
 *
 * @param value - The value of the filter.
 */
export function saturate(value) {
    return new Filter({ ...FILTERS.saturate, value });
}
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/hue-rotate | hue} filter.
 *
 * @param value - The value of the filter in degrees.
 */
export function hue(value) {
    return new Filter({ ...FILTERS.hue, value });
}
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/blur | blur} filter.
 *
 * @param value - The value of the filter in pixels.
 */
export function blur(value) {
    return new Filter({ ...FILTERS.blur, value });
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiRmlsdGVyLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9wYXJ0aWFscy9GaWx0ZXIudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUNMLFlBQVksRUFDWixHQUFHLEVBR0gsZUFBZSxHQUNoQixNQUFNLHFCQUFxQixDQUFDO0FBaUI3Qjs7OztHQUlHO0FBQ0gsTUFBTSxDQUFDLE1BQU0sT0FBTyxHQUF5QztJQUMzRCxNQUFNLEVBQUU7UUFDTixJQUFJLEVBQUUsUUFBUTtLQUNmO0lBQ0QsS0FBSyxFQUFFO1FBQ0wsSUFBSSxFQUFFLE9BQU87S0FDZDtJQUNELFNBQVMsRUFBRTtRQUNULElBQUksRUFBRSxXQUFXO0tBQ2xCO0lBQ0QsVUFBVSxFQUFFO1FBQ1YsSUFBSSxFQUFFLFlBQVk7UUFDbEIsT0FBTyxFQUFFLENBQUM7S0FDWDtJQUNELFFBQVEsRUFBRTtRQUNSLElBQUksRUFBRSxVQUFVO1FBQ2hCLE9BQU8sRUFBRSxDQUFDO0tBQ1g7SUFDRCxRQUFRLEVBQUU7UUFDUixJQUFJLEVBQUUsVUFBVTtRQUNoQixPQUFPLEVBQUUsQ0FBQztLQUNYO0lBQ0QsR0FBRyxFQUFFO1FBQ0gsSUFBSSxFQUFFLFlBQVk7UUFDbEIsSUFBSSxFQUFFLEtBQUs7UUFDWCxLQUFLLEVBQUUsQ0FBQztLQUNUO0lBQ0QsSUFBSSxFQUFFO1FBQ0osSUFBSSxFQUFFLE1BQU07UUFDWixTQUFTLEVBQUUsSUFBSTtRQUNmLElBQUksRUFBRSxJQUFJO1FBQ1YsS0FBSyxFQUFFLENBQUM7S0FDVDtDQUNGLENBQUM7QUFjRixNQUFNLE9BQU8sTUFBTTtJQUNqQixJQUFXLElBQUk7UUFDYixPQUFPLElBQUksQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDO0lBQ3pCLENBQUM7SUFFRCxJQUFXLE9BQU87UUFDaEIsT0FBTyxJQUFJLENBQUMsS0FBSyxDQUFDLE9BQU8sQ0FBQztJQUM1QixDQUFDO0lBS0QsWUFBbUIsS0FBMkI7UUFDNUMs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