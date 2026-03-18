import { Vector2 } from '@motion-canvas/core';
import { CurveProfile } from './CurveProfile';
/**
 * Calculate polygon perimeter
 * @param points - polygon points
 * @returns - perimeter of polygon
 */
export declare function polygonLength(points: Vector2[]): number;
/**
 * Calculate total moving point distance when morphing between polygon points
 * @param points - first polygon points
 * @param reference - second polygon points
 * @param offset - offset for first polygon points
 * @returns
 */
export declare function calculateLerpDistance(points: Vector2[], reference: Vector2[], offset: number): number;
/**
 * Interpolate between two polygon points.
 * @param from - source polygon points
 * @param to - target polygon points
 * @param value - interpolation progress
 * @returns - new polygon points
 */
export declare function polygonPointsLerp(from: Vector2[], to: Vector2[], value: number): Vector2[];
/**
 * Create interpolator to tween between two curve
 * @param a - source curve
 * @param b - target curve
 * @returns - curve interpolator
 */
export declare function createCurveProfileLerp(a: CurveProfile, b: CurveProfile): (progress: number) => CurveProfile;
//# sourceMappingURL=createCurveProfileLerp.d.ts.map