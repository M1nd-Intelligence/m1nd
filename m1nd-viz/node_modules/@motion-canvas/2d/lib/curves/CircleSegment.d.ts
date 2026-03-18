import { Vector2 } from '@motion-canvas/core';
import { CurvePoint } from './CurvePoint';
import { Segment } from './Segment';
export declare class CircleSegment extends Segment {
    private center;
    private radius;
    private from;
    private to;
    private counter;
    private readonly length;
    private readonly angle;
    readonly points: Vector2[];
    constructor(center: Vector2, radius: number, from: Vector2, to: Vector2, counter: boolean);
    get arcLength(): number;
    draw(context: CanvasRenderingContext2D | Path2D, from: number, to: number): [CurvePoint, CurvePoint];
    getPoint(distance: number): CurvePoint;
}
//# sourceMappingURL=CircleSegment.d.ts.map