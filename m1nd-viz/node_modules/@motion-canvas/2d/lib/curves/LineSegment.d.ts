import { Vector2 } from '@motion-canvas/core';
import { CurvePoint } from './CurvePoint';
import { Segment } from './Segment';
export declare class LineSegment extends Segment {
    readonly from: Vector2;
    readonly to: Vector2;
    private readonly length;
    private readonly vector;
    private readonly normal;
    readonly points: Vector2[];
    constructor(from: Vector2, to: Vector2);
    get arcLength(): number;
    draw(context: CanvasRenderingContext2D | Path2D, start?: number, end?: number, move?: boolean): [CurvePoint, CurvePoint];
    getPoint(distance: number): CurvePoint;
}
//# sourceMappingURL=LineSegment.d.ts.map