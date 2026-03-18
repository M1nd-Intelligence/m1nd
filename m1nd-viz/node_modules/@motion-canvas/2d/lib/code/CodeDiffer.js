import { resolveScope } from './CodeScope';
import { patienceDiff } from './diff';
/**
 * Default diffing function utilizing {@link code.patienceDiff}.
 *
 * @param from - The original code scope.
 * @param to - The new code scope.
 * @param tokenize - The inherited tokenizer to use.
 */
export function defaultDiffer(from, to, tokenize) {
    const fromString = resolveScope(from, false);
    const toString = resolveScope(to, true);
    const diff = patienceDiff(tokenize(fromString), tokenize(toString));
    const fragments = [];
    let before = '';
    let after = '';
    let lastAdded = false;
    const flush = () => {
        if (before !== '' || after !== '') {
            fragments.push({
                before,
                after,
            });
            before = '';
            after = '';
        }
    };
    for (const line of diff.lines) {
        if (line.aIndex === -1) {
            if (after !== '' && !lastAdded) {
                flush();
            }
            lastAdded = true;
            after += line.line;
        }
        else if (line.bIndex === -1) {
            if (before !== '' && lastAdded) {
                flush();
            }
            lastAdded = false;
            before += line.line;
        }
        else {
            flush();
            fragments.push(line.line);
        }
    }
    flush();
    return fragments;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZURpZmZlci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29kZS9Db2RlRGlmZmVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBcUIsWUFBWSxFQUFDLE1BQU0sYUFBYSxDQUFDO0FBRTdELE9BQU8sRUFBQyxZQUFZLEVBQUMsTUFBTSxRQUFRLENBQUM7QUFxQnBDOzs7Ozs7R0FNRztBQUNILE1BQU0sVUFBVSxhQUFhLENBQzNCLElBQWUsRUFDZixFQUFhLEVBQ2IsUUFBdUI7SUFFdkIsTUFBTSxVQUFVLEdBQUcsWUFBWSxDQUFDLElBQUksRUFBRSxLQUFLLENBQUMsQ0FBQztJQUM3QyxNQUFNLFFBQVEsR0FBRyxZQUFZLENBQUMsRUFBRSxFQUFFLElBQUksQ0FBQyxDQUFDO0lBRXhDLE1BQU0sSUFBSSxHQUFHLFlBQVksQ0FBQyxRQUFRLENBQUMsVUFBVSxDQUFDLEVBQUUsUUFBUSxDQUFDLFFBQVEsQ0FBQyxDQUFDLENBQUM7SUFFcEUsTUFBTSxTQUFTLEdBQWMsRUFBRSxDQUFDO0lBQ2hDLElBQUksTUFBTSxHQUFHLEVBQUUsQ0FBQztJQUNoQixJQUFJLEtBQUssR0FBRyxFQUFFLENBQUM7SUFDZixJQUFJLFNBQVMsR0FBRyxLQUFLLENBQUM7SUFDdEIsTUFBTSxLQUFLLEdBQUcsR0FBRyxFQUFFO1FBQ2pCLElBQUksTUFBTSxLQUFLLEVBQUUsSUFBSSxLQUFLLEtBQUssRUFBRSxFQUFFO1lBQ2pDLFNBQVMsQ0FBQyxJQUFJLENBQUM7Z0JBQ2IsTUFBTTtnQkFDTixLQUFLO2FBQ04sQ0FBQyxDQUFDO1lBQ0gsTUFBTSxHQUFHLEVBQUUsQ0FBQztZQUNaLEtBQUssR0FBRyxFQUFFLENBQUM7U0FDWjtJQUNILENBQUMsQ0FBQztJQUVGLEtBQUssTUFBTSxJQUFJLElBQUksSUFBSSxDQUFDLEtBQUssRUFBRTtRQUM3QixJQUFJLElBQUksQ0FBQyxNQUFNLEtBQUssQ0FBQyxDQUFDLEVBQUU7WUFDdEIsSUFBSSxLQUFLLEtBQUssRUFBRSxJQUFJLENBQUMsU0FBUyxFQUFFO2dCQUM5QixLQUFLLEVBQUUsQ0FBQzthQUNUO1lBQ0QsU0FBUyxHQUFHLElBQUksQ0FBQztZQUNqQixLQUFLLElBQUksSUFBSSxDQUFDLElBQUksQ0FBQztTQUNwQjthQUFNLElBQUksSUFBSSxDQUFDLE1BQU0sS0FBSyxDQUFDLENBQUMsRUFBRTtZQUM3QixJQUFJLE1BQU0sS0FBSyxFQUFFLElBQUksU0FBUyxFQUFFO2dCQUM5QixLQUFLLEVBQUUsQ0FBQzthQUNUO1lBQ0QsU0FBUyxHQUFHLEtBQUssQ0FBQztZQUNsQixNQUFNLElBQUksSUFBSSxDQUFDLElBQUksQ0FBQztTQUNyQjthQUFNO1lBQ0wsS0FBSyxFQUFFLENBQUM7WUFDUixTQUFTLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQztTQUMzQjtLQUNGO0lBQ0QsS0FBSyxFQUFFLENBQUM7SUFFUixPQUFPLFNBQVMsQ0FBQztBQUNuQixDQUFDIn0=