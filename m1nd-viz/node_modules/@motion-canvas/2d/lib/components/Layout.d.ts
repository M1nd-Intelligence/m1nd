import { BBox, Direction, InterpolationFunction, Origin, PossibleSpacing, PossibleVector2, SerializedVector2, Signal, SignalValue, SimpleSignal, SimpleVector2Signal, SpacingSignal, ThreadGenerator, TimingFunction, Vector2, Vector2Signal } from '@motion-canvas/core';
import { Vector2LengthSignal } from '../decorators';
import { DesiredLength, FlexBasis, FlexContent, FlexDirection, FlexItems, FlexWrap, LayoutMode, Length, LengthLimit, TextWrap } from '../partials';
import { Node, NodeProps } from './Node';
export interface LayoutProps extends NodeProps {
    layout?: LayoutMode;
    tagName?: keyof HTMLElementTagNameMap;
    width?: SignalValue<Length>;
    height?: SignalValue<Length>;
    maxWidth?: SignalValue<LengthLimit>;
    maxHeight?: SignalValue<LengthLimit>;
    minWidth?: SignalValue<LengthLimit>;
    minHeight?: SignalValue<LengthLimit>;
    ratio?: SignalValue<number>;
    marginTop?: SignalValue<number>;
    marginBottom?: SignalValue<number>;
    marginLeft?: SignalValue<number>;
    marginRight?: SignalValue<number>;
    margin?: SignalValue<PossibleSpacing>;
    paddingTop?: SignalValue<number>;
    paddingBottom?: SignalValue<number>;
    paddingLeft?: SignalValue<number>;
    paddingRight?: SignalValue<number>;
    padding?: SignalValue<PossibleSpacing>;
    direction?: SignalValue<FlexDirection>;
    basis?: SignalValue<FlexBasis>;
    grow?: SignalValue<number>;
    shrink?: SignalValue<number>;
    wrap?: SignalValue<FlexWrap>;
    justifyContent?: SignalValue<FlexContent>;
    alignContent?: SignalValue<FlexContent>;
    alignItems?: SignalValue<FlexItems>;
    alignSelf?: SignalValue<FlexItems>;
    rowGap?: SignalValue<Length>;
    columnGap?: SignalValue<Length>;
    gap?: SignalValue<PossibleVector2<Length>>;
    fontFamily?: SignalValue<string>;
    fontSize?: SignalValue<number>;
    fontStyle?: SignalValue<string>;
    fontWeight?: SignalValue<number>;
    lineHeight?: SignalValue<Length>;
    letterSpacing?: SignalValue<number>;
    textWrap?: SignalValue<TextWrap>;
    textDirection?: SignalValue<CanvasDirection>;
    textAlign?: SignalValue<CanvasTextAlign>;
    size?: SignalValue<PossibleVector2<Length>>;
    offsetX?: SignalValue<number>;
    offsetY?: SignalValue<number>;
    offset?: SignalValue<PossibleVector2>;
    /**
     * The position of the center of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the center ends
     * up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     * When {@link offset} is not set, this will be the same as the
     * {@link NodeProps.position}.
     */
    middle?: SignalValue<PossibleVector2>;
    /**
     * The position of the top edge of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the top edge
     * ends up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     */
    top?: SignalValue<PossibleVector2>;
    /**
     * The position of the bottom edge of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the bottom edge
     * ends up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     */
    bottom?: SignalValue<PossibleVector2>;
    /**
     * The position of the left edge of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the left edge
     * ends up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     */
    left?: SignalValue<PossibleVector2>;
    /**
     * The position of the right edge of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the right edge
     * ends up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     */
    right?: SignalValue<PossibleVector2>;
    /**
     * The position of the top left corner of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the top left
     * corner ends up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     */
    topLeft?: SignalValue<PossibleVector2>;
    /**
     * The position of the top right corner of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the top right
     * corner ends up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     */
    topRight?: SignalValue<PossibleVector2>;
    /**
     * The position of the bottom left corner of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the bottom left
     * corner ends up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     */
    bottomLeft?: SignalValue<PossibleVector2>;
    /**
     * The position of the bottom right corner of this node.
     *
     * @remarks
     * This shortcut property will set the node's position so that the bottom
     * right corner ends up in the given place.
     * If present, overrides the {@link NodeProps.position} property.
     */
    bottomRight?: SignalValue<PossibleVector2>;
    clip?: SignalValue<boolean>;
}
export declare class Layout extends Node {
    readonly layout: SimpleSignal<LayoutMode, this>;
    readonly maxWidth: SimpleSignal<LengthLimit, this>;
    readonly maxHeight: SimpleSignal<LengthLimit, this>;
    readonly minWidth: SimpleSignal<LengthLimit, this>;
    readonly minHeight: SimpleSignal<LengthLimit, this>;
    readonly ratio: SimpleSignal<number | null, this>;
    readonly margin: SpacingSignal<this>;
    readonly padding: SpacingSignal<this>;
    readonly direction: SimpleSignal<FlexDirection, this>;
    readonly basis: SimpleSignal<FlexBasis, this>;
    readonly grow: SimpleSignal<number, this>;
    readonly shrink: SimpleSignal<number, this>;
    readonly wrap: SimpleSignal<FlexWrap, this>;
    readonly justifyContent: SimpleSignal<FlexContent, this>;
    readonly alignContent: SimpleSignal<FlexContent, this>;
    readonly alignItems: SimpleSignal<FlexItems, this>;
    readonly alignSelf: SimpleSignal<FlexItems, this>;
    readonly gap: Vector2LengthSignal<this>;
    get columnGap(): Signal<Length, number, this>;
    get rowGap(): Signal<Length, number, this>;
    readonly fontFamily: SimpleSignal<string, this>;
    readonly fontSize: SimpleSignal<number, this>;
    readonly fontStyle: SimpleSignal<string, this>;
    readonly fontWeight: SimpleSignal<number, this>;
    readonly lineHeight: SimpleSignal<Length, this>;
    readonly letterSpacing: SimpleSignal<number, this>;
    readonly textWrap: SimpleSignal<TextWrap, this>;
    readonly textDirection: SimpleSignal<CanvasDirection, this>;
    readonly textAlign: SimpleSignal<CanvasTextAlign, this>;
    protected getX(): number;
    protected setX(value: SignalValue<number>): void;
    protected getY(): number;
    protected setY(value: SignalValue<number>): void;
    /**
     * Represents the size of this node.
     *
     * @remarks
     * A size is a two-dimensional vector, where `x` represents the `width`, and `y`
     * represents the `height`.
     *
     * The value of both x and y is of type {@link partials.Length} which is
     * either:
     * - `number` - the desired length in pixels
     * - `${number}%` - a string with the desired length in percents, for example
     *                  `'50%'`
     * - `null` - an automatic length
     *
     * When retrieving the size, all units are converted to pixels, using the
     * current state of the layout. For example, retrieving the width set to
     * `'50%'`, while the parent has a width of `200px` will result in the number
     * `100` being returned.
     *
     * When the node is not part of the layout, setting its size using percents
     * refers to the size of the entire scene.
     *
     * @example
     * Initializing the size:
     * ```tsx
     * // with a possible vector:
     * <Node size={['50%', 200]} />
     * // with individual components:
     * <Node width={'50%'} height={200} />
     * ```
     *
     * Accessing the size:
     * ```tsx
     * // retrieving the vector:
     * const size = node.size();
     * // retrieving an individual component:
     * const width = node.size.x();
     * ```
     *
     * Setting the size:
     * ```tsx
     * // with a possible vector:
     * node.size(['50%', 200]);
     * node.size(() => ['50%', 200]);
     * // with individual components:
     * node.size.x('50%');
     * node.size.x(() => '50%');
     * ```
     */
    readonly size: Vector2LengthSignal<this>;
    get width(): Signal<Length, number, this>;
    get height(): Signal<Length, number, this>;
    protected getWidth(): number;
    protected setWidth(value: SignalValue<Length>): void;
    protected tweenWidth(value: SignalValue<Length>, time: number, timingFunction: TimingFunction, interpolationFunction: InterpolationFunction<Length>): ThreadGenerator;
    protected getHeight(): number;
    protected setHeight(value: SignalValue<Length>): void;
    protected tweenHeight(value: SignalValue<Length>, time: number, timingFunction: TimingFunction, interpolationFunction: InterpolationFunction<Length>): ThreadGenerator;
    /**
     * Get the desired size of this node.
     *
     * @remarks
     * This method can be used to control the size using external factors.
     * By default, the returned size is the same as the one declared by the user.
     */
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected tweenSize(value: SignalValue<SerializedVector2<Length>>, time: number, timingFunction: TimingFunction, interpolationFunction: InterpolationFunction<Vector2>): ThreadGenerator;
    /**
     * Represents the offset of this node's origin.
     *
     * @remarks
     * By default, the origin of a node is located at its center. The origin
     * serves as the pivot point when rotating and scaling a node, but it doesn't
     * affect the placement of its children.
     *
     * The value is relative to the size of this node. A value of `1` means as far
     * to the right/bottom as possible. Here are a few examples of offsets:
     * - `[-1, -1]` - top left corner
     * - `[1, -1]` - top right corner
     * - `[0, 1]` - bottom edge
     * - `[-1, 1]` - bottom left corner
     */
    readonly offset: Vector2Signal<this>;
    /**
     * The position of the center of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the center ends up in the given place.
     *
     * If the {@link offset} has not been changed, this will be the same as the
     * {@link position}.
     *
     * When retrieved, it will return the position of the center in the parent
     * space.
     */
    readonly middle: SimpleVector2Signal<this>;
    /**
     * The position of the top edge of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the top edge ends up in the given place.
     *
     * When retrieved, it will return the position of the top edge in the parent
     * space.
     */
    readonly top: SimpleVector2Signal<this>;
    /**
     * The position of the bottom edge of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the bottom edge ends up in the given place.
     *
     * When retrieved, it will return the position of the bottom edge in the
     * parent space.
     */
    readonly bottom: SimpleVector2Signal<this>;
    /**
     * The position of the left edge of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the left edge ends up in the given place.
     *
     * When retrieved, it will return the position of the left edge in the parent
     * space.
     */
    readonly left: SimpleVector2Signal<this>;
    /**
     * The position of the right edge of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the right edge ends up in the given place.
     *
     * When retrieved, it will return the position of the right edge in the parent
     * space.
     */
    readonly right: SimpleVector2Signal<this>;
    /**
     * The position of the top left corner of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the top left corner ends up in the given place.
     *
     * When retrieved, it will return the position of the top left corner in the
     * parent space.
     */
    readonly topLeft: SimpleVector2Signal<this>;
    /**
     * The position of the top right corner of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the top right corner ends up in the given place.
     *
     * When retrieved, it will return the position of the top right corner in the
     * parent space.
     */
    readonly topRight: SimpleVector2Signal<this>;
    /**
     * The position of the bottom left corner of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the bottom left corner ends up in the given place.
     *
     * When retrieved, it will return the position of the bottom left corner in
     * the parent space.
     */
    readonly bottomLeft: SimpleVector2Signal<this>;
    /**
     * The position of the bottom right corner of this node.
     *
     * @remarks
     * When set, this shortcut property will modify the node's position so that
     * the bottom right corner ends up in the given place.
     *
     * When retrieved, it will return the position of the bottom right corner in
     * the parent space.
     */
    readonly bottomRight: SimpleVector2Signal<this>;
    /**
     * Get the cardinal point corresponding to the given origin.
     *
     * @param origin - The origin or direction of the point.
     */
    cardinalPoint(origin: Origin | Direction): SimpleVector2Signal<this>;
    readonly clip: SimpleSignal<boolean, this>;
    element: HTMLElement;
    styles: CSSStyleDeclaration;
    protected readonly sizeLockCounter: SimpleSignal<number, this>;
    constructor(props: LayoutProps);
    lockSize(): void;
    releaseSize(): void;
    protected parentTransform(): Layout | null;
    anchorPosition(): Vector2;
    /**
     * Get the resolved layout mode of this node.
     *
     * @remarks
     * When the mode is `null`, its value will be inherited from the parent.
     *
     * Use {@link layout} to get the raw mode set for this node (without
     * inheritance).
     */
    layoutEnabled(): boolean;
    isLayoutRoot(): boolean;
    localToParent(): DOMMatrix;
    /**
     * A simplified version of {@link localToParent} matrix used for transforming
     * direction vectors.
     *
     * @internal
     */
    protected scalingRotationMatrix(): DOMMatrix;
    protected getComputedLayout(): BBox;
    computedPosition(): Vector2;
    protected computedSize(): Vector2;
    /**
     * Find the closest layout root and apply any new layout changes.
     */
    protected requestLayoutUpdate(): void;
    protected appendedToView(): boolean;
    /**
     * Apply any new layout changes to this node and its children.
     */
    protected updateLayout(): void;
    protected layoutChildren(): Layout[];
    /**
     * Apply any new font changes to this node and all of its ancestors.
     */
    protected requestFontUpdate(): void;
    protected getCacheBBox(): BBox;
    protected draw(context: CanvasRenderingContext2D): void;
    drawOverlay(context: CanvasRenderingContext2D, matrix: DOMMatrix): void;
    getOriginDelta(origin: Origin): Vector2;
    /**
     * Update the offset of this node and adjust the position to keep it in the
     * same place.
     *
     * @param offset - The new offset.
     */
    moveOffset(offset: Vector2): void;
    protected parsePixels(value: number | null): string;
    protected parseLength(value: number | string | null): string;
    protected applyFlex(): void;
    protected applyFont(): void;
    dispose(): void;
    hit(position: Vector2): Node | null;
}
//# sourceMappingURL=Layout.d.ts.map