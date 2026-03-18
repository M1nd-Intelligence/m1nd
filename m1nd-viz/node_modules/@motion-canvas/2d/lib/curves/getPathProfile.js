import { Vector2, clamp } from '@motion-canvas/core';
import parse from 'parse-svg-path';
import { ArcSegment } from './ArcSegment';
import { CubicBezierSegment } from './CubicBezierSegment';
import { LineSegment } from './LineSegment';
import { QuadBezierSegment } from './QuadBezierSegment';
function addSegmentToProfile(profile, segment) {
    profile.segments.push(segment);
    profile.arcLength += segment.arcLength;
}
function getArg(command, argumentIndex) {
    return command[argumentIndex + 1];
}
function getVector2(command, argumentIndex) {
    return new Vector2(command[argumentIndex + 1], command[argumentIndex + 2]);
}
function getPoint(command, argumentIndex, isRelative, currentPoint) {
    const point = getVector2(command, argumentIndex);
    return isRelative ? currentPoint.add(point) : point;
}
function reflectControlPoint(control, currentPoint) {
    return currentPoint.add(currentPoint.sub(control));
}
function updateMinSin(profile) {
    for (let i = 0; i < profile.segments.length; i++) {
        const segmentA = profile.segments[i];
        const segmentB = profile.segments[(i + 1) % profile.segments.length];
        // In cubic bezier this equal p2.sub(p3)
        const startVector = segmentA.getPoint(1).tangent.scale(-1);
        // In cubic bezier this equal p1.sub(p0)
        const endVector = segmentB.getPoint(0).tangent;
        const dot = startVector.dot(endVector);
        const angleBetween = Math.acos(clamp(-1, 1, dot));
        const angleSin = Math.sin(angleBetween / 2);
        profile.minSin = Math.min(profile.minSin, Math.abs(angleSin));
    }
}
export function getPathProfile(data) {
    const profile = {
        segments: [],
        arcLength: 0,
        minSin: 1,
    };
    const segments = parse(data);
    let currentPoint = new Vector2(0, 0);
    let firstPoint = null;
    for (const segment of segments) {
        const command = segment[0].toLowerCase();
        const isRelative = segment[0] === command;
        if (command === 'm') {
            currentPoint = getPoint(segment, 0, isRelative, currentPoint);
            firstPoint = currentPoint;
        }
        else if (command === 'l') {
            const nextPoint = getPoint(segment, 0, isRelative, currentPoint);
            addSegmentToProfile(profile, new LineSegment(currentPoint, nextPoint));
            currentPoint = nextPoint;
        }
        else if (command === 'h') {
            const x = getArg(segment, 0);
            const nextPoint = isRelative
                ? currentPoint.addX(x)
                : new Vector2(x, currentPoint.y);
            addSegmentToProfile(profile, new LineSegment(currentPoint, nextPoint));
            currentPoint = nextPoint;
        }
        else if (command === 'v') {
            const y = getArg(segment, 0);
            const nextPoint = isRelative
                ? currentPoint.addY(y)
                : new Vector2(currentPoint.x, y);
            addSegmentToProfile(profile, new LineSegment(currentPoint, nextPoint));
            currentPoint = nextPoint;
        }
        else if (command === 'q') {
            const controlPoint = getPoint(segment, 0, isRelative, currentPoint);
            const nextPoint = getPoint(segment, 2, isRelative, currentPoint);
            addSegmentToProfile(profile, new QuadBezierSegment(currentPoint, controlPoint, nextPoint));
            currentPoint = nextPoint;
        }
        else if (command === 't') {
            const lastSegment = profile.segments.at(-1);
            const controlPoint = lastSegment instanceof QuadBezierSegment
                ? reflectControlPoint(lastSegment.p1, currentPoint)
                : currentPoint;
            const nextPoint = getPoint(segment, 0, isRelative, currentPoint);
            addSegmentToProfile(profile, new QuadBezierSegment(currentPoint, controlPoint, nextPoint));
            currentPoint = nextPoint;
        }
        else if (command === 'c') {
            const startControlPoint = getPoint(segment, 0, isRelative, currentPoint);
            const endControlPoint = getPoint(segment, 2, isRelative, currentPoint);
            const nextPoint = getPoint(segment, 4, isRelative, currentPoint);
            addSegmentToProfile(profile, new CubicBezierSegment(currentPoint, startControlPoint, endControlPoint, nextPoint));
            currentPoint = nextPoint;
        }
        else if (command === 's') {
            const lastSegment = profile.segments.at(-1);
            const startControlPoint = lastSegment instanceof CubicBezierSegment
                ? reflectControlPoint(lastSegment.p2, currentPoint)
                : currentPoint;
            const endControlPoint = getPoint(segment, 0, isRelative, currentPoint);
            const nextPoint = getPoint(segment, 2, isRelative, currentPoint);
            addSegmentToProfile(profile, new CubicBezierSegment(currentPoint, startControlPoint, endControlPoint, nextPoint));
            currentPoint = nextPoint;
        }
        else if (command === 'a') {
            const radius = getVector2(segment, 0);
            const angle = getArg(segment, 2);
            const largeArcFlag = getArg(segment, 3);
            const sweepFlag = getArg(segment, 4);
            const nextPoint = getPoint(segment, 5, isRelative, currentPoint);
            addSegmentToProfile(profile, new ArcSegment(currentPoint, radius, angle, largeArcFlag, sweepFlag, nextPoint));
            currentPoint = nextPoint;
        }
        else if (command === 'z') {
            if (!firstPoint)
                continue;
            if (currentPoint.equals(firstPoint))
                continue;
            addSegmentToProfile(profile, new LineSegment(currentPoint, firstPoint));
            currentPoint = firstPoint;
        }
    }
    updateMinSin(profile);
    return profile;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZ2V0UGF0aFByb2ZpbGUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2N1cnZlcy9nZXRQYXRoUHJvZmlsZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUMsT0FBTyxFQUFFLEtBQUssRUFBQyxNQUFNLHFCQUFxQixDQUFDO0FBQ25ELE9BQU8sS0FBb0IsTUFBTSxnQkFBZ0IsQ0FBQztBQUNsRCxPQUFPLEVBQUMsVUFBVSxFQUFDLE1BQU0sY0FBYyxDQUFDO0FBQ3hDLE9BQU8sRUFBQyxrQkFBa0IsRUFBQyxNQUFNLHNCQUFzQixDQUFDO0FBRXhELE9BQU8sRUFBQyxXQUFXLEVBQUMsTUFBTSxlQUFlLENBQUM7QUFDMUMsT0FBTyxFQUFDLGlCQUFpQixFQUFDLE1BQU0scUJBQXFCLENBQUM7QUFHdEQsU0FBUyxtQkFBbUIsQ0FBQyxPQUFxQixFQUFFLE9BQWdCO0lBQ2xFLE9BQU8sQ0FBQyxRQUFRLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxDQUFDO0lBQy9CLE9BQU8sQ0FBQyxTQUFTLElBQUksT0FBTyxDQUFDLFNBQVMsQ0FBQztBQUN6QyxDQUFDO0FBRUQsU0FBUyxNQUFNLENBQUMsT0FBb0IsRUFBRSxhQUFxQjtJQUN6RCxPQUFPLE9BQU8sQ0FBQyxhQUFhLEdBQUcsQ0FBQyxDQUFXLENBQUM7QUFDOUMsQ0FBQztBQUVELFNBQVMsVUFBVSxDQUFDLE9BQW9CLEVBQUUsYUFBcUI7SUFDN0QsT0FBTyxJQUFJLE9BQU8sQ0FDaEIsT0FBTyxDQUFDLGFBQWEsR0FBRyxDQUFDLENBQVcsRUFDcEMsT0FBTyxDQUFDLGFBQWEsR0FBRyxDQUFDLENBQVcsQ0FDckMsQ0FBQztBQUNKLENBQUM7QUFFRCxTQUFTLFFBQVEsQ0FDZixPQUFvQixFQUNwQixhQUFxQixFQUNyQixVQUFtQixFQUNuQixZQUFxQjtJQUVyQixNQUFNLEtBQUssR0FBRyxVQUFVLENBQUMsT0FBTyxFQUFFLGFBQWEsQ0FBQyxDQUFD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