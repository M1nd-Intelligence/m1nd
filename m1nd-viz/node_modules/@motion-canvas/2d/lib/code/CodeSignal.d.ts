import { Signal, SignalContext, SignalValue, ThreadGenerator, TimingFunction } from '@motion-canvas/core';
import { CodeHighlighter } from './CodeHighlighter';
import { CodePoint, CodeRange } from './CodeRange';
import { CodeScope, CodeTag, PossibleCodeScope } from './CodeScope';
interface CodeModifier<TOwner> {
    (code: CodeTag): TOwner;
    (code: CodeTag, duration: number): ThreadGenerator;
    (duration?: number): TagGenerator;
}
interface CodeInsert<TOwner> {
    (point: CodePoint, code: CodeTag): TOwner;
    (point: CodePoint, code: CodeTag, duration: number): ThreadGenerator;
    (point: CodePoint, duration?: number): TagGenerator;
}
interface CodeRemove<TOwner> {
    (range: CodeRange): TOwner;
    (range: CodeRange, duration: number): ThreadGenerator;
}
interface CodeReplace<TOwner> {
    (range: CodeRange, code: CodeTag): TOwner;
    (range: CodeRange, code: CodeTag, duration: number): ThreadGenerator;
    (range: CodeRange, duration?: number): TagGenerator;
}
type TagGenerator = (strings: TemplateStringsArray, ...tags: CodeTag[]) => ThreadGenerator;
export interface CodeSignalHelpers<TOwner> {
    edit(duration?: number): TagGenerator;
    append: CodeModifier<TOwner>;
    prepend: CodeModifier<TOwner>;
    insert: CodeInsert<TOwner>;
    remove: CodeRemove<TOwner>;
    replace: CodeReplace<TOwner>;
}
export type CodeSignal<TOwner> = Signal<PossibleCodeScope, CodeScope, TOwner, CodeSignalContext<TOwner>> & CodeSignalHelpers<TOwner>;
export declare class CodeSignalContext<TOwner> extends SignalContext<PossibleCodeScope, CodeScope, TOwner> implements CodeSignalHelpers<TOwner> {
    private readonly highlighter?;
    private readonly progress;
    constructor(initial: SignalValue<PossibleCodeScope>, owner: TOwner, highlighter?: SignalValue<CodeHighlighter<unknown> | null> | undefined);
    tweener(value: SignalValue<PossibleCodeScope>, duration: number, timingFunction: TimingFunction): ThreadGenerator;
    edit(duration?: number): TagGenerator;
    append(code: CodeTag): TOwner;
    append(code: CodeTag, duration: number): ThreadGenerator;
    append(duration?: number): TagGenerator;
    prepend(code: CodeTag): TOwner;
    prepend(code: CodeTag, duration: number): ThreadGenerator;
    prepend(duration?: number): TagGenerator;
    insert(point: CodePoint, code: CodeTag): TOwner;
    insert(point: CodePoint, code: CodeTag, duration: number): ThreadGenerator;
    insert(point: CodePoint, duration?: number): TagGenerator;
    remove(range: CodeRange): TOwner;
    remove(range: CodeRange, duration: number): ThreadGenerator;
    replace(range: CodeRange, code: CodeTag): TOwner;
    replace(range: CodeRange, code: CodeTag, duration: number): ThreadGenerator;
    replace(range: CodeRange, duration?: number): TagGenerator;
    private replaceTween;
    private editTween;
    private appendTween;
    private prependTween;
    parse(value: PossibleCodeScope): CodeScope;
    toSignal(): CodeSignal<TOwner>;
}
export declare function codeSignal(): PropertyDecorator;
export {};
//# sourceMappingURL=CodeSignal.d.ts.map