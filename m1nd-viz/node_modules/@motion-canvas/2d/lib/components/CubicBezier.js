var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { CubicBezierSegment } from '../curves';
import { computed, vector2Signal } from '../decorators';
import { bezierCurveTo, lineTo, moveTo } from '../utils';
import { Bezier } from './Bezier';
/**
 * A node for drawing a cubic Bézier curve.
 *
 * @preview
 * ```tsx editor
 * import {makeScene2D, CubicBezier} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const bezier = createRef<CubicBezier>();
 *
 *   view.add(
 *     <CubicBezier
 *       ref={bezier}
 *       lineWidth={4}
 *       stroke={'lightseagreen'}
 *       p0={[-200, -100]}
 *       p1={[100, -100]}
 *       p2={[-100, 100]}
 *       p3={[200, 100]}
 *       end={0}
 *     />
 *   );
 *
 *   yield* bezier().end(1, 1);
 *   yield* bezier().start(1, 1).to(0, 1);
 * });
 * ```
 */
export class CubicBezier extends Bezier {
    constructor(props) {
        super(props);
    }
    segment() {
        return new CubicBezierSegment(this.p0(), this.p1(), this.p2(), this.p3());
    }
    overlayInfo(matrix) {
        const [p0, p1, p2, p3] = this.segment().transformPoints(matrix);
        const curvePath = new Path2D();
        moveTo(curvePath, p0);
        bezierCurveTo(curvePath, p1, p2, p3);
        const handleLinesPath = new Path2D();
        moveTo(handleLinesPath, p0);
        lineTo(handleLinesPath, p1);
        moveTo(handleLinesPath, p2);
        lineTo(handleLinesPath, p3);
        return {
            curve: curvePath,
            startPoint: p0,
            endPoint: p3,
            controlPoints: [p1, p2],
            handleLines: handleLinesPath,
        };
    }
}
__decorate([
    vector2Signal('p0')
], CubicBezier.prototype, "p0", void 0);
__decorate([
    vector2Signal('p1')
], CubicBezier.prototype, "p1", void 0);
__decorate([
    vector2Signal('p2')
], CubicBezier.prototype, "p2", void 0);
__decorate([
    vector2Signal('p3')
], CubicBezier.prototype, "p3", void 0);
__decorate([
    computed()
], CubicBezier.prototype, "segment", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ3ViaWNCZXppZXIuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2NvbXBvbmVudHMvQ3ViaWNCZXppZXIudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7Ozs7O0FBQ0EsT0FBTyxFQUFDLGtCQUFrQixFQUFDLE1BQU0sV0FBVyxDQUFDO0FBRTdDLE9BQU8sRUFBQyxRQUFRLEVBQUUsYUFBYSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBQ3RELE9BQU8sRUFBQyxhQUFhLEVBQUUsTUFBTSxFQUFFLE1BQU0sRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUN2RCxPQUFPLEVBQUMsTUFBTSxFQUFvQixNQUFNLFVBQVUsQ0FBQztBQXFCbkQ7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7R0E0Qkc7QUFDSCxNQUFNLE9BQU8sV0FBWSxTQUFRLE1BQU07SUF5QnJDLFlBQW1CLEtBQXVCO1FBQ3hDLEtBQUssQ0FBQyxLQUFLLENBQUMsQ0FBQztJQUNmLENBQUM7SUFHUyxPQUFPO1FBQ2YsT0FBTyxJQUFJLGtCQUFrQixDQUFDLElBQUksQ0FBQyxFQUFFLEVBQUUsRUFBRSxJQUFJLENBQUMsRUFBRSxFQUFFLEVBQUUsSUFBSSxDQUFDLEVBQUUsRUFBRSxFQUFFLElBQUksQ0FBQyxFQUFFLEVBQUUsQ0FBQyxDQUFDO0lBQzVFLENBQUM7SUFFUyxXQUFXLENBQUMsTUFBaUI7UUFDckMsTUFBTSxDQUFDLEVBQUUsRUFBRSxFQUFFLEVBQUUsRUFBRSxFQUFFLEVBQUUsQ0FBQyxHQUFHLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxlQUFlLENBQUMsTUFBTSxDQUFDLENBQUM7UUFFaEUsTUFBTSxTQUFTLEdBQUcsSUFBSSxNQUFNLEVBQUUsQ0FBQztRQUMvQixNQUFNLENBQUMsU0FBUyxFQUFFLEVBQUUsQ0FBQyxDQUFDO1FBQ3RCLGFBQWEsQ0FBQyxTQUFTLEVBQUUsRUFBRSxFQUFFLEVBQUUsRUFBRSxFQUFFLENBQUMsQ0FBQztRQUVyQyxNQUFNLGVBQWUsR0FBRyxJQUFJLE1BQU0sRUFBRSxDQUFDO1FBQ3JDLE1BQU0sQ0FBQyxlQUFlLEVBQUUsRUFBRSxDQUFDLENBQUM7UUFDNUIsTUFBTSxDQUFDLGVBQWUsRUFBRSxFQUFFLENBQUMsQ0FBQztRQUM1QixNQUFNLENBQUMsZUFBZSxFQUFFLEVBQUUsQ0FBQyxDQUFDO1FBQzVCLE1BQU0sQ0FBQyxlQUFlLEVBQUUsRUFBRSxDQUFDLENBQUM7UUFFNUIsT0FBTztZQUNMLEtBQUssRUFBRSxTQUFTO1lBQ2hCLFVBQVUsRUFBRSxFQUFFO1lBQ2QsUUFBUSxFQUFFLEVBQUU7WUFDWixhQUFhLEVBQUUsQ0FBQyxFQUFFLEVBQUUsRUFBRSxDQUFDO1lBQ3ZCLFdBQVcsRUFBRSxlQUFlO1NBQzdCLENBQUM7SUFDSixDQUFDO0NBQ0Y7QUFsRHlCO0lBRHZCLGFBQWEsQ0FBQyxJQUFJLENBQUM7dUNBQzRCO0FBTXhCO0lBRHZCLGFBQWEsQ0FBQyxJQUFJLENBQUM7dUNBQzRCO0FBTXhCO0lBRHZCLGFBQWEsQ0FBQyxJQUFJLENBQUM7dUNBQzRCO0FBTXhCO0lBRHZCLGFBQWEsQ0FBQyxJQUFJLENBQUM7dUNBQzRCO0FBT3RDO0lBRFQsUUFBUSxFQUFFOzBDQUdWIn0=