import { CurvePoint } from './CurvePoint';
import { PolynomialSegment } from './PolynomialSegment';
/**
 * Class to uniformly sample points on a given polynomial curve.
 *
 * @remarks
 * In order to uniformly sample points from non-linear curves, this sampler
 * re-parameterizes the curve by arclength.
 */
export declare class UniformPolynomialCurveSampler {
    private readonly curve;
    private sampledDistances;
    /**
     * @param curve - The curve to sample
     * @param samples - How many points to sample from the provided curve. The
     *                  more points get sampled, the higher the resolution–and
     *                  therefore precision–of the sampler.
     */
    constructor(curve: PolynomialSegment, samples?: number);
    /**
     * Discard all previously sampled points and resample the provided number of
     * points from the curve.
     *
     * @param samples - The number of points to sample.
     */
    resample(samples: number): void;
    /**
     * Return the point at the provided distance along the sampled curve's
     * arclength.
     *
     * @param distance - The distance along the curve's arclength for which to
     *                   retrieve the point.
     */
    pointAtDistance(distance: number): CurvePoint;
    /**
     * Return the t value for the point at the provided distance along the sampled
     * curve's arc length.
     *
     * @param distance - The distance along the arclength
     */
    distanceToT(distance: number): number;
}
//# sourceMappingURL=UniformPolynomialCurveSampler.d.ts.map