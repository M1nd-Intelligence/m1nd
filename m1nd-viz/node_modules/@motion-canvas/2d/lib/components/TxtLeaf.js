var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var TxtLeaf_1;
import { BBox, capitalize, lazy, textLerp, } from '@motion-canvas/core';
import { computed, initial, interpolation, nodeName, signal, } from '../decorators';
import { Shape } from './Shape';
import { Txt } from './Txt';
import { View2D } from './View2D';
let TxtLeaf = TxtLeaf_1 = class TxtLeaf extends Shape {
    constructor({ children, ...rest }) {
        super(rest);
        if (children) {
            this.text(children);
        }
    }
    parentTxt() {
        const parent = this.parent();
        return parent instanceof Txt ? parent : null;
    }
    draw(context) {
        this.requestFontUpdate();
        this.applyStyle(context);
        this.applyText(context);
        context.font = this.styles.font;
        context.textBaseline = 'bottom';
        if ('letterSpacing' in context) {
            context.letterSpacing = `${this.letterSpacing()}px`;
        }
        const fontOffset = context.measureText('').fontBoundingBoxAscent;
        const parentRect = this.element.getBoundingClientRect();
        const { width, height } = this.size();
        const range = document.createRange();
        let line = '';
        const lineRect = new BBox();
        for (const childNode of this.element.childNodes) {
            if (!childNode.textContent) {
                continue;
            }
            range.selectNodeContents(childNode);
            const rangeRect = range.getBoundingClientRect();
            const x = width / -2 + rangeRect.left - parentRect.left;
            const y = height / -2 + rangeRect.top - parentRect.top + fontOffset;
            if (lineRect.y === y) {
                lineRect.width += rangeRect.width;
                line += childNode.textContent;
            }
            else {
                this.drawText(context, line, lineRect);
                lineRect.x = x;
                lineRect.y = y;
                lineRect.width = rangeRect.width;
                lineRect.height = rangeRect.height;
                line = childNode.textContent;
            }
        }
        this.drawText(context, line, lineRect);
    }
    drawText(context, text, box) {
        const y = box.y;
        text = text.replace(/\s+/g, ' ');
        if (this.lineWidth() <= 0) {
            context.fillText(text, box.x, y);
        }
        else if (this.strokeFirst()) {
            context.strokeText(text, box.x, y);
            context.fillText(text, box.x, y);
        }
        else {
            context.fillText(text, box.x, y);
            context.strokeText(text, box.x, y);
        }
    }
    getCacheBBox() {
        const size = this.computedSize();
        const range = document.createRange();
        range.selectNodeContents(this.element);
        const bbox = range.getBoundingClientRect();
        const lineWidth = this.lineWidth();
        // We take the default value of the miterLimit as 10.
        const miterLimitCoefficient = this.lineJoin() === 'miter' ? 0.5 * 10 : 0.5;
        return new BBox(-size.width / 2, -size.height / 2, bbox.width, bbox.height)
            .expand([0, this.fontSize() * 0.5])
            .expand(lineWidth * miterLimitCoefficient);
    }
    applyFlex() {
        super.applyFlex();
        this.element.style.display = 'inline';
    }
    updateLayout() {
        this.applyFont();
        this.applyFlex();
        // Make sure the text is aligned correctly even if the text is smaller than
        // the container.
        if (this.justifyContent.isInitial()) {
            this.element.style.justifyContent =
                this.styles.getPropertyValue('text-align');
        }
        const wrap = this.styles.whiteSpace !== 'nowrap' && this.styles.whiteSpace !== 'pre';
        if (wrap) {
            this.element.innerText = '';
            if (TxtLeaf_1.segmenter) {
                for (const word of TxtLeaf_1.segmenter.segment(this.text())) {
                    this.element.appendChild(document.createTextNode(word.segment));
                }
            }
            else {
                for (const word of this.text().split('')) {
                    this.element.appendChild(document.createTextNode(word));
                }
            }
        }
        else if (this.styles.whiteSpace === 'pre') {
            this.element.innerText = '';
            for (const line of this.text().split('\n')) {
                this.element.appendChild(document.createTextNode(line + '\n'));
            }
        }
        else {
            this.element.innerText = this.text();
        }
    }
};
__decorate([
    initial(''),
    interpolation(textLerp),
    signal()
], TxtLeaf.prototype, "text", void 0);
__decorate([
    computed()
], TxtLeaf.prototype, "parentTxt", null);
__decorate([
    lazy(() => {
        const formatter = document.createElement('span');
        View2D.shadowRoot.append(formatter);
        return formatter;
    })
], TxtLeaf, "formatter", void 0);
__decorate([
    lazy(() => {
        try {
            return new Intl.Segmenter(undefined, {
                granularity: 'grapheme',
            });
        }
        catch (e) {
            return null;
        }
    })
], TxtLeaf, "segmenter", void 0);
TxtLeaf = TxtLeaf_1 = __decorate([
    nodeName('TxtLeaf')
], TxtLeaf);
export { TxtLeaf };
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
    TxtLeaf.prototype[`get${capitalize(prop)}`] = function () {
        return (this.parentTxt()?.[prop]() ??
            this[prop].context.getInitial());
    };
});
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiVHh0TGVhZi5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29tcG9uZW50cy9UeHRMZWFmLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7Ozs7QUFBQSxPQUFPLEVBQ0wsSUFBSSxFQUdKLFVBQVUsRUFDVixJQUFJLEVBQ0osUUFBUSxHQUNULE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUNMLFFBQVEsRUFDUixPQUFPLEVBQ1AsYUFBYSxFQUNiLFFBQVEsRUFDUixNQUFNLEdBQ1AsTUFBTSxlQUFlLENBQUM7QUFDdkIsT0FBTyxFQUFDLEtBQUssRUFBYSxNQUFNLFNBQVMsQ0FBQztBQUMxQyxPQUFPLEVBQUMsR0FBRyxFQUFDLE1BQU0sT0FBTyxDQUFDO0FBQzFCLE9BQU8sRUFBQyxNQUFNLEVBQUMsTUFBTSxVQUFVLENBQUM7QUFRekIsSUFBTSxPQUFPLGVBQWIsTUFBTSxPQUFRLFNBQVEsS0FBSztJQXdCaEMsWUFBbUIsRUFBQyxRQUFRLEVBQUUsR0FBRyxJQUFJLEVBQWU7UUFDbEQsS0FBSyxDQUFDLElBQUksQ0FBQyxDQUFDO1FBQ1osSUFBSSxRQUFRLEVBQUU7WUFDWixJQUFJLENBQUMsSUFBSSxDQUFDLFFBQVEsQ0FBQyxDQUFDO1NBQ3JCO0lBQ0gsQ0FBQztJQUdTLFNBQVM7UUFDakIsTUFBTSxNQUFNLEdBQUcsSUFBSSxDQUFDLE1BQU0sRUFBRSxDQUFDO1FBQzdCLE9BQU8sTUFBTSxZQUFZLEdBQUcsQ0FBQyxDQUFDLENBQUMsTUFBTSxDQUFDLENBQUMsQ0FBQyxJQUFJLENBQUM7SUFDL0MsQ0FBQztJQUVrQixJQUFJLENBQUMsT0FBaUM7UUFDdkQsSUFBSSxDQUFDLGlCQUFpQixFQUFFLENBQUM7UUFDekIsSUFBSSxDQUFDLFVBQVUsQ0FBQyxPQUFPLENBQUMsQ0FBQztRQUN6QixJQUFJLENBQUMsU0FBUyxDQUFDLE9BQU8sQ0FBQyxDQUFDO1FBQ3hCLE9BQU8sQ0FBQyxJQUFJLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUM7UUFDaEMsT0FBTyxDQUFDLFlBQVksR0FBRyxRQUFRLENBQUM7UUFDaEMsSUFBSSxlQUFlLElBQUksT0FBTyxFQUFFO1lBQzlCLE9BQU8sQ0FBQyxhQUFhLEdBQUcsR0FBRyxJQUFJLENBQUMsYUFBYSxFQUFFLElBQUksQ0FBQztTQUNyRDtRQUNELE1BQU0sVUFBVSxHQUFHLE9BQU8sQ0FBQyxXQUFXLENBQUMsRUFBRSxDQUFDLENBQUMscUJBQXFCLENBQUM7UUFFakUsTUFBTSxVQUFVLEdBQUcsSUFBSSxDQUFDLE9BQU8sQ0FBQyxxQkFBcUIsRUFBRSxDQUFDO1FBQ3hELE1BQU0sRUFBQyxLQUFLLEVBQUUsTUFBTSxFQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1FBQ3BDLE1BQU0sS0FBSyxHQUFHLFFBQVEsQ0FBQyxXQUFXLEVBQUUsQ0FBQztRQUNyQyxJQUFJLElBQUksR0FBRyxFQUFFLENBQUM7UUFDZCxNQUFNLFFBQVEsR0FBRyxJQUFJLElBQUksRUFBRSxDQUFDO1FBQzVCLEtBQUssTUFBTSxTQUFTLElBQUksSUFBSSxDQUFDLE9BQU8sQ0FBQyxVQUFVLEVBQUU7WUFDL0MsSUFBSSxDQUFDLFNBQVMsQ0FBQyxXQUFXLEVBQUU7Z0JBQzFCLFNBQVM7YUFDVjtZQUVELEtBQUssQ0FBQyxrQkFBa0IsQ0FBQyxTQUFTLENBQUMsQ0FBQztZQUNwQyxNQUFNLFNBQVMsR0FBRyxLQUFLLENBQUMscUJBQXFCLEVBQUUsQ0FBQztZQUVoRCxNQUFNLENBQUMsR0FBRyxLQUFLLEdBQUcsQ0FBQyxDQUFDLEdBQUcsU0FBUyxDQUFDLElBQUksR0FBRyxVQUFVLENBQUMsSUFBSSxDQUFDO1lBQ3hELE1BQU0sQ0FBQyxHQUFHLE1BQU0sR0FBRyxDQUFDLENBQUMsR0FBRyxTQUFTLENBQUMsR0FBRyxHQUFHLFVBQVUsQ0FBQyxHQUFHLEdBQUcsVUFBVSxDQUFDO1lBRXBFLElBQUksUUFBUSxDQUFDLENBQUMsS0FBSyxDQUFDLEVBQUU7Z0JBQ3BCLFFBQVEsQ0FBQyxLQUFLLElBQUksU0FBUyxDQUFDLEtBQUssQ0FBQztnQkFDbEMsSUFBSSxJQUFJLFNBQVMsQ0FBQyxXQUFXLENBQUM7YUFDL0I7aUJBQU07Z0JBQ0wsSUFBSSxDQUFDLFFBQVEsQ0FBQyxPQUFPLEVBQUUsSUFBSSxFQUFFLFFBQVEsQ0FBQyxDQUFDO2dCQUN2QyxRQUFRLENBQUMsQ0FBQyxHQUFHLENBQUMsQ0FBQztnQkFDZixRQUFRLENBQUMsQ0FBQyxHQUFHLENBQUMsQ0FBQztnQkFDZixRQUFRLENBQUMsS0FBSyxHQUFHLFNBQVMsQ0FBQyxLQUFLLENBQUM7Z0JBQ2pDLFFBQVEsQ0FBQyxNQUFNLEdBQUcsU0FBUyxDQUFDLE1BQU0sQ0FBQztnQkFDbkMsSUFBSSxHQUFHLFNBQVMsQ0FBQyxXQUFXLENBQUM7YUFDOUI7U0FDRjtRQUVELElBQUksQ0FBQyxRQUFRLENBQUMsT0FBTyxFQUFFLElBQUksRUFBRSxRQUFRLENBQUMsQ0FBQztJQUN6QyxDQUFDO0lBRVMsUUFBUSxDQUNoQixPQUFpQyxFQUNqQyxJQUFZLEVBQ1osR0FBUztRQUVULE1BQU0sQ0FBQyxHQUFHLEdBQUcsQ0FBQyxDQUFDLENBQUM7UUFDaEIsSUFBSSxHQUFHLElBQUksQ0FBQyxPQUFPLENBQUMsTUFBTSxFQUFFLEdBQUcsQ0FBQyxDQUFDO1FBRWpDLElBQUksSUFBSSxDQUFDLFNBQVMsRUFBRSxJQUFJLENBQUMsRUFBRTtZQUN6QixPQUFPLENBQUMsUUFBUSxDQUFDLElBQUksRUFBRSxHQUFHLENBQUMsQ0FBQyxFQUFFLENBQUMsQ0FBQyxDQUFDO1NBQ2xDO2FBQU0sSUFBSSxJQUFJLENBQUMsV0FBVyxFQUFFLEVBQUU7WUFDN0IsT0FBTyxDQUFDLFVBQVUsQ0FBQyxJQUFJLEVBQUUsR0FBRyxDQUFDLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQztZQUNuQyxPQUFPLENBQUMsUUFBUSxDQUFDLElBQUksRUFBRSxHQUFHLENBQUMsQ0FBQyxFQUFFLENBQUMsQ0FBQyxDQUFDO1NBQ2xDO2FBQU07WUFDTCxPQUFPLENBQUMsUUFBUSxDQUFDLElBQUksRUFBRSxHQUFHLENBQUMsQ0FBQyxFQUFFLENBQUMsQ0FBQyxDQUFDO1lBQ2pDLE9BQU8sQ0FBQyxVQUFVLENBQUMsSUFBSSxFQUFFLEdBQUcsQ0FBQyxDQUFDLEVBQUUsQ0FBQyxDQUFDLENBQUM7U0FDcEM7SUFDSCxDQUFDO0lBRWtCLFlBQVk7UUFDN0IsTUFBTSxJQUFJLEdBQUcsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDO1FBQ2pDLE1BQU0sS0FBSyxHQUFHLFFBQVEsQ0FBQyxXQUFXLEVBQUUsQ0FBQztRQUNyQyxLQUFLLENBQUMsa0JBQWtCLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxDQUFDO1FBQ3ZDLE1BQU0sSUFBSSxHQUFHLEtBQUssQ0FBQyxxQkFBcUIsRUFBRSxDQUFDO1FBRTNDLE1BQU0sU0FBUyxHQUFHLElBQUksQ0FBQyxTQUFTLEVBQUUsQ0FBQztRQUNuQyxxREFBcUQ7UUFDckQsTUFBTSxxQkFBcUIsR0FBRyxJQUFJLENBQUMsUUFBUSxFQUFFLEtBQUssT0FBTyxDQUFDLENBQUMsQ0FBQyxHQUFHLEdBQUcsRUFBRSxDQUFDLENBQUMsQ0FBQyxHQUFHLENBQUM7UUFFM0UsT0FBTyxJQUFJLElBQUksQ0FBQyxDQUFDLElBQUksQ0FBQyxLQUFLLEdBQUcsQ0FBQyxFQUFFLENBQUMsSUFBSSxDQUFDLE1BQU0sR0FBRyxDQUFDLEVBQUUsSUFBSSxDQUFDLEtBQUssRUFBRSxJQUFJLENBQUMsTUFBTSxDQUFDO2FBQ3hFLE1BQU0sQ0FBQyxDQUFDLENBQUMsRUFBRSxJQUFJLENBQUMsUUFBUSxFQUFFLEdBQUcsR0FBRyxDQUFDLENBQUM7YUFDbEMsTUFBTSxDQUFDLFNBQVMsR0FBRyxxQkFBcUIsQ0FBQyxDQUFDO0lBQy9DLENBQUM7SUFFa0IsU0FBUztRQUMxQixLQUFLLENBQUMsU0FBUyxFQUFFLENBQUM7UUFDbEIsSUFBSSxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUMsT0FBTyxHQUFHLFFBQVEsQ0FBQztJQUN4QyxDQUFDO0lBRWtCLFlBQVk7UUFDN0IsSUFBSSxDQUFDLFNBQVMsRUFBRSxDQUFDO1FBQ2pCLElBQUksQ0FBQyxTQUFTLEVBQUUsQ0FBQztRQUVqQiwyRUFBMkU7UUFDM0UsaUJBQWlCO1FBQ2pCLElBQUksSUFBSSxDQUFDLGNBQWMsQ0FBQyxTQUFTLEVBQUUsRUFBRTtZQUNuQyxJQUFJLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQyxjQUFjO2dCQUMvQixJQUFJLENBQUMsTUFBTSxDQUFDLGdCQUFnQixDQUFDLFlBQVksQ0FBQyxDQUFDO1NBQzlDO1FBRUQsTUFBTSxJQUFJLEdBQ1IsSUFBSSxDQUFDLE1BQU0sQ0FBQyxVQUFVLEtBQUssUUFBUSxJQUFJLElBQUksQ0FBQyxNQUFNLENBQUMsVUFBVSxLQUFLLEtBQUssQ0FBQztRQUUxRSxJQUFJLElBQUksRUFBRTtZQUNSLElBQUksQ0FBQyxPQUFPLENBQUMsU0FBUyxHQUFHLEVBQUUsQ0FBQztZQUU1QixJQUFJLFNBQU8sQ0FBQyxTQUFTLEVBQUU7Z0JBQ3JCLEtBQUssTUFBTSxJQUFJLElBQUksU0FBTyxDQUFDLFNBQVMsQ0FBQyxPQUFPLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDLEVBQUU7b0JBQ3pELElBQUksQ0FBQyxPQUFPLENBQUMsV0FBVyxDQUFDLFFBQVEsQ0FBQyxjQUFjLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxDQUFDLENBQUM7aUJBQ2pFO2FBQ0Y7aUJBQU07Z0JBQ0wsS0FBSyxNQUFNLElBQUksSUFBSSxJQUFJLENBQUMsSUFBSSxFQUFFLENBQUMsS0FBSyxDQUFDLEVBQUUsQ0FBQyxFQUFFO29CQUN4QyxJQUFJLENBQUMsT0FBTyxDQUFDLFdBQVcsQ0FBQyxRQUFRLENBQUMsY0FBYyxDQUFDLElBQUksQ0FBQyxDQUFDLENBQUM7aUJBQ3pEO2FBQ0Y7U0FDRjthQUFNLElBQUksSUFBSSxDQUFDLE1BQU0sQ0FBQyxVQUFVLEtBQUssS0FBSyxFQUFFO1lBQzNDLElBQUksQ0FBQyxPQUFPLENBQUMsU0FBUyxHQUFHLEVBQUUsQ0FBQztZQUM1QixLQUFLLE1BQU0sSUFBSSxJQUFJLElBQUksQ0FBQyxJQUFJLEVBQUUsQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLEVBQUU7Z0JBQzFDLElBQUksQ0FBQyxPQUFPLENBQUMsV0FBVyxDQUFDLFFBQVEsQ0FBQyxjQUFjLENBQUMsSUFBSSxHQUFHLElBQUksQ0FBQyxDQUFDLENBQUM7YUFDaEU7U0FDRjthQUFNO1lBQ0wsSUFBSSxDQUFDLE9BQU8sQ0FBQyxTQUFTLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1NBQ3RDO0lBQ0gsQ0FBQztDQUNGLENBQUE7QUFwSXlCO0lBSHZCLE9BQU8sQ0FBQyxFQUFFLENBQUM7SUFDWCxhQUFhLENBQUMsUUFBUSxDQUFDO0lBQ3ZCLE1BQU0sRUFBRTtxQ0FDZ0Q7QUFVL0M7SUFEVCxRQUFRLEVBQUU7d0NBSVY7QUE3QmdCO0lBTGhCLElBQUksQ0FBQyxHQUFHLEVBQUU7UUFDVCxNQUFNLFNBQVMsR0FBRyxRQUFRLENBQUMsYUFBYSxDQUFDLE1BQU0sQ0FBQyxDQUFDO1FBQ2pELE1BQU0sQ0FBQyxVQUFVLENBQUMsTUFBTSxDQUFDLFNBQVMsQ0FBQyxDQUFDO1FBQ3BDLE9BQU8sU0FBUyxDQUFDO0lBQ25CLENBQUMsQ0FBQztnQ0FDeUM7QUFXakI7SUFUekIsSUFBSSxDQUFDLEdBQUcsRUFBRTtRQUNULElBQUk7WUFDRixPQUFPLElBQUssSUFBWSxDQUFDLFNBQVMsQ0FBQyxTQUFTLEVBQUU7Z0JBQzVDLFdBQVcsRUFBRSxVQUFVO2FBQ3hCLENBQUMsQ0FBQztTQUNKO1FBQUMsT0FBTyxDQUFDLEVBQUU7WUFDVixPQUFPLElBQUksQ0FBQztTQUNiO0lBQ0gsQ0FBQyxDQUFDO2dDQUN1QztBQWpCOUIsT0FBTztJQURuQixRQUFRLENBQUMsU0FBUyxDQUFDO0dBQ1AsT0FBTyxDQTBKbkI7O0FBRUQ7SUFDRSxNQUFNO0lBQ04sUUFBUTtJQUNSLFdBQVc7SUFDWCxhQUFhO0lBQ2IsU0FBUztJQUNULFVBQVU7SUFDVixVQUFVO0lBQ1YsZ0JBQWdCO0NBQ2pCLENBQUMsT0FBTyxDQUFDLElBQUksQ0FBQyxFQUFFO0lBQ2QsT0FBTyxDQUFDLFNBQWlCLENBQUMsTUFBTSxVQUFVLENBQUMsSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHO1FBR3JELE9BQU8sQ0FDSixJQUFJLENBQUMsU0FBUyxFQUFVLEVBQUUsQ0FBQyxJQUFJLENBQUMsRUFBRTtZQUNsQyxJQUFZLENBQUMsSUFBSSxDQUFDLENBQUMsT0FBTyxDQUFDLFVBQVUsRUFBRSxDQUN6QyxDQUFDO0lBQ0osQ0FBQyxDQUFDO0FBQ0osQ0FBQyxDQUFDLENBQUMifQ==