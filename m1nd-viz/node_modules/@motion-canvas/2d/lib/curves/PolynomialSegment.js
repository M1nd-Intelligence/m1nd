import { moveTo } from '../utils';
import { Segment } from './Segment';
import { UniformPolynomialCurveSampler } from './UniformPolynomialCurveSampler';
export class PolynomialSegment extends Segment {
    get arcLength() {
        return this.length;
    }
    constructor(curve, length) {
        super();
        this.curve = curve;
        this.length = length;
        this.pointSampler = new UniformPolynomialCurveSampler(this);
    }
    getBBox() {
        return this.curve.getBounds();
    }
    /**
     * Evaluate the polynomial at the given t value.
     *
     * @param t - The t value at which to evaluate the curve.
     */
    eval(t) {
        const tangent = this.tangent(t);
        return {
            position: this.curve.eval(t),
            tangent,
            normal: tangent.perpendicular,
        };
    }
    getPoint(distance) {
        const closestPoint = this.pointSampler.pointAtDistance(this.arcLength * distance);
        return {
            position: closestPoint.position,
            tangent: closestPoint.tangent,
            normal: closestPoint.tangent.perpendicular,
        };
    }
    transformPoints(matrix) {
        return this.points.map(point => point.transformAsPoint(matrix));
    }
    /**
     * Return the tangent of the point that sits at the provided t value on the
     * curve.
     *
     * @param t - The t value at which to evaluate the curve.
     */
    tangent(t) {
        return this.curve.evalDerivative(t).normalized;
    }
    draw(context, start = 0, end = 1, move = true) {
        let curve = null;
        let startT = start;
        let endT = end;
        let points = this.points;
        if (start !== 0 || end !== 1) {
            const startDistance = this.length * start;
            const endDistance = this.length * end;
            startT = this.pointSampler.distanceToT(startDistance);
            endT = this.pointSampler.distanceToT(endDistance);
            const relativeEndT = (endT - startT) / (1 - startT);
            const [, startSegment] = this.split(startT);
            [curve] = startSegment.split(relativeEndT);
            points = curve.points;
        }
        if (move) {
            moveTo(context, points[0]);
        }
        (curve ?? this).doDraw(context);
        const startTangent = this.tangent(startT);
        const endTangent = this.tangent(endT);
        return [
            {
                position: points[0],
                tangent: startTangent,
                normal: startTangent.perpendicular,
            },
            {
                position: points.at(-1),
                tangent: endTangent,
                normal: endTangent.perpendicular,
            },
        ];
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUG9seW5vbWlhbFNlZ21lbnQuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2N1cnZlcy9Qb2x5bm9taWFsU2VnbWVudC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFDQSxPQUFPLEVBQUMsTUFBTSxFQUFDLE1BQU0sVUFBVSxDQUFDO0FBR2hDLE9BQU8sRUFBQyxPQUFPLEVBQUMsTUFBTSxXQUFXLENBQUM7QUFDbEMsT0FBTyxFQUFDLDZCQUE2QixFQUFDLE1BQU0saUNBQWlDLENBQUM7QUFFOUUsTUFBTSxPQUFnQixpQkFBa0IsU0FBUSxPQUFPO0lBR3JELElBQVcsU0FBUztRQUNsQixPQUFPLElBQUksQ0FBQyxNQUFNLENBQUM7SUFDckIsQ0FBQztJQUlELFlBQ3FCLEtBQW1CLEVBQ25CLE1BQWM7UUFFakMsS0FBSyxFQUFFLENBQUM7UUFIVyxVQUFLLEdBQUwsS0FBSyxDQUFjO1FBQ25CLFdBQU0sR0FBTixNQUFNLENBQVE7UUFHakMsSUFBSSxDQUFDLFlBQVksR0FBRyxJQUFJLDZCQUE2QixDQUFDLElBQUksQ0FBQyxDQUFDO0lBQzlELENBQUM7SUFFTSxPQUFPO1FBQ1osT0FBTyxJQUFJLENBQUMsS0FBSyxDQUFDLFNBQVMsRUFBRSxDQUFDO0lBQ2hDLENBQUM7SUFFRDs7OztPQUlHO0lBQ0ksSUFBSSxDQUFDLENBQVM7UUFDbkIsTUFBTSxPQUFPLEdBQUcsSUFBSSxDQUFDLE9BQU8sQ0FBQyxDQUFDLENBQUMsQ0FBQztRQUVoQyxPQUFPO1lBQ0wsUUFBUSxFQUFFLElBQUksQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBQztZQUM1QixPQUFPO1lBQ1AsTUFBTSxFQUFFLE9BQU8sQ0FBQyxhQUFhO1NBQzlCLENBQUM7SUFDSixDQUFDO0lBVU0sUUFBUSxDQUFDLFFBQWdCO1FBQzlCLE1BQU0sWUFBWSxHQUFHLElBQUksQ0FBQyxZQUFZLENBQUMsZUFBZSxDQUNwRCxJQUFJLENBQUMsU0FBUyxHQUFH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