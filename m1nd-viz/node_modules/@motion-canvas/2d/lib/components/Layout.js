var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var Layout_1;
import { BBox, boolLerp, Direction, modify, Origin, originToOffset, threadable, tween, Vector2, } from '@motion-canvas/core';
import { addInitializer, cloneable, computed, defaultStyle, getPropertyMeta, initial, interpolation, nodeName, signal, vector2Signal, } from '../decorators';
import { spacingSignal } from '../decorators/spacingSignal';
import { drawLine, drawPivot, is } from '../utils';
import { Node } from './Node';
let Layout = Layout_1 = class Layout extends Node {
    get columnGap() {
        return this.gap.x;
    }
    get rowGap() {
        return this.gap.y;
    }
    getX() {
        if (this.isLayoutRoot()) {
            return this.x.context.getter();
        }
        return this.computedPosition().x;
    }
    setX(value) {
        this.x.context.setter(value);
    }
    getY() {
        if (this.isLayoutRoot()) {
            return this.y.context.getter();
        }
        return this.computedPosition().y;
    }
    setY(value) {
        this.y.context.setter(value);
    }
    get width() {
        return this.size.x;
    }
    get height() {
        return this.size.y;
    }
    getWidth() {
        return this.computedSize().width;
    }
    setWidth(value) {
        this.width.context.setter(value);
    }
    *tweenWidth(value, time, timingFunction, interpolationFunction) {
        const width = this.desiredSize().x;
        const lock = typeof width !== 'number' || typeof value !== 'number';
        let from;
        if (lock) {
            from = this.size.x();
        }
        else {
            from = width;
        }
        let to;
        if (lock) {
            this.size.x(value);
            to = this.size.x();
        }
        else {
            to = value;
        }
        this.size.x(from);
        lock && this.lockSize();
        yield* tween(time, value => this.size.x(interpolationFunction(from, to, timingFunction(value))));
        this.size.x(value);
        lock && this.releaseSize();
    }
    getHeight() {
        return this.computedSize().height;
    }
    setHeight(value) {
        this.height.context.setter(value);
    }
    *tweenHeight(value, time, timingFunction, interpolationFunction) {
        const height = this.desiredSize().y;
        const lock = typeof height !== 'number' || typeof value !== 'number';
        let from;
        if (lock) {
            from = this.size.y();
        }
        else {
            from = height;
        }
        let to;
        if (lock) {
            this.size.y(value);
            to = this.size.y();
        }
        else {
            to = value;
        }
        this.size.y(from);
        lock && this.lockSize();
        yield* tween(time, value => this.size.y(interpolationFunction(from, to, timingFunction(value))));
        this.size.y(value);
        lock && this.releaseSize();
    }
    /**
     * Get the desired size of this node.
     *
     * @remarks
     * This method can be used to control the size using external factors.
     * By default, the returned size is the same as the one declared by the user.
     */
    desiredSize() {
        return {
            x: this.width.context.getter(),
            y: this.height.context.getter(),
        };
    }
    *tweenSize(value, time, timingFunction, interpolationFunction) {
        const size = this.desiredSize();
        let from;
        if (typeof size.x !== 'number' || typeof size.y !== 'number') {
            from = this.size();
        }
        else {
            from = new Vector2(size);
        }
        let to;
        if (typeof value === 'object' &&
            typeof value.x === 'number' &&
            typeof value.y === 'number') {
            to = new Vector2(value);
        }
        else {
            this.size(value);
            to = this.size();
        }
        this.size(from);
        this.lockSize();
        yield* tween(time, value => this.size(interpolationFunction(from, to, timingFunction(value))));
        this.releaseSize();
        this.size(value);
    }
    /**
     * Get the cardinal point corresponding to the given origin.
     *
     * @param origin - The origin or direction of the point.
     */
    cardinalPoint(origin) {
        switch (origin) {
            case Origin.TopLeft:
                return this.topLeft;
            case Origin.TopRight:
                return this.topRight;
            case Origin.BottomLeft:
                return this.bottomLeft;
            case Origin.BottomRight:
                return this.bottomRight;
            case Origin.Top:
            case Direction.Top:
                return this.top;
            case Origin.Bottom:
            case Direction.Bottom:
                return this.bottom;
            case Origin.Left:
            case Direction.Left:
                return this.left;
            case Origin.Right:
            case Direction.Right:
                return this.right;
            default:
                return this.middle;
        }
    }
    constructor(props) {
        super(props);
        this.element.dataset.motionCanvasKey = this.key;
    }
    lockSize() {
        this.sizeLockCounter(this.sizeLockCounter() + 1);
    }
    releaseSize() {
        this.sizeLockCounter(this.sizeLockCounter() - 1);
    }
    parentTransform() {
        return this.findAncestor(is(Layout_1));
    }
    anchorPosition() {
        const size = this.computedSize();
        const offset = this.offset();
        return size.scale(0.5).mul(offset);
    }
    /**
     * Get the resolved layout mode of this node.
     *
     * @remarks
     * When the mode is `null`, its value will be inherited from the parent.
     *
     * Use {@link layout} to get the raw mode set for this node (without
     * inheritance).
     */
    layoutEnabled() {
        return this.layout() ?? this.parentTransform()?.layoutEnabled() ?? false;
    }
    isLayoutRoot() {
        return !this.layoutEnabled() || !this.parentTransform()?.layoutEnabled();
    }
    localToParent() {
        const matrix = super.localToParent();
        const offset = this.offset();
        if (!offset.exactlyEquals(Vector2.zero)) {
            const translate = this.size().mul(offset).scale(-0.5);
            matrix.translateSelf(translate.x, translate.y);
        }
        return matrix;
    }
    /**
     * A simplified version of {@link localToParent} matrix used for transforming
     * direction vectors.
     *
     * @internal
     */
    scalingRotationMatrix() {
        const matrix = new DOMMatrix();
        matrix.rotateSelf(0, 0, this.rotation());
        matrix.scaleSelf(this.scale.x(), this.scale.y());
        const offset = this.offset();
        if (!offset.exactlyEquals(Vector2.zero)) {
            const translate = this.size().mul(offset).scale(-0.5);
            matrix.translateSelf(translate.x, translate.y);
        }
        return matrix;
    }
    getComputedLayout() {
        return new BBox(this.element.getBoundingClientRect());
    }
    computedPosition() {
        this.requestLayoutUpdate();
        const box = this.getComputedLayout();
        const position = new Vector2(box.x + (box.width / 2) * this.offset.x(), box.y + (box.height / 2) * this.offset.y());
        const parent = this.parentTransform();
        if (parent) {
            const parentRect = parent.getComputedLayout();
            position.x -= parentRect.x + (parentRect.width - box.width) / 2;
            position.y -= parentRect.y + (parentRect.height - box.height) / 2;
        }
        return position;
    }
    computedSize() {
        this.requestLayoutUpdate();
        return this.getComputedLayout().size;
    }
    /**
     * Find the closest layout root and apply any new layout changes.
     */
    requestLayoutUpdate() {
        const parent = this.parentTransform();
        if (this.appendedToView()) {
            parent?.requestFontUpdate();
            this.updateLayout();
        }
        else {
            parent.requestLayoutUpdate();
        }
    }
    appendedToView() {
        const root = this.isLayoutRoot();
        if (root) {
            this.view().element.append(this.element);
        }
        return root;
    }
    /**
     * Apply any new layout changes to this node and its children.
     */
    updateLayout() {
        this.applyFont();
        this.applyFlex();
        if (this.layoutEnabled()) {
            const children = this.layoutChildren();
            for (const child of children) {
                child.updateLayout();
            }
        }
    }
    layoutChildren() {
        const queue = [...this.children()];
        const result = [];
        const elements = [];
        while (queue.length) {
            const child = queue.shift();
            if (child instanceof Layout_1) {
                if (child.layoutEnabled()) {
                    result.push(child);
                    elements.push(child.element);
                }
            }
            else if (child) {
                queue.unshift(...child.children());
            }
        }
        this.element.replaceChildren(...elements);
        return result;
    }
    /**
     * Apply any new font changes to this node and all of its ancestors.
     */
    requestFontUpdate() {
        this.appendedToView();
        this.parentTransform()?.requestFontUpdate();
        this.applyFont();
    }
    getCacheBBox() {
        return BBox.fromSizeCentered(this.computedSize());
    }
    draw(context) {
        if (this.clip()) {
            const size = this.computedSize();
            if (size.width === 0 || size.height === 0) {
                return;
            }
            context.beginPath();
            context.rect(size.width / -2, size.height / -2, size.width, size.height);
            context.closePath();
            context.clip();
        }
        this.drawChildren(context);
    }
    drawOverlay(context, matrix) {
        const size = this.computedSize();
        const offset = size.mul(this.offset()).scale(0.5).transformAsPoint(matrix);
        const box = BBox.fromSizeCentered(size);
        const layout = box.transformCorners(matrix);
        const padding = box
            .addSpacing(this.padding().scale(-1))
            .transformCorners(matrix);
        const margin = box.addSpacing(this.margin()).transformCorners(matrix);
        context.beginPath();
        drawLine(context, margin);
        drawLine(context, layout);
        context.closePath();
        context.fillStyle = 'rgba(255,193,125,0.6)';
        context.fill('evenodd');
        context.beginPath();
        drawLine(context, layout);
        drawLine(context, padding);
        context.closePath();
        context.fillStyle = 'rgba(180,255,147,0.6)';
        context.fill('evenodd');
        context.beginPath();
        drawLine(context, layout);
        context.closePath();
        context.lineWidth = 1;
        context.strokeStyle = 'white';
        context.stroke();
        context.beginPath();
        drawPivot(context, offset);
        context.stroke();
    }
    getOriginDelta(origin) {
        const size = this.computedSize().scale(0.5);
        const offset = this.offset().mul(size);
        if (origin === Origin.Middle) {
            return offset.flipped;
        }
        const newOffset = originToOffset(origin).mul(size);
        return newOffset.sub(offset);
    }
    /**
     * Update the offset of this node and adjust the position to keep it in the
     * same place.
     *
     * @param offset - The new offset.
     */
    moveOffset(offset) {
        const size = this.computedSize().scale(0.5);
        const oldOffset = this.offset().mul(size);
        const newOffset = offset.mul(size);
        this.offset(offset);
        this.position(this.position().add(newOffset).sub(oldOffset));
    }
    parsePixels(value) {
        return value === null ? '' : `${value}px`;
    }
    parseLength(value) {
        if (value === null) {
            return '';
        }
        if (typeof value === 'string') {
            return value;
        }
        return `${value}px`;
    }
    applyFlex() {
        this.element.style.position = this.isLayoutRoot() ? 'absolute' : 'relative';
        const size = this.desiredSize();
        this.element.style.width = this.parseLength(size.x);
        this.element.style.height = this.parseLength(size.y);
        this.element.style.maxWidth = this.parseLength(this.maxWidth());
        this.element.style.minWidth = this.parseLength(this.minWidth());
        this.element.style.maxHeight = this.parseLength(this.maxHeight());
        this.element.style.minHeight = this.parseLength(this.minHeight());
        this.element.style.aspectRatio =
            this.ratio() === null ? '' : this.ratio().toString();
        this.element.style.marginTop = this.parsePixels(this.margin.top());
        this.element.style.marginBottom = this.parsePixels(this.margin.bottom());
        this.element.style.marginLeft = this.parsePixels(this.margin.left());
        this.element.style.marginRight = this.parsePixels(this.margin.right());
        this.element.style.paddingTop = this.parsePixels(this.padding.top());
        this.element.style.paddingBottom = this.parsePixels(this.padding.bottom());
        this.element.style.paddingLeft = this.parsePixels(this.padding.left());
        this.element.style.paddingRight = this.parsePixels(this.padding.right());
        this.element.style.flexDirection = this.direction();
        this.element.style.flexBasis = this.parseLength(this.basis());
        this.element.style.flexWrap = this.wrap();
        this.element.style.justifyContent = this.justifyContent();
        this.element.style.alignContent = this.alignContent();
        this.element.style.alignItems = this.alignItems();
        this.element.style.alignSelf = this.alignSelf();
        this.element.style.columnGap = this.parseLength(this.gap.x());
        this.element.style.rowGap = this.parseLength(this.gap.y());
        if (this.sizeLockCounter() > 0) {
            this.element.style.flexGrow = '0';
            this.element.style.flexShrink = '0';
        }
        else {
            this.element.style.flexGrow = this.grow().toString();
            this.element.style.flexShrink = this.shrink().toString();
        }
    }
    applyFont() {
        this.element.style.fontFamily = this.fontFamily.isInitial()
            ? ''
            : this.fontFamily();
        this.element.style.fontSize = this.fontSize.isInitial()
            ? ''
            : `${this.fontSize()}px`;
        this.element.style.fontStyle = this.fontStyle.isInitial()
            ? ''
            : this.fontStyle();
        if (this.lineHeight.isInitial()) {
            this.element.style.lineHeight = '';
        }
        else {
            const lineHeight = this.lineHeight();
            this.element.style.lineHeight =
                typeof lineHeight === 'string'
                    ? (parseFloat(lineHeight) / 100).toString()
                    : `${lineHeight}px`;
        }
        this.element.style.fontWeight = this.fontWeight.isInitial()
            ? ''
            : this.fontWeight().toString();
        this.element.style.letterSpacing = this.letterSpacing.isInitial()
            ? ''
            : `${this.letterSpacing()}px`;
        this.element.style.textAlign = this.textAlign.isInitial()
            ? ''
            : this.textAlign();
        if (this.textWrap.isInitial()) {
            this.element.style.whiteSpace = '';
        }
        else {
            const wrap = this.textWrap();
            if (typeof wrap === 'boolean') {
                this.element.style.whiteSpace = wrap ? 'normal' : 'nowrap';
            }
            else {
                this.element.style.whiteSpace = wrap;
            }
        }
    }
    dispose() {
        super.dispose();
        this.sizeLockCounter?.context.dispose();
        if (this.element) {
            this.element.remove();
            this.element.innerHTML = '';
        }
        this.element = null;
        this.styles = null;
    }
    hit(position) {
        const local = position.transformAsPoint(this.localToParent().inverse());
        if (this.cacheBBox().includes(local)) {
            return super.hit(position) ?? this;
        }
        return null;
    }
};
__decorate([
    initial(null),
    interpolation(boolLerp),
    signal()
], Layout.prototype, "layout", void 0);
__decorate([
    initial(null),
    signal()
], Layout.prototype, "maxWidth", void 0);
__decorate([
    initial(null),
    signal()
], Layout.prototype, "maxHeight", void 0);
__decorate([
    initial(null),
    signal()
], Layout.prototype, "minWidth", void 0);
__decorate([
    initial(null),
    signal()
], Layout.prototype, "minHeight", void 0);
__decorate([
    initial(null),
    signal()
], Layout.prototype, "ratio", void 0);
__decorate([
    spacingSignal('margin')
], Layout.prototype, "margin", void 0);
__decorate([
    spacingSignal('padding')
], Layout.prototype, "padding", void 0);
__decorate([
    initial('row'),
    signal()
], Layout.prototype, "direction", void 0);
__decorate([
    initial(null),
    signal()
], Layout.prototype, "basis", void 0);
__decorate([
    initial(0),
    signal()
], Layout.prototype, "grow", void 0);
__decorate([
    initial(1),
    signal()
], Layout.prototype, "shrink", void 0);
__decorate([
    initial('nowrap'),
    signal()
], Layout.prototype, "wrap", void 0);
__decorate([
    initial('start'),
    signal()
], Layout.prototype, "justifyContent", void 0);
__decorate([
    initial('normal'),
    signal()
], Layout.prototype, "alignContent", void 0);
__decorate([
    initial('stretch'),
    signal()
], Layout.prototype, "alignItems", void 0);
__decorate([
    initial('auto'),
    signal()
], Layout.prototype, "alignSelf", void 0);
__decorate([
    initial(0),
    vector2Signal({ x: 'columnGap', y: 'rowGap' })
], Layout.prototype, "gap", void 0);
__decorate([
    defaultStyle('font-family'),
    signal()
], Layout.prototype, "fontFamily", void 0);
__decorate([
    defaultStyle('font-size', parseFloat),
    signal()
], Layout.prototype, "fontSize", void 0);
__decorate([
    defaultStyle('font-style'),
    signal()
], Layout.prototype, "fontStyle", void 0);
__decorate([
    defaultStyle('font-weight', parseInt),
    signal()
], Layout.prototype, "fontWeight", void 0);
__decorate([
    defaultStyle('line-height', parseFloat),
    signal()
], Layout.prototype, "lineHeight", void 0);
__decorate([
    defaultStyle('letter-spacing', i => (i === 'normal' ? 0 : parseFloat(i))),
    signal()
], Layout.prototype, "letterSpacing", void 0);
__decorate([
    defaultStyle('white-space', i => (i === 'pre' ? 'pre' : i === 'normal')),
    signal()
], Layout.prototype, "textWrap", void 0);
__decorate([
    initial('inherit'),
    signal()
], Layout.prototype, "textDirection", void 0);
__decorate([
    defaultStyle('text-align'),
    signal()
], Layout.prototype, "textAlign", void 0);
__decorate([
    initial({ x: null, y: null }),
    vector2Signal({ x: 'width', y: 'height' })
], Layout.prototype, "size", void 0);
__decorate([
    threadable()
], Layout.prototype, "tweenWidth", null);
__decorate([
    threadable()
], Layout.prototype, "tweenHeight", null);
__decorate([
    computed()
], Layout.prototype, "desiredSize", null);
__decorate([
    threadable()
], Layout.prototype, "tweenSize", null);
__decorate([
    vector2Signal('offset')
], Layout.prototype, "offset", void 0);
__decorate([
    originSignal(Origin.Middle)
], Layout.prototype, "middle", void 0);
__decorate([
    originSignal(Origin.Top)
], Layout.prototype, "top", void 0);
__decorate([
    originSignal(Origin.Bottom)
], Layout.prototype, "bottom", void 0);
__decorate([
    originSignal(Origin.Left)
], Layout.prototype, "left", void 0);
__decorate([
    originSignal(Origin.Right)
], Layout.prototype, "right", void 0);
__decorate([
    originSignal(Origin.TopLeft)
], Layout.prototype, "topLeft", void 0);
__decorate([
    originSignal(Origin.TopRight)
], Layout.prototype, "topRight", void 0);
__decorate([
    originSignal(Origin.BottomLeft)
], Layout.prototype, "bottomLeft", void 0);
__decorate([
    originSignal(Origin.BottomRight)
], Layout.prototype, "bottomRight", void 0);
__decorate([
    initial(false),
    signal()
], Layout.prototype, "clip", void 0);
__decorate([
    initial(0),
    signal()
], Layout.prototype, "sizeLockCounter", void 0);
__decorate([
    computed()
], Layout.prototype, "parentTransform", null);
__decorate([
    computed()
], Layout.prototype, "anchorPosition", null);
__decorate([
    computed()
], Layout.prototype, "layoutEnabled", null);
__decorate([
    computed()
], Layout.prototype, "isLayoutRoot", null);
__decorate([
    computed()
], Layout.prototype, "scalingRotationMatrix", null);
__decorate([
    computed()
], Layout.prototype, "computedPosition", null);
__decorate([
    computed()
], Layout.prototype, "computedSize", null);
__decorate([
    computed()
], Layout.prototype, "requestLayoutUpdate", null);
__decorate([
    computed()
], Layout.prototype, "appendedToView", null);
__decorate([
    computed()
], Layout.prototype, "updateLayout", null);
__decorate([
    computed()
], Layout.prototype, "layoutChildren", null);
__decorate([
    computed()
], Layout.prototype, "requestFontUpdate", null);
__decorate([
    computed()
], Layout.prototype, "applyFlex", null);
__decorate([
    computed()
], Layout.prototype, "applyFont", null);
Layout = Layout_1 = __decorate([
    nodeName('Layout')
], Layout);
export { Layout };
function originSignal(origin) {
    return (target, key) => {
        signal()(target, key);
        cloneable(false)(target, key);
        const meta = getPropertyMeta(target, key);
        meta.parser = value => new Vector2(value);
        meta.getter = function () {
            return this.computedSize()
                .getOriginOffset(origin)
                .transformAsPoint(this.localToParent());
        };
        meta.setter = function (value) {
            this.position(modify(value, unwrapped => this.getOriginDelta(origin)
                .transform(this.scalingRotationMatrix())
                .flipped.add(unwrapped)));
            return this;
        };
    };
}
addInitializer(Layout.prototype, instance => {
    instance.element = document.createElement('div');
    instance.element.style.display = 'flex';
    instance.element.style.boxSizing = 'border-box';
    instance.styles = getComputedStyle(instance.element);
});
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiTGF5b3V0LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL0xheW91dC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQUEsT0FBTyxFQUNMLElBQUksRUFDSixRQUFRLEVBQ1IsU0FBUyxFQUVULE1BQU0sRUFDTixNQUFNLEVBQ04sY0FBYyxFQVNkLFVBQVUsRUFHVixLQUFLLEVBQ0wsT0FBTyxHQUVSLE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUNMLGNBQWMsRUFDZCxTQUFTLEVBQ1QsUUFBUSxFQUNSLFlBQVksRUFDWixlQUFlLEVBQ2YsT0FBTyxFQUNQLGFBQWEsRUFDYixRQUFRLEVBQ1IsTUFBTSxFQUVOLGFBQWEsR0FDZCxNQUFNLGVBQWUsQ0FBQztBQUN2QixPQUFPLEVBQUMsYUFBYSxFQUFDLE1BQU0sNkJBQTZCLENBQUM7QUFhMUQsT0FBTyxFQUFDLFFBQVEsRUFBRSxTQUFTLEVBQUUsRUFBRSxFQUFDLE1BQU0sVUFBVSxDQUFDO0FBQ2pELE9BQU8sRUFBQyxJQUFJLEVBQVksTUFBTSxRQUFRLENBQUM7QUE2SWhDLElBQU0sTUFBTSxjQUFaLE1BQU0sTUFBTyxTQUFRLElBQUk7SUEyRDlCLElBQVcsU0FBUztRQUNsQixPQUFPLElBQUksQ0FBQyxHQUFHLENBQUMsQ0FBQyxDQUFDO0lBQ3BCLENBQUM7SUFDRCxJQUFXLE1BQU07UUFDZixPQUFPLElBQUksQ0FBQyxHQUFHLENBQUMsQ0FBQyxDQUFDO0lBQ3BCLENBQUM7SUErQlMsSUFBSTtRQUNaLElBQUksSUFBSSxDQUFDLFlBQVksRUFBRSxFQUFFO1lBQ3ZCLE9BQU8sSUFBSSxDQUFDLENBQUMsQ0FBQyxPQUFPLENBQUMsTUFBTSxFQUFFLENBQUM7U0FDaEM7UUFFRCxPQUFPLElBQUksQ0FBQyxnQkFBZ0IsRUFBRSxDQUFDLENBQUMsQ0FBQztJQUNuQyxDQUFDO0lBQ1MsSUFBSSxDQUFDLEtBQTBCO1FBQ3ZDLElBQUks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