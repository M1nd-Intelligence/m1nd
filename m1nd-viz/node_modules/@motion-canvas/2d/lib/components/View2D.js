var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var View2D_1;
import { PlaybackState, lazy } from '@motion-canvas/core';
import { initial, signal } from '../decorators';
import { nodeName } from '../decorators/nodeName';
import { useScene2D } from '../scenes/useScene2D';
import { Rect } from './Rect';
let View2D = View2D_1 = class View2D extends Rect {
    constructor(props) {
        super({
            composite: true,
            fontFamily: 'Roboto',
            fontSize: 48,
            lineHeight: '120%',
            textWrap: false,
            fontStyle: 'normal',
            ...props,
        });
        this.view2D = this;
        View2D_1.shadowRoot.append(this.element);
        this.applyFlex();
    }
    dispose() {
        this.removeChildren();
        super.dispose();
    }
    render(context) {
        this.computedSize();
        this.computedPosition();
        super.render(context);
    }
    /**
     * Find a node by its key.
     *
     * @param key - The key of the node.
     */
    findKey(key) {
        return useScene2D().getNode(key) ?? null;
    }
    requestLayoutUpdate() {
        this.updateLayout();
    }
    requestFontUpdate() {
        this.applyFont();
    }
    view() {
        return this;
    }
};
__decorate([
    initial(PlaybackState.Paused),
    signal()
], View2D.prototype, "playbackState", void 0);
__decorate([
    initial(0),
    signal()
], View2D.prototype, "globalTime", void 0);
__decorate([
    signal()
], View2D.prototype, "assetHash", void 0);
__decorate([
    lazy(() => {
        const frameID = 'motion-canvas-2d-frame';
        let frame = document.querySelector(`#${frameID}`);
        if (!frame) {
            frame = document.createElement('div');
            frame.id = frameID;
            frame.style.position = 'absolute';
            frame.style.pointerEvents = 'none';
            frame.style.top = '0';
            frame.style.left = '0';
            frame.style.opacity = '0';
            frame.style.overflow = 'hidden';
            document.body.prepend(frame);
        }
        return frame.shadowRoot ?? frame.attachShadow({ mode: 'open' });
    })
], View2D, "shadowRoot", void 0);
View2D = View2D_1 = __decorate([
    nodeName('View2D')
], View2D);
export { View2D };
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiVmlldzJELmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL1ZpZXcyRC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQUEsT0FBTyxFQUFDLGFBQWEsRUFBZ0IsSUFBSSxFQUFDLE1BQU0scUJBQXFCLENBQUM7QUFDdEUsT0FBTyxFQUFDLE9BQU8sRUFBRSxNQUFNLEVBQUMsTUFBTSxlQUFlLENBQUM7QUFDOUMsT0FBTyxFQUFDLFFBQVEsRUFBQyxNQUFNLHdCQUF3QixDQUFDO0FBQ2hELE9BQU8sRUFBQyxVQUFVLEVBQUMsTUFBTSxzQkFBc0IsQ0FBQztBQUVoRCxPQUFPLEVBQUMsSUFBSSxFQUFZLE1BQU0sUUFBUSxDQUFDO0FBT2hDLElBQU0sTUFBTSxjQUFaLE1BQU0sTUFBTyxTQUFRLElBQUk7SUE4QjlCLFlBQW1CLEtBQWtCO1FBQ25DLEtBQUssQ0FBQztZQUNKLFNBQVMsRUFBRSxJQUFJO1lBQ2YsVUFBVSxFQUFFLFFBQVE7WUFDcEIsUUFBUSxFQUFFLEVBQUU7WUFDWixVQUFVLEVBQUUsTUFBTTtZQUNsQixRQUFRLEVBQUUsS0FBSztZQUNmLFNBQVMsRUFBRSxRQUFRO1lBQ25CLEdBQUcsS0FBSztTQUNULENBQUMsQ0FBQztRQUNILElBQUksQ0FBQyxNQUFNLEdBQUcsSUFBSSxDQUFDO1FBRW5CLFFBQU0sQ0FBQyxVQUFVLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxPQUFPLENBQUMsQ0FBQztRQUN2QyxJQUFJLENBQUMsU0FBUyxFQUFFLENBQUM7SUFDbkIsQ0FBQztJQUVlLE9BQU87UUFDckIsSUFBSSxDQUFDLGNBQWMsRUFBRSxDQUFDO1FBQ3RCLEtBQUssQ0FBQyxPQUFPLEVBQUUsQ0FBQztJQUNsQixDQUFDO0lBRWUsTUFBTSxDQUFDLE9BQWlDO1FBQ3RELElBQUksQ0FBQyxZQUFZLEVBQUUsQ0FBQztRQUNwQixJQUFJLENBQUMsZ0JBQWdCLEVBQUUsQ0FBQztRQUN4QixLQUFLLENBQUMsTUFBTSxDQUFDLE9BQU8sQ0FBQyxDQUFDO0lBQ3hCLENBQUM7SUFFRDs7OztPQUlHO0lBQ0ksT0FBTyxDQUF3QixHQUFXO1FBQy9DLE9BQVEsVUFBVSxFQUFFLENBQUMsT0FBTyxDQUFDLEdBQUcsQ0FBTyxJQUFJLElBQUksQ0FBQztJQUNsRCxDQUFDO0lBRWtCLG1CQUFtQjtRQUNwQyxJQUFJLENBQUMsWUFBWSxFQUFFLENBQUM7SUFDdEIsQ0FBQztJQUVrQixpQkFBaUI7UUFDbEMsSUFBSSxDQUFDLFNBQVMsRUFBRSxDQUFDO0lBQ25CLENBQUM7SUFFZSxJQUFJO1FBQ2xCLE9BQU8sSUFBSSxDQUFDO0lBQ2QsQ0FBQztDQUNGLENBQUE7QUF4RHlCO0lBRnZCLE9BQU8sQ0FBQyxhQUFhLENBQUMsTUFBTSxDQUFDO0lBQzdCLE1BQU0sRUFBRTs2Q0FDZ0U7QUFJakQ7SUFGdkIsT0FBTyxDQUFDLENBQUMsQ0FBQztJQUNWLE1BQU0sRUFBRTswQ0FDc0Q7QUFHdkM7SUFEdkIsTUFBTSxFQUFFO3lDQUNxRDtBQVhoRDtJQWhCYixJQUFJLENBQUMsR0FBRyxFQUFFO1FBQ1QsTUFBTSxPQUFPLEdBQUcsd0JBQXdCLENBQUM7UUFDekMsSUFBSSxLQUFLLEdBQUcsUUFBUSxDQUFDLGFBQWEsQ0FBaUIsSUFBSSxPQUFPLEVBQUUsQ0FBQyxDQUFDO1FBQ2xFLElBQUksQ0FBQyxLQUFLLEVBQUU7WUFDVixLQUFLLEdBQUcsUUFBUSxDQUFDLGFBQWEsQ0FBQyxLQUFLLENBQUMsQ0FBQztZQUN0QyxLQUFLLENBQUMsRUFBRSxHQUFHLE9BQU8sQ0FBQztZQUNuQixLQUFLLENBQUMsS0FBSyxDQUFDLFFBQVEsR0FBRyxVQUFVLENBQUM7WUFDbEMsS0FBSyxDQUFDLEtBQUssQ0FBQyxhQUFhLEdBQUcsTUFBTSxDQUFDO1lBQ25DLEtBQUssQ0FBQyxLQUFLLENBQUMsR0FBRyxHQUFHLEdBQUcsQ0FBQztZQUN0QixLQUFLLENBQUMsS0FBSyxDQUFDLElBQUksR0FBRyxHQUFHLENBQUM7WUFDdkIsS0FBSyxDQUFDLEtBQUssQ0FBQyxPQUFPLEdBQUcsR0FBRyxDQUFDO1lBQzFCLEtBQUssQ0FBQyxLQUFLLENBQUMsUUFBUSxHQUFHLFFBQVEsQ0FBQztZQUNoQyxRQUFRLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUMsQ0FBQztTQUM5QjtRQUNELE9BQU8sS0FBSyxDQUFDLFVBQVUsSUFBSSxLQUFLLENBQUMsWUFBWSxDQUFDLEVBQUMsSUFBSSxFQUFFLE1BQU0sRUFBQyxDQUFDLENBQUM7SUFDaEUsQ0FBQyxDQUFDO2dDQUNtQztBQWpCMUIsTUFBTTtJQURsQixRQUFRLENBQUMsUUFBUSxDQUFDO0dBQ04sTUFBTSxDQTZFbEIifQ==