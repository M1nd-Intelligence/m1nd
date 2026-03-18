import { Vector2 } from '@motion-canvas/core';
import { LineSegment } from './LineSegment';
import { getPointAtDistance } from './getPointAtDistance';
import { getPolylineProfile } from './getPolylineProfile';
/**
 * Split segments of polygon until distance between adjacent point is less than or equal maxLength. This function mutate original points.
 * @param points - Polygon points
 * @param maxLength - max distance between two point
 */
function bisect(points, maxLength) {
    for (let i = 0; i < points.length - 1; i++) {
        const a = points[i];
        let b = points[i + 1];
        while (a.sub(b).magnitude > maxLength) {
            b = Vector2.lerp(a, b, 0.5);
            points.splice(i + 1, 0, b);
        }
    }
}
/**
 * Convert curve which only contain LineSegment into polygon.
 * @param curve - curve to convert. curve must contain 1 subpath
 * @param maxLength - max distance between two point
 * @returns - null if curve contain segment other than LineSegment
 */
function exactPolygonPoints(curve, maxLength) {
    const points = [];
    let endPoint = null;
    for (const segment of curve.segments) {
        if (!(segment instanceof LineSegment))
            return null;
        points.push(segment.from);
        endPoint = segment.to;
    }
    if (endPoint)
        points.push(endPoint);
    if (!Number.isNaN(maxLength) && maxLength > 0) {
        bisect(points, maxLength);
    }
    return points;
}
/**
 * Calculate area of polygon
 * @param points - polygon points
 * @returns - area of polygon
 */
function polygonArea(points) {
    return (points.reduce((area, a, i) => {
        const b = points[(i + 1) % points.length];
        return area + (a.y * b.x - a.x * b.y);
    }, 0) / 2);
}
/**
 * Convert curve into polygon by sampling curve profile
 * @param curve - curve to convert. curve must contain only 1 subpath
 * @param maxLength - max distance between point
 * @returns - always return polygon points
 */
function approximatePolygonPoints(curve, maxLength) {
    const points = [];
    let numPoints = 3;
    if (!Number.isNaN(maxLength) && maxLength > 0) {
        numPoints = Math.max(numPoints, Math.ceil(curve.arcLength / maxLength));
    }
    for (let i = 0; i < numPoints; i += 1) {
        const point = getPointAtDistance(curve, curve.arcLength * (i / (numPoints - 1)));
        points.push(point.position);
    }
    if (polygonArea(points) > 0)
        points.reverse();
    return points;
}
/**
 * Split curve into subpaths
 * @param curve - curve to split
 * @returns - subpaths of curve
 */
function splitCurve(curve) {
    if (curve.segments.length === 0)
        return [];
    let current = {
        arcLength: 0,
        minSin: 0,
        segments: [],
        closed: false,
    };
    let endPoint = null;
    const composite = [current];
    for (const segment of curve.segments) {
        const start = segment.getPoint(0).position;
        if (endPoint && !start.equals(endPoint)) {
            current = {
                arcLength: 0,
                minSin: 0,
                segments: [],
                closed: false,
            };
            composite.push(current);
        }
        current.segments.push(segment);
        current.arcLength += segment.arcLength;
        endPoint = segment.getPoint(1).position;
    }
    for (const sub of composite) {
        sub.closed = sub.segments[0]
            .getPoint(0)
            .position.equals(sub.segments[sub.segments.length - 1].getPoint(1).position);
    }
    return composite;
}
/**
 * Convert curve into polygon use best possible method
 * @param curve - curve to convert
 * @param maxLength - max distance between two point
 * @returns - polgon points
 */
function subcurveToPolygon(curve, maxLength) {
    const points = exactPolygonPoints(curve, maxLength) ||
        approximatePolygonPoints(curve, maxLength);
    return {
        points: [...points],
        closed: curve.closed,
    };
}
/**
 * Calculate polygon perimeter
 * @param points - polygon points
 * @returns - perimeter of polygon
 */
export function polygonLength(points) {
    return points.reduce((length, point, i) => {
        if (i)
            return length + points[i - 1].sub(point).magnitude;
        return 0;
    }, 0);
}
/**s
 * Sample additional points for polygon to better match its pair. This will mutate original points.
 * @param points - polygon points
 * @param numPoints - number of points to be added
 */
function addPoints(points, numPoints) {
    const desiredLength = points.length + numPoints;
    const step = polygonLength(points) / numPoints;
    let i = 0;
    let cursor = 0;
    let insertAt = step / 2;
    while (points.length < desiredLength) {
        const a = points[i];
        const b = points[(i + 1) % points.length];
        const length = a.sub(b).magnitude;
        if (insertAt <= cursor + length) {
            points.splice(i + 1, 0, length
                ? Vector2.lerp(a, b, (insertAt - cursor) / length)
                : new Vector2(a));
            insertAt += step;
        }
        else {
            cursor += length;
            i += 1;
        }
    }
}
/**
 * Calculate total moving point distance when morphing between polygon points
 * @param points - first polygon points
 * @param reference - second polygon points
 * @param offset - offset for first polygon points
 * @returns
 */
export function calculateLerpDistance(points, reference, offset) {
    const len = points.length;
    let sumOfSquares = 0;
    for (let i = 0; i < reference.length; i += 1) {
        const a = points[(offset + i) % len];
        const b = reference[i];
        sumOfSquares += a.sub(b).squaredMagnitude;
    }
    return sumOfSquares;
}
/**
 * Rotate polygon in order to minimize moving points.
 * @param polygon - polygon to be rotated
 * @param reference - polygon to be reference
 */
function rotatePolygon(polygon, reference) {
    const { points, closed } = polygon;
    const len = points.length;
    if (!closed) {
        const originalDistance = calculateLerpDistance(points, reference.points, 0);
        const reversedPoints = [...points].reverse();
        const reversedDistance = calculateLerpDistance(reversedPoints, reference.points, 0);
        if (reversedDistance < originalDistance)
            polygon.points = reversedPoints;
    }
    else {
        let minDistance = Infinity;
        let bestOffset = 0;
        const last = points.pop();
        // Closed polygon first point must equal last point
        // When we rotate polygon, first point is changed which mean last point also must changed
        // When we remove last point, calculateLerpDistance will assume last point is equal first point
        // Proof:
        // len = points.length = reference.length - 1
        // When i = 0:
        // (offset + i) % len = offset % len
        // When i = reference.length - 1 or i = len
        // (offset + i) % len = (offset + len) % len = offset % len
        for (let offset = 0; offset < len; offset += 1) {
            const distance = calculateLerpDistance(points, reference.points, offset);
            if (distance < minDistance) {
                minDistance = distance;
                bestOffset = offset;
            }
        }
        if (last)
            points.push(last);
        if (bestOffset) {
            points.pop();
            const spliced = points.splice(0, bestOffset);
            points.splice(points.length, 0, ...spliced);
            points.push(points[0]);
        }
    }
}
/**
 * Round polygon's points coordinate to a specified amount of decimal
 * @param points - polygon point to be rounded
 * @param round - amount of decimal
 * @returns - new polygon point
 */
function roundPolygon({ points, ...rest }, round) {
    const pow = round >= 1 ? 10 ** round : 1;
    return {
        points: points.map(point => {
            const [x, y] = [point.x, point.y].map(n => Math.round(n * pow) / pow);
            return new Vector2(x, y);
        }),
        ...rest,
    };
}
/**
 * Create two polygon to tween between sub curve/path
 * @param from - source curve
 * @param to - targe curve
 * @param precision - desired distance between two point
 * @param round - amount of decimal when rounding
 * @returns two polygon ready to tween
 */
function getSubcurveInterpolationPolygon(from, to, precision, round) {
    const morphPrecision = precision;
    const fromRing = subcurveToPolygon(from, morphPrecision);
    const toRing = subcurveToPolygon(to, morphPrecision);
    const diff = fromRing.points.length - toRing.points.length;
    addPoints(fromRing.points, diff < 0 ? diff * -1 : 0);
    addPoints(toRing.points, diff > 0 ? diff : 0);
    if (!from.closed && to.closed)
        rotatePolygon(toRing, fromRing);
    else
        rotatePolygon(fromRing, toRing);
    return {
        from: roundPolygon(fromRing, round),
        to: roundPolygon(toRing, round),
    };
}
/**
 * Make two sub curve list have equal length
 * @param subcurves - List to add
 * @param reference - Reference list
 */
function balanceSubcurves(subcurves, reference) {
    for (let i = subcurves.length; i < reference.length; i++) {
        const point = reference[i].segments[0].getPoint(0).position;
        subcurves.push({
            arcLength: 0,
            closed: false,
            minSin: 0,
            segments: [new LineSegment(point, point)],
        });
    }
}
/**
 * Create two polygon to tween between curve
 * @param from - source curve
 * @param to - targe curve
 * @param precision - desired distance between two point
 * @param round - amount of decimal when rounding
 * @returns list that contain list of polygon before and after tween
 */
function getInterpolationPolygon(from, to, precision, round) {
    const fromSub = splitCurve(from);
    const toSub = splitCurve(to);
    if (fromSub.length < toSub.length)
        balanceSubcurves(fromSub, toSub);
    else
        balanceSubcurves(toSub, fromSub);
    return fromSub.map((sub, i) => getSubcurveInterpolationPolygon(sub, toSub[i], precision, round));
}
/**
 * Add curve into another curve
 * @param target - target curve
 * @param source - curve to add
 */
function addCurveToCurve(target, source) {
    const { segments, arcLength, minSin } = source;
    target.segments.push(...segments);
    target.arcLength += arcLength;
    target.minSin = Math.min(target.minSin, minSin);
}
/**
 * Interpolate between two polygon points.
 * @param from - source polygon points
 * @param to - target polygon points
 * @param value - interpolation progress
 * @returns - new polygon points
 */
export function polygonPointsLerp(from, to, value) {
    const points = [];
    if (value === 0)
        return [...from];
    if (value === 1)
        return [...to];
    for (let i = 0; i < from.length; i++) {
        const a = from[i];
        const b = to[i];
        points.push(Vector2.lerp(a, b, value));
    }
    return points;
}
/**
 * Create interpolator to tween between two curve
 * @param a - source curve
 * @param b - target curve
 * @returns - curve interpolator
 */
export function createCurveProfileLerp(a, b) {
    const interpolations = getInterpolationPolygon(a, b, 5, 4);
    return (progress) => {
        const curve = {
            segments: [],
            arcLength: 0,
            minSin: 1,
        };
        for (const { from, to } of interpolations) {
            const points = polygonPointsLerp(from.points, to.points, progress);
            addCurveToCurve(curve, getPolylineProfile(points, 0, false));
        }
        return curve;
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY3JlYXRlQ3VydmVQcm9maWxlTGVycC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY3VydmVzL2NyZWF0ZUN1cnZlUHJvZmlsZUxlcnAudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLE9BQU8sRUFBQyxNQUFNLHFCQUFxQixDQUFDO0FBRTVDLE9BQU8sRUFBQyxXQUFXLEVBQUMsTUFBTSxlQUFlLENBQUM7QUFDMUMsT0FBTyxFQUFDLGtCQUFrQixFQUFDLE1BQU0sc0JBQXNCLENBQUM7QUFDeEQsT0FBTyxFQUFDLGtCQUFrQixFQUFDLE1BQU0sc0JBQXNCLENBQUM7QUFnQnhEOzs7O0dBSUc7QUFFSCxTQUFTLE1BQU0sQ0FBQyxNQUFpQixFQUFFLFNBQWlCO0lBQ2xELEtBQUssSUFBSSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRyxNQUFNLENBQUMsTUFBTSxHQUFHLENBQUMsRUFBRSxDQUFDLEVBQUUsRUFBRTtRQUMxQyxNQUFNLENBQUMsR0FBRyxNQUFNLENBQUMsQ0FBQyxDQUFDLENBQUM7UUFDcEIsSUFBSSxDQUFDLEdBQUcsTUFBTSxDQUFDLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztRQUN0QixPQUFPLENBQUMsQ0FBQyxHQUFHLENBQUMsQ0FBQyxDQUFDLENBQUMsU0FBUyxHQUFHLFNBQVMsRUFBRTtZQUNyQyxDQUFDLEdBQUcsT0FBTyxDQUFDLElBQUksQ0FBQyxDQUFDLEVBQUUsQ0FBQyxFQUFFLEdBQUcsQ0FBQyxDQUFDO1lBQzVCLE1BQU0sQ0FBQyxNQUFNLENBQUMsQ0FBQyxHQUFHLENBQUMsRUFBRSxDQUFDLEVBQUUsQ0FBQyxDQUFDLENBQUM7U0FDNUI7S0FDRjtBQUNILENBQUM7QUFFRDs7Ozs7R0FLRztBQUVILFNBQVMsa0JBQWtCLENBQ3pCLEtBQXNCLEVBQ3RCLFNBQWlCO0lBRWpCLE1BQU0sTUFBTSxHQUFjLEVBQUUsQ0FBQztJQUU3QixJQUFJLFFBQVEsR0FB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