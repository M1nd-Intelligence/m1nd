import { InterpolationFunction, PossibleVector2, Reference, SignalValue, SimpleSignal, ThreadGenerator, TimingFunction, Vector2 } from '@motion-canvas/core';
import { Curve } from './Curve';
import { Node, NodeProps } from './Node';
import { Rect, RectProps } from './Rect';
export interface CameraProps extends NodeProps {
    /**
     * {@inheritDoc Camera.scene}
     */
    scene?: Node;
    /**
     * {@inheritDoc Camera.zoom}
     */
    zoom?: SignalValue<number>;
}
/**
 * A node representing an orthographic camera.
 *
 * @preview
 * ```tsx editor
 * import {Camera, Circle, makeScene2D, Rect} from '@motion-canvas/2d';
 * import {all, createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const camera = createRef<Camera>();
 *   const rect = createRef<Rect>();
 *   const circle = createRef<Circle>();
 *
 *   view.add(
 *     <>
 *       <Camera ref={camera}>
 *         <Rect
 *           ref={rect}
 *           fill={'lightseagreen'}
 *           size={100}
 *           position={[100, -50]}
 *         />
 *         <Circle
 *           ref={circle}
 *           fill={'hotpink'}
 *           size={120}
 *           position={[-100, 50]}
 *         />
 *       </Camera>
 *     </>,
 *   );
 *
 *   yield* all(
 *     camera().centerOn(rect(), 3),
 *     camera().rotation(180, 3),
 *     camera().zoom(1.8, 3),
 *   );
 *   yield* camera().centerOn(circle(), 2);
 *   yield* camera().reset(1);
 * });
 * ```
 */
export declare class Camera extends Node {
    /**
     * The scene node that the camera is rendering.
     */
    readonly scene: SimpleSignal<Node, this>;
    constructor({ children, ...props }: CameraProps);
    /**
     * The zoom level of the camera.
     *
     * @defaultValue 1
     */
    readonly zoom: SimpleSignal<number, this>;
    protected getZoom(): number;
    protected setZoom(value: SignalValue<number>): void;
    protected getDefaultZoom(): SignalValue<number> | undefined;
    protected tweenZoom(value: SignalValue<number>, duration: number, timingFunction: TimingFunction, interpolationFunction: InterpolationFunction<number>): ThreadGenerator;
    /**
     * Resets the camera's position, rotation and zoom level to their original
     * values.
     *
     * @param duration - The duration of the tween.
     * @param timingFunction - The timing function to use for the tween.
     */
    reset(duration: number, timingFunction?: TimingFunction): ThreadGenerator;
    /**
     * Centers the camera on the specified position without changing the zoom
     * level.
     *
     * @param position - The position to center the camera on.
     * @param duration - The duration of the tween.
     * @param timingFunction - The timing function to use for the tween.
     * @param interpolationFunction - The interpolation function to use for the
     * tween.
     */
    centerOn(position: PossibleVector2, duration: number, timingFunction?: TimingFunction, interpolationFunction?: InterpolationFunction<Vector2>): ThreadGenerator;
    /**
     * Centers the camera on the specified node without changing the zoom level.
     *
     * @param node - The node to center the camera on.
     * @param duration - The duration of the tween.
     * @param timingFunction - The timing function to use for the tween.
     * @param interpolationFunction - The interpolation function to use for the
     * tween.
     */
    centerOn(node: Node, duration: number, timingFunction?: TimingFunction, interpolationFunction?: InterpolationFunction<Vector2>): ThreadGenerator;
    /**
     * Makes the camera follow a path specified by the provided curve.
     *
     * @remarks
     * This will not change the orientation of the camera. To make the camera
     * orient itself along the curve, use {@link followCurveWithRotation} or
     * {@link followCurveWithRotationReverse}.
     *
     * If you want to follow the curve in reverse, use {@link followCurveReverse}.
     *
     * @param curve - The curve to follow.
     * @param duration - The duration of the tween.
     * @param timing - The timing function to use for the tween.
     */
    followCurve(curve: Curve, duration: number, timing?: TimingFunction): ThreadGenerator;
    /**
     * Makes the camera follow a path specified by the provided curve in reverse.
     *
     * @remarks
     * This will not change the orientation of the camera. To make the camera
     * orient itself along the curve, use {@link followCurveWithRotation} or
     * {@link followCurveWithRotationReverse}.
     *
     * If you want to follow the curve forward, use {@link followCurve}.
     *
     * @param curve - The curve to follow.
     * @param duration - The duration of the tween.
     * @param timing - The timing function to use for the tween.
     */
    followCurveReverse(curve: Curve, duration: number, timing?: TimingFunction): Generator<void | ThreadGenerator | Promise<any> | import("@motion-canvas/core").Promisable<any>, void, any>;
    /**
     * Makes the camera follow a path specified by the provided curve while
     * pointing the camera the direction of the tangent.
     *
     * @remarks
     * To make the camera follow the curve without changing its orientation, use
     * {@link followCurve} or {@link followCurveReverse}.
     *
     * If you want to follow the curve in reverse, use
     * {@link followCurveWithRotationReverse}.
     *
     * @param curve - The curve to follow.
     * @param duration - The duration of the tween.
     * @param timing - The timing function to use for the tween.
     */
    followCurveWithRotation(curve: Curve, duration: number, timing?: TimingFunction): Generator<void | ThreadGenerator | Promise<any> | import("@motion-canvas/core").Promisable<any>, void, any>;
    /**
     * Makes the camera follow a path specified by the provided curve in reverse
     * while pointing the camera the direction of the tangent.
     *
     * @remarks
     * To make the camera follow the curve without changing its orientation, use
     * {@link followCurve} or {@link followCurveReverse}.
     *
     * If you want to follow the curve forward, use
     * {@link followCurveWithRotation}.
     *
     * @param curve - The curve to follow.
     * @param duration - The duration of the tween.
     * @param timing - The timing function to use for the tween.
     */
    followCurveWithRotationReverse(curve: Curve, duration: number, timing?: TimingFunction): Generator<void | ThreadGenerator | Promise<any> | import("@motion-canvas/core").Promisable<any>, void, any>;
    protected transformContext(context: CanvasRenderingContext2D): void;
    hit(position: Vector2): Node | null;
    protected drawChildren(context: CanvasRenderingContext2D): void;
    static Stage({ children, cameraRef, scene, ...props }: RectProps & {
        cameraRef?: Reference<Camera>;
        scene?: Node;
    }): Rect;
}
//# sourceMappingURL=Camera.d.ts.map