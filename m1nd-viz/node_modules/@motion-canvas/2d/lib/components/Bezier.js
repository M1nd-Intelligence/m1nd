var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { BBox } from '@motion-canvas/core';
import { computed } from '../decorators';
import { arc, drawLine, drawPivot, moveTo } from '../utils';
import { Curve } from './Curve';
export class Bezier extends Curve {
    profile() {
        const segment = this.segment();
        return {
            segments: [segment],
            arcLength: segment.arcLength,
            minSin: 0,
        };
    }
    childrenBBox() {
        return BBox.fromPoints(...this.segment().points);
    }
    desiredSize() {
        return this.segment().getBBox().size;
    }
    offsetComputedLayout(box) {
        box.position = box.position.sub(this.segment().getBBox().center);
        return box;
    }
    drawOverlay(context, matrix) {
        const size = this.computedSize();
        const box = this.childrenBBox().transformCorners(matrix);
        const offset = size.mul(this.offset()).scale(0.5).transformAsPoint(matrix);
        const overlayInfo = this.overlayInfo(matrix);
        context.lineWidth = 1;
        context.strokeStyle = 'white';
        context.fillStyle = 'white';
        // Draw the curve itself first so everything else gets drawn on top of it
        context.stroke(overlayInfo.curve);
        context.fillStyle = 'white';
        context.globalAlpha = 0.5;
        context.beginPath();
        context.stroke(overlayInfo.handleLines);
        context.globalAlpha = 1;
        context.lineWidth = 2;
        // Draw start and end points
        for (const point of [overlayInfo.startPoint, overlayInfo.endPoint]) {
            moveTo(context, point);
            context.beginPath();
            arc(context, point, 4);
            context.closePath();
            context.stroke();
            context.fill();
        }
        // Draw the control points
        context.fillStyle = 'black';
        for (const point of overlayInfo.controlPoints) {
            moveTo(context, point);
            context.beginPath();
            arc(context, point, 4);
            context.closePath();
            context.fill();
            context.stroke();
        }
        // Draw the offset marker
        context.lineWidth = 1;
        context.beginPath();
        drawPivot(context, offset);
        context.stroke();
        // Draw the bounding box
        context.beginPath();
        drawLine(context, box);
        context.closePath();
        context.stroke();
    }
}
__decorate([
    computed()
], Bezier.prototype, "childrenBBox", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQmV6aWVyLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL0Jlemllci50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7QUFBQSxPQUFPLEVBQUMsSUFBSSxFQUE2QixNQUFNLHFCQUFxQixDQUFDO0FBR3JFLE9BQU8sRUFBQyxRQUFRLEVBQUMsTUFBTSxlQUFlLENBQUM7QUFFdkMsT0FBTyxFQUFDLEdBQUcsRUFBRSxRQUFRLEVBQUUsU0FBUyxFQUFFLE1BQU0sRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUMxRCxPQUFPLEVBQUMsS0FBSyxFQUFDLE1BQU0sU0FBUyxDQUFDO0FBVTlCLE1BQU0sT0FBZ0IsTUFBTyxTQUFRLEtBQUs7SUFDeEIsT0FBTztRQUNyQixNQUFNLE9BQU8sR0FBRyxJQUFJLENBQUMsT0FBTyxFQUFFLENBQUM7UUFDL0IsT0FBTztZQUNMLFFBQVEsRUFBRSxDQUFDLE9BQU8sQ0FBQztZQUNuQixTQUFTLEVBQUUsT0FBTyxDQUFDLFNBQVM7WUFDNUIsTUFBTSxFQUFFLENBQUM7U0FDVixDQUFDO0lBQ0osQ0FBQztJQU9TLFlBQVk7UUFDcEIsT0FBTyxJQUFJLENBQUMsVUFBVSxDQUFDLEdBQUcsSUFBSSxDQUFDLE9BQU8sRUFBRSxDQUFDLE1BQU0sQ0FBQyxDQUFDO0lBQ25ELENBQUM7SUFFa0IsV0FBVztRQUM1QixPQUFPLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxPQUFPLEVBQUUsQ0FBQyxJQUFJLENBQUM7SUFDdkMsQ0FBQztJQUVrQixvQkFBb0IsQ0FBQyxHQUFTO1FBQy9DLEdBQUcsQ0FBQyxRQUFRLEdBQUcsR0FBRyxDQUFDLFFBQVEsQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLE9BQU8sRUFBRSxDQUFDLE9BQU8sRUFBRSxDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBQ2pFLE9BQU8sR0FBRyxDQUFDO0lBQ2IsQ0FBQztJQUVlLFdBQVcsQ0FDekIsT0FBaUMsRUFDakMsTUFB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