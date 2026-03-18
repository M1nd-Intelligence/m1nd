import { MetaField } from './MetaField';
/**
 * Represents a range stored in a meta file.
 *
 * @remarks
 * Range is an array with two elements denoting the beginning and end of a
 * range, respectively.
 */
export declare class RangeMetaField extends MetaField<[
    number,
    number | null
], [
    number,
    number
]> {
    static readonly symbol: unique symbol;
    readonly type: symbol;
    parse(value: [number, number | null]): [number, number];
    /**
     * Convert the given range from frames to seconds and update this field.
     *
     * @remarks
     * This helper method applies additional validation to the range, preventing
     * it from overflowing the timeline.
     *
     * @param startFrame - The beginning of the range.
     * @param endFrame - The end of the range.
     * @param duration - The current duration in frames.
     * @param fps - The current framerate.
     */
    update(startFrame: number, endFrame: number, duration: number, fps: number): void;
    protected parseRange(duration: number, startFrame?: number, endFrame?: number): [number, number];
}
//# sourceMappingURL=RangeMetaField.d.ts.map