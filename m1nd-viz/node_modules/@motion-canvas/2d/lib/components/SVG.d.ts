import { SerializedVector2, SignalValue, SimpleSignal, ThreadGenerator, TimingFunction, Vector2 } from '@motion-canvas/core';
import { DesiredLength } from '../partials';
import { Node, NodeProps } from './Node';
import { Shape, ShapeProps } from './Shape';
/**
 * Represent SVG shape.
 * This only used single time because `node` may have reference to parent SVG renderer.
 */
export interface SVGShape {
    id: string;
    shape: Node;
}
/**
 * Data of SVGShape.
 * This can used many times  because it do not reference parent SVG.
 * This must build into SVGShape
 */
export interface SVGShapeData {
    id: string;
    type: new (props: NodeProps) => Node;
    props: ShapeProps;
    children?: SVGShapeData[];
}
/**
 * Represent SVG document that contains SVG shapes.
 * This only used single time because `nodes` have reference to parent SVG renderer.
 */
export interface SVGDocument {
    size: Vector2;
    nodes: SVGShape[];
}
/**
 * Data of SVGDocument.
 * This can used many times because it do not reference parent SVG.
 * This must build into SVGDocument
 */
export interface SVGDocumentData {
    size: Vector2;
    nodes: SVGShapeData[];
}
export interface SVGProps extends ShapeProps {
    svg: SignalValue<string>;
}
/**
A Node for drawing and animating SVG images.

@remarks
If you're not interested in animating SVG, you can use {@link Img} instead.
 */
export declare class SVG extends Shape {
    protected static containerElement: HTMLDivElement;
    private static svgNodesPool;
    /**
     * SVG string to be rendered
     */
    readonly svg: SimpleSignal<string, this>;
    /**
     * Child to wrap all SVG node
     */
    wrapper: Node;
    private lastTweenTargetSrc;
    private lastTweenTargetDocument;
    constructor(props: SVGProps);
    /**
     * Get all SVG nodes with the given id.
     * @param id - An id to query.
     */
    getChildrenById(id: string): Node[];
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected getCurrentSize(): {
        x: number | null;
        y: number | null;
    };
    protected calculateWrapperScale(documentSize: Vector2, parentSize: SerializedVector2<number | null>): Vector2;
    /**
     * Convert `SVGDocumentData` to `SVGDocument`.
     * @param data - `SVGDocumentData` to convert.
     */
    protected buildDocument(data: SVGDocumentData): SVGDocument;
    /**
     * Convert `SVGShapeData` to `SVGShape`.
     * @param data - `SVGShapeData` to convert.
     */
    protected buildShape({ id, type, props, children }: SVGShapeData): SVGShape;
    /**
     * Convert an SVG string to `SVGDocument`.
     * @param svg - An SVG string to be parsed.
     */
    protected parseSVG(svg: string): SVGDocument;
    /**
     * Create a tweening list to tween between two SVG nodes.
     * @param from - The initial node,
     * @param to - The final node.
     * @param duration - The duration of the tween.
     * @param timing - The timing function.
     */
    protected generateTransformer(from: Node, to: Node, duration: number, timing: TimingFunction): Generator<ThreadGenerator>;
    protected tweenSvg(value: SignalValue<string>, time: number, timingFunction: TimingFunction): Generator<void | ThreadGenerator | Promise<any> | import("@motion-canvas/core").Promisable<any>, void, any>;
    private wrapperScale;
    /**
     * Get the current `SVGDocument`.
     */
    private document;
    /**
     * Get current document nodes.
     */
    private documentNodes;
    /**
     * Convert SVG colors in Shape properties to Motion Canvas colors.
     * @param param - Shape properties.
     * @returns Converted Shape properties.
     */
    private processElementStyle;
    /**
     * Parse an SVG string as `SVGDocumentData`.
     * @param svg - And SVG string to be parsed.
     * @returns `SVGDocumentData` that can be used to build SVGDocument.
     */
    protected static parseSVGData(svg: string): SVGDocumentData;
    /**
     * Get position, rotation and scale from Matrix transformation as Shape properties
     * @param transform - Matrix transformation
     * @returns MotionCanvas Shape properties
     */
    protected static getMatrixTransformation(transform: DOMMatrix): ShapeProps;
    /**
     * Convert an SVG color into a Motion Canvas color.
     * @param color - SVG color.
     * @returns Motion Canvas color.
     */
    private static processSVGColor;
    /**
     * Get the final transformation matrix for the given SVG element.
     * @param element - SVG element.
     * @param parentTransform - The transformation matrix of the parent.
     */
    private static getElementTransformation;
    private static parseLineCap;
    private static parseLineJoin;
    private static parseLineDash;
    private static parseDashOffset;
    private static parseOpacity;
    /**
     * Convert the SVG element's style to a Motion Canvas Shape properties.
     * @param element - An SVG element whose style should be converted.
     * @param inheritedStyle - The parent style that should be inherited.
     */
    private static getElementStyle;
    /**
     * Extract `SVGShapeData` list from the SVG element's children.
     * This will not extract the current element's shape.
     * @param element - An element whose children will be extracted.
     * @param svgRoot - The SVG root ("svg" tag) of the element.
     * @param parentTransform - The transformation matrix applied to the parent.
     * @param inheritedStyle - The style of the current SVG `element` that the children should inherit.
     */
    private static extractGroupNodes;
    /**
     * Parse a number from an SVG element attribute.
     * @param element - SVG element whose attribute will be parsed.
     * @param name - The name of the attribute to parse.
     * @returns a parsed number or `0` if the attribute is not defined.
     */
    private static parseNumberAttribute;
    /**
     * Extract `SVGShapeData` list from the SVG element.
     * This will also recursively extract shapes from its children.
     * @param child - An SVG element to extract.
     * @param svgRoot - The SVG root ("svg" tag) of the element.
     * @param parentTransform - The transformation matrix applied to the parent.
     * @param inheritedStyle - The style of the parent SVG element that the element should inherit.
     */
    private static extractElementNodes;
}
//# sourceMappingURL=SVG.d.ts.map