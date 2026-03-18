var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { Color, unwrap, } from '@motion-canvas/core';
import { computed } from '../decorators/computed';
import { initial, initializeSignals, signal } from '../decorators/signal';
import { vector2Signal } from '../decorators/vector2Signal';
export class Gradient {
    constructor(props) {
        initializeSignals(this, props);
    }
    canvasGradient(context) {
        let gradient;
        switch (this.type()) {
            case 'linear':
                gradient = context.createLinearGradient(this.from.x(), this.from.y(), this.to.x(), this.to.y());
                break;
            case 'conic':
                gradient = context.createConicGradient(this.angle(), this.from.x(), this.from.y());
                break;
            case 'radial':
                gradient = context.createRadialGradient(this.from.x(), this.from.y(), this.fromRadius(), this.to.x(), this.to.y(), this.toRadius());
                break;
        }
        for (const { offset, color } of this.stops()) {
            gradient.addColorStop(unwrap(offset), new Color(unwrap(color)).serialize());
        }
        return gradient;
    }
}
__decorate([
    initial('linear'),
    signal()
], Gradient.prototype, "type", void 0);
__decorate([
    vector2Signal('from')
], Gradient.prototype, "from", void 0);
__decorate([
    vector2Signal('to')
], Gradient.prototype, "to", void 0);
__decorate([
    initial(0),
    signal()
], Gradient.prototype, "angle", void 0);
__decorate([
    initial(0),
    signal()
], Gradient.prototype, "fromRadius", void 0);
__decorate([
    initial(0),
    signal()
], Gradient.prototype, "toRadius", void 0);
__decorate([
    initial([]),
    signal()
], Gradient.prototype, "stops", void 0);
__decorate([
    computed()
], Gradient.prototype, "canvasGradient", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiR3JhZGllbnQuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL3BhcnRpYWxzL0dyYWRpZW50LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7OztBQUFBLE9BQU8sRUFDTCxLQUFLLEVBTUwsTUFBTSxHQUNQLE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUFDLFFBQVEsRUFBQyxNQUFNLHdCQUF3QixDQUFDO0FBQ2hELE9BQU8sRUFBQyxPQUFPLEVBQUUsaUJBQWlCLEVBQUUsTUFBTSxFQUFDLE1BQU0sc0JBQXNCLENBQUM7QUFDeEUsT0FBTyxFQUFDLGFBQWEsRUFBQyxNQUFNLDZCQUE2QixDQUFDO0FBdUIxRCxNQUFNLE9BQU8sUUFBUTtJQXdCbkIsWUFBbUIsS0FBb0I7UUFDckMsaUJBQWlCLENBQUMsSUFBSSxFQUFFLEtBQUssQ0FBQyxDQUFDO0lBQ2pDLENBQUM7SUFHTSxjQUFjLENBQUMsT0FBaUM7UUFDckQsSUFBSSxRQUF3QixDQUFDO1FBQzdCLFFBQVEsSUFBSSxDQUFDLElBQUksRUFBRSxFQUFFO1lBQ25CLEtBQUssUUFBUTtnQkFDWCxRQUFRLEdBQUcsT0FBTyxDQUFDLG9CQUFvQixDQUNyQyxJQUFJLENBQUMsSUFBSSxDQUFDLENBQUMsRUFBRSxFQUNiLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxFQUFFLEVBQ2IsSUFBSSxDQUFDLEVBQUUsQ0FBQyxDQUFDLEVBQUUsRUFDWCxJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsRUFBRSxDQUNaLENBQUM7Z0JBQ0YsTUFBTTtZQUNSLEtBQUssT0FBTztnQkFDVixRQUFRLEdBQUcsT0FBTyxDQUFDLG1CQUFtQixDQUNwQyxJQUFJLENBQUMsS0FBSyxFQUFFLEVBQ1osSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDLEVBQUUsRUFDYixJQUFJLENBQUMsSUFBSSxDQUFDLENBQUMsRUFBRSxDQUNkLENBQUM7Z0JBQ0YsTUFBTTtZQUNSLEtBQUssUUFBUTtnQkFDWCxRQUFRLEdBQUcsT0FBTyxDQUFDLG9CQUFvQixDQUNyQyxJQUFJLENBQUMsSUFBSSxDQUFDLENBQUMsRUFBRSxFQUNiLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxFQUFFLEVBQ2IsSUFBSSxDQUFDLFVBQVUsRUFBRSxFQUNqQixJQUFJLENBQUMsRUFBRSxDQUFDLENBQUMsRUFBRSxFQUNYLElBQUksQ0FBQyxFQUFFLENBQUMsQ0FBQyxFQUFFLEVBQ1gsSUFBSSxDQUFDLFFBQVEsRUFBRSxDQUNoQixDQUFDO2dCQUNGLE1BQU07U0FDVDtRQUVELEtBQUssTUFBTSxFQUFDLE1BQU0sRUFBRSxLQUFLLEVBQUMsSUFBSSxJQUFJLENBQUMsS0FBSyxFQUFFLEVBQUU7WUFDMUMsUUFBUSxDQUFDLFlBQVksQ0FDbkIsTUFBTSxDQUFDLE1BQU0sQ0FBQyxFQUNkLElBQUksS0FBSyxDQUFDLE1BQU0sQ0FBQyxLQUFLLENBQUMsQ0FBQyxDQUFDLFNBQVMsRUFBRSxDQUNyQyxDQUFDO1NBQ0g7UUFFRCxPQUFPLFFBQVEsQ0FBQztJQUNsQixDQUFDO0NBQ0Y7QUFqRXlCO0lBRnZCLE9BQU8sQ0FBQyxRQUFRLENBQUM7SUFDakIsTUFBTSxFQUFFO3NDQUNzRDtBQUd2QztJQUR2QixhQUFhLENBQUMsTUFBTSxDQUFDO3NDQUM0QjtBQUcxQjtJQUR2QixhQUFhLENBQUMsSUFBSSxDQUFDO29DQUM0QjtBQUl4QjtJQUZ2QixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFO3VDQUNpRDtBQUdsQztJQUZ2QixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFOzRDQUNzRDtBQUd2QztJQUZ2QixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFOzBDQUNvRDtBQUdyQztJQUZ2QixPQUFPLENBQUMsRUFBRSxDQUFDO0lBQ1gsTUFBTSxFQUFFO3VDQUN5RDtBQU8zRDtJQUROLFFBQVEsRUFBRTs4Q0F1Q1YifQ==