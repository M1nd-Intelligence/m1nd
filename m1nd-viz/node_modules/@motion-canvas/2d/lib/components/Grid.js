var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { map, } from '@motion-canvas/core';
import { initial, nodeName, signal, vector2Signal } from '../decorators';
import { Shape } from './Shape';
/**
 * A node for drawing a two-dimensional grid.
 *
 * @preview
 * ```tsx editor
 * import {Grid, makeScene2D} from '@motion-canvas/2d';
 * import {all, createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const grid = createRef<Grid>();
 *
 *   view.add(
 *     <Grid
 *       ref={grid}
 *       width={'100%'}
 *       height={'100%'}
 *       stroke={'#666'}
 *       start={0}
 *       end={1}
 *     />,
 *   );
 *
 *   yield* all(
 *     grid().end(0.5, 1).to(1, 1).wait(1),
 *     grid().start(0.5, 1).to(0, 1).wait(1),
 *   );
 * });
 * ```
 */
let Grid = class Grid extends Shape {
    constructor(props) {
        super(props);
    }
    drawShape(context) {
        context.save();
        this.applyStyle(context);
        this.drawRipple(context);
        const spacing = this.spacing();
        const size = this.computedSize().scale(0.5);
        const steps = size.div(spacing).floored;
        for (let x = -steps.x; x <= steps.x; x++) {
            const [from, to] = this.mapPoints(-size.height, size.height);
            context.beginPath();
            context.moveTo(spacing.x * x, from);
            context.lineTo(spacing.x * x, to);
            context.stroke();
        }
        for (let y = -steps.y; y <= steps.y; y++) {
            const [from, to] = this.mapPoints(-size.width, size.width);
            context.beginPath();
            context.moveTo(from, spacing.y * y);
            context.lineTo(to, spacing.y * y);
            context.stroke();
        }
        context.restore();
    }
    mapPoints(start, end) {
        let from = map(start, end, this.start());
        let to = map(start, end, this.end());
        if (to < from) {
            [from, to] = [to, from];
        }
        return [from, to];
    }
};
__decorate([
    initial(80),
    vector2Signal('spacing')
], Grid.prototype, "spacing", void 0);
__decorate([
    initial(0),
    signal()
], Grid.prototype, "start", void 0);
__decorate([
    initial(1),
    signal()
], Grid.prototype, "end", void 0);
Grid = __decorate([
    nodeName('Grid')
], Grid);
export { Grid };
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiR3JpZC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29tcG9uZW50cy9HcmlkLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7OztBQUFBLE9BQU8sRUFLTCxHQUFHLEdBQ0osTUFBTSxxQkFBcUIsQ0FBQztBQUM3QixPQUFPLEVBQUMsT0FBTyxFQUFFLFFBQVEsRUFBRSxNQUFNLEVBQUUsYUFBYSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBQ3ZFLE9BQU8sRUFBQyxLQUFLLEVBQWEsTUFBTSxTQUFTLENBQUM7QUFpQjFDOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0dBNEJHO0FBRUksSUFBTSxJQUFJLEdBQVYsTUFBTSxJQUFLLFNBQVEsS0FBSztJQWtDN0IsWUFBbUIsS0FBZ0I7UUFDakMsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFDO0lBQ2YsQ0FBQztJQUVrQixTQUFTLENBQUMsT0FBaUM7UUFDNUQsT0FBTyxDQUFDLElBQUksRUFBRSxDQUFDO1FBQ2YsSUFBSSxDQUFDLFVBQVUsQ0FBQyxPQUFPLENBQUMsQ0FBQztRQUN6QixJQUFJLENBQUMsVUFBVSxDQUFDLE9BQU8sQ0FBQyxDQUFDO1FBRXpCLE1BQU0sT0FBTyxHQUFHLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQztRQUMvQixNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsWUFBWSxFQUFFLENBQUMsS0FBSyxDQUFDLEdBQUcsQ0FBQyxDQUFDO1FBQzVDLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsT0FBTyxDQUFDLENBQUMsT0FBTyxDQUFDO1FBRXhDLEtBQUssSUFBSSxDQUFDLEdBQUcsQ0FBQyxLQUFLLENBQUMsQ0FBQyxFQUFFLENBQUMsSUFBSSxLQUFLLENBQUMsQ0FBQyxFQUFFLENBQUMsRUFBRSxFQUFFO1lBQ3hDLE1BQU0sQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUFDLEdBQUcsSUFBSSxDQUFDLFNBQVMsQ0FBQyxDQUFDLElBQUksQ0FBQyxNQUFNLEVBQUUsSUFBSSxDQUFDLE1BQU0sQ0FBQyxDQUFDO1lBRTdELE9BQU8sQ0FBQyxTQUFTLEVBQUUsQ0FBQztZQUNwQixPQUFPLENBQUMsTUFBTSxDQUFDLE9BQU8sQ0FBQyxDQUFDLEdBQUcsQ0FBQyxFQUFFLElBQUksQ0FBQyxDQUFDO1lBQ3BDLE9BQU8sQ0FBQyxNQUFNLENBQUMsT0FBTyxDQUFDLENBQUMsR0FBRyxDQUFDLEVBQUUsRUFBRSxDQUFDLENBQUM7WUFDbEMsT0FBTyxDQUFDLE1BQU0sRUFBRSxDQUFDO1NBQ2xCO1FBRUQsS0FBSyxJQUFJLENBQUMsR0FBRyxDQUFDLEtBQUssQ0FBQyxDQUFDLEVBQUUsQ0FBQyxJQUFJLEtBQUssQ0FBQyxDQUFDLEVBQUUsQ0FBQyxFQUFFLEVBQUU7WUFDeEMsTUFBTSxDQUFDLElBQUksRUFBRSxFQUFFLENBQUMsR0FBRyxJQUFJLENBQUMsU0FBUyxDQUFDLENBQUMsSUFBSSxDQUFDLEtBQUssRUFBRSxJQUFJLENBQUMsS0FBSyxDQUFDLENBQUM7WUFFM0QsT0FBTyxDQUFDLFNBQVMsRUFBRSxDQUFDO1lBQ3BCLE9BQU8sQ0FBQyxNQUFNLENBQUMsSUFBSSxFQUFFLE9BQU8sQ0FBQyxDQUFDLEdBQUcsQ0FBQyxDQUFDLENBQUM7WUFDcEMsT0FBTyxDQUFDLE1BQU0sQ0FBQyxFQUFFLEVBQUUsT0FBTyxDQUFDLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztZQUNsQyxPQUFPLENBQUMsTUFBTSxFQUFFLENBQUM7U0FDbEI7UUFFRCxPQUFPLENBQUMsT0FBTyxFQUFFLENBQUM7SUFDcEIsQ0FBQztJQUVPLFNBQVMsQ0FBQyxLQUFhLEVBQUUsR0FBVztRQUMxQyxJQUFJLElBQUksR0FBRyxHQUFHLENBQUMsS0FBSyxFQUFFLEdBQUcsRUFBRSxJQUFJLENBQUMsS0FBSyxFQUFFLENBQUMsQ0FBQztRQUN6QyxJQUFJLEVBQUUsR0FBRyxHQUFHLENBQUMsS0FBSyxFQUFFLEdBQUcsRUFBRSxJQUFJLENBQUMsR0FBRyxFQUFFLENBQUMsQ0FBQztRQUVyQyxJQUFJLEVBQUUsR0FBRyxJQUFJLEVBQUU7WUFDYixDQUFDLElBQUksRUFBRSxFQUFFLENBQUMsR0FBRyxDQUFDLEVBQUUsRUFBRSxJQUFJLENBQUMsQ0FBQztTQUN6QjtRQUVELE9BQU8sQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUFDLENBQUM7SUFDcEIsQ0FBQztDQUNGLENBQUE7QUF4RXlCO0lBRnZCLE9BQU8sQ0FBQyxFQUFFLENBQUM7SUFDWCxhQUFhLENBQUMsU0FBUyxDQUFDO3FDQUM0QjtBQWE3QjtJQUZ2QixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFO21DQUNpRDtBQWFsQztJQUZ2QixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFO2lDQUMrQztBQWhDN0MsSUFBSTtJQURoQixRQUFRLENBQUMsTUFBTSxDQUFDO0dBQ0osSUFBSSxDQThFaEIifQ==