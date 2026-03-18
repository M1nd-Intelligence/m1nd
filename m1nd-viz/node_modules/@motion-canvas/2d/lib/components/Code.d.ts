import { BBox, SerializedVector2, Signal, SignalValue, SimpleSignal, ThreadGenerator, TimingFunction, Vector2 } from '@motion-canvas/core';
import { CodeFragmentDrawingInfo, CodeHighlighter, CodePoint, CodeRange, CodeSelection, CodeSignal, PossibleCodeScope, PossibleCodeSelection } from '../code';
import { DesiredLength } from '../partials';
import { Shape, ShapeProps } from './Shape';
/**
 * @experimental
 */
export interface DrawTokenHook {
    (ctx: CanvasRenderingContext2D, text: string, position: Vector2, color: string, selection: number): void;
}
/**
 * Describes custom drawing logic used by the Code node.
 *
 * @experimental
 */
export interface DrawHooks {
    /**
     * Custom drawing logic for individual code tokens.
     *
     * @example
     * ```ts
     * token(ctx, text, position, color, selection) {
     *   const blur = map(3, 0, selection);
     *   const alpha = map(0.5, 1, selection);
     *   ctx.globalAlpha *= alpha;
     *   ctx.filter = `blur(${blur}px)`;
     *   ctx.fillStyle = color;
     *   ctx.fillText(text, position.x, position.y);
     * }
     * ```
     */
    token: DrawTokenHook;
}
export interface CodeProps extends ShapeProps {
    /**
     * {@inheritDoc Code.highlighter}
     */
    highlighter?: SignalValue<CodeHighlighter | null>;
    /**
     * {@inheritDoc Code.code}
     */
    code?: SignalValue<PossibleCodeScope>;
    /**
     * {@inheritDoc Code.selection}
     */
    selection?: SignalValue<PossibleCodeSelection>;
    /**
     * {@inheritDoc Code.drawHooks}
     */
    drawHooks?: SignalValue<DrawHooks>;
}
/**
 * A node for displaying and animating code.
 *
 * @preview
 * ```tsx editor
 * import {Code, makeScene2D} from '@motion-canvas/2d';
 * import {createRef} from '@motion-canvas/core';
 *
 * export default makeScene2D(function* (view) {
 *   const code = createRef<Code>();
 *
 *   view.add(
 *     <Code
 *       ref={code}
 *       offset={-1}
 *       position={view.size().scale(-0.5).add(60)}
 *       fontFamily={'JetBrains Mono, monospace'}
 *       fontSize={36}
 *       code={`\
 * function hello() {
 *   console.log('Hello');
 * }`}
 *     />,
 *   );
 *
 *   yield* code()
 *     .code(
 *       `\
 * function hello() {
 *   console.warn('Hello World');
 * }`,
 *       1,
 *     )
 *     .wait(0.5)
 *     .back(1)
 *     .wait(0.5);
 * });
 * ```
 */
export declare class Code extends Shape {
    /**
     * Create a standalone code signal.
     *
     * @param initial - The initial code.
     * @param highlighter - Custom highlighter to use.
     */
    static createSignal(initial: SignalValue<PossibleCodeScope>, highlighter?: SignalValue<CodeHighlighter>): CodeSignal<void>;
    static defaultHighlighter: CodeHighlighter | null;
    /**
     * The code highlighter to use for this code node.
     *
     * @remarks
     * Defaults to a shared {@link code.LezerHighlighter}.
     */
    readonly highlighter: SimpleSignal<CodeHighlighter | null, this>;
    /**
     * The code to display.
     */
    readonly code: CodeSignal<this>;
    /**
     * Custom drawing logic for the code.
     *
     * @remarks
     * Check out {@link DrawHooks} for available render hooks.
     *
     * @experimental
     *
     * @example
     * Make the unselected code blurry and transparent:
     * ```tsx
     * <Code
     *   drawHooks={{
     *     token(ctx, text, position, color, selection) {
     *       const blur = map(3, 0, selection);
     *       const alpha = map(0.5, 1, selection);
     *       ctx.globalAlpha *= alpha;
     *       ctx.filter = `blur(${blur}px)`;
     *       ctx.fillStyle = color;
     *       ctx.fillText(text, position.x, position.y);
     *     },
     *   }}
     *   // ...
     * />
     * ```
     */
    readonly drawHooks: SimpleSignal<DrawHooks, this>;
    protected setDrawHooks(value: DrawHooks): void;
    /**
     * The currently selected code range.
     *
     * @remarks
     * Either a single {@link code.CodeRange} or an array of them
     * describing which parts of the code should be visually emphasized.
     *
     * You can use {@link code.word} and
     * {@link code.lines} to quickly create ranges.
     *
     * @example
     * The following will select the word "console" in the code.
     * Both lines and columns are 0-based. So it will select a 7-character-long
     * (`7`) word in the second line (`1`) starting at the third character (`2`).
     * ```tsx
     * <Code
     *   selection={word(1, 2, 7)}
     *   code={`\
     * function hello() => {
     *   console.log('Hello');
     * }`}
     *   // ...
     * />
     * ```
     */
    readonly selection: Signal<PossibleCodeSelection, CodeSelection, this>;
    oldSelection: CodeSelection | null;
    selectionProgress: SimpleSignal<number | null, void>;
    protected tweenSelection(value: CodeRange[], duration: number, timingFunction: TimingFunction): ThreadGenerator;
    /**
     * Get the currently displayed code as a string.
     */
    parsed(): string;
    highlighterCache(): {
        before: unknown;
        after: unknown;
    } | null;
    private cursorCache;
    private get cursor();
    constructor(props: CodeProps);
    /**
     * Create a child code signal.
     *
     * @param initial - The initial code.
     */
    createSignal(initial: SignalValue<PossibleCodeScope>): CodeSignal<this>;
    /**
     * Find all code ranges that match the given pattern.
     *
     * @param pattern - Either a string or a regular expression to match.
     */
    findAllRanges(pattern: string | RegExp): CodeRange[];
    /**
     * Find the first code range that matches the given pattern.
     *
     * @param pattern - Either a string or a regular expression to match.
     */
    findFirstRange(pattern: string | RegExp): CodeRange;
    /**
     * Find the last code range that matches the given pattern.
     *
     * @param pattern - Either a string or a regular expression to match.
     */
    findLastRange(pattern: string | RegExp): CodeRange;
    /**
     * Return the bounding box of the given point (character) in the code.
     *
     * @remarks
     * The returned bound box is in local space of the `Code` node.
     *
     * @param point - The point to get the bounding box for.
     */
    getPointBBox(point: CodePoint): BBox;
    /**
     * Return bounding boxes of all characters in the selection.
     *
     * @remarks
     * The returned bounding boxes are in local space of the `Code` node.
     * Each line of code has a separate bounding box.
     *
     * @param selection - The selection to get the bounding boxes for.
     */
    getSelectionBBox(selection: PossibleCodeSelection): BBox[];
    protected drawingInfo(): {
        fragments: CodeFragmentDrawingInfo[];
        verticalOffset: number;
        fontHeight: number;
    };
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected draw(context: CanvasRenderingContext2D): void;
    protected applyText(context: CanvasRenderingContext2D): void;
    protected collectAsyncResources(): void;
}
//# sourceMappingURL=Code.d.ts.map