var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { all, DEFAULT, easeInOutCubic, modify, threadable, tween, unwrap, Vector2, } from '@motion-canvas/core';
import { cloneable, signal } from '../decorators';
import { Node } from './Node';
import { Rect } from './Rect';
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
export class Camera extends Node {
    constructor({ children, ...props }) {
        super(props);
        if (!this.scene()) {
            this.scene(new Node({}));
        }
        if (children) {
            this.scene().add(children);
        }
    }
    getZoom() {
        return 1 / this.scale.x();
    }
    setZoom(value) {
        this.scale(modify(value, unwrapped => 1 / unwrapped));
    }
    getDefaultZoom() {
        return this.scale.x.context.getInitial();
    }
    *tweenZoom(value, duration, timingFunction, interpolationFunction) {
        const from = this.scale.x();
        yield* tween(duration, v => {
            this.zoom(1 / interpolationFunction(from, 1 / unwrap(value), timingFunction(v)));
        });
    }
    /**
     * Resets the camera's position, rotation and zoom level to their original
     * values.
     *
     * @param duration - The duration of the tween.
     * @param timingFunction - The timing function to use for the tween.
     */
    *reset(duration, timingFunction = easeInOutCubic) {
        yield* all(this.position(DEFAULT, duration, timingFunction), this.zoom(DEFAULT, duration, timingFunction), this.rotation(DEFAULT, duration, timingFunction));
    }
    *centerOn(positionOrNode, duration, timing = easeInOutCubic, interpolationFunction = Vector2.lerp) {
        const position = positionOrNode instanceof Node
            ? positionOrNode
                .absolutePosition()
                .transformAsPoint(this.scene().worldToLocal())
            : positionOrNode;
        yield* this.position(position, duration, timing, interpolationFunction);
    }
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
    *followCurve(curve, duration, timing = easeInOutCubic) {
        yield* tween(duration, value => {
            const t = timing(value);
            const point = curve
                .getPointAtPercentage(t)
                .position.transformAsPoint(curve.localToWorld());
            this.position(point);
        });
    }
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
    *followCurveReverse(curve, duration, timing = easeInOutCubic) {
        yield* tween(duration, value => {
            const t = 1 - timing(value);
            const point = curve
                .getPointAtPercentage(t)
                .position.transformAsPoint(curve.localToWorld());
            this.position(point);
        });
    }
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
    *followCurveWithRotation(curve, duration, timing = easeInOutCubic) {
        yield* tween(duration, value => {
            const t = timing(value);
            const { position, normal } = curve.getPointAtPercentage(t);
            const point = position.transformAsPoint(curve.localToWorld());
            const angle = normal.flipped.perpendicular.degrees;
            this.position(point);
            this.rotation(angle);
        });
    }
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
    *followCurveWithRotationReverse(curve, duration, timing = easeInOutCubic) {
        yield* tween(duration, value => {
            const t = 1 - timing(value);
            const { position, normal } = curve.getPointAtPercentage(t);
            const point = position.transformAsPoint(curve.localToWorld());
            const angle = normal.flipped.perpendicular.degrees;
            this.position(point);
            this.rotation(angle);
        });
    }
    transformContext(context) {
        const matrix = this.localToParent().inverse();
        context.transform(matrix.a, matrix.b, matrix.c, matrix.d, matrix.e, matrix.f);
    }
    hit(position) {
        const local = position.transformAsPoint(this.localToParent());
        return this.scene().hit(local);
    }
    drawChildren(context) {
        this.scene().drawChildren(context);
    }
    // eslint-disable-next-line @typescript-eslint/naming-convention
    static Stage({ children, cameraRef, scene, ...props }) {
        const camera = new Camera({ scene: scene, children });
        cameraRef?.(camera);
        return new Rect({
            clip: true,
            ...props,
            children: [camera],
        });
    }
}
__decorate([
    signal()
], Camera.prototype, "scene", void 0);
__decorate([
    cloneable(false),
    signal()
], Camera.prototype, "zoom", void 0);
__decorate([
    threadable()
], Camera.prototype, "reset", null);
__decorate([
    threadable()
], Camera.prototype, "centerOn", null);
__decorate([
    threadable()
], Camera.prototype, "followCurve", null);
__decorate([
    threadable()
], Camera.prototype, "followCurveReverse", null);
__decorate([
    threadable()
], Camera.prototype, "followCurveWithRotation", null);
__decorate([
    threadable()
], Camera.prototype, "followCurveWithRotationReverse", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ2FtZXJhLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL0NhbWVyYS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7QUFBQSxPQUFPLEVBQ0wsR0FBRyxFQUNILE9BQU8sRUFDUCxjQUFjLEVBRWQsTUFBTSxFQUtOLFVBQVUsRUFHVixLQUFLLEVBQ0wsTUFBTSxFQUNOLE9BQU8sR0FDUixNQUFNLHFCQUFxQixDQUFDO0FBQzdCLE9BQU8sRUFBQyxTQUFTLEVBQUUsTUFBTSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBRWhELE9BQU8sRUFBQyxJQUFJLEVBQVksTUFBTSxRQUFRLENBQUM7QUFDdkMsT0FBTyxFQUFDLElBQUksRUFBWSxNQUFNLFFBQVEsQ0FBQztBQWN2Qzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7R0F5Q0c7QUFDSCxNQUFNLE9BQU8sTUFBTyxTQUFRLElBQUk7SUFPOUIsWUFBbUIsRUFBQyxRQUFRLEVBQUUsR0FBRyxLQUFLLEVBQWM7UUFDbEQsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBRWIsSUFBSSxDQUFDLElBQUksQ0FBQyxLQUFLLEVBQUUsRUFBRTtZQUNqQixJQUFJLENBQUMsS0FBSyxDQUFDLElBQUksSUFBSSxDQUFDLEVBQUUsQ0FBQyxDQUFDLENBQUM7U0FDMUI7UUFFRCxJQUFJLFFBQVEsRUFBRTtZQUNaLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQyxHQUFHLENBQUMsUUFBUSxDQUFDLENBQUM7U0FDNUI7SUFDSCxDQUFDO0lBV1MsT0FBTztRQUNmLE9BQU8sQ0FBQyxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsQ0FBQyxFQUFFLENBQUM7SUFDNUIsQ0FBQztJQUVTLE9BQU8sQ0FBQyxLQUEwQjtRQUMxQyxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sQ0FBQyxLQUFLLEVBQUUsU0FBUyxDQUFDLEVBQUUsQ0FBQyxDQUFDLEdBQUcsU0FB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