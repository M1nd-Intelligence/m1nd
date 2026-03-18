import { BBox, PossibleVector2, SignalValue, Vector2Signal } from '@motion-canvas/core';
import { CurveProfile } from '../curves';
import { Curve, CurveProps } from './Curve';
export interface RayProps extends CurveProps {
    /**
     * {@inheritDoc Ray.from}
     */
    from?: SignalValue<PossibleVector2>;
    fromX?: SignalValue<number>;
    fromY?: SignalValue<number>;
    /**
     * {@inheritDoc Ray.to}
     */
    to?: SignalValue<PossibleVector2>;
    toX?: SignalValue<number>;
    toY?: SignalValue<number>;
}
/**
 * A node for drawing an individual line segment.
 *
 * @preview
 * ```tsx editor
 * import {makeScene2D} from '@motion-canvas/2d';
 * import {Ray} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const ray = createRef<Ray>();
 *
 *   view.add(
 *     <Ray
 *       ref={ray}
 *       lineWidth={8}
 *       endArrow
 *       stroke={'lightseagreen'}
 *       fromX={-200}
 *       toX={200}
 *     />,
 *   );
 *
 *   yield* ray().start(1, 1);
 *   yield* ray().start(0).end(0).start(1, 1);
 * });
 * ```
 */
export declare class Ray extends Curve {
    /**
     * The starting point of the ray.
     */
    readonly from: Vector2Signal<this>;
    /**
     * The ending point of the ray.
     */
    readonly to: Vector2Signal<this>;
    constructor(props: RayProps);
    protected childrenBBox(): BBox;
    profile(): CurveProfile;
    drawOverlay(context: CanvasRenderingContext2D, matrix: DOMMatrix): void;
}
//# sourceMappingURL=Ray.d.ts.map