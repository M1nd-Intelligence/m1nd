var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { BBox, } from '@motion-canvas/core';
import { getRectProfile } from '../curves/getRectProfile';
import { computed, initial, nodeName, signal } from '../decorators';
import { spacingSignal } from '../decorators/spacingSignal';
import { drawRoundRect } from '../utils';
import { Curve } from './Curve';
let Rect = class Rect extends Curve {
    constructor(props) {
        super(props);
    }
    profile() {
        return getRectProfile(this.childrenBBox(), this.radius(), this.smoothCorners(), this.cornerSharpness());
    }
    desiredSize() {
        return {
            x: this.width.context.getter(),
            y: this.height.context.getter(),
        };
    }
    offsetComputedLayout(box) {
        return box;
    }
    childrenBBox() {
        return BBox.fromSizeCentered(this.computedSize());
    }
    getPath() {
        if (this.requiresProfile()) {
            return this.curveDrawingInfo().path;
        }
        const path = new Path2D();
        const radius = this.radius();
        const smoothCorners = this.smoothCorners();
        const cornerSharpness = this.cornerSharpness();
        const box = BBox.fromSizeCentered(this.size());
        drawRoundRect(path, box, radius, smoothCorners, cornerSharpness);
        return path;
    }
    getCacheBBox() {
        return super.getCacheBBox().expand(this.rippleSize());
    }
    getRipplePath() {
        const path = new Path2D();
        const rippleSize = this.rippleSize();
        const radius = this.radius().addScalar(rippleSize);
        const smoothCorners = this.smoothCorners();
        const cornerSharpness = this.cornerSharpness();
        const box = BBox.fromSizeCentered(this.size()).expand(rippleSize);
        drawRoundRect(path, box, radius, smoothCorners, cornerSharpness);
        return path;
    }
};
__decorate([
    spacingSignal('radius')
], Rect.prototype, "radius", void 0);
__decorate([
    initial(false),
    signal()
], Rect.prototype, "smoothCorners", void 0);
__decorate([
    initial(0.6),
    signal()
], Rect.prototype, "cornerSharpness", void 0);
__decorate([
    computed()
], Rect.prototype, "profile", null);
Rect = __decorate([
    nodeName('Rect')
], Rect);
export { Rect };
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUmVjdC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29tcG9uZW50cy9SZWN0LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7OztBQUFBLE9BQU8sRUFDTCxJQUFJLEdBTUwsTUFBTSxxQkFBcUIsQ0FBQztBQUM3QixPQUFPLEVBQUMsY0FBYyxFQUFDLE1BQU0sMEJBQTBCLENBQUM7QUFDeEQsT0FBTyxFQUFDLFFBQVEsRUFBRSxPQUFPLEVBQUUsUUFBUSxFQUFFLE1BQU0sRUFBQyxNQUFNLGVBQWUsQ0FBQztBQUNsRSxPQUFPLEVBQUMsYUFBYSxFQUFDLE1BQU0sNkJBQTZCLENBQUM7QUFFMUQsT0FBTyxFQUFDLGFBQWEsRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUN2QyxPQUFPLEVBQUMsS0FBSyxFQUFhLE1BQU0sU0FBUyxDQUFDO0FBb0JuQyxJQUFNLElBQUksR0FBVixNQUFNLElBQUssU0FBUSxLQUFLO0lBNkY3QixZQUFtQixLQUFnQjtRQUNqQyxLQUFLLENBQUMsS0FBSyxDQUFDLENBQUM7SUFDZixDQUFDO0lBR00sT0FBTztRQUNaLE9BQU8sY0FBYyxDQUNuQixJQUFJLENBQUMsWUFBWSxFQUFFLEVBQ25CLElBQUksQ0FBQyxNQUFNLEVBQUUsRUFDYixJQUFJLENBQUMsYUFBYSxFQUFFLEVBQ3BCLElBQUksQ0FBQyxlQUFlLEVBQUUsQ0FDdkIsQ0FBQztJQUNKLENBQUM7SUFFa0IsV0FBVztRQUM1QixPQUFPO1lBQ0wsQ0FBQyxFQUFFLElBQUksQ0FBQyxLQUFLLENBQUMsT0FBTyxDQUFDLE1BQU0sRUFBRTtZQUM5QixDQUFDLEVBQUUsSUFBSSxDQUFDLE1BQU0sQ0FBQyxPQUFPLENBQUMsTUFBTSxFQUFFO1NBQ2hDLENBQUM7SUFDSixDQUFDO0lBRWtCLG9CQUFvQixDQUFDLEdBQVM7UUFDL0MsT0FBTyxHQUFHLENBQUM7SUFDYixDQUFDO0lBRWtCLFlBQVk7UUFDN0IsT0FBTyxJQUFJLENBQUMsZ0JBQWdCLENBQUMsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDLENBQUM7SUFDcEQsQ0FBQztJQUVrQixPQUFPO1FBQ3hCLElBQUksSUFBSSxDQUFDLGVBQWUsRUFBRSxFQUFFO1lBQzFCLE9BQU8sSUFBSSxDQUFDLGdCQUFnQixFQUFFLENBQUMsSUFBSSxDQUFDO1NBQ3JDO1FBRUQsTUFBTSxJQUFJLEdBQUcsSUFBSSxNQUFNLEVBQUUsQ0FBQztRQUMxQixNQUFNLE1BQU0sR0FBRyxJQUFJLENBQUMsTUFBTSxFQUFFLENBQUM7UUFDN0IsTUFBTSxhQUFhLEdBQUcsSUFBSSxDQUFDLGFBQWEsRUFBRSxDQUFDO1FBQzNDLE1BQU0sZUFBZSxHQUFHLElBQUksQ0FBQyxlQUFlLEVBQUUsQ0FBQztRQUMvQyxNQUFNLEdBQUcsR0FBRyxJQUFJLENBQUMsZ0JBQWdCLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDLENBQUM7UUFDL0MsYUFBYSxDQUFDLElBQUksRUFBRSxHQUFHLEVBQUUsTUFBTSxFQUFFLGFBQWEsRUFBRSxlQUFlLENBQUMsQ0FBQztRQUVqRSxPQUFPLElBQUksQ0FBQztJQUNkLENBQUM7SUFFa0IsWUFBWTtRQUM3QixPQUFPLEtBQUssQ0FBQyxZQUFZLEVBQUUsQ0FBQyxNQUFNLENBQUMsSUFBSSxDQUFDLFVBQVUsRUFBRSxDQUFDLENBQUM7SUFDeEQsQ0FBQztJQUVrQixhQUFhO1FBQzlCLE1BQU0sSUFBSSxHQUFHLElBQUksTUFBTSxFQUFFLENBQUM7UUFDMUIsTUFBTSxVQUFVLEdBQUcsSUFBSSxDQUFDLFVBQVUsRUFBRSxDQUFDO1FBQ3JDLE1BQU0sTUFBTSxHQUFHLElBQUksQ0FBQyxNQUFNLEVBQUUsQ0FBQyxTQUFTLENBQUMsVUFBVSxDQUFDLENBQUM7UUFDbkQsTUFBTSxhQUFhLEdBQUcsSUFBSSxDQUFDLGFBQWEsRUFBRSxDQUFDO1FBQzNDLE1BQU0sZUFBZSxHQUFHLElBQUksQ0FBQyxlQUFlLEVBQUUsQ0FBQztRQUMvQyxNQUFNLEdBQUcsR0FBRyxJQUFJLENBQUMsZ0JBQWdCLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDLENBQUMsTUFBTSxDQUFDLFVBQVUsQ0FBQyxDQUFDO1FBQ2xFLGFBQWEsQ0FBQyxJQUFJLEVBQUUsR0FBRyxFQUFFLE1BQU0sRUFBRSxhQUFhLEVBQUUsZUFBZSxDQUFDLENBQUM7UUFFakUsT0FBTyxJQUFJLENBQUM7SUFDZCxDQUFDO0NBQ0YsQ0FBQTtBQWhIeUI7SUFEdkIsYUFBYSxDQUFDLFFBQVEsQ0FBQztvQ0FDNEI7QUEyQjVCO0lBRnZCLE9BQU8sQ0FBQyxLQUFLLENBQUM7SUFDZCxNQUFNLEVBQUU7MkNBQzBEO0FBd0IzQztJQUZ2QixPQUFPLENBQUMsR0FBRyxDQUFDO0lBQ1osTUFBTSxFQUFFOzZDQUMyRDtBQU83RDtJQUROLFFBQVEsRUFBRTttQ0FRVjtBQXpHVSxJQUFJO0lBRGhCLFFBQVEsQ0FBQyxNQUFNLENBQUM7R0FDSixJQUFJLENBd0poQiJ9