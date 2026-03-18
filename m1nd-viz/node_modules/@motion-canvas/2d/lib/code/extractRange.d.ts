import { CodeRange } from './CodeRange';
import { CodeTag } from './CodeScope';
/**
 * Transform the fragments to isolate the given range into its own fragment.
 *
 * @remarks
 * This function will try to preserve the original fragments, resolving them
 * only if they overlap with the range.
 *
 * @param range - The range to extract.
 * @param fragments - The fragments to transform.
 *
 * @returns A tuple containing the transformed fragments and the index of the
 *          isolated fragment within.
 */
export declare function extractRange(range: CodeRange, fragments: CodeTag[]): [CodeTag[], number];
//# sourceMappingURL=extractRange.d.ts.map