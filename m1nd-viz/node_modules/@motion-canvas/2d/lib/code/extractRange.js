import { resolveCodeTag } from './CodeScope';
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
export function extractRange(range, fragments) {
    const [from, to] = range;
    let [fromRow, fromColumn] = from;
    let [toRow, toColumn] = to;
    if (fromRow > toRow || (fromRow === toRow && fromColumn > toColumn)) {
        [fromRow, fromColumn] = to;
        [toRow, toColumn] = from;
    }
    let currentRow = 0;
    let currentColumn = 0;
    const newFragments = [];
    let index = -1;
    let found = false;
    let extracted = '';
    for (const fragment of fragments) {
        if (found) {
            newFragments.push(fragment);
            continue;
        }
        const resolved = resolveCodeTag(fragment, false);
        const lines = resolved.split('\n');
        const newRows = lines.length - 1;
        const lastColumn = lines[newRows].length;
        const nextColumn = newRows > 0 ? lastColumn : currentColumn + lastColumn;
        if (fromRow > currentRow + newRows ||
            (fromRow === currentRow + newRows && fromColumn > nextColumn)) {
            currentRow += newRows;
            currentColumn = nextColumn;
            newFragments.push(fragment);
            continue;
        }
        for (let i = 0; i < resolved.length; i++) {
            const char = resolved.charAt(i);
            if (fromRow === currentRow && fromColumn >= currentColumn) {
                if (fromColumn === currentColumn) {
                    index = newFragments.length + 1;
                    newFragments.push(resolved.slice(0, i), '');
                }
                else if (char === '\n') {
                    index = newFragments.length + 1;
                    newFragments.push(resolved.slice(0, i) + ' '.repeat(fromColumn - currentColumn), '');
                }
            }
            if (index !== -1 && toRow === currentRow && toColumn >= currentColumn) {
                if (toColumn === currentColumn) {
                    newFragments.push(resolved.slice(i));
                    found = true;
                    break;
                }
                if (char === '\n') {
                    if (currentColumn < toColumn) {
                        extracted += '\n';
                        if (i + 1 < resolved.length) {
                            newFragments.push(resolved.slice(i + 1));
                        }
                    }
                    else {
                        newFragments.push(resolved.slice(i));
                    }
                    found = true;
                    break;
                }
            }
            if (index !== -1) {
                extracted += char;
            }
            if (char === '\n') {
                currentRow++;
                currentColumn = 0;
            }
            else {
                currentColumn++;
            }
        }
        if (index === -1) {
            newFragments.push(fragment);
        }
    }
    if (index === -1) {
        index = newFragments.length + 1;
        const missingRows = fromRow - currentRow;
        const missingColumns = missingRows > 0 ? fromColumn : fromColumn - currentColumn;
        newFragments.push('\n'.repeat(missingRows) + ' '.repeat(missingColumns), '');
    }
    newFragments[index] = extracted;
    return [newFragments, index];
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZXh0cmFjdFJhbmdlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb2RlL2V4dHJhY3RSYW5nZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFDQSxPQUFPLEVBQVUsY0FBYyxFQUFDLE1BQU0sYUFBYSxDQUFDO0FBRXBEOzs7Ozs7Ozs7Ozs7R0FZRztBQUNILE1BQU0sVUFBVSxZQUFZLENBQzFCLEtBQWdCLEVBQ2hCLFNBQW9CO0lBRXBCLE1BQU0sQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUFDLEdBQUcsS0FBSyxDQUFDO0lBQ3pCLElBQUksQ0FBQyxPQUFPLEVBQUUsVUFBVSxDQUFDLEdBQUcsSUFBSSxDQUFDO0lBQ2pDLElBQUksQ0FBQyxLQUFLLEVBQUUsUUFBUSxDQUFDLEdBQUcsRUFBRSxDQUFDO0lBQzNCLElBQUksT0FBTyxHQUFHLEtBQUssSUFBSSxDQUFDLE9BQU8sS0FBSyxLQUFLLElBQUksVUFBVSxHQUFHLFFBQVEsQ0FBQyxFQUFFO1FBQ25FLENBQUMsT0FBTyxFQUFFLFVBQVUsQ0FBQyxHQUFHLEVBQUUsQ0FBQztRQUMzQixDQUFDLEtBQUssRUFBRSxRQUFRLENBQUMsR0FBRyxJQUFJLENBQUM7S0FDMUI7SUFFRCxJQUFJLFVBQVUsR0FBRyxDQUFDLENBQUM7SUFDbkIsSUFBSSxhQUFhLEdBQUcsQ0FBQyxDQUFDO0lBQ3RCLE1BQU0sWUFBWSxHQUFjLEVBQUUsQ0FBQztJQUNuQyxJQUFJLEtBQUssR0FBRyxDQUFDLENBQUMsQ0FBQztJQUNmLElBQUksS0FBSyxHQUFHLEtBQUssQ0FBQztJQUNsQixJQUFJLFNBQVMsR0FBRyxFQUFFLENBQUM7SUFFbkIsS0FBSyxNQUFNLFFBQVEsSUFBSSxTQUFTLEVBQUU7UUFDaEMsSUFBSSxLQUFLLEVBQUU7WUFDVCxZQUFZLENBQUMsSUFBSSxDQUFDLFFBQVEsQ0FBQyxDQUFDO1lBQzVCLFNBQVM7U0FDVjtRQUVELE1BQU0sUUFBUSxHQUFHLGNBQWMsQ0FBQyxRQUFR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