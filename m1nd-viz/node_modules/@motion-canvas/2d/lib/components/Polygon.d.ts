import { BBox, SerializedVector2, SignalValue, SimpleSignal, Vector2 } from '@motion-canvas/core';
import { CurveProfile } from '../curves';
import { DesiredLength } from '../partials';
import { Curve, CurveProps } from './Curve';
export interface PolygonProps extends CurveProps {
    /**
     * {@inheritDoc Polygon.sides}
     */
    sides?: SignalValue<number>;
    /**
     * {@inheritDoc Polygon.radius}
     */
    radius?: SignalValue<number>;
}
/**
 * A node for drawing regular polygons.
 *
 * @remarks
 * This node can be used to render shapes such as: triangle, pentagon,
 * hexagon and more.
 *
 * Note that the polygon is inscribed in a circle defined by the height
 * and width. If height and width are unequal, the polygon is inscribed
 * in the resulting ellipse.
 *
 * Since the polygon is inscribed in the circle, the actual displayed
 * height and width may differ somewhat from the bounding rectangle. This
 * will be particularly noticeable if the number of sides is low, e.g. for a
 * triangle.
 *
 * @preview
 * ```tsx editor
 * // snippet Polygon
 * import {makeScene2D, Polygon} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const ref = createRef<Polygon>();
 *   view.add(
 *     <Polygon
 *       ref={ref}
 *       sides={6}
 *       size={160}
 *       fill={'lightseagreen'}
 *     />
 *   );
 *
 *   yield* ref().sides(3, 2).to(6, 2);
 * });
 *
 * // snippet Pentagon outline
 * import {makeScene2D, Polygon} from '@motion-canvas/2d';
 *
 * export default makeScene2D(function* (view) {
 *   view.add(
 *     <Polygon
 *       sides={5}
 *       size={160}
 *       radius={30}
 *       stroke={'lightblue'}
 *       lineWidth={8}
 *     />
 *   );
 * });
 *
 * // snippet Accessing vertex data
 * import {Circle, Polygon, makeScene2D} from '@motion-canvas/2d';
 * import {createRef, range} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const polygon = createRef<Polygon>();
 *   view.add(
 *     <Polygon ref={polygon} sides={3} lineWidth={4} stroke={'white'} size={160}>
 *       {range(6).map(index => (
 *         <Circle
 *           fill={'white'}
 *           size={20}
 *           position={() => polygon().vertex(index)}
 *           opacity={() => polygon().vertexCompletion(index)}
 *         />
 *       ))}
 *     </Polygon>,
 *   );
 *
 *   yield* polygon().sides(6, 2).wait(0.5).back(2);
 * });
 * ```
 */
export declare class Polygon extends Curve {
    /**
     * The number of sides of the polygon.
     *
     * @remarks
     * For example, a value of 6 creates a hexagon.
     *
     * @example
     * ```tsx
     * <Polygon
     *   size={320}
     *   sides={7}
     *   stroke={'#fff'}
     *   lineWidth={8}
     *   fill={'lightseagreen'}
     * />
     * ```
     */
    readonly sides: SimpleSignal<number, this>;
    /**
     * The radius of the polygon's corners.
     *
     * @example
     * ```tsx
     * <Polygon
     *   radius={30}
     *   size={320}
     *   sides={3}
     *   stroke={'#fff'}
     *   lineWidth={8}
     * />
     * ```
     */
    readonly radius: SimpleSignal<number, this>;
    constructor(props: PolygonProps);
    /**
     * Get the position of the nth vertex in the local space of this polygon.
     *
     * @param index - The index of the vertex.
     */
    vertex(index: number): Vector2;
    /**
     * Get the completion of the nth vertex.
     *
     * @remarks
     * The completion is a value between `0` and `1` that describes how the given
     * vertex partakes in the polygon.
     *
     * For integer values of {@link sides}, the completion is simply `1` for
     * each index making up the polygon and `0` for any other index. If `sides`
     * includes a fraction, the last index of the polygon will have a completion
     * equal to said fraction.
     *
     * Check out the {@link Polygon | Accessing vertex data} example for a
     * demonstration.
     *
     * @param index - The index of the vertex.
     */
    vertexCompletion(index: number): number;
    profile(): CurveProfile;
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected offsetComputedLayout(box: BBox): BBox;
    protected childrenBBox(): BBox;
    protected requiresProfile(): boolean;
    protected getPath(): Path2D;
    protected getRipplePath(): Path2D;
    protected createPath(expand?: number): Path2D;
}
//# sourceMappingURL=Polygon.d.ts.map