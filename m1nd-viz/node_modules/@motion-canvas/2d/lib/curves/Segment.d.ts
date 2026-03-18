import { Vector2 } from '@motion-canvas/core';
import { CurvePoint } from './CurvePoint';
export declare abstract class Segment {
    abstract readonly points: Vector2[];
    abstract draw(context: CanvasRenderingContext2D | Path2D, start: number, end: number, move: boolean): [CurvePoint, CurvePoint];
    abstract getPoint(distance: number): CurvePoint;
    abstract get arcLength(): number;
}
//# sourceMappingURL=Segment.d.ts.map