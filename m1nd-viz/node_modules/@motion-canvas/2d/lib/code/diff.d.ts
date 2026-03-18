/**
 * Performs a patience diff on two arrays of strings, returning an object
 * containing the lines that were deleted, inserted, and potentially moved
 * lines. The plus parameter can result in a significant performance hit due
 * to additional Longest Common Substring searches.
 *
 * @param aLines - The original array of strings
 * @param bLines - The new array of strings
 * @param plus - Whether to return the moved lines
 *
 * Adapted from Jonathan "jonTrent" Trent's patience-diff algorithm.
 * Types and tests added by Hunter "hhenrichsen" Henrichsen.
 *
 * {@link https://github.com/jonTrent/PatienceDiff}
 */
export declare function patienceDiff(aLines: string[], bLines: string[]): {
    lines: {
        line: string;
        aIndex: number;
        bIndex: number;
    }[];
    lineCountDeleted: number;
    lineCountInserted: number;
};
/**
 * Utility function for debugging patienceDiff.
 *
 * @internal
 */
export declare function printDiff(diff: ReturnType<typeof patienceDiff>): void;
//# sourceMappingURL=diff.d.ts.map