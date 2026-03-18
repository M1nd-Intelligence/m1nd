var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { Vector2, } from '@motion-canvas/core';
import { cloneable, compound, computed, initial, parser, signal, wrapper, } from '../decorators';
import { Node } from './Node';
/**
 * A node representing a knot of a {@link Spline}.
 */
export class Knot extends Node {
    get startHandleAuto() {
        return this.auto.startHandle;
    }
    get endHandleAuto() {
        return this.auto.endHandle;
    }
    constructor(props) {
        super(props.startHandle === undefined && props.endHandle === undefined
            ? { auto: 1, ...props }
            : props);
    }
    points() {
        const hasExplicitHandles = !this.startHandle.isInitial() || !this.endHandle.isInitial();
        const startHandle = hasExplicitHandles ? this.startHandle() : Vector2.zero;
        const endHandle = hasExplicitHandles ? this.endHandle() : Vector2.zero;
        return {
            position: this.position(),
            startHandle: startHandle.transformAsPoint(this.localToParent()),
            endHandle: endHandle.transformAsPoint(this.localToParent()),
            auto: { start: this.startHandleAuto(), end: this.endHandleAuto() },
        };
    }
    getDefaultEndHandle() {
        return this.startHandle().flipped;
    }
    getDefaultStartHandle() {
        return this.endHandle().flipped;
    }
}
__decorate([
    wrapper(Vector2),
    signal()
], Knot.prototype, "startHandle", void 0);
__decorate([
    wrapper(Vector2),
    signal()
], Knot.prototype, "endHandle", void 0);
__decorate([
    cloneable(false),
    initial(() => ({ startHandle: 0, endHandle: 0 })),
    parser((value) => {
        if (typeof value === 'object' && !Array.isArray(value)) {
            return value;
        }
        if (typeof value === 'number') {
            value = [value, value];
        }
        return { startHandle: value[0], endHandle: value[1] };
    }),
    compound({ startHandle: 'startHandleAuto', endHandle: 'endHandleAuto' })
], Knot.prototype, "auto", void 0);
__decorate([
    computed()
], Knot.prototype, "points", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiS25vdC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29tcG9uZW50cy9Lbm90LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7OztBQUFBLE9BQU8sRUFJTCxPQUFPLEdBRVIsTUFBTSxxQkFBcUIsQ0FBQztBQUU3QixPQUFPLEVBQ0wsU0FBUyxFQUNULFFBQVEsRUFDUixRQUFRLEVBQ1IsT0FBTyxFQUNQLE1BQU0sRUFDTixNQUFNLEVBQ04sT0FBTyxHQUNSLE1BQU0sZUFBZSxDQUFDO0FBQ3ZCLE9BQU8sRUFBQyxJQUFJLEVBQVksTUFBTSxRQUFRLENBQUM7QUE4QnZDOztHQUVHO0FBQ0gsTUFBTSxPQUFPLElBQUssU0FBUSxJQUFJO0lBb0U1QixJQUFXLGVBQWU7UUFDeEIsT0FBTyxJQUFJLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FBQztJQUMvQixDQUFDO0lBQ0QsSUFBVyxhQUFhO1FBQ3RCLE9BQU8sSUFBSSxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUM7SUFDN0IsQ0FBQztJQUVELFlBQW1CLEtBQWdCO1FBQ2pDLEtBQUssQ0FDSCxLQUFLLENBQUMsV0FBVyxLQUFLLFNBQVMsSUFBSSxLQUFLLENBQUMsU0FBUyxLQUFLLFNBQVM7WUFDOUQsQ0FBQyxDQUFDLEVBQUMsSUFBSSxFQUFFLENBQUMsRUFBRSxHQUFHLEtBQUssRUFBQztZQUNyQixDQUFDLENBQUMsS0FBSyxDQUNWLENBQUM7SUFDSixDQUFDO0lBR00sTUFBTTtRQUNYLE1BQU0sa0JBQWtCLEdBQ3RCLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FBQyxTQUFTLEVBQUUsSUFBSSxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUMsU0FBUyxFQUFFLENBQUM7UUFDL0QsTUFBTSxXQUFXLEdBQUcsa0JBQWtCLENBQUMsQ0FBQyxDQUFDLElBQUksQ0FBQyxXQUFXLEVBQUUsQ0FBQyxDQUFDLENBQUMsT0FBTyxDQUFDLElBQUksQ0FBQztRQUMzRSxNQUFNLFNBQVMsR0FBRyxrQkFBa0IsQ0FBQyxDQUFDLENBQUMsSUFBSSxDQUFDLFNBQVMsRUFBRSxDQUFDLENBQUMsQ0FBQyxPQUFPLENBQUMsSUFBSSxDQUFDO1FBRXZFLE9BQU87WUFDTCxRQUFRLEVBQUUsSUFBSSxDQUFDLFFBQVEsRUFBRTtZQUN6QixXQUFXLEVBQUUsV0FBVyxDQUFDLGdCQUFnQixDQUFDLElBQUksQ0FBQyxhQUFhLEVBQUUsQ0FBQztZQUMvRCxTQUFTLEVBQUUsU0FBUyxDQUFDLGdCQUFnQixDQUFDLElBQUksQ0FBQyxhQUFhLEVBQUUsQ0FBQztZQUMzRCxJQUFJLEVBQUUsRUFBQyxLQUFLLEVBQUUsSUFBSSxDQUFDLGVBQWUsRUFBRSxFQUFFLEdBQUcsRUFBRSxJQUFJLENBQUMsYUFBYSxFQUFFLEVBQUM7U0FDakUsQ0FBQztJQUNKLENBQUM7SUFFTyxtQkFBbUI7UUFDekIsT0FBTyxJQUFJLENBQUMsV0FBVyxFQUFFLENBQUMsT0FBTyxDQUFDO0lBQ3BDLENBQUM7SUFFTyxxQkFBcUI7UUFDM0IsT0FBTyxJQUFJLENBQUMsU0FBUyxFQUFFLENBQUMsT0FBTyxDQUFDO0lBQ2xDLENBQUM7Q0FDRjtBQXBGeUI7SUFGdkIsT0FBTyxDQUFDLE9BQU8sQ0FBQztJQUNoQixNQUFNLEVBQUU7eUNBQ2dEO0FBc0JqQztJQUZ2QixPQUFPLENBQUMsT0FBTyxDQUFDO0lBQ2hCLE1BQU0sRUFBRTt1Q0FDOEM7QUF3Qi9CO0lBWnZCLFNBQVMsQ0FBQyxLQUFLLENBQUM7SUFDaEIsT0FBTyxDQUFDLEdBQUcsRUFBRSxDQUFDLENBQUMsRUFBQyxXQUFXLEVBQUUsQ0FBQyxFQUFFLFNBQVMsRUFBRSxDQUFDLEVBQUMsQ0FBQyxDQUFDO0lBQy9DLE1BQU0sQ0FBQyxDQUFDLEtBQXVCLEVBQUUsRUFBRTtRQUNsQyxJQUFJLE9BQU8sS0FBSyxLQUFLLFFBQVEsSUFBSSxDQUFDLEtBQUssQ0FBQyxPQUFPLENBQUMsS0FBSyxDQUFDLEVBQUU7WUFDdEQsT0FBTyxLQUFLLENBQUM7U0FDZDtRQUNELElBQUksT0FBTyxLQUFLLEtBQUssUUFBUSxFQUFFO1lBQzdCLEtBQUssR0FBRyxDQUFDLEtBQUssRUFBRSxLQUFLLENBQUMsQ0FBQztTQUN4QjtRQUNELE9BQU8sRUFBQyxXQUFXLEVBQUUsS0FBSyxDQUFDLENBQUMsQ0FBQyxFQUFFLFNBQVMsRUFBRSxLQUFLLENBQUMsQ0FBQyxDQUFDLEVBQUMsQ0FBQztJQUN0RCxDQUFDLENBQUM7SUFDRCxRQUFRLENBQUMsRUFBQyxXQUFXLEVBQUUsaUJBQWlCLEVBQUUsU0FBUyxFQUFFLGVBQWUsRUFBQyxDQUFDO2tDQUNwQjtBQWlCNUM7SUFETixRQUFRLEVBQUU7a0NBYVYifQ==