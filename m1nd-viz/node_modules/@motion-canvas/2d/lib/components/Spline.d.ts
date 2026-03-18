import { BBox, PossibleVector2, SerializedVector2, SignalValue, SimpleSignal } from '@motion-canvas/core';
import { CurveProfile, KnotInfo } from '../curves';
import { DesiredLength } from '../partials';
import { Curve, CurveProps } from './Curve';
export interface SplineProps extends CurveProps {
    /**
     * {@inheritDoc Spline.smoothness}
     */
    smoothness?: SignalValue<number>;
    /**
     * {@inheritDoc Spline.points}
     */
    points?: SignalValue<SignalValue<PossibleVector2[]>>;
}
/**
 * A node for drawing a smooth line through a number of points.
 *
 * @remarks
 * This node uses Bézier curves for drawing each segment of the spline.
 *
 * @example
 * Defining knots using the `points` property. This will automatically
 * calculate the handle positions for each knot do draw a smooth curve. You
 * can control the smoothness of the resulting curve via the
 * {@link Spline.smoothness} property:
 *
 * ```tsx
 * <Spline
 *   lineWidth={4}
 *   stroke={'white'}
 *   smoothness={0.4}
 *   points={[
 *     [-400, 0],
 *     [-200, -300],
 *     [0, 0],
 *     [200, -300],
 *     [400, 0],
 *   ]}
 * />
 * ```
 *
 * Defining knots with {@link Knot} nodes:
 *
 * ```tsx
 * <Spline lineWidth={4} stroke={'white'}>
 *   <Knot position={[-400, 0]} />
 *   <Knot position={[-200, -300]} />
 *   <Knot
 *     position={[0, 0]}
 *     startHandle={[-100, 200]}
 *     endHandle={[100, 200]}
 *   />
 *   <Knot position={[200, -300]} />
 *   <Knot position={[400, 0]} />
 * </Spline>
 * ```
 */
export declare class Spline extends Curve {
    /**
     * The smoothness of the spline when using auto-calculated handles.
     *
     * @remarks
     * This property is only applied to knots that don't use explicit handles.
     *
     * @defaultValue 0.4
     */
    readonly smoothness: SimpleSignal<number>;
    /**
     * The knots of the spline as an array of knots with auto-calculated handles.
     *
     * @remarks
     * You can control the smoothness of the resulting curve
     * via the {@link smoothness} property.
     */
    readonly points: SimpleSignal<SignalValue<PossibleVector2>[] | null, this>;
    constructor(props: SplineProps);
    profile(): CurveProfile;
    knots(): KnotInfo[];
    protected childrenBBox(): BBox;
    protected lineWidthCoefficient(): number;
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected offsetComputedLayout(box: BBox): BBox;
    private getTightBBox;
    drawOverlay(context: CanvasRenderingContext2D, matrix: DOMMatrix): void;
    private isKnot;
}
//# sourceMappingURL=Spline.d.ts.map