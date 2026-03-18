import { unwrap } from '@motion-canvas/core';
import { isCodeMetrics } from './CodeMetrics';
export function CODE(strings, ...tags) {
    const result = [];
    for (let i = 0; i < strings.length; i++) {
        result.push(strings[i]);
        const tag = tags[i];
        if (tag !== undefined) {
            if (Array.isArray(tag)) {
                result.push(...tag);
            }
            else {
                result.push(tag);
            }
        }
    }
    return result;
}
export function isCodeScope(value) {
    return value?.fragments !== undefined;
}
export function parseCodeScope(value) {
    if (typeof value === 'string') {
        return {
            progress: 0,
            fragments: [value],
        };
    }
    if (Array.isArray(value)) {
        return {
            progress: 0,
            fragments: value,
        };
    }
    return value;
}
export function resolveScope(scope, isAfter) {
    let code = '';
    const after = typeof isAfter === 'boolean' ? isAfter : isAfter(scope);
    for (const wrapped of scope.fragments) {
        code += resolveCodeTag(wrapped, after, isAfter);
    }
    return code;
}
export function resolveCodeTag(wrapped, after, isAfter = after) {
    const fragment = unwrap(wrapped);
    if (typeof fragment === 'string') {
        return fragment;
    }
    else if (isCodeScope(fragment)) {
        return resolveScope(fragment, isAfter);
    }
    else if (isCodeMetrics(fragment)) {
        return fragment.content;
    }
    else if (Array.isArray(fragment)) {
        return resolveScope({
            progress: 0,
            fragments: fragment,
        }, isAfter);
    }
    else {
        return after
            ? typeof fragment.after === 'string'
                ? fragment.after
                : fragment.after.content
            : typeof fragment.before === 'string'
                ? fragment.before
                : fragment.before.content;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZVNjb3BlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb2RlL0NvZGVTY29wZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQWMsTUFBTSxFQUFDLE1BQU0scUJBQXFCLENBQUM7QUFFeEQsT0FBTyxFQUFDLGFBQWEsRUFBQyxNQUFNLGVBQWUsQ0FBQztBQVc1QyxNQUFNLFVBQVUsSUFBSSxDQUNsQixPQUE2QixFQUM3QixHQUFHLElBQWU7SUFFbEIsTUFBTSxNQUFNLEdBQWMsRUFBRSxDQUFDO0lBQzdCLEtBQUssSUFBSSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRyxPQUFPLENBQUMsTUFBTSxFQUFFLENBQUMsRUFBRSxFQUFFO1FBQ3ZDLE1BQU0sQ0FBQyxJQUFJLENBQUMsT0FBTyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUM7UUFDeEIsTUFBTSxHQUFHLEdBQUcsSUFBSSxDQUFDLENBQUMsQ0FBQyxDQUFDO1FBQ3BCLElBQUksR0FBRyxLQUFLLFNBQVMsRUFBRTtZQUNyQixJQUFJLEtBQUssQ0FBQyxPQUFPLENBQUMsR0FBRyxDQUFDLEVBQUU7Z0JBQ3RCLE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxHQUFHLENBQUMsQ0FBQzthQUNyQjtpQkFBTTtnQkFDTCxNQUFNLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDO2FBQ2xCO1NBQ0Y7S0FDRjtJQUVELE9BQU8sTUFBTSxDQUFDO0FBQ2hCLENBQUM7QUFFRCxNQUFNLFVBQVUsV0FBVyxDQUFDLEtBQVU7SUFDcEMsT0FBTyxLQUFLLEVBQUUsU0FBUyxLQUFLLFNBQVMsQ0FBQztBQUN4QyxDQUFDO0FBRUQsTUFBTSxVQUFVLGNBQWMsQ0FBQyxLQUF3QjtJQUNyRCxJQUFJLE9BQU8sS0FBSyxLQUFLLFFBQVEsRUFBRTtRQUM3QixPQUFPO1lBQ0wsUUFBUSxFQUFFLENBQUM7WUFDWCxTQUFTLEVBQUUsQ0FBQyxLQUFLLENBQUM7U0FDbkIsQ0FBQztLQUNIO0lBRUQsSUFBSSxLQUFLLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQyxFQUFFO1FBQ3hCLE9BQU87WUFDTCxRQUFRLEVBQUUsQ0FBQztZQUNYLFNBQVMsRUFBRSxLQUFLO1NBQ2pCLENBQUM7S0FDSDtJQUVELE9BQU8sS0FBSyxDQUFDO0FBQ2YsQ0FBQztBQUlELE1BQU0sVUFBVSxZQUFZLENBQzFCLEtBQWdCLEVBQ2hCLE9BQXlCO0lBRXpCLElBQUksSUFBSSxHQUFHLEVBQUUsQ0FBQztJQUNkLE1BQU0sS0FBSyxHQUFHLE9BQU8sT0FBTyxLQUFLLFNBQVMsQ0FBQyxDQUFDLENBQUMsT0FBTyxDQUFDLENBQUMsQ0FBQyxPQUFPLENBQUMsS0FBSyxDQUFDLENBQUM7SUFDdEUsS0FBSyxNQUFNLE9BQU8sSUFBSSxLQUFLLENBQUMsU0FBUyxFQUFFO1FBQ3JDLElBQUksSUFBSSxjQUFjLENBQUMsT0FBTyxFQUFFLEtBQUssRUFBRSxPQUFPLENBQUMsQ0FBQztLQUNqRDtJQUVELE9BQU8sSUFBSSxDQUFDO0FBQ2QsQ0FBQztBQUVELE1BQU0sVUFBVSxjQUFjLENBQzVCLE9BQWdCLEVBQ2hCLEtBQWMsRUFDZCxVQUE0QixLQUFLO0lBRWpDLE1BQU0sUUFBUSxHQUFHLE1BQU0sQ0FBQyxPQUFPLENBQUMsQ0FBQztJQUNqQyxJQUFJLE9BQU8sUUFBUSxLQUFLLFFBQVEsRUFBRTtRQUNoQyxPQUFPLFFBQVEsQ0FBQztLQUNqQjtTQUFNLElBQUksV0FBVyxDQUFDLFFBQVEsQ0FBQyxFQUFFO1FBQ2hDLE9BQU8sWUFBWSxDQUFDLFFBQVEsRUFBRSxPQUFPLENBQUMsQ0FBQztLQUN4QztTQUFNLElBQUksYUFBYSxDQUFDLFFBQVEsQ0FBQyxFQUFFO1FBQ2xDLE9BQU8sUUFBUSxDQUFDLE9BQU8sQ0FBQztLQUN6QjtTQUFNLElBQUksS0FBSyxDQUFDLE9BQU8sQ0FBQyxRQUFRLENBQUMsRUFBRTtRQUNsQyxPQUFPLFlBQVksQ0FDakI7WUFDRSxRQUFRLEVBQUUsQ0FBQztZQUNYLFNBQVMsRUFBRSxRQUFRO1NBQ3BCLEVBQ0QsT0FBTyxDQUNSLENBQUM7S0FDSDtTQUFNO1FBQ0wsT0FBTyxLQUFLO1lBQ1YsQ0FBQyxDQUFDLE9BQU8sUUFBUSxDQUFDLEtBQUssS0FBSyxRQUFRO2dCQUNsQyxDQUFDLENBQUMsUUFBUSxDQUFDLEtBQUs7Z0JBQ2hCLENBQUMsQ0FBQyxRQUFRLENBQUMsS0FBSyxDQUFDLE9BQU87WUFDMUIsQ0FBQyxDQUFDLE9BQU8sUUFBUSxDQUFDLE1BQU0sS0FBSyxRQUFRO2dCQUNuQyxDQUFDLENBQUMsUUFBUSxDQUFDLE1BQU07Z0JBQ2pCLENBQUMsQ0FBQyxRQUFRLENBQUMsTUFBTSxDQUFDLE9BQU8sQ0FBQztLQUMvQjtBQUNILENBQUMifQ==