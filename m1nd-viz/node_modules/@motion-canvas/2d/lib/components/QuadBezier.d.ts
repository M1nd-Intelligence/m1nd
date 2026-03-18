import { PossibleVector2, SignalValue, Vector2Signal } from '@motion-canvas/core';
import { PolynomialSegment } from '../curves/PolynomialSegment';
import { Bezier, BezierOverlayInfo } from './Bezier';
import { CurveProps } from './Curve';
export interface QuadBezierProps extends CurveProps {
    p0?: SignalValue<PossibleVector2>;
    p0X?: SignalValue<number>;
    p0Y?: SignalValue<number>;
    p1?: SignalValue<PossibleVector2>;
    p1X?: SignalValue<number>;
    p1Y?: SignalValue<number>;
    p2?: SignalValue<PossibleVector2>;
    p2X?: SignalValue<number>;
    p2Y?: SignalValue<number>;
}
/**
 * A node for drawing a quadratic Bézier curve.
 *
 * @preview
 * ```tsx editor
 * import {makeScene2D, QuadBezier} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const bezier = createRef<QuadBezier>();
 *
 *   view.add(
 *     <QuadBezier
 *       ref={bezier}
 *       lineWidth={4}
 *       stroke={'lightseagreen'}
 *       p0={[-200, 0]}
 *       p1={[0, -200]}
 *       p2={[200, 0]}
 *       end={0}
 *     />
 *   );
 *
 *   yield* bezier().end(1, 1);
 *   yield* bezier().start(1, 1).to(0, 1);
 * });
 * ```
 */
export declare class QuadBezier extends Bezier {
    /**
     * The start point of the Bézier curve.
     */
    readonly p0: Vector2Signal<this>;
    /**
     * The control point of the Bézier curve.
     */
    readonly p1: Vector2Signal<this>;
    /**
     * The end point of the Bézier curve.
     */
    readonly p2: Vector2Signal<this>;
    constructor(props: QuadBezierProps);
    protected segment(): PolynomialSegment;
    protected overlayInfo(matrix: DOMMatrix): BezierOverlayInfo;
}
//# sourceMappingURL=QuadBezier.d.ts.map