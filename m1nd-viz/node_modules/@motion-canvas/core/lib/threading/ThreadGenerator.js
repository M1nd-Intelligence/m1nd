export function isPromisable(value) {
    return (value &&
        (typeof value === 'object' || typeof value === 'function') &&
        'toPromise' in value);
}
/**
 * Check if the given value is a {@link ThreadGenerator}.
 *
 * @param value - A possible thread {@link ThreadGenerator}.
 */
export function isThreadGenerator(value) {
    return (value !== null &&
        typeof value === 'object' &&
        Symbol.iterator in value &&
        'next' in value);
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiVGhyZWFkR2VuZXJhdG9yLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3RocmVhZGluZy9UaHJlYWRHZW5lcmF0b3IudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBTUEsTUFBTSxVQUFVLFlBQVksQ0FBQyxLQUFVO0lBQ3JDLE9BQU8sQ0FDTCxLQUFLO1FBQ0wsQ0FBQyxPQUFPLEtBQUssS0FBSyxRQUFRLElBQUksT0FBTyxLQUFLLEtBQUssVUFBVSxDQUFDO1FBQzFELFdBQVcsSUFBSSxLQUFLLENBQ3JCLENBQUM7QUFDSixDQUFDO0FBa0NEOzs7O0dBSUc7QUFDSCxNQUFNLFVBQVUsaUJBQWlCLENBQUMsS0FBYztJQUM5QyxPQUFPLENBQ0wsS0FBSyxLQUFLLElBQUk7UUFDZCxPQUFPLEtBQUssS0FBSyxRQUFRO1FBQ3pCLE1BQU0sQ0FBQyxRQUFRLElBQUksS0FBSztRQUN4QixNQUFNLElBQUksS0FBSyxDQUNoQixDQUFDO0FBQ0osQ0FBQyJ9