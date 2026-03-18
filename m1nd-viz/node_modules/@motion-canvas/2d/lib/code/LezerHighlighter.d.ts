import { HighlightStyle } from '@codemirror/language';
import { Parser, Tree } from '@lezer/common';
import { CodeHighlighter, HighlightResult } from './CodeHighlighter';
interface LezerCache {
    tree: Tree;
    code: string;
    colorLookup: Map<string, string>;
}
export declare class LezerHighlighter implements CodeHighlighter<LezerCache | null> {
    private readonly parser;
    private readonly style;
    private static classRegex;
    private readonly classLookup;
    constructor(parser: Parser, style?: HighlightStyle);
    initialize(): boolean;
    prepare(code: string): LezerCache | null;
    highlight(index: number, cache: LezerCache | null): HighlightResult;
    tokenize(code: string): string[];
    private getNodeId;
}
export {};
//# sourceMappingURL=LezerHighlighter.d.ts.map