import { SignalValue } from '@motion-canvas/core';
import { PossibleCodeFragment } from './CodeFragment';
export interface CodeScope {
    progress: SignalValue<number>;
    fragments: CodeTag[];
}
export type PossibleCodeScope = CodeScope | CodeTag[] | string;
export type CodeTag = SignalValue<PossibleCodeFragment | CodeScope | CodeTag[]>;
export declare function CODE(strings: TemplateStringsArray, ...tags: CodeTag[]): CodeTag[];
export declare function isCodeScope(value: any): value is CodeScope;
export declare function parseCodeScope(value: PossibleCodeScope): CodeScope;
type IsAfterPredicate = ((scope: CodeScope) => boolean) | boolean;
export declare function resolveScope(scope: CodeScope, isAfter: IsAfterPredicate): string;
export declare function resolveCodeTag(wrapped: CodeTag, after: boolean, isAfter?: IsAfterPredicate): string;
export {};
//# sourceMappingURL=CodeScope.d.ts.map