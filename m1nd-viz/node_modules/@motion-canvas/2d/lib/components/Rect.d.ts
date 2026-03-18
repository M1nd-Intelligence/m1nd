import { BBox, PossibleSpacing, SerializedVector2, SignalValue, SimpleSignal, SpacingSignal } from '@motion-canvas/core';
import { DesiredLength } from '../partials';
import { Curve, CurveProps } from './Curve';
export interface RectProps extends CurveProps {
    /**
     * {@inheritDoc Rect.radius}
     */
    radius?: SignalValue<PossibleSpacing>;
    /**
     * {@inheritDoc Rect.smoothCorners}
     */
    smoothCorners?: SignalValue<boolean>;
    /**
     * {@inheritDoc Rect.cornerSharpness}
     */
    cornerSharpness?: SignalValue<number>;
}
export declare class Rect extends Curve {
    /**
     * Rounds the corners of this rectangle.
     *
     * @remarks
     * The value represents the radius of the quarter circle that is used to round
     * the corners. If the value is a number, the same radius is used for all
     * corners. Passing an array of two to four numbers will set individual radii
     * for each corner. Individual radii correspond to different corners depending
     * on the number of values passed:
     *
     * ```ts
     * // top-left-and-bottom-right | top-right-and-bottom-left
     * [10, 30]
     * // top-left | top-right-and-bottom-left | bottom-right
     * [10, 20, 30]
     * // top-left | top-right | bottom-right | bottom-left
     * [10, 20, 30, 40]
     * ```
     *
     * @example
     * One uniform radius:
     * ```tsx
     * <Rect
     *   size={320}
     *   radius={40}
     *   fill={'white'}
     * />
     * ```
     * @example
     * Individual radii for each corner:
     * ```tsx
     * <Rect
     *   size={320}
     *   radius={[10, 20, 30, 40]}
     *   fill={'white'}
     * />
     * ```
     */
    readonly radius: SpacingSignal<this>;
    /**
     * Enables corner smoothing.
     *
     * @remarks
     * This property only affects the way rounded corners are drawn. To control
     * the corner radius use the {@link radius} property.
     *
     * When enabled, rounded corners are drawn continuously using Bézier curves
     * rather than quarter circles. The sharpness of the curve can be controlled
     * with {@link cornerSharpness}.
     *
     * You can read more about corner smoothing in
     * [this article by Nick Lawrence](https://uxplanet.org/ui-ux-design-corner-smoothing-720509d1ae48).
     *
     * @example
     * ```tsx
     * <Rect
     *   width={300}
     *   height={300}
     *   smoothCorners={true}
     * />
     * ```
     */
    readonly smoothCorners: SimpleSignal<boolean, this>;
    /**
     * Controls the sharpness of {@link smoothCorners}.
     *
     * @remarks
     * This property only affects the way rounded corners are drawn. To control
     * the corner radius use the {@link radius} property.
     *
     * Requires {@link smoothCorners} to be enabled to have any effect.
     * By default, corner sharpness is set to `0.6` which represents a smooth,
     * circle-like rounding. At `0` the edges are squared off.
     *
     * @example
     * ```tsx
     * <Rect
     *   size={300}
     *   smoothCorners={true}
     *   cornerSharpness={0.7}
     * />
     * ```
     */
    readonly cornerSharpness: SimpleSignal<number, this>;
    constructor(props: RectProps);
    profile(): import("..").CurveProfile;
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected offsetComputedLayout(box: BBox): BBox;
    protected childrenBBox(): BBox;
    protected getPath(): Path2D;
    protected getCacheBBox(): BBox;
    protected getRipplePath(): Path2D;
}
//# sourceMappingURL=Rect.d.ts.map