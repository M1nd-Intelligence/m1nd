import { BBox, Vector2 } from '@motion-canvas/core';
import { CurvePoint } from './CurvePoint';
import { Polynomial2D } from './Polynomial2D';
import { Segment } from './Segment';
import { UniformPolynomialCurveSampler } from './UniformPolynomialCurveSampler';
export declare abstract class PolynomialSegment extends Segment {
    protected readonly curve: Polynomial2D;
    protected readonly length: number;
    protected readonly pointSampler: UniformPolynomialCurveSampler;
    get arcLength(): number;
    abstract get points(): Vector2[];
    protected constructor(curve: Polynomial2D, length: number);
    getBBox(): BBox;
    /**
     * Evaluate the polynomial at the given t value.
     *
     * @param t - The t value at which to evaluate the curve.
     */
    eval(t: number): CurvePoint;
    /**
     * Split the curve into two separate polynomials at the given t value. The two
     * resulting curves form the same overall shape as the original curve.
     *
     * @param t - The t value at which to split the curve.
     */
    abstract split(t: number): [PolynomialSegment, PolynomialSegment];
    getPoint(distance: number): CurvePoint;
    transformPoints(matrix: DOMMatrix): Vector2[];
    /**
     * Return the tangent of the point that sits at the provided t value on the
     * curve.
     *
     * @param t - The t value at which to evaluate the curve.
     */
    tangent(t: number): Vector2;
    draw(context: CanvasRenderingContext2D | Path2D, start?: number, end?: number, move?: boolean): [CurvePoint, CurvePoint];
    protected abstract doDraw(context: CanvasRenderingContext2D | Path2D): void;
}
//# sourceMappingURL=PolynomialSegment.d.ts.map