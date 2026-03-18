var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { BBox, Vector2, unwrap, useLogger, } from '@motion-canvas/core';
import { CubicBezierSegment, getBezierSplineProfile, } from '../curves';
import { computed, initial, signal } from '../decorators';
import { arc, bezierCurveTo, drawLine, drawPivot, lineTo, moveTo, quadraticCurveTo, } from '../utils';
import { Curve } from './Curve';
import { Knot } from './Knot';
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
export class Spline extends Curve {
    constructor(props) {
        super(props);
        if ((props.children === undefined ||
            !Array.isArray(props.children) ||
            props.children.length < 2) &&
            (props.points === undefined ||
                (typeof props.points !== 'function' && props.points.length < 2)) &&
            props.spawner === undefined) {
            useLogger().warn({
                message: 'Insufficient number of knots specified for spline. A spline needs at least two knots.',
                remarks: "<p>The spline won&#39;t be visible unless you specify at least two knots:</p>\n<pre class=\"\"><code class=\"language-tsx\">&lt;<span class=\"hljs-title class_\">Spline</span>\n  stroke=<span class=\"hljs-string\">&quot;#fff&quot;</span>\n  lineWidth={<span class=\"hljs-number\">8</span>}\n  points={[\n    [<span class=\"hljs-number\">100</span>, <span class=\"hljs-number\">0</span>],\n    [<span class=\"hljs-number\">0</span>, <span class=\"hljs-number\">0</span>],\n    [<span class=\"hljs-number\">0</span>, <span class=\"hljs-number\">100</span>],\n  ]}\n/&gt;</code></pre><p>For more control over the knot handles, you can alternatively provide the knots\nas children to the spline using the <code>Knot</code> component:</p>\n<pre class=\"\"><code class=\"language-tsx\">&lt;<span class=\"hljs-title class_\">Spline</span> stroke=<span class=\"hljs-string\">&quot;#fff&quot;</span> lineWidth={<span class=\"hljs-number\">8</span>}&gt;\n  <span class=\"language-xml\"><span class=\"hljs-tag\">&lt;<span class=\"hljs-name\">Knot</span> <span class=\"hljs-attr\">x</span>=<span class=\"hljs-string\">{100}</span> <span class=\"hljs-attr\">endHandle</span>=<span class=\"hljs-string\">{[-50,</span> <span class=\"hljs-attr\">0</span>]} /&gt;</span></span>\n  <span class=\"language-xml\"><span class=\"hljs-tag\">&lt;<span class=\"hljs-name\">Knot</span> /&gt;</span></span>\n  <span class=\"language-xml\"><span class=\"hljs-tag\">&lt;<span class=\"hljs-name\">Knot</span> <span class=\"hljs-attr\">y</span>=<span class=\"hljs-string\">{100}</span> <span class=\"hljs-attr\">startHandle</span>=<span class=\"hljs-string\">{[-100,</span> <span class=\"hljs-attr\">50</span>]} /&gt;</span></span>\n&lt;/<span class=\"hljs-title class_\">Spline</span>&gt;</code></pre>",
                inspect: this.key,
            });
        }
    }
    profile() {
        return getBezierSplineProfile(this.knots(), this.closed(), this.smoothness());
    }
    knots() {
        const points = this.points();
        if (points) {
            return points.map(signal => {
                const point = new Vector2(unwrap(signal));
                return {
                    position: point,
                    startHandle: point,
                    endHandle: point,
                    auto: { start: 1, end: 1 },
                };
            });
        }
        return this.children()
            .filter(this.isKnot)
            .map(knot => knot.points());
    }
    childrenBBox() {
        const points = this.profile().segments.flatMap(segment => segment.points);
        return BBox.fromPoints(...points);
    }
    lineWidthCoefficient() {
        const join = this.lineJoin();
        let coefficient = super.lineWidthCoefficient();
        if (join !== 'miter') {
            return coefficient;
        }
        const { minSin } = this.profile();
        if (minSin > 0) {
            coefficient = Math.max(coefficient, 0.5 / minSin);
        }
        return coefficient;
    }
    desiredSize() {
        return this.getTightBBox().size;
    }
    offsetComputedLayout(box) {
        box.position = box.position.sub(this.getTightBBox().center);
        return box;
    }
    getTightBBox() {
        const bounds = this.profile().segments.map(segment => segment.getBBox());
        return BBox.fromBBoxes(...bounds);
    }
    drawOverlay(context, matrix) {
        const size = this.computedSize();
        const box = this.childrenBBox().transformCorners(matrix);
        const offset = size.mul(this.offset()).scale(0.5).transformAsPoint(matrix);
        const segments = this.profile().segments;
        context.lineWidth = 1;
        context.strokeStyle = 'white';
        context.fillStyle = 'white';
        const splinePath = new Path2D();
        // Draw the actual spline first so that all control points get drawn on top of it.
        for (let i = 0; i < segments.length; i++) {
            const segment = segments[i];
            const [from, startHandle, endHandle, to] = segment.transformPoints(matrix);
            moveTo(splinePath, from);
            if (segment instanceof CubicBezierSegment) {
                bezierCurveTo(splinePath, startHandle, endHandle, to);
            }
            else {
                quadraticCurveTo(splinePath, startHandle, endHandle);
            }
        }
        context.stroke(splinePath);
        for (let i = 0; i < segments.length; i++) {
            const segment = segments[i];
            context.fillStyle = 'white';
            const [from, startHandle, endHandle, to] = segment.transformPoints(matrix);
            const handlePath = new Path2D();
            context.globalAlpha = 0.5;
            // Line from p0 to p1
            moveTo(handlePath, from);
            lineTo(handlePath, startHandle);
            if (segment instanceof CubicBezierSegment) {
                // Line from p2 to p3
                moveTo(handlePath, endHandle);
                lineTo(handlePath, to);
                context.beginPath();
                context.stroke(handlePath);
            }
            else {
                // Line from p1 to p2
                lineTo(handlePath, endHandle);
                context.beginPath();
                context.stroke(handlePath);
            }
            context.globalAlpha = 1;
            context.lineWidth = 2;
            // Draw first point of segment
            moveTo(context, from);
            context.beginPath();
            arc(context, from, 4);
            context.closePath();
            context.stroke();
            context.fill();
            // Draw final point of segment only if we're on the last segment.
            // Otherwise, it will get drawn as the start point of the next segment.
            if (i === segments.length - 1) {
                if (to !== undefined) {
                    moveTo(context, to);
                    context.beginPath();
                    arc(context, to, 4);
                    context.closePath();
                    context.stroke();
                    context.fill();
                }
            }
            // Draw the control points
            context.fillStyle = 'black';
            for (const point of [startHandle, endHandle]) {
                if (point.magnitude > 0) {
                    moveTo(context, point);
                    context.beginPath();
                    arc(context, point, 4);
                    context.closePath();
                    context.fill();
                    context.stroke();
                }
            }
        }
        context.lineWidth = 1;
        context.beginPath();
        drawPivot(context, offset);
        context.stroke();
        context.beginPath();
        drawLine(context, box);
        context.closePath();
        context.stroke();
    }
    isKnot(node) {
        return node instanceof Knot;
    }
}
__decorate([
    initial(0.4),
    signal()
], Spline.prototype, "smoothness", void 0);
__decorate([
    initial(null),
    signal()
], Spline.prototype, "points", void 0);
__decorate([
    computed()
], Spline.prototype, "profile", null);
__decorate([
    computed()
], Spline.prototype, "knots", null);
__decorate([
    computed()
], Spline.prototype, "childrenBBox", null);
__decorate([
    computed()
], Spline.prototype, "getTightBBox", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiU3BsaW5lLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL1NwbGluZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7QUFBQSxPQUFPLEVBQ0wsSUFBSSxFQUtKLE9BQU8sRUFDUCxNQUFNLEVBQ04sU0FBUyxHQUNWLE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUNMLGtCQUFrQixFQUdsQixzQkFBc0IsR0FDdkIsTUFBTSxXQUFXLENBQUM7QUFFbkIsT0FBTyxFQUFDLFFBQVEsRUFBRSxPQUFPLEVBQUUsTUFBTSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBRXhELE9BQU8sRUFDTCxHQUFHLEVBQ0gsYUFBYSxFQUNiLFFBQVEsRUFDUixTQUFTLEVBQ1QsTUFBTSxFQUNOLE1BQU0sRUFDTixnQkFBZ0IsR0FDakIsTUFBTSxVQUFVLENBQUM7QUFDbEIsT0FBTyxFQUFDLEtBQUssRUFBYSxNQUFNLFNBQVMsQ0FBQztBQUMxQyxPQUFPLEVBQUMsSUFBSSxFQUFDLE1BQU0sUUFBUSxDQUFDO0FBZ0I1Qjs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0dBMENHO0FBQ0gsTUFBTSxPQUFPLE1BQU8sU0FBUSxLQUFLO0lBMkIvQixZQUFtQixLQUFrQjtRQUNuQyxLQUFLLENBQUMsS0FBSyxDQUFDLENBQUM7UUFFYixJQUNFLENBQUMsS0FBSyxDQUFDLFFBQVEsS0FBSyxTQUFTO1lBQzNCLENBQUMsS0FBSyxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUMsUUFBUSxDQUFDO1lBQzlCLEtBQUssQ0FBQyxRQUFRLENBQUMsTUFBTSxHQUFHLENBQUMsQ0FBQztZQUM1QixDQUFDLEtBQUssQ0FBQyxNQUFNLEtBQUssU0FBUztnQkFDekIsQ0FBQyxPQUFPLEtBQUssQ0FBQyxNQUFNLEtBQUssVUFBVSxJQUFJLEtBQUssQ0FBQyxNQUFNLENBQUMsTUFBTSxHQUFHLENBQUMsQ0FBQyxDQUFDO1lBQ2xFLEtBQUssQ0FBQyxPQUFPLEtBQUssU0FBUyxFQUMzQjtZQUNBLFNBQVMsRUFBRSxDQUFDLElBQUksQ0FBQztnQkFDZixPQUFPLEVBQ0wsdUZBQXVGO2dCQUN6RixPQUFPLCt1REFBNkI7Z0JBQ3BDLE9BQU8sRUFBRSxJQUFJLENBQUMsR0FBRzthQUNsQixDQUFDLENBQUM7U0FDSjtJQUNILENBQUM7SUFHZSxPQUFPO1FBQ3JCLE9BQU8sc0JBQXNCLENBQzNCLElBQUksQ0FBQyxLQUFLLEVBQUUsRUFDWixJQUFJLENBQUMsTUFBTSxFQUFFLEVBQ2IsSUFBSSxDQUFDLFVBQVUsRUFBRSxDQUNsQixDQUFDO0lBQ0osQ0FBQztJQUdNLEtBQUs7UUFDVixNQUFNLE1BQU0sR0FBRyxJQUFJLENBQUMsTUFBTSxFQUFFLENBQUM7UUFFN0IsSUFBSSxNQUFNLEVBQUU7WUFDVixPQUFPLE1BQU0sQ0FBQyxHQUFHLENBQUMsTUFBTSxDQUFDLEVBQUU7Z0JBQ3pCLE1BQU0sS0FBSyxHQUFHLElBQUksT0FBTyxDQUFDLE1BQU0sQ0FBQyxNQUFNLENBQUMsQ0FBQyxDQUFDO2dCQUUxQyxPQUFPO29CQUNMLFFBQVEsRUFBRSxLQUFLO29CQUNmLFdBQVcsRUFBRSxLQUFLO29CQUNsQixTQUFTLEVBQUUsS0FBSztvQkFDaEIsSUFBSSxFQUFFLEVBQUMsS0FBSyxFQUFFLENBQUMsRUFBRSxHQUFHLEVBQUUsQ0FBQyxFQUFDO2lCQUN6QixDQUFDO1lBQ0osQ0FBQyxDQUFDLENBQUM7U0FDSjtRQUVELE9BQU8sSUFBSSxDQUFDLFFBQVEsRUFBRTthQUNuQixNQUFNLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQzthQUNuQixHQUFHLENBQUMsSUFBSSxDQUFDLEVBQUUsQ0FBQyxJQUFJLENBQUMsTUFBTSxFQUFFLENBQUMsQ0FBQztJQUNoQyxDQUFDO0lBR1MsWUFBWTtRQUNwQixNQUFNLE1BQU0sR0FBSSxJQUFJLENBQUMsT0FBTyxFQUFFLENBQUMsUUFBZ0MsQ0FBQyxPQUFPLENBQ3JFLE9BQU8sQ0FBQyxFQUFFLENBQUMsT0FBTyxDQUFDLE1BQU0sQ0FDMUIsQ0FBQztRQUNGLE9BQU8sSUFBSSxDQUFDLFVBQVUsQ0FBQyxHQUFHLE1BQU0sQ0FBQyxDQUFDO0lBQ3BDLENBQUM7SUFFa0Isb0JBQW9CO1FBQ3JDLE1BQU0sSUFBSSxHQUFHLElBQUksQ0FBQyxRQUFRLEVBQUUsQ0FBQztRQUU3QixJQUFJLFdBQVcsR0FBRyxLQUFLLENBQUMsb0JBQW9CLEVBQUUsQ0FBQztRQUUvQyxJQUFJLElBQUksS0FBSyxPQUFPLEVBQUU7WUFDcEIsT0FBTyxXQUFXLENBQUM7U0FDcEI7UUFFRCxNQUFNLEVBQUMsTUFBTSxFQUFDLEdBQUcsSUFBSSxDQUFDLE9BQU8sRUFBRSxDQUFDO1FBQ2hDLElBQUksTUFBTSxHQUFHLENBQUMsRUFBRTtZQUNkLFdBQVcsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLFdBQVcsRUFBRSxHQUFHLEdBQUcsTUFBTSxDQUFDLENBQUM7U0FDbkQ7UUFFRCxPQUFPLFdBQVcsQ0FBQztJQUNyQixDQUFDO0lBRWtCLFdBQVc7UUFDNUIsT0FBTyxJQUFJLENBQUMsWUFBWSxFQUFFLENBQUMsSUFBSSxDQUFDO0lBQ2xDLENBQUM7SUFFa0Isb0JBQW9CLENBQUMsR0FBUztRQUMvQyxHQUFHLENBQUMsUUFBUSxHQUFHLEdBQUcsQ0FBQyxRQUFRLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxZQUFZLEVBQUUsQ0FBQyxNQUFNLENBQUMsQ0FBQztRQUM1RCxPQUFPLEdBQUcsQ0FBQztJQUNiLENBQUM7SUFHTyxZQUFZO1FBQ2xCLE1BQU0sTUFBTSxHQUFJLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxRQUFnQyxDQUFDLEdBQUcsQ0FDakUsT0FBTyxDQUFDLEVBQUUsQ0FBQyxPQUFPLENBQUMsT0FBTyxFQUFFLENBQzdCLENBQUM7UUFDRixPQUFPLElBQUksQ0FBQyxVQUFVLENBQUMsR0FBRyxNQUFNLENBQUMsQ0FBQztJQUNwQyxDQUFDO0lBRWUsV0FBVyxDQUN6QixPQUFpQyxFQUNqQyxNQUFpQjtRQUVqQixNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsWUFBWSxFQUFFLENBQUM7UUFDakMsTUFBTSxHQUFHLEdBQUcsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDLGdCQUFnQixDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBQ3pELE1BQU0sTUFBTSxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLE1BQU0sRUFBRSxDQUFDLENBQUMsS0FBSyxDQUFDLEdBQUcsQ0FBQyxDQUFDLGdCQUFnQixDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBQzNFLE1BQU0sUUFBUSxHQUFHLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxRQUErQixDQUFDO1FBRWhFLE9BQU8sQ0FBQyxTQUFTLEdBQUcsQ0FBQyxDQUFDO1FBQ3RCLE9BQU8sQ0FBQyxXQUFXLEdBQUcsT0FBTyxDQUFDO1FBQzlCLE9BQU8sQ0FBQyxTQUFTLEdBQUcsT0FBTyxDQUFDO1FBRTVCLE1BQU0sVUFBVSxHQUFHLElBQUksTUFBTSxFQUFFLENBQUM7UUFFaEMsa0ZBQWtGO1FBQ2xGLEtBQUssSUFBSSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRyxRQUFRLENBQUMsTUFBTSxFQUFFLENBQUMsRUFBRSxFQUFFO1lBQ3hDLE1BQU0sT0FBTyxHQUFHLFFBQVEsQ0FBQyxDQUFDLENBQUMsQ0FBQztZQUM1QixNQUFNLENBQUMsSUFBSSxFQUFFLFdBQVcsRUFBRSxTQUFTLEVBQUUsRUFBRSxDQUFDLEdBQ3RDLE9BQU8sQ0FBQyxlQUFlLENBQUMsTUFBTSxDQUFDLENBQUM7WUFFbEMsTUFBTSxDQUFDLFVBQVUsRUFBRSxJQUFJLENBQUMsQ0FBQztZQUN6QixJQUFJLE9BQU8sWUFBWSxrQkFBa0IsRUFBRTtnQkFDekMsYUFBYSxDQUFDLFVBQVUsRUFBRSxXQUFXLEVBQUUsU0FBUyxFQUFFLEVBQWEsQ0FBQyxDQUFDO2FBQ2xFO2lCQUFNO2dCQUNMLGdCQUFnQixDQUFDLFVBQVUsRUFBRSxXQUFXLEVBQUUsU0FBUyxDQUFDLENBQUM7YUFDdEQ7U0FDRjtRQUNELE9BQU8sQ0FBQyxNQUFNLENBQUMsVUFBVSxDQUFDLENBQUM7UUFFM0IsS0FBSyxJQUFJLENBQUMsR0FBRyxDQUFDLEVBQUUsQ0FBQyxHQUFHLFFBQVEsQ0FBQyxNQUFNLEVBQUUsQ0FBQyxFQUFFLEVBQUU7WUFDeEMsTUFBTSxPQUFPLEdBQUcsUUFBUSxDQUFDLENBQUMsQ0FBQyxDQUFDO1lBQzVCLE9BQU8sQ0FBQyxTQUFTLEdBQUcsT0FBTyxDQUFDO1lBRTVCLE1BQU0sQ0FBQyxJQUFJLEVBQUUsV0FBVyxFQUFFLFNBQVMsRUFBRSxFQUFFLENBQUMsR0FDdEMsT0FBTyxDQUFDLGVBQWUsQ0FBQyxNQUFNLENBQUMsQ0FBQztZQUVsQyxNQUFNLFVBQVUsR0FBRyxJQUFJLE1BQU0sRUFBRSxDQUFDO1lBRWhDLE9BQU8sQ0FBQyxXQUFXLEdBQUcsR0FBRyxDQUFDO1lBQzFCLHFCQUFxQjtZQUNyQixNQUFNLENBQUMsVUFBVSxFQUFFLElBQUksQ0FBQyxDQUFDO1lBQ3pCLE1BQU0sQ0FBQyxVQUFVLEVBQUUsV0FBVyxDQUFDLENBQUM7WUFFaEMsSUFBSSxPQUFPLFlBQVksa0JBQWtCLEVBQUU7Z0JBQ3pDLHFCQUFxQjtnQkFDckIsTUFBTSxDQUFDLFVBQVUsRUFBRSxTQUFTLENBQUMsQ0FBQztnQkFDOUIsTUFBTSxDQUFDLFVBQVUsRUFBRSxFQUFhLENBQUMsQ0FBQztnQkFDbEMsT0FBTyxDQUFDLFNBQVMsRUFBRSxDQUFDO2dCQUNwQixPQUFPLENBQUMsTUFBTSxDQUFDLFVBQVUsQ0FBQyxDQUFDO2FBQzVCO2lCQUFNO2dCQUNMLHFCQUFxQjtnQkFDckIsTUFBTSxDQUFDLFVBQVUsRUFBRSxTQUFTLENBQUMsQ0FBQztnQkFDOUIsT0FBTyxDQUFDLFNBQVMsRUFBRSxDQUFDO2dCQUNwQixPQUFPLENBQUMsTUFBTSxDQUFDLFVBQVUsQ0FBQyxDQUFDO2FBQzVCO1lBRUQsT0FBTyxDQUFDLFdBQVcsR0FBRyxDQUFDLENBQUM7WUFDeEIsT0FBTyxDQUFDLFNBQVMsR0FBRyxDQUFDLENBQUM7WUFFdEIsOEJBQThCO1lBQzlCLE1BQU0sQ0FBQyxPQUFPLEVBQUUsSUFBSSxDQUFDLENBQUM7WUFDdEIsT0FBTyxDQUFDLFNBQVMsRUFBRSxDQUFDO1lBQ3BCLEdBQUcsQ0FBQyxPQUFPLEVBQUUsSUFBSSxFQUFFLENBQUMsQ0FBQyxDQUFDO1lBQ3RCLE9BQU8sQ0FBQyxTQUFTLEVBQUUsQ0FBQztZQUNwQixPQUFPLENBQUMsTUFBTSxFQUFFLENBQUM7WUFDakIsT0FBTyxDQUFDLElBQUksRUFBRSxDQUFDO1lBRWYsaUVBQWlFO1lBQ2pFLHVFQUF1RTtZQUN2RSxJQUFJLENBQUMsS0FBSyxRQUFRLENBQUMsTUFBTSxHQUFHLENBQUMsRUFBRTtnQkFDN0IsSUFBSSxFQUFFLEtBQUssU0FBUyxFQUFFO29CQUNwQixNQUFNLENBQUMsT0FBTyxFQUFFLEVBQUUsQ0FBQyxDQUFDO29CQUNwQixPQUFPLENBQUMsU0FBUyxFQUFFLENBQUM7b0JBQ3BCLEdBQUcsQ0FBQyxPQUFPLEVBQUUsRUFBRSxFQUFFLENBQUMsQ0FBQyxDQUFDO29CQUNwQixPQUFPLENBQUMsU0FBUyxFQUFFLENBQUM7b0JBQ3BCLE9BQU8sQ0FBQyxNQUFNLEVBQUUsQ0FBQztvQkFDakIsT0FBTyxDQUFDLElBQUksRUFBRSxDQUFDO2lCQUNoQjthQUNGO1lBRUQsMEJBQTBCO1lBQzFCLE9BQU8sQ0FBQyxTQUFTLEdBQUcsT0FBTyxDQUFDO1lBQzVCLEtBQUssTUFBTSxLQUFLLElBQUksQ0FBQyxXQUFXLEVBQUUsU0FBUyxDQUFDLEVBQUU7Z0JBQzVDLElBQUksS0FBSyxDQUFDLFNBQVMsR0FBRyxDQUFDLEVBQUU7b0JBQ3ZCLE1BQU0sQ0FBQyxPQUFPLEVBQUUsS0FBSyxDQUFDLENBQUM7b0JBQ3ZCLE9BQU8sQ0FBQyxTQUFTLEVBQUUsQ0FBQztvQkFDcEIsR0FBRyxDQUFDLE9BQU8sRUFBRSxLQUFLLEVBQUUsQ0FBQyxDQUFDLENBQUM7b0JBQ3ZCLE9BQU8sQ0FBQyxTQUFTLEVBQUUsQ0FBQztvQkFDcEIsT0FBTyxDQUFDLElBQUksRUFBRSxDQUFDO29CQUNmLE9BQU8sQ0FBQyxNQUFNLEVBQUUsQ0FBQztpQkFDbEI7YUFDRjtTQUNGO1FBRUQsT0FBTyxDQUFDLFNBQVMsR0FBRyxDQUFDLENBQUM7UUFDdEIsT0FBTyxDQUFDLFNBQVMsRUFBRSxDQUFDO1FBQ3BCLFNBQVMsQ0FBQyxPQUFPLEVBQUUsTUFBTSxDQUFDLENBQUM7UUFDM0IsT0FBTyxDQUFDLE1BQU0sRUFBRSxDQUFDO1FBRWpCLE9BQU8sQ0FBQyxTQUFTLEVBQUUsQ0FBQztRQUNwQixRQUFRLENBQUMsT0FBTyxFQUFFLEdBQUcsQ0FBQyxDQUFDO1FBQ3ZCLE9BQU8sQ0FBQyxTQUFTLEVBQUUsQ0FBQztRQUNwQixPQUFPLENBQUMsTUFBTSxFQUFFLENBQUM7SUFDbkIsQ0FBQztJQUVPLE1BQU0sQ0FBQyxJQUFVO1FBQ3ZCLE9BQU8sSUFBSSxZQUFZLElBQUksQ0FBQztJQUM5QixDQUFDO0NBQ0Y7QUExTnlCO0lBRnZCLE9BQU8sQ0FBQyxHQUFHLENBQUM7SUFDWixNQUFNLEVBQUU7MENBQ2dEO0FBV2pDO0lBRnZCLE9BQU8sQ0FBQyxJQUFJLENBQUM7SUFDYixNQUFNLEVBQUU7c0NBSVA7QUF1QmM7SUFEZixRQUFRLEVBQUU7cUNBT1Y7QUFHTTtJQUROLFFBQVEsRUFBRTttQ0FvQlY7QUFHUztJQURULFFBQVEsRUFBRTswQ0FNVjtBQTZCTztJQURQLFFBQVEsRUFBRTswQ0FNViJ9