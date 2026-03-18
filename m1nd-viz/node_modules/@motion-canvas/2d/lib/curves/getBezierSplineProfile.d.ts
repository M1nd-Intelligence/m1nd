import { CurveProfile } from './CurveProfile';
import { KnotInfo } from './KnotInfo';
/**
 * Calculate the curve profile of a spline based on a set of knots.
 *
 * @param knots - The knots defining the spline
 * @param closed - Whether the spline should be closed or not
 * @param smoothness - The desired smoothness of the spline when using auto
 *                     calculated handles.
 */
export declare function getBezierSplineProfile(knots: KnotInfo[], closed: boolean, smoothness: number): CurveProfile;
//# sourceMappingURL=getBezierSplineProfile.d.ts.map