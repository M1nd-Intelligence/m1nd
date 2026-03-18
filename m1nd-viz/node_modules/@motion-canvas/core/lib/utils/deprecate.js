import { useLogger } from './useScene';
/**
 * Mark the given function as deprecated.
 *
 * @param fn - The function to deprecate.
 * @param message - The log message.
 * @param remarks - The optional log remarks.
 */
export function deprecate(fn, message, remarks) {
    return function (...args) {
        useLogger().warn({ message, remarks, stack: new Error().stack });
        return fn.apply(this, args);
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZGVwcmVjYXRlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3V0aWxzL2RlcHJlY2F0ZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUMsU0FBUyxFQUFDLE1BQU0sWUFBWSxDQUFDO0FBRXJDOzs7Ozs7R0FNRztBQUNILE1BQU0sVUFBVSxTQUFTLENBQ3ZCLEVBQStCLEVBQy9CLE9BQWUsRUFDZixPQUFnQjtJQUVoQixPQUFPLFVBQXFCLEdBQUcsSUFBSTtRQUNqQyxTQUFTLEVBQUUsQ0FBQyxJQUFJLENBQUMsRUFBQyxPQUFPLEVBQUUsT0FBTyxFQUFFLEtBQUssRUFBRSxJQUFJLEtBQUssRUFBRSxDQUFDLEtBQUssRUFBQyxDQUFDLENBQUM7UUFDL0QsT0FBTyxFQUFFLENBQUMsS0FBSyxDQUFDLElBQUksRUFBRSxJQUFJLENBQUMsQ0FBQztJQUM5QixDQUFDLENBQUM7QUFDSixDQUFDIn0=