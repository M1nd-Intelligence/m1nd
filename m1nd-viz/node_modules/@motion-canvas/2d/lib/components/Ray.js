var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { BBox, } from '@motion-canvas/core';
import { LineSegment } from '../curves';
import { nodeName, vector2Signal } from '../decorators';
import { arc, drawLine, drawPivot } from '../utils';
import { Curve } from './Curve';
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
let Ray = class Ray extends Curve {
    constructor(props) {
        super(props);
    }
    childrenBBox() {
        return BBox.fromPoints(this.from(), this.to());
    }
    profile() {
        const segment = new LineSegment(this.from(), this.to());
        return {
            arcLength: segment.arcLength,
            minSin: 1,
            segments: [segment],
        };
    }
    drawOverlay(context, matrix) {
        const box = this.childrenBBox().transformCorners(matrix);
        const size = this.computedSize();
        const offset = size.mul(this.offset()).scale(0.5).transformAsPoint(matrix);
        const from = this.from().transformAsPoint(matrix);
        const to = this.to().transformAsPoint(matrix);
        context.fillStyle = 'white';
        context.strokeStyle = 'black';
        context.lineWidth = 1;
        context.beginPath();
        arc(context, from, 4);
        context.fill();
        context.stroke();
        context.beginPath();
        arc(context, to, 4);
        context.fill();
        context.stroke();
        context.strokeStyle = 'white';
        context.beginPath();
        drawLine(context, [from, to]);
        context.stroke();
        context.beginPath();
        drawPivot(context, offset);
        context.stroke();
        context.beginPath();
        drawLine(context, box);
        context.closePath();
        context.stroke();
    }
};
__decorate([
    vector2Signal('from')
], Ray.prototype, "from", void 0);
__decorate([
    vector2Signal('to')
], Ray.prototype, "to", void 0);
Ray = __decorate([
    nodeName('Ray')
], Ray);
export { Ray };
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUmF5LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL1JheS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7QUFBQSxPQUFPLEVBQ0wsSUFBSSxHQUlMLE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUFlLFdBQVcsRUFBQyxNQUFNLFdBQVcsQ0FBQztBQUNwRCxPQUFPLEVBQUMsUUFBUSxFQUFFLGFBQWEsRUFBQyxNQUFNLGVBQWUsQ0FBQztBQUN0RCxPQUFPLEVBQUMsR0FBRyxFQUFFLFFBQVEsRUFBRSxTQUFTLEVBQUMsTUFBTSxVQUFVLENBQUM7QUFDbEQsT0FBTyxFQUFDLEtBQUssRUFBYSxNQUFNLFNBQVMsQ0FBQztBQWtCMUM7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztHQTJCRztBQUVJLElBQU0sR0FBRyxHQUFULE1BQU0sR0FBSSxTQUFRLEtBQUs7SUFhNUIsWUFBbUIsS0FBZTtRQUNoQyxLQUFLLENBQUMsS0FBSyxDQUFDLENBQUM7SUFDZixDQUFDO0lBRWtCLFlBQVk7UUFDN0IsT0FBTyxJQUFJLENBQUMsVUFBVSxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsRUFBRSxJQUFJLENBQUMsRUFBRSxFQUFFLENBQUMsQ0FBQztJQUNqRCxDQUFDO0lBRWUsT0FBTztRQUNyQixNQUFNLE9BQU8sR0FBRyxJQUFJLFdBQVcsQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLEVBQUUsSUFBSSxDQUFDLEVBQUUsRUFBRSxDQUFDLENBQUM7UUFFeEQsT0FBTztZQUNMLFNBQVMsRUFBRSxPQUFPLENBQUMsU0FBUztZQUM1QixNQUFNLEVBQUUsQ0FBQztZQUNULFFBQVEsRUFBRSxDQUFDLE9BQU8sQ0FBQztTQUNwQixDQUFDO0lBQ0osQ0FBQztJQUVlLFdBQVcsQ0FDekIsT0FBaUMsRUFDakMsTUFBaUI7UUFFakIsTUFBTSxHQUFHLEdBQUcsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDLGdCQUFnQixDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBQ3pELE1BQU0sSUFBSSxHQUFHLElBQUksQ0FBQyxZQUFZLEVBQUUsQ0FBQztRQUNqQyxNQUFNLE1BQU0sR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxNQUFNLEVBQUUsQ0FBQyxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsQ0FBQyxnQkFBZ0IsQ0FBQyxNQUFNLENBQUMsQ0FBQztRQUMzRSxNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsSUFBSSxFQUFFLENBQUMsZ0JBQWdCLENBQUMsTUFBTSxDQUFDLENBQUM7UUFDbEQsTUFBTSxFQUFFLEdBQUcsSUFBSSxDQUFDLEVBQUUsRUFBRSxDQUFDLGdCQUFnQixDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBRTlDLE9BQU8sQ0FBQyxTQUFTLEdBQUcsT0FBTyxDQUFDO1FBQzVCLE9BQU8sQ0FBQyxXQUFXLEdBQUcsT0FBTyxDQUFDO1FBQzlCLE9BQU8sQ0FBQyxTQUFTLEdBQUcsQ0FBQyxDQUFDO1FBRXRCLE9BQU8sQ0FBQyxTQUFTLEVBQUUsQ0FBQztRQUNwQixHQUFHLENBQUMsT0FBTyxFQUFFLElBQUksRUFBRSxDQUFDLENBQUMsQ0FBQztRQUN0QixPQUFPLENBQUMsSUFBSSxFQUFFLENBQUM7UUFDZixPQUFPLENBQUMsTUFBTSxFQUFFLENBQUM7UUFFakIsT0FBTyxDQUFDLFNBQVMsRUFBRSxDQUFDO1FBQ3BCLEdBQUcsQ0FBQyxPQUFPLEVBQUUsRUFBRSxFQUFFLENBQUMsQ0FBQyxDQUFDO1FBQ3BCLE9BQU8sQ0FBQyxJQUFJLEVBQUUsQ0FBQztRQUNmLE9BQU8sQ0FBQyxNQUFNLEVBQUUsQ0FBQztRQUVqQixPQUFPLENBQUMsV0FBVyxHQUFHLE9BQU8sQ0FBQztRQUM5QixPQUFPLENBQUMsU0FBUyxFQUFFLENBQUM7UUFDcEIsUUFBUSxDQUFDLE9BQU8sRUFBRSxDQUFDLElBQUksRUFBRSxFQUFFLENBQUMsQ0FBQyxDQUFDO1FBQzlCLE9BQU8sQ0FBQyxNQUFNLEVBQUUsQ0FBQztRQUVqQixPQUFPLENBQUMsU0FBUyxFQUFFLENBQUM7UUFDcEIsU0FBUyxDQUFDLE9BQU8sRUFBRSxNQUFNLENBQUMsQ0FBQztRQUMzQixPQUFPLENBQUMsTUFBTSxFQUFFLENBQUM7UUFFakIsT0FBTyxDQUFDLFNBQVMsRUFBRSxDQUFDO1FBQ3BCLFFBQVEsQ0FBQyxPQUFPLEVBQUUsR0FBRyxDQUFDLENBQUM7UUFDdkIsT0FBTyxDQUFDLFNBQVMsRUFBRSxDQUFDO1FBQ3BCLE9BQU8sQ0FBQyxNQUFNLEVBQUUsQ0FBQztJQUNuQixDQUFDO0NBQ0YsQ0FBQTtBQWhFeUI7SUFEdkIsYUFBYSxDQUFDLE1BQU0sQ0FBQztpQ0FDNEI7QUFNMUI7SUFEdkIsYUFBYSxDQUFDLElBQUksQ0FBQzsrQkFDNEI7QUFYckMsR0FBRztJQURmLFFBQVEsQ0FBQyxLQUFLLENBQUM7R0FDSCxHQUFHLENBcUVmIn0=