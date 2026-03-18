import { PossibleVector2, SignalValue, SimpleSignal, Vector2Signal } from '@motion-canvas/core';
import { Shape, ShapeProps } from './Shape';
export interface GridProps extends ShapeProps {
    /**
     * {@inheritDoc Grid.spacing}
     */
    spacing?: SignalValue<PossibleVector2>;
    /**
     * {@inheritDoc Grid.start}
     */
    start?: SignalValue<number>;
    /**
     * {@inheritDoc Grid.end}
     */
    end?: SignalValue<number>;
}
/**
 * A node for drawing a two-dimensional grid.
 *
 * @preview
 * ```tsx editor
 * import {Grid, makeScene2D} from '@motion-canvas/2d';
 * import {all, createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const grid = createRef<Grid>();
 *
 *   view.add(
 *     <Grid
 *       ref={grid}
 *       width={'100%'}
 *       height={'100%'}
 *       stroke={'#666'}
 *       start={0}
 *       end={1}
 *     />,
 *   );
 *
 *   yield* all(
 *     grid().end(0.5, 1).to(1, 1).wait(1),
 *     grid().start(0.5, 1).to(0, 1).wait(1),
 *   );
 * });
 * ```
 */
export declare class Grid extends Shape {
    /**
     * The spacing between the grid lines.
     */
    readonly spacing: Vector2Signal<this>;
    /**
     * The percentage that should be clipped from the beginning of each grid line.
     *
     * @remarks
     * The portion of each grid line that comes before the given percentage will
     * be made invisible.
     *
     * This property is useful for animating the grid appearing on-screen.
     */
    readonly start: SimpleSignal<number, this>;
    /**
     * The percentage that should be clipped from the end of each grid line.
     *
     * @remarks
     * The portion of each grid line that comes after the given percentage will
     * be made invisible.
     *
     * This property is useful for animating the grid appearing on-screen.
     */
    readonly end: SimpleSignal<number, this>;
    constructor(props: GridProps);
    protected drawShape(context: CanvasRenderingContext2D): void;
    private mapPoints;
}
//# sourceMappingURL=Grid.d.ts.map