var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { BBox, Vector2, } from '@motion-canvas/core';
import { getPolylineProfile } from '../curves';
import { computed, initial, signal } from '../decorators';
import { drawPolygon } from '../utils';
import { Curve } from './Curve';
/**
 * A node for drawing regular polygons.
 *
 * @remarks
 * This node can be used to render shapes such as: triangle, pentagon,
 * hexagon and more.
 *
 * Note that the polygon is inscribed in a circle defined by the height
 * and width. If height and width are unequal, the polygon is inscribed
 * in the resulting ellipse.
 *
 * Since the polygon is inscribed in the circle, the actual displayed
 * height and width may differ somewhat from the bounding rectangle. This
 * will be particularly noticeable if the number of sides is low, e.g. for a
 * triangle.
 *
 * @preview
 * ```tsx editor
 * // snippet Polygon
 * import {makeScene2D, Polygon} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const ref = createRef<Polygon>();
 *   view.add(
 *     <Polygon
 *       ref={ref}
 *       sides={6}
 *       size={160}
 *       fill={'lightseagreen'}
 *     />
 *   );
 *
 *   yield* ref().sides(3, 2).to(6, 2);
 * });
 *
 * // snippet Pentagon outline
 * import {makeScene2D, Polygon} from '@motion-canvas/2d';
 *
 * export default makeScene2D(function* (view) {
 *   view.add(
 *     <Polygon
 *       sides={5}
 *       size={160}
 *       radius={30}
 *       stroke={'lightblue'}
 *       lineWidth={8}
 *     />
 *   );
 * });
 *
 * // snippet Accessing vertex data
 * import {Circle, Polygon, makeScene2D} from '@motion-canvas/2d';
 * import {createRef, range} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const polygon = createRef<Polygon>();
 *   view.add(
 *     <Polygon ref={polygon} sides={3} lineWidth={4} stroke={'white'} size={160}>
 *       {range(6).map(index => (
 *         <Circle
 *           fill={'white'}
 *           size={20}
 *           position={() => polygon().vertex(index)}
 *           opacity={() => polygon().vertexCompletion(index)}
 *         />
 *       ))}
 *     </Polygon>,
 *   );
 *
 *   yield* polygon().sides(6, 2).wait(0.5).back(2);
 * });
 * ```
 */
export class Polygon extends Curve {
    constructor(props) {
        super(props);
    }
    /**
     * Get the position of the nth vertex in the local space of this polygon.
     *
     * @param index - The index of the vertex.
     */
    vertex(index) {
        const size = this.computedSize().scale(0.5);
        const theta = (index * 2 * Math.PI) / this.sides();
        const direction = Vector2.fromRadians(theta).perpendicular;
        return direction.mul(size);
    }
    /**
     * Get the completion of the nth vertex.
     *
     * @remarks
     * The completion is a value between `0` and `1` that describes how the given
     * vertex partakes in the polygon.
     *
     * For integer values of {@link sides}, the completion is simply `1` for
     * each index making up the polygon and `0` for any other index. If `sides`
     * includes a fraction, the last index of the polygon will have a completion
     * equal to said fraction.
     *
     * Check out the {@link Polygon | Accessing vertex data} example for a
     * demonstration.
     *
     * @param index - The index of the vertex.
     */
    vertexCompletion(index) {
        const sides = this.sides();
        if (index < 0 || index > sides) {
            return 0;
        }
        if (index < sides - 1) {
            return 1;
        }
        return sides - index;
    }
    profile() {
        const sides = this.sides();
        const radius = this.radius();
        const points = [];
        const size = this.computedSize().scale(0.5);
        for (let i = 0; i < sides; i++) {
            const theta = (i * 2 * Math.PI) / sides;
            const direction = Vector2.fromRadians(theta).perpendicular;
            points.push(direction.mul(size));
        }
        return getPolylineProfile(points, radius, true);
    }
    desiredSize() {
        return {
            x: this.width.context.getter(),
            y: this.height.context.getter(),
        };
    }
    offsetComputedLayout(box) {
        return box;
    }
    childrenBBox() {
        return BBox.fromSizeCentered(this.computedSize());
    }
    requiresProfile() {
        return super.requiresProfile() || this.radius() > 0;
    }
    getPath() {
        if (this.requiresProfile()) {
            return this.curveDrawingInfo().path;
        }
        return this.createPath();
    }
    getRipplePath() {
        return this.createPath(this.rippleSize());
    }
    createPath(expand = 0) {
        const path = new Path2D();
        const sides = this.sides();
        const box = BBox.fromSizeCentered(this.size()).expand(expand);
        drawPolygon(path, box, sides);
        return path;
    }
}
__decorate([
    initial(6),
    signal()
], Polygon.prototype, "sides", void 0);
__decorate([
    initial(0),
    signal()
], Polygon.prototype, "radius", void 0);
__decorate([
    computed()
], Polygon.prototype, "profile", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUG9seWdvbi5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29tcG9uZW50cy9Qb2x5Z29uLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7OztBQUFBLE9BQU8sRUFDTCxJQUFJLEVBSUosT0FBTyxHQUNSLE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUFlLGtCQUFrQixFQUFDLE1BQU0sV0FBVyxDQUFDO0FBQzNELE9BQU8sRUFBQyxRQUFRLEVBQUUsT0FBTyxFQUFFLE1BQU0sRUFBQyxNQUFNLGVBQWUsQ0FBQztBQUV4RCxPQUFPLEVBQUMsV0FBVyxFQUFDLE1BQU0sVUFBVSxDQUFDO0FBQ3JDLE9BQU8sRUFBQyxLQUFLLEVBQWEsTUFBTSxTQUFTLENBQUM7QUFhMUM7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7R0F5RUc7QUFDSCxNQUFNLE9BQU8sT0FBUSxTQUFRLEtBQUs7SUF3Q2hDLFlBQW1CLEtBQW1CO1FBQ3BDLEtBQUssQ0FBQyxLQUFLLENBQUMsQ0FBQztJQUNmLENBQUM7SUFFRDs7OztPQUlHO0lBQ0ksTUFBTSxDQUFDLEtBQWE7UUFDekIsTUFBTSxJQUFJLEdBQUcsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsQ0FBQztRQUM1QyxNQUFNLEtBQUssR0FBRyxDQUFDLEtBQUssR0FBRyxDQUFDLEdBQUcsSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztRQUNuRCxNQUFNLFNBQVMsR0FBRyxPQUFPLENBQUMsV0FBVyxDQUFDLEtBQUssQ0FBQyxDQUFDLGFBQWEsQ0FBQztRQUMzRCxPQUFPLFNBQVMsQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLENBQUM7SUFDN0IsQ0FBQztJQUVEOzs7Ozs7Ozs7Ozs7Ozs7O09BZ0JHO0lBQ0ksZ0JBQWdC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