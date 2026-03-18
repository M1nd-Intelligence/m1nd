var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { Vector2, clamp, } from '@motion-canvas/core';
import { getPointAtDistance } from '../curves/getPointAtDistance';
import { computed, initial, nodeName, signal } from '../decorators';
import { lineTo, moveTo, resolveCanvasStyle } from '../utils';
import { Shape } from './Shape';
let Curve = class Curve extends Shape {
    desiredSize() {
        return this.childrenBBox().size;
    }
    constructor(props) {
        super(props);
        this.canHaveSubpath = false;
    }
    /**
     * Convert a percentage along the curve to a distance.
     *
     * @remarks
     * The returned distance is given in relation to the full curve, not
     * accounting for {@link startOffset} and {@link endOffset}.
     *
     * @param value - The percentage along the curve.
     */
    percentageToDistance(value) {
        return clamp(0, this.baseArcLength(), this.startOffset() + this.offsetArcLength() * value);
    }
    /**
     * Convert a distance along the curve to a percentage.
     *
     * @remarks
     * The distance should be given in relation to the full curve, not
     * accounting for {@link startOffset} and {@link endOffset}.
     *
     * @param value - The distance along the curve.
     */
    distanceToPercentage(value) {
        return (value - this.startOffset()) / this.offsetArcLength();
    }
    /**
     * The base arc length of this curve.
     *
     * @remarks
     * This is the entire length of this curve, not accounting for
     * {@link startOffset | the offsets}.
     */
    baseArcLength() {
        return this.profile().arcLength;
    }
    /**
     * The offset arc length of this curve.
     *
     * @remarks
     * This is the length of the curve that accounts for
     * {@link startOffset | the offsets}.
     */
    offsetArcLength() {
        const startOffset = this.startOffset();
        const endOffset = this.endOffset();
        const baseLength = this.baseArcLength();
        return clamp(0, baseLength, baseLength - startOffset - endOffset);
    }
    /**
     * The visible arc length of this curve.
     *
     * @remarks
     * This arc length accounts for both the offset and the {@link start} and
     * {@link end} properties.
     */
    arcLength() {
        return this.offsetArcLength() * Math.abs(this.start() - this.end());
    }
    /**
     * The percentage of the curve that's currently visible.
     *
     * @remarks
     * The returned value is the ratio between the visible length (as defined by
     * {@link start} and {@link end}) and the offset length of the curve.
     */
    completion() {
        return Math.abs(this.start() - this.end());
    }
    processSubpath(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _path, 
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _startPoint, 
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _endPoint) {
        // do nothing
    }
    curveDrawingInfo() {
        const path = new Path2D();
        let subpath = new Path2D();
        const profile = this.profile();
        let start = this.percentageToDistance(this.start());
        let end = this.percentageToDistance(this.end());
        if (start > end) {
            [start, end] = [end, start];
        }
        const distance = end - start;
        const arrowSize = Math.min(distance / 2, this.arrowSize());
        if (this.startArrow()) {
            start += arrowSize / 2;
        }
        if (this.endArrow()) {
            end -= arrowSize / 2;
        }
        let length = 0;
        let startPoint = null;
        let startTangent = null;
        let endPoint = null;
        let endTangent = null;
        for (const segment of profile.segments) {
            const previousLength = length;
            length += segment.arcLength;
            if (length < start) {
                continue;
            }
            const relativeStart = (start - previousLength) / segment.arcLength;
            const relativeEnd = (end - previousLength) / segment.arcLength;
            const clampedStart = clamp(0, 1, relativeStart);
            const clampedEnd = clamp(0, 1, relativeEnd);
            if (this.canHaveSubpath &&
                endPoint &&
                !segment.getPoint(0).position.equals(endPoint)) {
                path.addPath(subpath);
                this.processSubpath(subpath, startPoint, endPoint);
                subpath = new Path2D();
                startPoint = null;
            }
            const [startCurvePoint, endCurvePoint] = segment.draw(subpath, clampedStart, clampedEnd, startPoint === null);
            if (startPoint === null) {
                startPoint = startCurvePoint.position;
                startTangent = startCurvePoint.normal.flipped.perpendicular;
            }
            endPoint = endCurvePoint.position;
            endTangent = endCurvePoint.normal.flipped.perpendicular;
            if (length > end) {
                break;
            }
        }
        if (this.closed() &&
            this.start.isInitial() &&
            this.end.isInitial() &&
            this.startOffset.isInitial() &&
            this.endOffset.isInitial()) {
            subpath.closePath();
        }
        this.processSubpath(subpath, startPoint, endPoint);
        path.addPath(subpath);
        return {
            startPoint: startPoint ?? Vector2.zero,
            startTangent: startTangent ?? Vector2.right,
            endPoint: endPoint ?? Vector2.zero,
            endTangent: endTangent ?? Vector2.right,
            arrowSize,
            path,
            startOffset: start,
        };
    }
    getPointAtDistance(value) {
        return getPointAtDistance(this.profile(), value + this.startOffset());
    }
    getPointAtPercentage(value) {
        return getPointAtDistance(this.profile(), this.percentageToDistance(value));
    }
    getComputedLayout() {
        return this.offsetComputedLayout(super.getComputedLayout());
    }
    offsetComputedLayout(box) {
        box.position = box.position.sub(this.childrenBBox().center);
        return box;
    }
    getPath() {
        return this.curveDrawingInfo().path;
    }
    getCacheBBox() {
        const box = this.childrenBBox();
        const arrowSize = this.startArrow() || this.endArrow() ? this.arrowSize() : 0;
        const lineWidth = this.lineWidth();
        const coefficient = this.lineWidthCoefficient();
        return box.expand(Math.max(0, arrowSize, lineWidth * coefficient));
    }
    lineWidthCoefficient() {
        return this.lineCap() === 'square' ? 0.5 * 1.4143 : 0.5;
    }
    /**
     * Check if the path requires a profile.
     *
     * @remarks
     * The profile is only required if certain features are used. Otherwise, the
     * profile generation can be skipped, and the curve can be drawn directly
     * using the 2D context.
     */
    requiresProfile() {
        return (!this.start.isInitial() ||
            !this.startOffset.isInitial() ||
            !this.startArrow.isInitial() ||
            !this.end.isInitial() ||
            !this.endOffset.isInitial() ||
            !this.endArrow.isInitial());
    }
    drawShape(context) {
        super.drawShape(context);
        if (this.startArrow() || this.endArrow()) {
            this.drawArrows(context);
        }
    }
    drawArrows(context) {
        const { startPoint, startTangent, endPoint, endTangent, arrowSize } = this.curveDrawingInfo();
        if (arrowSize < 0.001) {
            return;
        }
        context.save();
        context.beginPath();
        if (this.endArrow()) {
            this.drawArrow(context, endPoint, endTangent.flipped, arrowSize);
        }
        if (this.startArrow()) {
            this.drawArrow(context, startPoint, startTangent, arrowSize);
        }
        context.fillStyle = resolveCanvasStyle(this.stroke(), context);
        context.closePath();
        context.fill();
        context.restore();
    }
    drawArrow(context, center, tangent, arrowSize) {
        const normal = tangent.perpendicular;
        const origin = center.add(tangent.scale(-arrowSize / 2));
        moveTo(context, origin);
        lineTo(context, origin.add(tangent.add(normal).scale(arrowSize)));
        lineTo(context, origin.add(tangent.sub(normal).scale(arrowSize)));
        lineTo(context, origin);
        context.closePath();
    }
};
__decorate([
    initial(false),
    signal()
], Curve.prototype, "closed", void 0);
__decorate([
    initial(0),
    signal()
], Curve.prototype, "start", void 0);
__decorate([
    initial(0),
    signal()
], Curve.prototype, "startOffset", void 0);
__decorate([
    initial(false),
    signal()
], Curve.prototype, "startArrow", void 0);
__decorate([
    initial(1),
    signal()
], Curve.prototype, "end", void 0);
__decorate([
    initial(0),
    signal()
], Curve.prototype, "endOffset", void 0);
__decorate([
    initial(false),
    signal()
], Curve.prototype, "endArrow", void 0);
__decorate([
    initial(24),
    signal()
], Curve.prototype, "arrowSize", void 0);
__decorate([
    computed()
], Curve.prototype, "arcLength", null);
__decorate([
    computed()
], Curve.prototype, "curveDrawingInfo", null);
Curve = __decorate([
    nodeName('Curve')
], Curve);
export { Curve };
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ3VydmUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2NvbXBvbmVudHMvQ3VydmUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7Ozs7O0FBQUEsT0FBTyxFQUtMLE9BQU8sRUFDUCxLQUFLLEdBQ04sTUFBTSxxQkFBcUIsQ0FBQztBQUk3QixPQUFPLEVBQUMsa0JBQWtCLEVBQUMsTUFBTSw4QkFBOEIsQ0FBQztBQUNoRSxPQUFPLEVBQUMsUUFBUSxFQUFFLE9BQU8sRUFBRSxRQUFRLEVBQUUsTUFBTSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBRWxFLE9BQU8sRUFBQyxNQUFNLEVBQUUsTUFBTSxFQUFFLGtCQUFrQixFQUFDLE1BQU0sVUFBVSxDQUFDO0FBQzVELE9BQU8sRUFBQyxLQUFLLEVBQWEsTUFBTSxTQUFTLENBQUM7QUFzQ25DLElBQWUsS0FBSyxHQUFwQixNQUFlLEtBQU0sU0FBUSxLQUFLO0lBNEdwQixXQUFXO1FBQzVCLE9BQU8sSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDLElBQUksQ0FBQztJQUNsQyxDQUFDO0lBRUQsWUFBbUIsS0FBaUI7UUFDbEMsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBUEwsbUJBQWMsR0FBRyxLQUFLLENBQUM7SUFRakMsQ0FBQztJQU1EOzs7Ozs7OztPQVFHO0lBQ0ksb0JBQW9CLENBQUMsS0FBYTtRQUN2QyxPQUFPLEtBQUssQ0FDVixDQUFDLEVBQ0QsSUFBSSxDQUFDLGFBQWEsRUFBRSxFQUNwQixJQUFJLENBQUMsV0FBVyxFQUFFLEdBQUcsSUFBSSxDQUFDLGVBQWUsRUFBRSxHQUFHLEtBQUssQ0FDcEQsQ0FBQztJQUNKLENBQUM7SUFFRDs7Ozs7Ozs7T0FRRztJQUNJLG9CQUFvQixDQUFDLEtBQWE7UUFDdkMsT0FBTyxDQUFDLEtBQUssR0FBRyxJQUFJLENBQUMsV0FBVyxFQUFFLENBQUMsR0FBRyxJQUFJLENBQUMsZUFBZSxFQUFFLENBQUM7SUFDL0QsQ0FBQztJQUVE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