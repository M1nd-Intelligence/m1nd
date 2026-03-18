var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { Vector2, lazy } from '@motion-canvas/core';
import { quadraticCurveTo } from '../utils';
import { Polynomial2D } from './Polynomial2D';
import { PolynomialSegment } from './PolynomialSegment';
/**
 * A spline segment representing a quadratic Bézier curve.
 */
export class QuadBezierSegment extends PolynomialSegment {
    get points() {
        return [this.p0, this.p1, this.p2];
    }
    constructor(p0, p1, p2) {
        super(new Polynomial2D(p0, 
        // 2*(-p0+p1)
        p0.flipped.add(p1).scale(2), 
        // p0-2*p1+p2
        p0.sub(p1.scale(2)).add(p2)), QuadBezierSegment.getLength(p0, p1, p2));
        this.p0 = p0;
        this.p1 = p1;
        this.p2 = p2;
    }
    split(t) {
        const a = new Vector2(this.p0.x + (this.p1.x - this.p0.x) * t, this.p0.y + (this.p1.y - this.p0.y) * t);
        const b = new Vector2(this.p1.x + (this.p2.x - this.p1.x) * t, this.p1.y + (this.p2.y - this.p1.y) * t);
        const p = new Vector2(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
        const left = new QuadBezierSegment(this.p0, a, p);
        const right = new QuadBezierSegment(p, b, this.p2);
        return [left, right];
    }
    static getLength(p0, p1, p2) {
        // Let the browser do the work for us instead of calculating the arclength
        // manually.
        QuadBezierSegment.el.setAttribute('d', `M ${p0.x} ${p0.y} Q ${p1.x} ${p1.y} ${p2.x} ${p2.y}`);
        return QuadBezierSegment.el.getTotalLength();
    }
    doDraw(context) {
        quadraticCurveTo(context, this.p1, this.p2);
    }
}
__decorate([
    lazy(() => document.createElementNS('http://www.w3.org/2000/svg', 'path'))
], QuadBezierSegment, "el", void 0);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUXVhZEJlemllclNlZ21lbnQuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2N1cnZlcy9RdWFkQmV6aWVyU2VnbWVudC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7QUFBQSxPQUFPLEVBQUMsT0FBTyxFQUFFLElBQUksRUFBQyxNQUFNLHFCQUFxQixDQUFDO0FBQ2xELE9BQU8sRUFBQyxnQkFBZ0IsRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUMxQyxPQUFPLEVBQUMsWUFBWSxFQUFDLE1BQU0sZ0JBQWdCLENBQUM7QUFDNUMsT0FBTyxFQUFDLGlCQUFpQixFQUFDLE1BQU0scUJBQXFCLENBQUM7QUFFdEQ7O0dBRUc7QUFDSCxNQUFNLE9BQU8saUJBQWtCLFNBQVEsaUJBQWlCO0lBSXRELElBQVcsTUFBTTtRQUNmLE9BQU8sQ0FBQyxJQUFJLENBQUMsRUFBRSxFQUFFLElBQUksQ0FBQyxFQUFFLEVBQUUsSUFBSSxDQUFDLEVBQUUsQ0FBQyxDQUFDO0lBQ3JDLENBQUM7SUFFRCxZQUNrQixFQUFXLEVBQ1gsRUFBVyxFQUNYLEVBQVc7UUFFM0IsS0FBSyxDQUNILElBQUksWUFBWSxDQUNkLEVBQUU7UUFDRixhQUFhO1FBQ2IsRUFBRSxDQUFDLE9BQU8sQ0FBQyxHQUFHLENBQUMsRUFBRSxDQUFDLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQztRQUMzQixhQUFhO1FBQ2IsRUFBRSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUMsR0FBRyxDQUFDLEVBQUUsQ0FBQyxDQUM1QixFQUNELGlCQUFpQixDQUFDLFNBQVMsQ0FBQyxFQUFFLEVBQUUsRUFBRSxFQUFFLEVBQUUsQ0FBQyxDQUN4QyxDQUFDO1FBYmMsT0FBRSxHQUFGLEVBQUUsQ0FBUztRQUNYLE9BQUUsR0FBRixFQUFFLENBQVM7UUFDWCxPQUFFLEdBQUYsRUFBRSxDQUFTO0lBWTdCLENBQUM7SUFFTSxLQUFLLENBQUMsQ0FBUztRQUNwQixNQUFNLENBQUMsR0FBRyxJQUFJLE9BQU8sQ0FDbkIsSUFBSSxDQUFDLEVBQUUsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsR0FBRyxJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQyxHQUFHLENBQUMsRUFDdkMsSUFBSSxDQUFDLEVBQUUsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsR0FBRyxJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQyxHQUFHLENBQUMsQ0FDeEMsQ0FBQztRQUNGLE1BQU0sQ0FBQyxHQUFHLElBQUksT0FBTyxDQUNuQixJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxFQUFFLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxFQUFFLENBQUMsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxFQUN2QyxJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxFQUFFLENBQUMsQ0FBQyxHQUFHLElBQUksQ0FBQyxFQUFFLENBQUMsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxDQUN4QyxDQUFDO1FBQ0YsTUFBTSxDQUFDLEdBQUcsSUFBSSxPQUFPLENBQUMsQ0FBQyxDQUFDLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxDQUFDLENBQUMsQ0FBQyxHQUFHLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQyxHQUFHLENBQUMsQ0FBQyxDQUFDLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxDQUFDLENBQUM7UUFFcEUsTUFBTSxJQUFJLEdBQUcsSUFBSSxpQkFBaUIsQ0FBQyxJQUFJLENBQUMsRUFBRSxFQUFFLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQztRQUNsRCxNQUFNLEtBQUssR0FBRyxJQUFJLGlCQUFpQixDQUFDLENBQUMsRUFBRSxDQUFDLEVBQUUsSUFBSSxDQUFDLEVBQUUsQ0FBQyxDQUFDO1FBRW5ELE9BQU8sQ0FBQyxJQUFJLEVBQUUsS0FBSyxDQUFDLENBQUM7SUFDdkIsQ0FBQztJQUVTLE1BQU0sQ0FBQyxTQUFTLENBQUMsRUFBVyxFQUFFLEVBQVcsRUFBRSxFQUFXO1FBQzlELDBFQUEwRTtRQUMxRSxZQUFZO1FBQ1osaUJBQWlCLENBQUMsRUFBRSxDQUFDLFlBQVksQ0FDL0IsR0FBRyxFQUNILEtBQUssRUFBRSxDQUFDLENBQUMsSUFBSSxFQUFFLENBQUMsQ0FBQyxNQUFNLEVBQUUsQ0FBQyxDQUFDLElBQUksRUFBRSxDQUFDLENBQUMsSUFBSSxFQUFFLENBQUMsQ0FBQyxJQUFJLEVBQUUsQ0FBQyxDQUFDLEVBQUUsQ0FDdEQsQ0FBQztRQUNGLE9BQU8saUJBQWlCLENBQUMsRUFBRSxDQUFDLGNBQWMsRUFBRSxDQUFDO0lBQy9DLENBQUM7SUFFa0IsTUFBTSxDQUFDLE9BQTBDO1FBQ2xFLGdCQUFnQixDQUFDLE9BQU8sRUFBRSxJQUFJLENBQUMsRUFBRSxFQUFFLElBQUksQ0FBQyxFQUFFLENBQUMsQ0FBQztJQUM5QyxDQUFDO0NBQ0Y7QUFyRGdCO0lBRGQsSUFBSSxDQUFDLEdBQUcsRUFBRSxDQUFDLFFBQVEsQ0FBQyxlQUFlLENBQUMsNEJBQTRCLEVBQUUsTUFBTSxDQUFDLENBQUM7bUNBQ3pDIn0=