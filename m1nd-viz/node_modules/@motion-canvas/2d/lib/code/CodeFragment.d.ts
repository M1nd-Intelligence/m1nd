import { CodeMetrics } from './CodeMetrics';
export interface CodeFragment {
    before: CodeMetrics;
    after: CodeMetrics;
}
export interface RawCodeFragment {
    before: string;
    after: string;
}
export type PossibleCodeFragment = CodeFragment | CodeMetrics | RawCodeFragment | string;
export declare function metricsToFragment(value: CodeMetrics): CodeFragment;
export declare function parseCodeFragment(value: PossibleCodeFragment, context: CanvasRenderingContext2D, monoWidth: number): CodeFragment;
/**
 * Create a code fragment that represents an insertion of code.
 *
 * @remarks
 * Can be used in conjunction with {@link code.CodeSignalHelpers.edit}.
 *
 * @param code - The code to insert.
 */
export declare function insert(code: string): RawCodeFragment;
/**
 * Create a code fragment that represents a change from one piece of code to
 * another.
 *
 * @remarks
 * Can be used in conjunction with {@link code.CodeSignalHelpers.edit}.
 *
 * @param before - The code to change from.
 * @param after - The code to change to.
 */
export declare function replace(before: string, after: string): RawCodeFragment;
/**
 * Create a code fragment that represents a removal of code.
 *
 * @remarks
 * Can be used in conjunction with {@link code.CodeSignalHelpers.edit}.
 *
 * @param code - The code to remove.
 */
export declare function remove(code: string): RawCodeFragment;
//# sourceMappingURL=CodeFragment.d.ts.map