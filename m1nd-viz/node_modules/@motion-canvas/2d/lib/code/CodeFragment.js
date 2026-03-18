import { isCodeMetrics, measureString } from './CodeMetrics';
export function metricsToFragment(value) {
    return {
        before: value,
        after: value,
    };
}
export function parseCodeFragment(value, context, monoWidth) {
    let fragment;
    if (typeof value === 'string') {
        fragment = metricsToFragment(measureString(context, monoWidth, value));
    }
    else if (isCodeMetrics(value)) {
        fragment = metricsToFragment(value);
    }
    else {
        fragment = {
            before: typeof value.before === 'string'
                ? measureString(context, monoWidth, value.before)
                : value.before,
            after: typeof value.after === 'string'
                ? measureString(context, monoWidth, value.after)
                : value.after,
        };
    }
    return fragment;
}
/**
 * Create a code fragment that represents an insertion of code.
 *
 * @remarks
 * Can be used in conjunction with {@link code.CodeSignalHelpers.edit}.
 *
 * @param code - The code to insert.
 */
export function insert(code) {
    return {
        before: '',
        after: code,
    };
}
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
export function replace(before, after) {
    return {
        before,
        after,
    };
}
/**
 * Create a code fragment that represents a removal of code.
 *
 * @remarks
 * Can be used in conjunction with {@link code.CodeSignalHelpers.edit}.
 *
 * @param code - The code to remove.
 */
export function remove(code) {
    return {
        before: code,
        after: '',
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZUZyYWdtZW50LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb2RlL0NvZGVGcmFnbWVudC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQWMsYUFBYSxFQUFFLGFBQWEsRUFBQyxNQUFNLGVBQWUsQ0FBQztBQWlCeEUsTUFBTSxVQUFVLGlCQUFpQixDQUFDLEtBQWtCO0lBQ2xELE9BQU87UUFDTCxNQUFNLEVBQUUsS0FBSztRQUNiLEtBQUssRUFBRSxLQUFLO0tBQ2IsQ0FBQztBQUNKLENBQUM7QUFFRCxNQUFNLFVBQVUsaUJBQWlCLENBQy9CLEtBQTJCLEVBQzNCLE9BQWlDLEVBQ2pDLFNBQWlCO0lBRWpCLElBQUksUUFBc0IsQ0FBQztJQUMzQixJQUFJLE9BQU8sS0FBSyxLQUFLLFFBQVEsRUFBRTtRQUM3QixRQUFRLEdBQUcsaUJBQWlCLENBQUMsYUFBYSxDQUFDLE9BQU8sRUFBRSxTQUFTLEVBQUUsS0FBSyxDQUFDLENBQUMsQ0FBQztLQUN4RTtTQUFNLElBQUksYUFBYSxDQUFDLEtBQUssQ0FBQyxFQUFFO1FBQy9CLFFBQVEsR0FBRyxpQkFBaUIsQ0FBQyxLQUFLLENBQUMsQ0FBQztLQUNyQztTQUFNO1FBQ0wsUUFBUSxHQUFHO1lBQ1QsTUFBTSxFQUNKLE9BQU8sS0FBSyxDQUFDLE1BQU0sS0FBSyxRQUFRO2dCQUM5QixDQUFDLENBQUMsYUFBYSxDQUFDLE9BQU8sRUFBRSxTQUFTLEVBQUUsS0FBSyxDQUFDLE1BQU0sQ0FBQztnQkFDakQsQ0FBQyxDQUFDLEtBQUssQ0FBQyxNQUFNO1lBQ2xCLEtBQUssRUFDSCxPQUFPLEtBQUssQ0FBQyxLQUFLLEtBQUssUUFBUTtnQkFDN0IsQ0FBQyxDQUFDLGFBQWEsQ0FBQyxPQUFPLEVBQUUsU0FBUyxFQUFFLEtBQUssQ0FBQyxLQUFLLENBQUM7Z0JBQ2hELENBQUMsQ0FBQyxLQUFLLENBQUMsS0FBSztTQUNsQixDQUFDO0tBQ0g7SUFFRCxPQUFPLFFBQVEsQ0FBQztBQUNsQixDQUFDO0FBRUQ7Ozs7Ozs7R0FPRztBQUNILE1BQU0sVUFBVSxNQUFNLENBQUMsSUFBWTtJQUNqQyxPQUFPO1FBQ0wsTUFBTSxFQUFFLEVBQUU7UUFDVixLQUFLLEVBQUUsSUFBSTtLQUNaLENBQUM7QUFDSixDQUFDO0FBRUQ7Ozs7Ozs7OztHQVNHO0FBQ0gsTUFBTSxVQUFVLE9BQU8sQ0FBQyxNQUFjLEVBQUUsS0FBYTtJQUNuRCxPQUFPO1FBQ0wsTUFBTTtRQUNOLEtBQUs7S0FDTixDQUFDO0FBQ0osQ0FBQztBQUVEOzs7Ozs7O0dBT0c7QUFDSCxNQUFNLFVBQVUsTUFBTSxDQUFDLElBQVk7SUFDakMsT0FBTztRQUNMLE1BQU0sRUFBRSxJQUFJO1FBQ1osS0FBSyxFQUFFLEVBQUU7S0FDVixDQUFDO0FBQ0osQ0FBQyJ9