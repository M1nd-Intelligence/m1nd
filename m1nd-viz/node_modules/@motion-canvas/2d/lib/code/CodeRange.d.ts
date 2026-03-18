export type CodePoint = [number, number];
export type CodeRange = [CodePoint, CodePoint];
export declare function isCodeRange(value: unknown): value is CodeRange;
/**
 * Create a code range that spans the given lines.
 *
 * @param from - The line from which the range starts.
 * @param to - The line at which the range ends. If omitted, the range will
 *             cover only one line.
 */
export declare function lines(from: number, to?: number): CodeRange;
/**
 * Create a code range that highlights the given word.
 *
 * @param line - The line at which the word appears.
 * @param from - The column at which the word starts.
 * @param length - The length of the word. If omitted, the range will cover the
 *                 rest of the line.
 */
export declare function word(line: number, from: number, length?: number): CodeRange;
/**
 * Create a custom selection range.
 *
 * @param startLine - The line at which the selection starts.
 * @param startColumn - The column at which the selection starts.
 * @param endLine - The line at which the selection ends.
 * @param endColumn - The column at which the selection ends.
 */
export declare function pointToPoint(startLine: number, startColumn: number, endLine: number, endColumn: number): CodeRange;
export declare function isPointInCodeRange(point: CodePoint, range: CodeRange): boolean;
export declare function consolidateCodeRanges(ranges: CodeRange[]): CodeRange[];
export declare function inverseCodeRange(ranges: CodeRange[]): CodeRange[];
/**
 * Find all code ranges that match the given pattern.
 *
 * @param code - The code to search in.
 * @param pattern - Either a string or a regular expression to search for.
 * @param limit - An optional limit on the number of ranges to find.
 */
export declare function findAllCodeRanges(code: string, pattern: string | RegExp, limit?: number): CodeRange[];
//# sourceMappingURL=CodeRange.d.ts.map