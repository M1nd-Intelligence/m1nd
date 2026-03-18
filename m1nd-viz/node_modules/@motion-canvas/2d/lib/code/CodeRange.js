function isCodePoint(value) {
    return (Array.isArray(value) &&
        value.length === 2 &&
        typeof value[0] === 'number' &&
        typeof value[1] === 'number');
}
export function isCodeRange(value) {
    return (Array.isArray(value) &&
        value.length === 2 &&
        isCodePoint(value[0]) &&
        isCodePoint(value[1]));
}
/**
 * Create a code range that spans the given lines.
 *
 * @param from - The line from which the range starts.
 * @param to - The line at which the range ends. If omitted, the range will
 *             cover only one line.
 */
export function lines(from, to) {
    return [
        [from, 0],
        [to ?? from, Infinity],
    ];
}
/**
 * Create a code range that highlights the given word.
 *
 * @param line - The line at which the word appears.
 * @param from - The column at which the word starts.
 * @param length - The length of the word. If omitted, the range will cover the
 *                 rest of the line.
 */
export function word(line, from, length) {
    return [
        [line, from],
        [line, from + (length ?? Infinity)],
    ];
}
/**
 * Create a custom selection range.
 *
 * @param startLine - The line at which the selection starts.
 * @param startColumn - The column at which the selection starts.
 * @param endLine - The line at which the selection ends.
 * @param endColumn - The column at which the selection ends.
 */
export function pointToPoint(startLine, startColumn, endLine, endColumn) {
    return [
        [startLine, startColumn],
        [endLine, endColumn],
    ];
}
export function isPointInCodeRange(point, range) {
    const [y, x] = point;
    const [[startLine, startColumn], [endLine, endColumn]] = range;
    return (((y === startLine && x >= startColumn) || y > startLine) &&
        ((y === endLine && x < endColumn) || y < endLine));
}
export function consolidateCodeRanges(ranges) {
    // Sort by start position
    ranges.sort((a, b) => {
        const lines = b[0][0] - a[0][0];
        // Break ties on start column
        if (lines === 0) {
            return b[0][1] - a[0][1];
        }
        return lines;
    });
    const staged = [...ranges];
    const results = [];
    while (staged.length > 0) {
        let current = staged.pop();
        if (!current) {
            continue;
        }
        const [[initStartLine, initStartColumn], [initEndLine, initEndColumn]] = current;
        for (const targetRange of staged) {
            const [[targetStartLine, targetStartColumn], [targetEndLine, targetEndColumn],] = targetRange;
            if (isPointInCodeRange(targetRange[0], current) ||
                isPointInCodeRange(targetRange[1], current)) {
                staged.pop();
                let startColumn;
                if (initStartLine === targetStartLine) {
                    startColumn = Math.min(initStartColumn, targetStartColumn);
                }
                else if (initStartLine < targetStartLine) {
                    startColumn = initStartColumn;
                }
                else {
                    startColumn = targetStartColumn;
                }
                let endColumn;
                if (initEndLine === targetEndLine) {
                    endColumn = Math.max(initEndColumn, targetEndColumn);
                }
                else if (initEndLine > targetEndLine) {
                    endColumn = initEndColumn;
                }
                else {
                    endColumn = targetEndColumn;
                }
                // Update the current to the consolidated one and get rid of the
                // remaining instance of the unconsolidated target
                current = [
                    [Math.min(initStartLine, targetStartLine), startColumn],
                    [Math.max(initEndLine, targetEndLine), endColumn],
                ];
            }
        }
        results.push(current);
    }
    return results;
}
export function inverseCodeRange(ranges) {
    if (ranges.length === 0) {
        return [
            [
                [0, 0],
                [Infinity, Infinity],
            ],
        ];
    }
    const firstRange = ranges[0];
    const result = [];
    for (let first = 0; first < ranges.length - 1; first++) {
        const range1 = ranges[first];
        const range2 = ranges[first + 1];
        result.push([range1[1], range2[0]]);
    }
    const lastRange = ranges.slice(-1)[0];
    return [
        [[0, 0], firstRange[0]],
        ...result,
        [lastRange[1], [Infinity, Infinity]],
    ];
}
/**
 * Find all code ranges that match the given pattern.
 *
 * @param code - The code to search in.
 * @param pattern - Either a string or a regular expression to search for.
 * @param limit - An optional limit on the number of ranges to find.
 */
export function findAllCodeRanges(code, pattern, limit = Infinity) {
    if (typeof pattern === 'string') {
        pattern = new RegExp(pattern, 'g');
    }
    const matches = code.matchAll(pattern);
    const ranges = [];
    let index = 0;
    let line = 0;
    let column = 0;
    for (const match of matches) {
        if (match.index === undefined || ranges.length >= limit) {
            continue;
        }
        let from = [line, column];
        while (index <= code.length) {
            if (index === match.index) {
                from = [line, column];
            }
            if (index === match.index + match[0].length) {
                ranges.push([from, [line, column]]);
                break;
            }
            if (code[index] === '\n') {
                line++;
                column = 0;
            }
            else {
                column++;
            }
            index++;
        }
    }
    return ranges;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZVJhbmdlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb2RlL0NvZGVSYW5nZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFFQSxTQUFTLFdBQVcsQ0FBQyxLQUFjO0lBQ2pDLE9BQU8sQ0FDTCxLQUFLLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQztRQUNwQixLQUFLLENBQUMsTUFBTSxLQUFLLENBQUM7UUFDbEIsT0FBTyxLQUFLLENBQUMsQ0FBQyxDQUFDLEtBQUssUUFBUTtRQUM1QixPQUFPLEtBQUssQ0FBQyxDQUFDLENBQUMsS0FBSyxRQUFRLENBQzdCLENBQUM7QUFDSixDQUFDO0FBSUQsTUFBTSxVQUFVLFdBQVcsQ0FBQyxLQUFjO0lBQ3hDLE9BQU8sQ0FDTCxLQUFLLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQztRQUNwQixLQUFLLENBQUMsTUFBTSxLQUFLLENBQUM7UUFDbEIsV0FBVyxDQUFDLEtBQUssQ0FBQyxDQUFDLENBQUMsQ0FBQztRQUNyQixXQUFXLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQ3RCLENBQUM7QUFDSixDQUFDO0FBRUQ7Ozs7OztHQU1HO0FBQ0gsTUFBTSxVQUFVLEtBQUssQ0FBQyxJQUFZLEVBQUUsRUFBVztJQUM3QyxPQUFPO1FBQ0wsQ0FBQyxJQUFJLEVBQUUsQ0FBQyxDQUFDO1FBQ1QsQ0FBQyxFQUFFLElBQUksSUFBSSxFQUFFLFFBQVEsQ0FBQztLQUN2QixDQUFDO0FBQ0osQ0FBQztBQUVEOzs7Ozs7O0dBT0c7QUFDSCxNQUFNLFVBQVUsSUFBSSxDQUFDLElBQVksRUFBRSxJQUFZLEVBQUUsTUFBZTtJQUM5RCxPQUFPO1FBQ0wsQ0FBQyxJQUFJLEVBQUUsSUFBSSxDQUFDO1FBQ1osQ0FBQyxJQUFJLEVBQUUsSUFBSSxHQUFHLENBQUMsTUFBTSxJQUFJLFFBQVEsQ0FBQyxDQUFDO0tBQ3BDLENBQUM7QUFDSixDQUFDO0FBRUQ7Ozs7Ozs7R0FPRztBQUNI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