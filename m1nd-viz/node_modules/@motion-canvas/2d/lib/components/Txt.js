var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var Txt_1;
import { DEFAULT, all, capitalize, threadable, } from '@motion-canvas/core';
import { computed, initial, nodeName, signal } from '../decorators';
import { is } from '../utils';
import { Shape } from './Shape';
import { TxtLeaf } from './TxtLeaf';
let Txt = Txt_1 = class Txt extends Shape {
    /**
     * Create a bold text node.
     *
     * @remarks
     * This is a shortcut for
     * ```tsx
     * <Txt fontWeight={700} />
     * ```
     *
     * @param props - Additional text properties.
     */
    static b(props) {
        return new Txt_1({ ...props, fontWeight: 700 });
    }
    /**
     * Create an italic text node.
     *
     * @remarks
     * This is a shortcut for
     * ```tsx
     * <Txt fontStyle={'italic'} />
     * ```
     *
     * @param props - Additional text properties.
     */
    static i(props) {
        return new Txt_1({ ...props, fontStyle: 'italic' });
    }
    getText() {
        return this.innerText();
    }
    setText(value) {
        const children = this.children();
        let leaf = null;
        for (let i = 0; i < children.length; i++) {
            const child = children[i];
            if (leaf === null && child instanceof TxtLeaf) {
                leaf = child;
            }
            else {
                child.parent(null);
            }
        }
        if (leaf === null) {
            leaf = new TxtLeaf({ text: value });
            leaf.parent(this);
        }
        else {
            leaf.text(value);
        }
        this.setParsedChildren([leaf]);
    }
    setChildren(value) {
        if (this.children.context.raw() === value) {
            return;
        }
        if (typeof value === 'string') {
            this.text(value);
        }
        else {
            super.setChildren(value);
        }
    }
    *tweenText(value, time, timingFunction, interpolationFunction) {
        const children = this.children();
        if (children.length !== 1 || !(children[0] instanceof TxtLeaf)) {
            this.text.save();
        }
        const leaf = this.childAs(0);
        const oldText = leaf.text.context.raw();
        const oldSize = this.size.context.raw();
        leaf.text(value);
        const newSize = this.size();
        leaf.text(oldText ?? DEFAULT);
        if (this.height() === 0) {
            this.height(newSize.height);
        }
        yield* all(this.size(newSize, time, timingFunction), leaf.text(value, time, timingFunction, interpolationFunction));
        this.children.context.setter(value);
        this.size(oldSize);
    }
    getLayout() {
        return true;
    }
    constructor({ children, text, ...props }) {
        super(props);
        this.children(text ?? children);
    }
    innerText() {
        const children = this.childrenAs();
        let text = '';
        for (const child of children) {
            text += child.text();
        }
        return text;
    }
    parentTxt() {
        const parent = this.parent();
        return parent instanceof Txt_1 ? parent : null;
    }
    parseChildren(children) {
        const result = [];
        const array = Array.isArray(children) ? children : [children];
        for (const child of array) {
            if (child instanceof Txt_1 || child instanceof TxtLeaf) {
                result.push(child);
            }
            else if (typeof child === 'string') {
                result.push(new TxtLeaf({ text: child }));
            }
        }
        return result;
    }
    applyFlex() {
        super.applyFlex();
        this.element.style.display = this.findAncestor(is(Txt_1))
            ? 'inline'
            : 'block';
    }
    draw(context) {
        this.drawChildren(context);
    }
};
__decorate([
    initial(''),
    signal()
], Txt.prototype, "text", void 0);
__decorate([
    threadable()
], Txt.prototype, "tweenText", null);
__decorate([
    computed()
], Txt.prototype, "innerText", null);
__decorate([
    computed()
], Txt.prototype, "parentTxt", null);
Txt = Txt_1 = __decorate([
    nodeName('Txt')
], Txt);
export { Txt };
[
    'fill',
    'stroke',
    'lineWidth',
    'strokeFirst',
    'lineCap',
    'lineJoin',
    'lineDash',
    'lineDashOffset',
].forEach(prop => {
    Txt.prototype[`getDefault${capitalize(prop)}`] = function (initial) {
        return this.parentTxt()?.[prop]() ?? initial;
    };
});
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiVHh0LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL1R4dC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQUEsT0FBTyxFQUNMLE9BQU8sRUFNUCxHQUFHLEVBQ0gsVUFBVSxFQUNWLFVBQVUsR0FDWCxNQUFNLHFCQUFxQixDQUFDO0FBQzdCLE9BQU8sRUFBQyxRQUFRLEVBQUUsT0FBTyxFQUFFLFFBQVEsRUFBRSxNQUFNLEVBQUMsTUFBTSxlQUFlLENBQUM7QUFDbEUsT0FBTyxFQUFDLEVBQUUsRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUU1QixPQUFPLEVBQUMsS0FBSyxFQUFhLE1BQU0sU0FBUyxDQUFDO0FBQzFDLE9BQU8sRUFBQyxPQUFPLEVBQUMsTUFBTSxXQUFXLENBQUM7QUFZM0IsSUFBTSxHQUFHLFdBQVQsTUFBTSxHQUFJLFNBQVEsS0FBSztJQUM1Qjs7Ozs7Ozs7OztPQVVHO0lBQ0ksTUFBTSxDQUFDLENBQUMsQ0FBQyxLQUFlO1FBQzdCLE9BQU8sSUFBSSxLQUFHLENBQUMsRUFBQyxHQUFHLEtBQUssRUFBRSxVQUFVLEVBQUUsR0FBRyxFQUFDLENBQUMsQ0FBQztJQUM5QyxDQUFDO0lBRUQ7Ozs7Ozs7Ozs7T0FVRztJQUNJLE1BQU0sQ0FBQyxDQUFDLENBQUMsS0FBZTtRQUM3QixPQUFPLElBQUksS0FBRyxDQUFDLEVBQUMsR0FBRyxLQUFLLEVBQUUsU0FBUyxFQUFFLFFBQVEsRUFBQyxDQUFDLENBQUM7SUFDbEQsQ0FBQztJQU1TLE9BQU87UUFDZixPQUFPLElBQUksQ0FBQyxTQUFTLEVBQUUsQ0FBQztJQUMxQixDQUFDO0lBRVMsT0FBTyxDQUFDLEtBQTBCO1FBQzFDLE1BQU0sUUFBUSxHQUFHLElBQUksQ0FBQyxRQUFRLEVBQUUsQ0FBQztRQUNqQyxJQUFJLElBQUksR0FBbUIsSUFBSSxDQUFDO1FBQ2hDLEtBQUssSUFBSSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRyxRQUFRLENBQUMsTUFBTSxFQUFFLENBQUMsRUFBRSxFQUFFO1lBQ3hDLE1BQU0sS0FBSyxHQUFHLFFBQVEsQ0FBQyxDQUFDLENBQUMsQ0FBQztZQUMxQixJQUFJLElBQUksS0FBSyxJQUFJLElBQUksS0FBSyxZQUFZLE9BQU8sRUFBRTtnQkFDN0MsSUFBSSxHQUFHLEtBQUssQ0FBQzthQUNkO2lCQUFNO2dCQUNMLEtBQUssQ0FBQyxNQUFNLENBQUMsSUFBSSxDQUFDLENBQUM7YUFDcEI7U0FDRjtRQUVELElBQUksSUFBSSxLQUFLLElBQUksRUFBRTtZQUNqQixJQUFJLEdBQUcsSUFBSSxPQUFPLENBQUMsRUFBQyxJQUFJLEVBQUUsS0FBSyxFQUFDLENBQUMsQ0FBQztZQUNsQyxJQUFJLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxDQUFDO1NBQ25CO2FBQU07WUFDTCxJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDO1NBQ2xCO1FBRUQsSUFBSSxDQUFDLGlCQUFpQixDQUFDLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBQztJQUNqQyxDQUFDO0lBRWtCLFdBQVcsQ0FBQyxLQUFxQztRQUNsRSxJQUFJLElBQUksQ0FBQyxRQUFRLENBQUMsT0FBTyxDQUFDLEdBQUcsRUFBRSxLQUFLLEtBQUssRUFBRTtZQUN6QyxPQUFPO1NBQ1I7UUFFRCxJQUFJLE9BQU8sS0FBSyxLQUFLLFFBQVEsRUFBRTtZQUM3QixJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDO1NBQ2xCO2FBQU07WUFDTCxLQUFLLENBQUMsV0FBVyxDQUFDLEtBQUssQ0FBQyxDQUFDO1NBQzFCO0lBQ0gsQ0FBQztJQUdVLEFBQUQsQ0FBQyxTQUFTLENBQ2xCLEtBQTBCLEVBQzFCLElBQVksRUFDWixjQUE4QixFQUM5QixxQkFBb0Q7UUFFcEQsTUFBTSxRQUFRLEdBQUcsSUFBSSxDQUFDLFFBQVEsRUFBRSxDQUFDO1FBQ2pDLElBQUksUUFBUSxDQUFDLE1BQU0sS0FBSyxDQUFDLElBQUksQ0FBQyxDQUFDLFFBQVEsQ0FBQyxDQUFDLENBQUMsWUFBWSxPQUFPLENBQUMsRUFBRTtZQUM5RCxJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1NBQ2xCO1FBRUQsTUFBTSxJQUFJLEdBQUcsSUFBSSxDQUFDLE9BQU8sQ0FBVSxDQUFDLENBQUUsQ0FBQztRQUN2QyxNQUFNLE9BQU8sR0FBRyxJQUFJLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxHQUFHLEVBQUUsQ0FBQztRQUN4QyxNQUFNLE9BQU8sR0FBRyxJQUFJLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxHQUFHLEVBQUUsQ0FBQztRQUN4QyxJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBQ2pCLE1BQU0sT0FBTyxHQUFHLElBQUksQ0FBQyxJQUFJLEVBQUUsQ0FBQztRQUM1QixJQUFJLENBQUMsSUFBSSxDQUFDLE9BQU8sSUFBSSxPQUFPLENBQUMsQ0FBQztRQUU5QixJQUFJLElBQUksQ0FBQyxNQUFNLEVBQUUsS0FBSyxDQUFDLEVBQUU7WUFDdkIsSUFBSSxDQUFDLE1BQU0sQ0FBQyxPQUFPLENBQUMsTUFBTSxDQUFDLENBQUM7U0FDN0I7UUFFRCxLQUFLLENBQUMsQ0FBQyxHQUFHLENBQ1IsSUFBSSxDQUFDLElBQUksQ0FBQyxPQUFPLEVBQUUsSUFBSSxFQUFFLGNBQWMsQ0FBQyxFQUN4QyxJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssRUFBRSxJQUFJLEVBQUUsY0FBYyxFQUFFLHFCQUFxQixDQUFDLENBQzlELENBQUM7UUFFRixJQUFJLENBQUMsUUFBUSxDQUFDLE9BQU8sQ0FBQyxNQUFNLENBQUMsS0FBSyxDQUFDLENBQUM7UUFDcEMsSUFBSSxDQUFDLElBQUksQ0FBQyxPQUFPLENBQUMsQ0FBQztJQUNyQixDQUFDO0lBRVMsU0FBUztRQUNqQixPQUFPLElBQUksQ0FBQztJQUNkLENBQUM7SUFFRCxZQUFtQixFQUFDLFFBQVEsRUFBRSxJQUFJLEVBQUUsR0FBRyxLQUFLLEVBQVc7UUFDckQsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBQ2IsSUFBSSxDQUFDLFFBQVEsQ0FBQyxJQUFJLElBQUksUUFBUSxDQUFDLENBQUM7SUFDbEMsQ0FBQztJQUdTLFNBQVM7UUFDakIsTUFBTSxRQUFRLEdBQUcsSUFBSSxDQUFDLFVBQVUsRUFBaUIsQ0FBQztRQUNsRCxJQUFJLElBQUksR0FBRyxFQUFFLENBQUM7UUFDZCxLQUFLLE1BQU0sS0FBSyxJQUFJLFFBQVEsRUFBRTtZQUM1QixJQUFJLElBQUksS0FBSyxDQUFDLElBQUksRUFBRSxDQUFDO1NBQ3RCO1FBRUQsT0FBTyxJQUFJLENBQUM7SUFDZCxDQUFDO0lBR1MsU0FBUztRQUNqQixNQUFNLE1BQU0sR0FBRyxJQUFJLENBQUMsTUFBTSxFQUFFLENBQUM7UUFDN0IsT0FBTyxNQUFNLFlBQVksS0FBRyxDQUFDLENBQUMsQ0FBQyxNQUFNLENBQUMsQ0FBQyxDQUFDLElBQUksQ0FBQztJQUMvQyxDQUFDO0lBRWtCLGFBQWEsQ0FBQyxRQUEyQjtRQUMxRCxNQUFNLE1BQU0sR0FBYSxFQUFFLENBQUM7UUFDNUIsTUFBTSxLQUFLLEdBQUcsS0FBSyxDQUFDLE9BQU8sQ0FBQyxRQUFRLENBQUMsQ0FBQyxDQUFDLENBQUMsUUFBUSxDQUFDLENBQUMsQ0FBQyxDQUFDLFFBQVEsQ0FBQyxDQUFDO1FBQzlELEtBQUssTUFBTSxLQUFLLElBQUksS0FBSyxFQUFFO1lBQ3pCLElBQUksS0FBSyxZQUFZLEtBQUcsSUFBSSxLQUFLLFlBQVksT0FBTyxFQUFFO2dCQUNwRCxNQUFNLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDO2FBQ3BCO2lCQUFNLElBQUksT0FBTyxLQUFLLEtBQUssUUFBUSxFQUFFO2dCQUNwQyxNQUFNLENBQUMsSUFBSSxDQUFDLElBQUksT0FBTyxDQUFDLEVBQUMsSUFBSSxFQUFFLEtBQUssRUFBQyxDQUFDLENBQUMsQ0FBQzthQUN6QztTQUNGO1FBRUQsT0FBTyxNQUFNLENBQUM7SUFDaEIsQ0FBQztJQUVrQixTQUFTO1FBQzFCLEtBQUssQ0FBQyxTQUFTLEVBQUUsQ0FBQztRQUNsQixJQUFJLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQyxPQUFPLEdBQUcsSUFBSSxDQUFDLFlBQVksQ0FBQyxFQUFFLENBQUMsS0FBRyxDQUFDLENBQUM7WUFDckQsQ0FBQyxDQUFDLFFBQVE7WUFDVixDQUFDLENBQUMsT0FBTyxDQUFDO0lBQ2QsQ0FBQztJQUVrQixJQUFJLENBQUMsT0FBaUM7UUFDdkQsSUFBSSxDQUFDLFlBQVksQ0FBQyxPQUFPLENBQUMsQ0FBQztJQUM3QixDQUFDO0NBQ0YsQ0FBQTtBQTFIeUI7SUFGdkIsT0FBTyxDQUFDLEVBQUUsQ0FBQztJQUNYLE1BQU0sRUFBRTtpQ0FDZ0Q7QUF5QzlDO0lBRFYsVUFBVSxFQUFFO29DQThCWjtBQVlTO0lBRFQsUUFBUSxFQUFFO29DQVNWO0FBR1M7SUFEVCxRQUFRLEVBQUU7b0NBSVY7QUFqSVUsR0FBRztJQURmLFFBQVEsQ0FBQyxLQUFLLENBQUM7R0FDSCxHQUFHLENBMkpmOztBQUVEO0lBQ0UsTUFBTTtJQUNOLFFBQVE7SUFDUixXQUFXO0lBQ1gsYUFBYTtJQUNiLFNBQVM7SUFDVCxVQUFVO0lBQ1YsVUFBVTtJQUNWLGdCQUFnQjtDQUNqQixDQUFDLE9BQU8sQ0FBQyxJQUFJLENBQUMsRUFBRTtJQUNkLEdBQUcsQ0FBQyxTQUFpQixDQUFDLGFBQWEsVUFBVSxDQUFDLElBQUksQ0FBQyxFQUFFLENBQUMsR0FBRyxVQUV4RCxPQUFnQjtRQUVoQixPQUFRLElBQUksQ0FBQyxTQUFTLEVBQVUsRUFBRSxDQUFDLElBQUksQ0FBQyxFQUFFLElBQUksT0FBTyxDQUFDO0lBQ3hELENBQUMsQ0FBQztBQUNKLENBQUMsQ0FBQyxDQUFDIn0=