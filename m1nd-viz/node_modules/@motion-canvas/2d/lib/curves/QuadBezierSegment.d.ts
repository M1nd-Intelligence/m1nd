import { Vector2 } from '@motion-canvas/core';
import { PolynomialSegment } from './PolynomialSegment';
/**
 * A spline segment representing a quadratic Bézier curve.
 */
export declare class QuadBezierSegment extends PolynomialSegment {
    readonly p0: Vector2;
    readonly p1: Vector2;
    readonly p2: Vector2;
    private static el;
    get points(): Vector2[];
    constructor(p0: Vector2, p1: Vector2, p2: Vector2);
    split(t: number): [PolynomialSegment, PolynomialSegment];
    protected static getLength(p0: Vector2, p1: Vector2, p2: Vector2): number;
    protected doDraw(context: CanvasRenderingContext2D | Path2D): void;
}
//# sourceMappingURL=QuadBezierSegment.d.ts.map