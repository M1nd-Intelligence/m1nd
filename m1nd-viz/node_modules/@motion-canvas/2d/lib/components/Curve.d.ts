import { BBox, SerializedVector2, SignalValue, SimpleSignal, Vector2 } from '@motion-canvas/core';
import { CurveDrawingInfo } from '../curves/CurveDrawingInfo';
import { CurvePoint } from '../curves/CurvePoint';
import { CurveProfile } from '../curves/CurveProfile';
import { DesiredLength } from '../partials';
import { Shape, ShapeProps } from './Shape';
export interface CurveProps extends ShapeProps {
    /**
     * {@inheritDoc Curve.closed}
     */
    closed?: SignalValue<boolean>;
    /**
     * {@inheritDoc Curve.start}
     */
    start?: SignalValue<number>;
    /**
     * {@inheritDoc Curve.startOffset}
     */
    startOffset?: SignalValue<number>;
    /**
     * {@inheritDoc Curve.startArrow}
     */
    startArrow?: SignalValue<boolean>;
    /**
     * {@inheritDoc Curve.end}
     */
    end?: SignalValue<number>;
    /**
     * {@inheritDoc Curve.endOffset}
     */
    endOffset?: SignalValue<number>;
    /**
     * {@inheritDoc Curve.endArrow}
     */
    endArrow?: SignalValue<boolean>;
    /**
     * {@inheritDoc Curve.arrowSize}
     */
    arrowSize?: SignalValue<number>;
}
export declare abstract class Curve extends Shape {
    /**
     * Whether the curve should be closed.
     *
     * @remarks
     * Closed curves have their start and end points connected.
     */
    readonly closed: SimpleSignal<boolean, this>;
    /**
     * A percentage from the start before which the curve should be clipped.
     *
     * @remarks
     * The portion of the curve that comes before the given percentage will be
     * made invisible.
     *
     * This property is usefully for animating the curve appearing on the screen.
     * The value of `0` means the very start of the curve (accounting for the
     * {@link startOffset}) while `1` means the very end (accounting for the
     * {@link endOffset}).
     */
    readonly start: SimpleSignal<number, this>;
    /**
     * The offset in pixels from the start of the curve.
     *
     * @remarks
     * This property lets you specify where along the defined curve the actual
     * visible portion starts. For example, setting it to `20` will make the first
     * 20 pixels of the curve invisible.
     *
     * This property is useful for trimming the curve using a fixed distance.
     * If you want to animate the curve appearing on the screen, use {@link start}
     * instead.
     */
    readonly startOffset: SimpleSignal<number, this>;
    /**
     * Whether to display an arrow at the start of the visible curve.
     *
     * @remarks
     * Use {@link arrowSize} to control the size of the arrow.
     */
    readonly startArrow: SimpleSignal<boolean, this>;
    /**
     * A percentage from the start after which the curve should be clipped.
     *
     * @remarks
     * The portion of the curve that comes after the given percentage will be
     * made invisible.
     *
     * This property is usefully for animating the curve appearing on the screen.
     * The value of `0` means the very start of the curve (accounting for the
     * {@link startOffset}) while `1` means the very end (accounting for the
     * {@link endOffset}).
     */
    readonly end: SimpleSignal<number, this>;
    /**
     * The offset in pixels from the end of the curve.
     *
     * @remarks
     * This property lets you specify where along the defined curve the actual
     * visible portion ends. For example, setting it to `20` will make the last
     * 20 pixels of the curve invisible.
     *
     * This property is useful for trimming the curve using a fixed distance.
     * If you want to animate the curve appearing on the screen, use {@link end}
     * instead.
     */
    readonly endOffset: SimpleSignal<number, this>;
    /**
     * Whether to display an arrow at the end of the visible curve.
     *
     * @remarks
     * Use {@link arrowSize} to control the size of the arrow.
     */
    readonly endArrow: SimpleSignal<boolean, this>;
    /**
     * Controls the size of the end and start arrows.
     *
     * @remarks
     * To make the arrows visible make sure to enable {@link startArrow} and/or
     * {@link endArrow}.
     */
    readonly arrowSize: SimpleSignal<number, this>;
    protected canHaveSubpath: boolean;
    protected desiredSize(): SerializedVector2<DesiredLength>;
    constructor(props: CurveProps);
    protected abstract childrenBBox(): BBox;
    abstract profile(): CurveProfile;
    /**
     * Convert a percentage along the curve to a distance.
     *
     * @remarks
     * The returned distance is given in relation to the full curve, not
     * accounting for {@link startOffset} and {@link endOffset}.
     *
     * @param value - The percentage along the curve.
     */
    percentageToDistance(value: number): number;
    /**
     * Convert a distance along the curve to a percentage.
     *
     * @remarks
     * The distance should be given in relation to the full curve, not
     * accounting for {@link startOffset} and {@link endOffset}.
     *
     * @param value - The distance along the curve.
     */
    distanceToPercentage(value: number): number;
    /**
     * The base arc length of this curve.
     *
     * @remarks
     * This is the entire length of this curve, not accounting for
     * {@link startOffset | the offsets}.
     */
    baseArcLength(): number;
    /**
     * The offset arc length of this curve.
     *
     * @remarks
     * This is the length of the curve that accounts for
     * {@link startOffset | the offsets}.
     */
    offsetArcLength(): number;
    /**
     * The visible arc length of this curve.
     *
     * @remarks
     * This arc length accounts for both the offset and the {@link start} and
     * {@link end} properties.
     */
    arcLength(): number;
    /**
     * The percentage of the curve that's currently visible.
     *
     * @remarks
     * The returned value is the ratio between the visible length (as defined by
     * {@link start} and {@link end}) and the offset length of the curve.
     */
    completion(): number;
    protected processSubpath(_path: Path2D, _startPoint: Vector2 | null, _endPoint: Vector2 | null): void;
    protected curveDrawingInfo(): CurveDrawingInfo;
    protected getPointAtDistance(value: number): CurvePoint;
    getPointAtPercentage(value: number): CurvePoint;
    protected getComputedLayout(): BBox;
    protected offsetComputedLayout(box: BBox): BBox;
    protected getPath(): Path2D;
    protected getCacheBBox(): BBox;
    protected lineWidthCoefficient(): number;
    /**
     * Check if the path requires a profile.
     *
     * @remarks
     * The profile is only required if certain features are used. Otherwise, the
     * profile generation can be skipped, and the curve can be drawn directly
     * using the 2D context.
     */
    protected requiresProfile(): boolean;
    protected drawShape(context: CanvasRenderingContext2D): void;
    private drawArrows;
    private drawArrow;
}
//# sourceMappingURL=Curve.d.ts.map