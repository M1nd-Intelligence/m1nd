import { SerializedVector2, Signal, SignalValue, SimpleSignal, ThreadGenerator, TimingFunction, Vector2 } from '@motion-canvas/core';
import { Code, CodeStyle, CodeTree, Token } from 'code-fns';
import { DesiredLength } from '../partials';
import { Shape, ShapeProps } from './Shape';
type CodePoint = [number, number];
type CodeRange = [CodePoint, CodePoint];
export interface CodeProps extends ShapeProps {
    language?: string;
    children?: Code;
    code?: SignalValue<Code>;
    selection?: CodeRange[];
    theme?: CodeStyle;
}
export interface CodeModification {
    from: Code;
    to: Code;
}
/**
 * @deprecated Use {@link Code} instead.
 */
export declare class CodeBlock extends Shape {
    private static initialized;
    readonly language: SimpleSignal<string, this>;
    readonly code: Signal<Code, CodeTree, this>;
    readonly theme: Signal<CodeStyle | null, CodeStyle, this>;
    readonly selection: SimpleSignal<CodeRange[], this>;
    protected tweenSelection(value: CodeRange[], duration: number, timingFunction: TimingFunction): ThreadGenerator;
    readonly unselectedOpacity: SimpleSignal<number, this>;
    private codeProgress;
    private selectionProgress;
    private oldSelection;
    private diffed;
    private currentLineCount;
    private newLineCount;
    protected getLineCountOfTokenArray(tokens: Token[]): number;
    lineCount(): number;
    protected parsed(): Token[];
    constructor({ children, ...rest }: CodeProps);
    protected characterSize(): Vector2;
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected getTokensSize(tokens: Token[]): {
        x: number;
        y: number;
    };
    protected collectAsyncResources(): void;
    set(strings: string[], ...rest: any[]): void;
    /**
     * Smoothly edit the code.
     *
     * @remarks
     * This method returns a tag function that should be used together with a
     * template literal to define what to edit. Expressions can be used to either
     * {@link insert}, {@link remove}, or {@link edit} the code.
     *
     * @example
     * ```ts
     * yield* codeBlock().edit()`
     *   const ${edit('a', 'b')} = [${insert('1, 2, 3')}];${remove(`
     *   // this comment will be removed`)}
     * `;
     * ```
     *
     * @param duration - The duration of the transition.
     * @param changeSelection - When set to `true`, the selection will be modified
     *                          to highlight the newly inserted code. Setting it
     *                          to `false` leaves the selection untouched.
     *                          Providing a custom {@link CodeRange} will select
     *                          it instead.
     */
    edit(duration?: number, changeSelection?: CodeRange[] | boolean): (strings: TemplateStringsArray, ...rest: (Code | CodeModification)[]) => ThreadGenerator;
    tweenCode(code: CodeTree, time: number, timingFunction: TimingFunction): Generator<void | ThreadGenerator | Promise<any> | import("@motion-canvas/core").Promisable<any>, void, any>;
    protected draw(context: CanvasRenderingContext2D): void;
    protected selectionStrength(x: number, y: number): number;
    protected static selectionStrength(selection: CodeRange[], x: number, y: number): number;
}
/**
 * Create a code modification that inserts a piece of code.
 *
 * @remarks
 * Should be used in conjunction with {@link CodeBlock.edit}.
 *
 * @param content - The code to insert.
 */
export declare function insert(content: Code): CodeModification;
/**
 * Create a code modification that removes a piece of code.
 *
 * @remarks
 * Should be used in conjunction with {@link CodeBlock.edit}.
 *
 * @param content - The code to remove.
 */
export declare function remove(content: Code): CodeModification;
/**
 * Create a code modification that changes one piece of code into another.
 *
 * @remarks
 * Should be used in conjunction with {@link CodeBlock.edit}.
 *
 * @param from - The code to change from.
 * @param to - The code to change to.
 */
export declare function edit(from: Code, to: Code): CodeModification;
/**
 * Create a selection range that highlights the given lines.
 *
 * @param from - The line from which the selection starts.
 * @param to - The line at which the selection ends. If omitted, the selection
 *             will cover only one line.
 */
export declare function lines(from: number, to?: number): CodeRange[];
/**
 * Create a selection range that highlights the given word.
 *
 * @param line - The line at which the word appears.
 * @param from - The column at which the word starts.
 * @param length - The length of the word. If omitted, the selection will cover
 *                 the rest of the line.
 */
export declare function word(line: number, from: number, length?: number): CodeRange[];
/**
 * Create a custom selection range.
 *
 * @param startLine - The line at which the selection starts.
 * @param startColumn - The column at which the selection starts.
 * @param endLine - The line at which the selection ends.
 * @param endColumn - The column at which the selection ends.
 */
export declare function range(startLine: number, startColumn: number, endLine: number, endColumn: number): CodeRange[];
export {};
//# sourceMappingURL=CodeBlock.d.ts.map