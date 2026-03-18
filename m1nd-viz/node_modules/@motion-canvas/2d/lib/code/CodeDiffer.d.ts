import { CodeScope, CodeTag } from './CodeScope';
import { CodeTokenizer } from './CodeTokenizer';
/**
 * A function that compares two code snippets and returns a list of
 * {@link CodeTag}s describing a transition between them.
 */
export type CodeDiffer = (
/**
 * The original code scope.
 */
from: CodeScope, 
/**
 * The new code scope.
 */
to: CodeScope, 
/**
 * The inherited tokenizer to use.
 */
tokenize: CodeTokenizer) => CodeTag[];
/**
 * Default diffing function utilizing {@link code.patienceDiff}.
 *
 * @param from - The original code scope.
 * @param to - The new code scope.
 * @param tokenize - The inherited tokenizer to use.
 */
export declare function defaultDiffer(from: CodeScope, to: CodeScope, tokenize: CodeTokenizer): CodeTag[];
//# sourceMappingURL=CodeDiffer.d.ts.map