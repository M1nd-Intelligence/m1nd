var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { BBox, createSignal, isReactive, threadable, tween, } from '@motion-canvas/core';
import { createCurveProfileLerp } from '../curves/createCurveProfileLerp';
import { getPathProfile } from '../curves/getPathProfile';
import { computed, signal } from '../decorators';
import { drawLine, drawPivot } from '../utils';
import { Curve } from './Curve';
export class Path extends Curve {
    constructor(props) {
        super(props);
        this.currentProfile = createSignal(null);
        this.canHaveSubpath = true;
    }
    profile() {
        return this.currentProfile() ?? getPathProfile(this.data());
    }
    childrenBBox() {
        const points = this.profile().segments.flatMap(segment => segment.points);
        return BBox.fromPoints(...points);
    }
    lineWidthCoefficient() {
        const join = this.lineJoin();
        let coefficient = super.lineWidthCoefficient();
        if (join === 'miter') {
            const { minSin } = this.profile();
            if (minSin > 0) {
                coefficient = Math.max(coefficient, 0.5 / minSin);
            }
        }
        return coefficient;
    }
    processSubpath(path, startPoint, endPoint) {
        if (startPoint && endPoint && startPoint.equals(endPoint)) {
            path.closePath();
        }
    }
    *tweenData(newPath, time, timingFunction) {
        const fromProfile = this.profile();
        const toProfile = getPathProfile(isReactive(newPath) ? newPath() : newPath);
        const interpolator = createCurveProfileLerp(fromProfile, toProfile);
        this.currentProfile(fromProfile);
        yield* tween(time, value => {
            const progress = timingFunction(value);
            this.currentProfile(interpolator(progress));
        }, () => {
            this.currentProfile(null);
            this.data(newPath);
        });
    }
    drawOverlay(context, matrix) {
        const box = this.childrenBBox().transformCorners(matrix);
        const size = this.computedSize();
        const offset = size.mul(this.offset()).scale(0.5).transformAsPoint(matrix);
        const segments = this.profile().segments;
        context.lineWidth = 1;
        context.strokeStyle = 'white';
        context.fillStyle = 'white';
        context.save();
        context.setTransform(matrix);
        let endPoint = null;
        let path = new Path2D();
        for (const segment of segments) {
            if (endPoint && !segment.getPoint(0).position.equals(endPoint)) {
                context.stroke(path);
                path = new Path2D();
                endPoint = null;
            }
            const [, end] = segment.draw(path, 0, 1, endPoint == null);
            endPoint = end.position;
        }
        context.stroke(path);
        context.restore();
        context.beginPath();
        drawPivot(context, offset);
        context.stroke();
        context.beginPath();
        drawLine(context, box);
        context.closePath();
        context.stroke();
    }
}
__decorate([
    signal()
], Path.prototype, "data", void 0);
__decorate([
    computed()
], Path.prototype, "profile", null);
__decorate([
    threadable()
], Path.prototype, "tweenData", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUGF0aC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29tcG9uZW50cy9QYXRoLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7OztBQUFBLE9BQU8sRUFDTCxJQUFJLEVBQ0osWUFBWSxFQUNaLFVBQVUsRUFHVixVQUFVLEVBRVYsS0FBSyxHQUVOLE1BQU0scUJBQXFCLENBQUM7QUFFN0IsT0FBTyxFQUFDLHNCQUFzQixFQUFDLE1BQU0sa0NBQWtDLENBQUM7QUFDeEUsT0FBTyxFQUFDLGNBQWMsRUFBQyxNQUFNLDBCQUEwQixDQUFDO0FBQ3hELE9BQU8sRUFBQyxRQUFRLEVBQUUsTUFBTSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBQy9DLE9BQU8sRUFBQyxRQUFRLEVBQUUsU0FBUyxFQUFDLE1BQU0sVUFBVSxDQUFDO0FBQzdDLE9BQU8sRUFBQyxLQUFLLEVBQWEsTUFBTSxTQUFTLENBQUM7QUFNMUMsTUFBTSxPQUFPLElBQUssU0FBUSxLQUFLO0lBSzdCLFlBQW1CLEtBQWdCO1FBQ2pDLEtBQUssQ0FBQyxLQUFLLENBQUMsQ0FBQztRQUxQLG1CQUFjLEdBQUcsWUFBWSxDQUFzQixJQUFJLENBQUMsQ0FBQztRQU0vRCxJQUFJLENBQUMsY0FBYyxHQUFHLElBQUksQ0FBQztJQUM3QixDQUFDO0lBR2UsT0FBTztRQUNyQixPQUFPLElBQUksQ0FBQyxjQUFjLEVBQUUsSUFBSSxjQUFjLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDLENBQUM7SUFDOUQsQ0FBQztJQUVrQixZQUFZO1FBQzdCLE1BQU0sTUFBTSxHQUFHLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxRQUFRLENBQUMsT0FBTyxDQUFDLE9BQU8sQ0FBQyxFQUFFLENBQUMsT0FBTyxDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBQzFFLE9BQU8sSUFBSSxDQUFDLFVBQVUsQ0FBQyxHQUFHLE1BQU0sQ0FBQyxDQUFDO0lBQ3BDLENBQUM7SUFFa0Isb0JBQW9CO1FB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