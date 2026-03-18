import { DetailedError } from './DetailedError';
const ThreadStack = [];
/**
 * Get a reference to the current thread.
 */
export function useThread() {
    const thread = ThreadStack.at(-1);
    if (!thread) {
        throw new DetailedError('The thread is not available in the current context.', "<p><code>useThread()</code> can only be called from within generator functions.\n      It&#39;s not available during rendering.</p>\n");
    }
    return thread;
}
export function startThread(thread) {
    ThreadStack.push(thread);
}
export function endThread(thread) {
    if (ThreadStack.pop() !== thread) {
        throw new Error('startThread/endThread was called out of order.');
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidXNlVGhyZWFkLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3V0aWxzL3VzZVRocmVhZC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFDQSxPQUFPLEVBQUMsYUFBYSxFQUFDLE1BQU0saUJBQWlCLENBQUM7QUFFOUMsTUFBTSxXQUFXLEdBQWEsRUFBRSxDQUFDO0FBRWpDOztHQUVHO0FBQ0gsTUFBTSxVQUFVLFNBQVM7SUFDdkIsTUFBTSxNQUFNLEdBQUcsV0FBVyxDQUFDLEVBQUUsQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFDO0lBQ2xDLElBQUksQ0FBQyxNQUFNLEVBQUU7UUFDWCxNQUFNLElBQUksYUFBYSxDQUNyQixxREFBcUQsMElBSXRELENBQUM7S0FDSDtJQUNELE9BQU8sTUFBTSxDQUFDO0FBQ2hCLENBQUM7QUFFRCxNQUFNLFVBQVUsV0FBVyxDQUFDLE1BQWM7SUFDeEMsV0FBVyxDQUFDLElBQUksQ0FBQyxNQUFNLENBQUMsQ0FBQztBQUMzQixDQUFDO0FBRUQsTUFBTSxVQUFVLFNBQVMsQ0FBQyxNQUFjO0lBQ3RDLElBQUksV0FBVyxDQUFDLEdBQUcsRUFBRSxLQUFLLE1BQU0sRUFBRTtRQUNoQyxNQUFNLElBQUksS0FBSyxDQUFDLGdEQUFnRCxDQUFDLENBQUM7S0FDbkU7QUFDSCxDQUFDIn0=