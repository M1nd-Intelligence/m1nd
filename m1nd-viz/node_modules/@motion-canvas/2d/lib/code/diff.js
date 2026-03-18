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
export function patienceDiff(aLines, bLines) {
    /**
     * Finds all unique values in lines[start...end], inclusive. This
     * function is used in preparation for determining the longest common
     * subsequence.
     *
     * @param lines - The array to search
     * @param start - The starting index (inclusive)
     * @param end - The ending index (inclusive)
     * @returns A map of the unique lines to their index
     */
    function findUnique(lines, start, end) {
        const lineMap = new Map();
        for (let i = start; i <= end; i++) {
            const line = lines[i];
            const data = lineMap.get(line);
            if (data) {
                data.count++;
                data.index = i;
            }
            else {
                lineMap.set(line, { count: 1, index: i });
            }
        }
        const newMap = new Map();
        for (const [key, value] of lineMap) {
            if (value.count === 1) {
                newMap.set(key, value.index);
            }
        }
        return newMap;
    }
    /**
     * Finds all the unique common entries between aArray[aStart...aEnd] and
     * bArray[bStart...bEnd], inclusive. This function uses findUnique to pare
     * down the aArray and bArray ranges first, before then walking the
     * comparison between the two arrays.
     *
     *
     * @param aArray - The original array
     * @param aStart - The start of the original array to search
     * @param aEnd - The end of the original array to search, inclusive
     * @param bArray - The new array
     * @param bStart - the start of the new array to search
     * @param bEnd - The end of the new array to search, inclusive
     * @returns a Map, with the key as the common line between aArray and
     * bArray, with the value as an object containing the array indices of the
     * matching uniqe lines.
     */
    function uniqueCommon(aArray, aStart, aEnd, bArray, bStart, bEnd) {
        const aUnique = findUnique(aArray, aStart, aEnd);
        const bUnique = findUnique(bArray, bStart, bEnd);
        return [...aUnique.entries()].reduce((paired, [key, value]) => {
            const bIndex = bUnique.get(key);
            if (bIndex !== undefined) {
                paired.set(key, {
                    aIndex: value,
                    bIndex,
                });
            }
            return paired;
        }, new Map());
    }
    /**
     * Takes a map from the unique common lines between two arrays and determines
     * the longest common subsequence.
     *
     * @see uniqueCommon
     *
     * @param abMap - The map of unique common lines between two arrays.
     * @returns An array of objects containing the indices of the longest common
     * subsequence.
     */
    function longestCommonSubsequence(abMap) {
        const jagged = [];
        abMap.forEach(value => {
            let i = 0;
            while (jagged[i] && jagged[i].at(-1).bIndex < value.bIndex) {
                i++;
            }
            if (i > 0) {
                value.prev = jagged[i - 1].at(-1);
            }
            if (!jagged[i]) {
                jagged[i] = [value];
            }
            else {
                jagged[i].push(value);
            }
        });
        // Pull out the longest common subsequence
        let lcs = [];
        if (jagged.length > 0) {
            lcs = [jagged.at(-1).at(-1)];
            let cursor = lcs.at(-1);
            while (cursor?.prev) {
                cursor = cursor.prev;
                lcs.push(cursor);
            }
        }
        return lcs.reverse();
    }
    /**
     * Keeps track of the aLines that have been deleted, are shared between aLines
     * and bLines, and bLines that have been inserted.
     */
    const result = [];
    let deleted = 0;
    let inserted = 0;
    function addToResult(aIndex, bIndex) {
        if (bIndex < 0) {
            deleted++;
        }
        else if (aIndex < 0) {
            inserted++;
        }
        result.push({
            line: 0 <= aIndex ? aLines[aIndex] : bLines[bIndex],
            aIndex,
            bIndex,
            moved: false,
        });
    }
    function addSubMatch(aStart, aEnd, bStart, bEnd) {
        // Match any lines at the beginning of aLines and bLines.
        while (aStart <= aEnd &&
            bStart <= bEnd &&
            aLines[aStart] === bLines[bStart]) {
            addToResult(aStart++, bStart++);
        }
        // Match any lines at the end of aLines and bLines, but don't place them
        // in the "result" array just yet, as the lines between these matches at
        // the beginning and the end need to be analyzed first.
        const aEndTemp = aEnd;
        while (aStart <= aEnd && bStart <= bEnd && aLines[aEnd] === bLines[bEnd]) {
            aEnd--;
            bEnd--;
        }
        // Now, check to determine with the remaining lines in the subsequence
        // whether there are any unique common lines between aLines and bLines.
        //
        // If not, add the subsequence to the result (all aLines having been
        // deleted, and all bLines having been inserted).
        //
        // If there are unique common lines between aLines and bLines, then let's
        // recursively perform the patience diff on the subsequence.
        const uniqueCommonMap = uniqueCommon(aLines, aStart, aEnd, bLines, bStart, bEnd);
        if (uniqueCommonMap.size === 0) {
            while (aStart <= aEnd) {
                addToResult(aStart++, -1);
            }
            while (bStart <= bEnd) {
                addToResult(-1, bStart++);
            }
        }
        else {
            recurseLCS(aStart, aEnd, bStart, bEnd, uniqueCommonMap);
        }
        // Finally, let's add the matches at the end to the result.
        while (aEnd < aEndTemp) {
            addToResult(++aEnd, ++bEnd);
        }
    }
    /**
     * Finds the longest common subsequence between the arrays
     * aLines[aStart...aEnd] and bLines[bStart...bEnd], inclusive. Then for each
     * subsequence, recursively performs another LCS search (via addSubMatch),
     * until there are none found, at which point the subsequence is dumped to
     * the result.
     *
     * @param aStart - The start of the original array to search
     * @param aEnd - The end of the original array to search, inclusive
     * @param bStart - The start of the new array to search
     * @param bEnd - The end of the new array to search, inclusive
     * @param uniqueCommonMap - A map of the unique common lines between
     * aLines[aStart...aEnd] and bLines[bStart...bEnd], inclusive.
     */
    function recurseLCS(aStart, aEnd, bStart, bEnd, uniqueCommonMap = uniqueCommon(aLines, aStart, aEnd, bLines, bStart, bEnd)) {
        const lcs = longestCommonSubsequence(uniqueCommonMap);
        if (lcs.length === 0) {
            addSubMatch(aStart, aEnd, bStart, bEnd);
        }
        else {
            if (aStart < lcs[0].aIndex || bStart < lcs[0].bIndex) {
                addSubMatch(aStart, lcs[0].aIndex - 1, bStart, lcs[0].bIndex - 1);
            }
            let i;
            for (i = 0; i < lcs.length - 1; i++) {
                addSubMatch(lcs[i].aIndex, lcs[i + 1].aIndex - 1, lcs[i].bIndex, lcs[i + 1].bIndex - 1);
            }
            if (lcs[i].aIndex <= aEnd || lcs[i].bIndex <= bEnd) {
                addSubMatch(lcs[i].aIndex, aEnd, lcs[i].bIndex, bEnd);
            }
        }
    }
    recurseLCS(0, aLines.length - 1, 0, bLines.length - 1);
    return {
        lines: result,
        lineCountDeleted: deleted,
        lineCountInserted: inserted,
    };
}
/**
 * Utility function for debugging patienceDiff.
 *
 * @internal
 */
export function printDiff(diff) {
    diff.lines.forEach(line => {
        if (line.bIndex < 0) {
            console.log(`- ${line.line}`);
        }
        else if (line.aIndex < 0) {
            console.log(`+ ${line.line}`);
        }
        else {
            console.log(`  ${line.line}`);
        }
    });
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZGlmZi5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29kZS9kaWZmLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQU1BOzs7Ozs7Ozs7Ozs7OztHQWNHO0FBQ0gsTUFBTSxVQUFVLFlBQVksQ0FDMUIsTUFBZ0IsRUFDaEIsTUFBZ0I7SUFVaEI7Ozs7Ozs7OztPQVNHO0lBQ0gsU0FBUyxVQUFVLENBQUMsS0FBZSxFQUFFLEtBQWEsRUFBRSxHQUFXO1FBQzdELE1BQU0sT0FBTyxHQUFHLElBQUksR0FBRyxFQUEwQyxDQUFDO1FBQ2xFLEtBQUssSUFBSSxDQUFDLEdBQUcsS0FBSyxFQUFFLENBQUMsSUFBSSxHQUFHLEVBQUUsQ0FBQyxFQUFFLEVBQUU7WUFDakMsTUFBTSxJQUFJLEdBQUcsS0FBSyxDQUFDLENBQUMsQ0FBQyxDQUFDO1lBQ3RCLE1BQU0sSUFBSSxHQUFHLE9BQU8sQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLENBQUM7WUFDL0IsSUFBSSxJQUFJLEVBQUU7Z0JBQ1IsSUFBSSxDQUFDLEtBQUssRUFBRSxDQUFDO2dCQUNiLElBQUksQ0FBQyxLQUFLLEdBQUcsQ0FBQyxDQUFDO2FBQ2hCO2lCQUFNO2dCQUNMLE9BQU8sQ0FBQyxHQUFHLENBQUMsSUFBSSxFQUFFLEVBQUMsS0FBSyxFQUFFLENBQUMsRUFBRSxLQUFLLEVBQUUsQ0FBQyxFQUFDLENBQUMsQ0FBQzthQUN6QztTQUNGO1FBRUQsTUFBTSxNQUFNLEdBQUcsSUFBSSxHQUFHLEVBQWtCLENBQUM7UUFDekMsS0FBSyxNQUFNLENBQUMsR0FBRyxFQUFFLEtBQUssQ0FBQyxJQUFJLE9BQU8sRUFBRTtZQUNsQyxJQUFJLEtBQUssQ0FBQyxLQUFLLEtBQUssQ0FBQyxFQUFFO2dCQUNyQixNQUFNLENBQUMsR0FBRyxDQUFDLEdBQUcsRUFBRSxLQUFLLENBQUMsS0FBSyxDQUFDLENBQUM7YUFDOUI7U0FDRjtRQUVELE9BQU8sTUFBTSxDQUFDO0lBQ2hCLENBQUM7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