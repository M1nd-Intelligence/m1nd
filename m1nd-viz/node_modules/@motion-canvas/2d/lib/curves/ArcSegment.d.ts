import { Vector2 } from '@motion-canvas/core';
import { CurvePoint } from './CurvePoint';
import { Segment } from './Segment';
export declare class ArcSegment extends Segment {
    readonly startPoint: Vector2;
    readonly radius: Vector2;
    readonly xAxisRotationDegree: number;
    readonly largeArcFlag: number;
    readonly sweepFlag: number;
    readonly endPoint: Vector2;
    private static el;
    readonly center: Vector2;
    readonly startAngle: number;
    readonly deltaAngle: number;
    readonly xAxisRotation: number;
    private xAxisRotationMatrix;
    readonly points: Vector2[];
    private length;
    constructor(startPoint: Vector2, radius: Vector2, xAxisRotationDegree: number, largeArcFlag: number, sweepFlag: number, endPoint: Vector2);
    getAnglePosition(angle: number): Vector2;
    getAngleDerivative(angle: number): Vector2;
    draw(context: CanvasRenderingContext2D | Path2D, start: number, end: number, move: boolean): [CurvePoint, CurvePoint];
    getPoint(distance: number): CurvePoint;
    get arcLength(): number;
}
//# sourceMappingURL=ArcSegment.d.ts.map