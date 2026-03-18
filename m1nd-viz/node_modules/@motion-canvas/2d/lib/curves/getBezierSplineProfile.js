import { Vector2, clamp } from '@motion-canvas/core';
import { CubicBezierSegment } from './CubicBezierSegment';
import { QuadBezierSegment } from './QuadBezierSegment';
function isCubicSegment(segment) {
    return segment instanceof CubicBezierSegment;
}
/**
 * Update a given knot's handles to be a blend between the user provided handles
 * and a set of auto calculated handles that smoothly connect the knot to its
 * two neighboring knots.
 *
 * @param knot - The knot for which to calculate the handles
 * @param previous - The previous knot in the spline, relative to the provided
 *                   knot.
 * @param next - The next knot in the spline, relative to the provided knot.
 * @param smoothness - The desired smoothness of the spline. Affects the scaling
 *                     of the auto calculated handles.
 */
function calculateSmoothHandles(knot, previous, next, smoothness) {
    if (knot.auto.start === 0 && knot.auto.end === 0) {
        return;
    }
    // See for reference:
    // http://scaledinnovation.com/analytics/splines/aboutSplines.html
    const distanceToPrev = knot.position.sub(previous.position).magnitude;
    const distanceToNext = next.position.sub(knot.position).magnitude;
    const fa = (smoothness * distanceToPrev) / (distanceToPrev + distanceToNext);
    const fb = smoothness - fa;
    const startHandle = new Vector2(knot.position.x - fa * (next.position.x - previous.position.x), knot.position.y - fa * (next.position.y - previous.position.y));
    const endHandle = new Vector2(knot.position.x + fb * (next.position.x - previous.position.x), knot.position.y + fb * (next.position.y - previous.position.y));
    knot.startHandle = knot.startHandle.lerp(startHandle, knot.auto.start);
    knot.endHandle = knot.endHandle.lerp(endHandle, knot.auto.end);
}
/**
 * Calculate the `minSin` value of the curve profile so that miter joins get
 * taken into account properly.
 */
function updateMinSin(profile) {
    for (let i = 0; i < profile.segments.length; i++) {
        const segmentA = profile.segments[i];
        const segmentB = profile.segments[(i + 1) % profile.segments.length];
        // Quadratic Bézier segments will always join smoothly with the previous
        // segment. This means that we can skip the segment since it's impossible
        // to have a miter join between the two segments.
        if (!isCubicSegment(segmentA) || !isCubicSegment(segmentB)) {
            continue;
        }
        const startVector = segmentA.p2.sub(segmentA.p3).normalized.safe;
        const endVector = segmentB.p1.sub(segmentB.p0).normalized.safe;
        const dot = startVector.dot(endVector);
        // A miter join can only occur if the handle is broken, so we can skip the
        // segment if the handles are mirrored.
        const isBroken = 1 - Math.abs(dot) > 0.0001;
        if (!isBroken) {
            continue;
        }
        const angleBetween = Math.acos(clamp(-1, 1, dot));
        const angleSin = Math.sin(angleBetween / 2);
        profile.minSin = Math.min(profile.minSin, Math.abs(angleSin));
    }
}
function addSegmentToProfile(profile, p0, p1, p2, p3) {
    const segment = p3 !== undefined
        ? new CubicBezierSegment(p0, p1, p2, p3)
        : new QuadBezierSegment(p0, p1, p2);
    profile.segments.push(segment);
    profile.arcLength += segment.arcLength;
}
/**
 * Calculate the curve profile of a spline based on a set of knots.
 *
 * @param knots - The knots defining the spline
 * @param closed - Whether the spline should be closed or not
 * @param smoothness - The desired smoothness of the spline when using auto
 *                     calculated handles.
 */
export function getBezierSplineProfile(knots, closed, smoothness) {
    const profile = {
        segments: [],
        arcLength: 0,
        minSin: 1,
    };
    if (knots.length < 2) {
        return profile;
    }
    // First, we want to calculate the actual handle positions for each knot. We
    // do so using the knot's `auto` value to blend between the user-provided
    // handles and the auto calculated smooth handles.
    const numberOfKnots = knots.length;
    for (let i = 0; i < numberOfKnots; i++) {
        // Calculating the auto handles for a given knot requires both of the knot's
        // neighboring knots. To make sure that this works properly for the first
        // and last knots of the spline, we want to make sure to wrap around to the
        // beginning and end of the array, respectively.
        const prevIndex = (i - 1 + numberOfKnots) % numberOfKnots;
        const nextIndex = (i + 1) % numberOfKnots;
        calculateSmoothHandles(knots[i], knots[prevIndex], knots[nextIndex], smoothness);
    }
    const firstKnot = knots[0];
    const secondKnot = knots[1];
    // Drawing the first and last segments of a spline has a few edge cases we
    // need to consider:
    // If the spline is not closed and the first knot should use the auto
    // calculated handles, we want to draw a quadratic Bézier curve instead of a
    // cubic one.
    if (!closed && firstKnot.auto.start === 1 && firstKnot.auto.end === 1) {
        addSegmentToProfile(profile, firstKnot.position, secondKnot.startHandle, secondKnot.position);
    }
    else {
        // Otherwise, draw a cubic Bézier segment like we do for the other segments.
        addSegmentToProfile(profile, firstKnot.position, firstKnot.endHandle, secondKnot.startHandle, secondKnot.position);
    }
    // Add all intermediate spline segments as cubic Bézier curve segments.
    for (let i = 1; i < numberOfKnots - 2; i++) {
        const start = knots[i];
        const end = knots[i + 1];
        addSegmentToProfile(profile, start.position, start.endHandle, end.startHandle, end.position);
    }
    const lastKnot = knots.at(-1);
    const secondToLastKnot = knots.at(-2);
    if (knots.length > 2) {
        // Similar to the first segment, we also want to draw the last segment as a
        // quadratic Bézier curve if the curve is not closed and the knot should
        // use the auto calculated handles.
        if (!closed && lastKnot.auto.start === 1 && lastKnot.auto.end === 1) {
            addSegmentToProfile(profile, secondToLastKnot.position, secondToLastKnot.endHandle, lastKnot.position);
        }
        else {
            addSegmentToProfile(profile, secondToLastKnot.position, secondToLastKnot.endHandle, lastKnot.startHandle, lastKnot.position);
        }
    }
    // If the spline should be closed, add one final cubic Bézier segment
    // connecting the last and first knots.
    if (closed) {
        addSegmentToProfile(profile, lastKnot.position, lastKnot.endHandle, firstKnot.startHandle, firstKnot.position);
    }
    updateMinSin(profile);
    return profile;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZ2V0QmV6aWVyU3BsaW5lUHJvZmlsZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY3VydmVzL2dldEJlemllclNwbGluZVByb2ZpbGUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLE9BQU8sRUFBRSxLQUFLLEVBQUMsTUFBTSxxQkFBcUIsQ0FBQztBQUNuRCxPQUFPLEVBQUMsa0JBQWtCLEVBQUMsTUFBTSxzQkFBc0IsQ0FBQztBQUl4RCxPQUFPLEVBQUMsaUJBQWlCLEVBQUMsTUFBTSxxQkFBcUIsQ0FBQztBQUV0RCxTQUFTLGNBQWMsQ0FDckIsT0FBMEI7SUFFMUIsT0FBTyxPQUFPLFlBQVksa0JBQWtCLENBQUM7QUFDL0MsQ0FBQztBQUVEOzs7Ozs7Ozs7OztHQVdHO0FBQ0gsU0FBUyxzQkFBc0IsQ0FDN0IsSUFBYyxFQUNkLFFBQWtCLEVBQ2xCLElBQWMsRUFDZCxVQUFrQjtJQUVsQixJQUFJLElBQUksQ0FBQyxJQUFJLENBQUMsS0FBSyxLQUFLLENBQUMsSUFBSSxJQUFJLENBQUMsSUFBSSxDQUFDLEdBQUcsS0FBSyxDQUFDLEVBQUU7UUFDaEQsT0FBTztLQUNSO0lBRUQscUJBQXFCO0lBQ3JCLGtFQUFrRTtJQUNsRSxNQUFNLGNBQWMsR0FBRyxJQUFJLENBQUMsUUFBUSxDQUFDLEdBQUcsQ0FBQyxRQUFRLENBQUMsUUFBUSxDQUFDLENBQUMsU0FBUyxDQUFDO0lBQ3RFLE1BQU0sY0FBYyxHQUFHLElBQUksQ0FBQyxRQUFRLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxRQUFRLENBQUMsQ0FBQyxTQUFTLENBQUM7SUFDbEUsTUFBTSxFQUFFLEdBQUcsQ0FBQyxVQUFVLEdBQUcsY0FBYyxDQUFDLEdBQUcsQ0FBQyxjQUFjLEdBQUcsY0FBYyxDQUFDLENBQUM7SUFDN0UsTUFBTSxFQUFFLEdBQUcsVUFBVSxHQUFHLEVBQUUsQ0FBQztJQUMzQixNQUFNLFdBQVcsR0FBRyxJQUFJLE9BQU8s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