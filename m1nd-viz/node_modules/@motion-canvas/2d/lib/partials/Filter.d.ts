import { SignalValue, SimpleSignal } from '@motion-canvas/core';
/**
 * All possible CSS filter names.
 *
 * @internal
 */
export type FilterName = 'invert' | 'sepia' | 'grayscale' | 'brightness' | 'contrast' | 'saturate' | 'hue' | 'blur';
/**
 * Definitions of all possible CSS filters.
 *
 * @internal
 */
export declare const FILTERS: Record<string, Partial<FilterProps>>;
/**
 * A unified abstraction for all CSS filters.
 */
export interface FilterProps {
    name: string;
    value: SignalValue<number>;
    unit: string;
    scale: number;
    transform: boolean;
    default: number;
}
export declare class Filter {
    get name(): string;
    get default(): number;
    readonly value: SimpleSignal<number, Filter>;
    private readonly props;
    constructor(props: Partial<FilterProps>);
    isActive(): boolean;
    serialize(matrix: DOMMatrix): string;
}
/**
 * Create an {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/invert | invert} filter.
 *
 * @param value - The value of the filter.
 */
export declare function invert(value?: SignalValue<number>): Filter;
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/sepia | sepia} filter.
 *
 * @param value - The value of the filter.
 */
export declare function sepia(value?: SignalValue<number>): Filter;
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/grayscale | grayscale} filter.
 *
 * @param value - The value of the filter.
 */
export declare function grayscale(value?: SignalValue<number>): Filter;
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/brightness | brightness} filter.
 *
 * @param value - The value of the filter.
 */
export declare function brightness(value?: SignalValue<number>): Filter;
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/contrast | contrast} filter.
 *
 * @param value - The value of the filter.
 */
export declare function contrast(value?: SignalValue<number>): Filter;
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/saturate | saturate} filter.
 *
 * @param value - The value of the filter.
 */
export declare function saturate(value?: SignalValue<number>): Filter;
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/hue-rotate | hue} filter.
 *
 * @param value - The value of the filter in degrees.
 */
export declare function hue(value?: SignalValue<number>): Filter;
/**
 * Create a {@link https://developer.mozilla.org/en-US/docs/Web/CSS/filter-function/blur | blur} filter.
 *
 * @param value - The value of the filter in pixels.
 */
export declare function blur(value?: SignalValue<number>): Filter;
//# sourceMappingURL=Filter.d.ts.map