import { isCodeRange, isPointInCodeRange, } from './CodeRange';
export function parseCodeSelection(value) {
    return isCodeRange(value) ? [value] : value;
}
export function isPointInCodeSelection(point, selection) {
    for (const range of selection) {
        if (isPointInCodeRange(point, range)) {
            return true;
        }
    }
    return false;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZVNlbGVjdGlvbi5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29kZS9Db2RlU2VsZWN0aW9uLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFHTCxXQUFXLEVBQ1gsa0JBQWtCLEdBQ25CLE1BQU0sYUFBYSxDQUFDO0FBS3JCLE1BQU0sVUFBVSxrQkFBa0IsQ0FDaEMsS0FBNEI7SUFFNUIsT0FBTyxXQUFXLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQyxDQUFDLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQyxDQUFDLEtBQUssQ0FBQztBQUM5QyxDQUFDO0FBRUQsTUFBTSxVQUFVLHNCQUFzQixDQUNwQyxLQUFnQixFQUNoQixTQUF3QjtJQUV4QixLQUFLLE1BQU0sS0FBSyxJQUFJLFNBQVMsRUFBRTtRQUM3QixJQUFJLGtCQUFrQixDQUFDLEtBQUssRUFBRSxLQUFLLENBQUMsRUFBRTtZQUNwQyxPQUFPLElBQUksQ0FBQztTQUNiO0tBQ0Y7SUFFRCxPQUFPLEtBQUssQ0FBQztBQUNmLENBQUMifQ==