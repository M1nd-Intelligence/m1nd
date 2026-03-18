import { BBox, SerializedVector2, Vector2 } from '@motion-canvas/core';
import { CurveProfile } from '../curves';
import { PolynomialSegment } from '../curves/PolynomialSegment';
import { DesiredLength } from '../partials';
import { Curve } from './Curve';
export interface BezierOverlayInfo {
    curve: Path2D;
    handleLines: Path2D;
    controlPoints: Vector2[];
    startPoint: Vector2;
    endPoint: Vector2;
}
export declare abstract class Bezier extends Curve {
    profile(): CurveProfile;
    protected abstract segment(): PolynomialSegment;
    protected abstract overlayInfo(matrix: DOMMatrix): BezierOverlayInfo;
    protected childrenBBox(): BBox;
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected offsetComputedLayout(box: BBox): BBox;
    drawOverlay(context: CanvasRenderingContext2D, matrix: DOMMatrix): void;
}
//# sourceMappingURL=Bezier.d.ts.map