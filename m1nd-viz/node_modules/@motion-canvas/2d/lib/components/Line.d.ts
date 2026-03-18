import { BBox, PossibleVector2, SignalValue, SimpleSignal, ThreadGenerator, TimingFunction, Vector2 } from '@motion-canvas/core';
import { CurveProfile } from '../curves';
import { Curve, CurveProps } from './Curve';
export interface LineProps extends CurveProps {
    /**
     * {@inheritDoc Line.radius}
     */
    radius?: SignalValue<number>;
    /**
     * {@inheritDoc Line.points}
     */
    points?: SignalValue<SignalValue<PossibleVector2>[]>;
}
/**
 * A node for drawing lines and polygons.
 *
 * @remarks
 * This node can be used to render any polygonal shape defined by a set of
 * points.
 *
 * @preview
 * ```tsx editor
 * // snippet Simple line
 * import {makeScene2D, Line} from '@motion-canvas/2d';
 *
 * export default makeScene2D(function* (view) {
 *   view.add(
 *     <Line
 *       points={[
 *         [150, 50],
 *         [0, -50],
 *         [-150, 50],
 *       ]}
 *       stroke={'lightseagreen'}
 *       lineWidth={8}
 *       radius={40}
 *       startArrow
 *     />,
 *   );
 * });
 *
 * // snippet Polygon
 * import {makeScene2D, Line} from '@motion-canvas/2d';
 *
 * export default makeScene2D(function* (view) {
 *   view.add(
 *     <Line
 *       points={[
 *         [-200, 70],
 *         [150, 70],
 *         [100, -70],
 *         [-100, -70],
 *       ]}
 *       fill={'lightseagreen'}
 *       closed
 *     />,
 *   );
 * });
 *
 * // snippet Using signals
 * import {makeScene2D, Line} from '@motion-canvas/2d';
 * import {createSignal} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const tip = createSignal(-150);
 *   view.add(
 *     <Line
 *       points={[
 *         [-150, 70],
 *         [150, 70],
 *         // this point is dynamically calculated based on the signal:
 *         () => [tip(), -70],
 *       ]}
 *       stroke={'lightseagreen'}
 *       lineWidth={8}
 *       closed
 *     />,
 *   );
 *
 *   yield* tip(150, 1).back(1);
 * });
 *
 * // snippet Tweening points
 * import {makeScene2D, Line} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const line = createRef<Line>();
 *   view.add(
 *     <Line
 *       ref={line}
 *       points={[
 *         [-150, 70],
 *         [150, 70],
 *         [0, -70],
 *       ]}
 *       stroke={'lightseagreen'}
 *       lineWidth={8}
 *       radius={20}
 *       closed
 *     />,
 *   );
 *
 *   yield* line()
 *     .points(
 *       [
 *         [-150, 0],
 *         [0, 100],
 *         [150, 0],
 *         [150, -70],
 *         [-150, -70],
 *       ],
 *       2,
 *     )
 *     .back(2);
 * });
 * ```
 */
export declare class Line extends Curve {
    /**
     * Rotate the points to minimize the overall distance traveled when tweening.
     *
     * @param points - The points to rotate.
     * @param reference - The reference points to which the distance is measured.
     * @param closed - Whether the points form a closed polygon.
     */
    private static rotatePoints;
    /**
     * Distribute additional points along the polyline.
     *
     * @param points - The points of a polyline along which new points should be
     *                 distributed.
     * @param count - The number of points to add.
     */
    private static distributePoints;
    /**
     * The radius of the line's corners.
     */
    readonly radius: SimpleSignal<number, this>;
    /**
     * The points of the line.
     *
     * @remarks
     * When set to `null`, the Line will use the positions of its children as
     * points.
     */
    readonly points: SimpleSignal<SignalValue<PossibleVector2>[] | null, this>;
    protected tweenPoints(value: SignalValue<SignalValue<PossibleVector2>[] | null>, time: number, timingFunction: TimingFunction): ThreadGenerator;
    private tweenedPoints;
    constructor(props: LineProps);
    protected childrenBBox(): BBox;
    parsedPoints(): Vector2[];
    profile(): CurveProfile;
    protected lineWidthCoefficient(): number;
    drawOverlay(context: CanvasRenderingContext2D, matrix: DOMMatrix): void;
    private parsePoints;
}
//# sourceMappingURL=Line.d.ts.map