import { CodePoint, CodeRange } from './CodeRange';
export type CodeSelection = CodeRange[];
export type PossibleCodeSelection = CodeRange | CodeRange[];
export declare function parseCodeSelection(value: PossibleCodeSelection): CodeSelection;
export declare function isPointInCodeSelection(point: CodePoint, selection: CodeSelection): boolean;
//# sourceMappingURL=CodeSelection.d.ts.map