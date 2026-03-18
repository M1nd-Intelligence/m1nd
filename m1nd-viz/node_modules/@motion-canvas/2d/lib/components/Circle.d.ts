import { BBox, SerializedVector2, SignalValue, SimpleSignal } from '@motion-canvas/core';
import { CurveProfile } from '../curves';
import { DesiredLength } from '../partials';
import { Curve, CurveProps } from './Curve';
export interface CircleProps extends CurveProps {
    /**
     * {@inheritDoc Circle.startAngle}
     */
    startAngle?: SignalValue<number>;
    /**
     * {@inheritDoc Circle.endAngle}
     */
    endAngle?: SignalValue<number>;
    /**
     * {@inheritDoc Circle.counterclockwise}
     */
    counterclockwise?: SignalValue<boolean>;
    /**
     * {@inheritDoc Circle.closed}
     */
    closed?: SignalValue<boolean>;
}
/**
 * A node for drawing circular shapes.
 *
 * @remarks
 * This node can be used to render shapes such as: circle, ellipse, arc, and
 * sector (pie chart).
 *
 * @preview
 * ```tsx editor
 * // snippet Simple circle
 * import {makeScene2D, Circle} from '@motion-canvas/2d';
 *
 * export default makeScene2D(function* (view) {
 *   view.add(
 *     <Circle
 *       size={160}
 *       fill={'lightseagreen'}
 *     />
 *    );
 * });
 *
 * // snippet Ellipse
 * import {makeScene2D, Circle} from '@motion-canvas/2d';
 *
 * export default makeScene2D(function* (view) {
 *   view.add(
 *     <Circle
 *       width={160}
 *       height={80}
 *       fill={'lightseagreen'}
 *     />
 *   );
 * });
 *
 * // snippet Sector (pie chart):
 * import {makeScene2D, Circle} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const ref = createRef<Circle>();
 *   view.add(
 *     <Circle
 *       ref={ref}
 *       size={160}
 *       fill={'lightseagreen'}
 *       startAngle={30}
 *       endAngle={270}
 *       closed={true}
 *     />
 *   );
 *
 *   yield* ref().startAngle(270, 2).to(30, 2);
 * });
 *
 * // snippet Arc:
 * import {makeScene2D, Circle} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const ref = createRef<Circle>();
 *   view.add(
 *     <Circle
 *       ref={ref}
 *       size={160}
 *       stroke={'lightseagreen'}
 *       lineWidth={8}
 *       startAngle={-90}
 *       endAngle={90}
 *     />
 *   );
 *
 *   yield* ref().startAngle(-270, 2).to(-90, 2);
 * });
 *
 * // snippet Curve properties:
 * import {makeScene2D, Circle} from '@motion-canvas/2d';
 * import {all, createRef, easeInCubic, easeOutCubic} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const ref = createRef<Circle>();
 *   view.add(
 *     <Circle
 *       ref={ref}
 *       size={160}
 *       stroke={'lightseagreen'}
 *       lineWidth={8}
 *       endAngle={270}
 *       endArrow
 *     />,
 *   );
 *
 *   yield* all(ref().start(1, 1), ref().rotation(180, 1, easeInCubic));
 *   ref().start(0).end(0);
 *   yield* all(ref().end(1, 1), ref().rotation(360, 1, easeOutCubic));
 * });
 * ```
 */
export declare class Circle extends Curve {
    /**
     * The starting angle in degrees for the circle sector.
     *
     * @remarks
     * This property can be used together with {@link startAngle} to turn this
     * circle into a sector (when using fill) or an arc (when using stroke).
     *
     * @defaultValue 0
     */
    readonly startAngle: SimpleSignal<number, this>;
    /**
     * The ending angle in degrees for the circle sector.
     *
     * @remarks
     * This property can be used together with {@link endAngle} to turn this
     * circle into a sector (when using fill) or an arc (when using stroke).
     *
     * @defaultValue 360
     */
    readonly endAngle: SimpleSignal<number, this>;
    /**
     * Whether the circle sector should be drawn counterclockwise.
     *
     * @remarks
     * By default, the circle begins at {@link startAngle} and is drawn clockwise
     * until reaching {@link endAngle}. Setting this property to true will reverse
     * this direction.
     */
    readonly counterclockwise: SimpleSignal<boolean, this>;
    /**
     * Whether the path of this circle should be closed.
     *
     * @remarks
     * When set to true, the path of this circle will start and end at the center.
     * This can be used to fine-tune how sectors are rendered.
     *
     * @example
     * A closed circle will look like a pie chart:
     * ```tsx
     * <Circle
     *   size={300}
     *   fill={'lightseagreen'}
     *   endAngle={230}
     *   closed={true}
     * />
     * ```
     * An open one will look like an arc:
     * ```tsx
     * <Circle
     *   size={300}
     *   stroke={'lightseagreen'}
     *   lineWidth={8}
     *   endAngle={230}
     *   closed={false}
     * />
     * ```
     *
     * @defaultValue false
     */
    readonly closed: SimpleSignal<boolean, this>;
    constructor(props: CircleProps);
    profile(): CurveProfile;
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected offsetComputedLayout(box: BBox): BBox;
    protected childrenBBox(): BBox;
    protected getPath(): Path2D;
    protected getRipplePath(): Path2D;
    protected getCacheBBox(): BBox;
    protected createPath(expand?: number): Path2D;
}
//# sourceMappingURL=Circle.d.ts.map