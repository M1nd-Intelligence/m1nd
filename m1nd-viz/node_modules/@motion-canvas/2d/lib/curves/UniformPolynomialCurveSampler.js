import { clamp, remap } from '@motion-canvas/core';
/**
 * Class to uniformly sample points on a given polynomial curve.
 *
 * @remarks
 * In order to uniformly sample points from non-linear curves, this sampler
 * re-parameterizes the curve by arclength.
 */
export class UniformPolynomialCurveSampler {
    /**
     * @param curve - The curve to sample
     * @param samples - How many points to sample from the provided curve. The
     *                  more points get sampled, the higher the resolution–and
     *                  therefore precision–of the sampler.
     */
    constructor(curve, samples = 20) {
        this.curve = curve;
        this.sampledDistances = [];
        this.resample(samples);
    }
    /**
     * Discard all previously sampled points and resample the provided number of
     * points from the curve.
     *
     * @param samples - The number of points to sample.
     */
    resample(samples) {
        this.sampledDistances = [0];
        let length = 0;
        let previous = this.curve.eval(0).position;
        for (let i = 1; i < samples; i++) {
            const t = i / (samples - 1);
            const curvePoint = this.curve.eval(t);
            const segmentLength = previous.sub(curvePoint.position).magnitude;
            length += segmentLength;
            this.sampledDistances.push(length);
            previous = curvePoint.position;
        }
        // Account for any accumulated floating point errors and explicitly set the
        // distance of the last point to the arclength of the curve.
        this.sampledDistances[this.sampledDistances.length - 1] =
            this.curve.arcLength;
    }
    /**
     * Return the point at the provided distance along the sampled curve's
     * arclength.
     *
     * @param distance - The distance along the curve's arclength for which to
     *                   retrieve the point.
     */
    pointAtDistance(distance) {
        return this.curve.eval(this.distanceToT(distance));
    }
    /**
     * Return the t value for the point at the provided distance along the sampled
     * curve's arc length.
     *
     * @param distance - The distance along the arclength
     */
    distanceToT(distance) {
        const samples = this.sampledDistances.length;
        distance = clamp(0, this.curve.arcLength, distance);
        for (let i = 0; i < samples; i++) {
            const lower = this.sampledDistances[i];
            const upper = this.sampledDistances[i + 1];
            if (distance >= lower && distance <= upper) {
                return remap(lower, upper, i / (samples - 1), (i + 1) / (samples - 1), distance);
            }
        }
        return 1;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiVW5pZm9ybVBvbHlub21pYWxDdXJ2ZVNhbXBsZXIuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2N1cnZlcy9Vbmlmb3JtUG9seW5vbWlhbEN1cnZlU2FtcGxlci50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQVUsS0FBSyxFQUFFLEtBQUssRUFBQyxNQUFNLHFCQUFxQixDQUFDO0FBSTFEOzs7Ozs7R0FNRztBQUNILE1BQU0sT0FBTyw2QkFBNkI7SUFHeEM7Ozs7O09BS0c7SUFDSCxZQUNtQixLQUF3QixFQUN6QyxPQUFPLEdBQUcsRUFBRTtRQURLLFVBQUssR0FBTCxLQUFLLENBQW1CO1FBVG5DLHFCQUFnQixHQUFhLEVBQUUsQ0FBQztRQVl0QyxJQUFJLENBQUMsUUFBUSxDQUFDLE9BQU8sQ0FBQyxDQUFDO0lBQ3pCLENBQUM7SUFFRDs7Ozs7T0FLRztJQUNJLFFBQVEsQ0FBQyxPQUFlO1FBQzdCLElBQUksQ0FBQyxnQkFBZ0IsR0FBRyxDQUFDLENBQUMsQ0FBQyxDQUFDO1FBRTVCLElBQUksTUFBTSxHQUFHLENBQUMsQ0FBQztRQUNmLElBQUksUUFBUSxHQUFZLElBQUksQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBQyxDQUFDLFFBQVEsQ0FBQztRQUNwRCxLQUFLLElBQUksQ0FBQyxHQUFHLENBQUMsRUFBRSxDQUFDLEdBQUcsT0FBTyxFQUFFLENBQUMsRUFBRSxFQUFFO1lBQ2hDLE1BQU0sQ0FBQyxHQUFHLENBQUMsR0FBRyxDQUFDLE9BQU8sR0FBRyxDQUFDLENBQUMsQ0FBQztZQUM1QixNQUFNLFVBQVUsR0FBRyxJQUFJLENBQUMsS0FBSyxDQUFDLElBQUksQ0FBQyxDQUFDLENBQUMsQ0FBQztZQUN0QyxNQUFNLGFBQWEsR0FBRyxRQUFRLENBQUMsR0FBRyxDQUFDLFVBQVUsQ0FBQyxRQUFRLENBQUMsQ0FBQyxTQUFTLENBQUM7WUFFbEUsTUFBTSxJQUFJLGFBQWEsQ0FBQztZQUV4QixJQUFJLENBQUMsZ0JBQWdCLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxDQUFDO1lBQ25DLFFBQVEsR0FBRyxVQUFVLENBQUMsUUFBUSxDQUFDO1NBQ2hDO1FBRUQsMkVBQTJFO1FBQzNFLDREQUE0RDtRQUM1RCxJQUFJLENBQUMsZ0JBQWdCLENBQUMsSUFBSSxDQUFDLGdCQUFnQixDQUFDLE1BQU0sR0FBRyxDQUFDLENBQUM7WUFDckQsSUFBSSxDQUFDLEtBQUssQ0FBQyxTQUFTLENBQUM7SUFDekIsQ0FBQztJQUVEOzs7Ozs7T0FNRztJQUNJLGVBQWUsQ0FBQyxRQUFnQjtRQUNyQyxPQUFPLElBQUksQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQUMsUUFBUSxDQUFDLENBQUMsQ0FBQztJQUNyRCxDQUFDO0lBRUQ7Ozs7O09BS0c7SUFDSSxXQUFXLENBQUMsUUFBZ0I7UUFDakMsTUFBTSxPQUFPLEdBQUcsSUFBSSxDQUFDLGdCQUFnQixDQUFDLE1BQU0sQ0FBQztRQUM3QyxRQUFRLEdBQUcsS0FBSyxDQUFDLENBQUMsRUFBRSxJQUFJLENBQUMsS0FBSyxDQUFDLFNBQVMsRUFBRSxRQUFRLENBQUMsQ0FBQztRQUVwRCxLQUFLLElBQUksQ0FBQyxHQUFHLENBQUMsRUFBRSxDQUFDLEdBQUcsT0FBTyxFQUFFLENBQUMsRUFBRSxFQUFFO1lBQ2hDLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxnQkFBZ0IsQ0FBQyxDQUFDLENBQUMsQ0FBQztZQUN2QyxNQUFNLEtBQUssR0FBRyxJQUFJLENBQUMsZ0JBQWdCLENBQUMsQ0FBQyxHQUFHLENBQUMsQ0FBQyxDQUFDO1lBQzNDLElBQUksUUFBUSxJQUFJLEtBQUssSUFBSSxRQUFRLElBQUksS0FBSyxFQUFFO2dCQUMxQyxPQUFPLEtBQUssQ0FDVixLQUFLLEVBQ0wsS0FBSyxFQUNMLENBQUMsR0FBRyxDQUFDLE9BQU8sR0FBRyxDQUFDLENBQUMsRUFDakIsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxPQUFPLEdBQUcsQ0FBQyxDQUFDLEVBQ3ZCLFFBQVEsQ0FDVCxDQUFDO2FBQ0g7U0FDRjtRQUVELE9BQU8sQ0FBQyxDQUFDO0lBQ1gsQ0FBQztDQUNGIn0=