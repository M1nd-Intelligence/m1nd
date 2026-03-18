var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { QuadBezierSegment } from '../curves';
import { computed, vector2Signal } from '../decorators';
import { lineTo, moveTo, quadraticCurveTo } from '../utils';
import { Bezier } from './Bezier';
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
export class QuadBezier extends Bezier {
    constructor(props) {
        super(props);
    }
    segment() {
        return new QuadBezierSegment(this.p0(), this.p1(), this.p2());
    }
    overlayInfo(matrix) {
        const [p0, p1, p2] = this.segment().transformPoints(matrix);
        const curvePath = new Path2D();
        moveTo(curvePath, p0);
        quadraticCurveTo(curvePath, p1, p2);
        const handleLinesPath = new Path2D();
        moveTo(handleLinesPath, p0);
        lineTo(handleLinesPath, p1);
        lineTo(handleLinesPath, p2);
        return {
            curve: curvePath,
            startPoint: p0,
            endPoint: p2,
            controlPoints: [p1],
            handleLines: handleLinesPath,
        };
    }
}
__decorate([
    vector2Signal('p0')
], QuadBezier.prototype, "p0", void 0);
__decorate([
    vector2Signal('p1')
], QuadBezier.prototype, "p1", void 0);
__decorate([
    vector2Signal('p2')
], QuadBezier.prototype, "p2", void 0);
__decorate([
    computed()
], QuadBezier.prototype, "segment", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUXVhZEJlemllci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29tcG9uZW50cy9RdWFkQmV6aWVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7OztBQUNBLE9BQU8sRUFBQyxpQkFBaUIsRUFBQyxNQUFNLFdBQVcsQ0FBQztBQUU1QyxPQUFPLEVBQUMsUUFBUSxFQUFFLGFBQWEsRUFBQyxNQUFNLGVBQWUsQ0FBQztBQUN0RCxPQUFPLEVBQUMsTUFBTSxFQUFFLE1BQU0sRUFBRSxnQkFBZ0IsRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUMxRCxPQUFPLEVBQUMsTUFBTSxFQUFvQixNQUFNLFVBQVUsQ0FBQztBQWlCbkQ7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztHQTJCRztBQUNILE1BQU0sT0FBTyxVQUFXLFNBQVEsTUFBTTtJQW1CcEMsWUFBbUIsS0FBc0I7UUFDdkMsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFDO0lBQ2YsQ0FBQztJQUdTLE9BQU87UUFDZixPQUFPLElBQUksaUJBQWlCLENBQUMsSUFBSSxDQUFDLEVBQUUsRUFBRSxFQUFFLElBQUksQ0FBQyxFQUFFLEVBQUUsRUFBRSxJQUFJLENBQUMsRUFBRSxFQUFFLENBQUMsQ0FBQztJQUNoRSxDQUFDO0lBRVMsV0FBVyxDQUFDLE1BQWlCO1FBQ3JDLE1BQU0sQ0FBQyxFQUFFLEVBQUUsRUFBRSxFQUFFLEVBQUUsQ0FBQyxHQUFHLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxlQUFlLENBQUMsTUFBTSxDQUFDLENBQUM7UUFFNUQsTUFBTSxTQUFTLEdBQUcsSUFBSSxNQUFNLEVBQUUsQ0FBQztRQUMvQixNQUFNLENBQUMsU0FBUyxFQUFFLEVBQUUsQ0FBQyxDQUFDO1FBQ3RCLGdCQUFnQixDQUFDLFNBQVMsRUFBRSxFQUFFLEVBQUUsRUFBRSxDQUFDLENBQUM7UUFFcEMsTUFBTSxlQUFlLEdBQUcsSUFBSSxNQUFNLEVBQUUsQ0FBQztRQUNyQyxNQUFNLENBQUMsZUFBZSxFQUFFLEVBQUUsQ0FBQyxDQUFDO1FBQzVCLE1BQU0sQ0FBQyxlQUFlLEVBQUUsRUFBRSxDQUFDLENBQUM7UUFDNUIsTUFBTSxDQUFDLGVBQWUsRUFBRSxFQUFFLENBQUMsQ0FBQztRQUU1QixPQUFPO1lBQ0wsS0FBSyxFQUFFLFNBQVM7WUFDaEIsVUFBVSxFQUFFLEVBQUU7WUFDZCxRQUFRLEVBQUUsRUFBRTtZQUNaLGFBQWEsRUFBRSxDQUFDLEVBQUUsQ0FBQztZQUNuQixXQUFXLEVBQUUsZUFBZTtTQUM3QixDQUFDO0lBQ0osQ0FBQztDQUNGO0FBM0N5QjtJQUR2QixhQUFhLENBQUMsSUFBSSxDQUFDO3NDQUM0QjtBQU14QjtJQUR2QixhQUFhLENBQUMsSUFBSSxDQUFDO3NDQUM0QjtBQU14QjtJQUR2QixhQUFhLENBQUMsSUFBSSxDQUFDO3NDQUM0QjtBQU90QztJQURULFFBQVEsRUFBRTt5Q0FHViJ9