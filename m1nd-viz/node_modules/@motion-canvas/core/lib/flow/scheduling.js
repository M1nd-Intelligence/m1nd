import { decorate, threadable } from '../decorators';
import { useDuration, usePlayback, useThread } from '../utils';
decorate(waitUntil, threadable());
/**
 * Wait until the given time event.
 *
 * @remarks
 * Time events are displayed on the timeline and can be edited to adjust the
 * delay. By default, an event happens immediately - without any delay.
 *
 * @example
 * ```ts
 * yield waitUntil('event');
 * ```
 *
 * @param event - The name of the time event.
 * @param after - An optional task to be run after the function completes.
 */
export function* waitUntil(event, after) {
    yield* waitFor(useDuration(event));
    if (after) {
        yield* after;
    }
}
decorate(waitFor, threadable());
/**
 * Wait for the given amount of time.
 *
 * @example
 * ```ts
 * // current time: 0s
 * yield waitFor(2);
 * // current time: 2s
 * yield waitFor(3);
 * // current time: 5s
 * ```
 *
 * @param seconds - The relative time in seconds.
 * @param after - An optional task to be run after the function completes.
 */
export function* waitFor(seconds = 0, after) {
    const thread = useThread();
    const step = usePlayback().framesToSeconds(1);
    const targetTime = thread.time() + seconds;
    // subtracting the step is not necessary, but it keeps the thread time ahead
    // of the project time.
    while (targetTime - step > thread.fixed) {
        yield;
    }
    thread.time(targetTime);
    if (after) {
        yield* after;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2NoZWR1bGluZy5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9mbG93L3NjaGVkdWxpbmcudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLFFBQVEsRUFBRSxVQUFVLEVBQUMsTUFBTSxlQUFlLENBQUM7QUFFbkQsT0FBTyxFQUFDLFdBQVcsRUFBRSxXQUFXLEVBQUUsU0FBUyxFQUFDLE1BQU0sVUFBVSxDQUFDO0FBRTdELFFBQVEsQ0FBQyxTQUFTLEVBQUUsVUFBVSxFQUFFLENBQUMsQ0FBQztBQUNsQzs7Ozs7Ozs7Ozs7Ozs7R0FjRztBQUNILE1BQU0sU0FBUyxDQUFDLENBQUMsU0FBUyxDQUN4QixLQUFhLEVBQ2IsS0FBdUI7SUFFdkIsS0FBSyxDQUFDLENBQUMsT0FBTyxDQUFDLFdBQVcsQ0FBQyxLQUFLLENBQUMsQ0FBQyxDQUFDO0lBRW5DLElBQUksS0FBSyxFQUFFO1FBQ1QsS0FBSyxDQUFDLENBQUMsS0FBSyxDQUFDO0tBQ2Q7QUFDSCxDQUFDO0FBRUQsUUFBUSxDQUFDLE9BQU8sRUFBRSxVQUFVLEVBQUUsQ0FBQyxDQUFDO0FBQ2hDOzs7Ozs7Ozs7Ozs7OztHQWNHO0FBQ0gsTUFBTSxTQUFTLENBQUMsQ0FBQyxPQUFPLENBQ3RCLE9BQU8sR0FBRyxDQUFDLEVBQ1gsS0FBdUI7SUFFdkIsTUFBTSxNQUFNLEdBQUcsU0FBUyxFQUFFLENBQUM7SUFDM0IsTUFBTSxJQUFJLEdBQUcsV0FBVyxFQUFFLENBQUMsZUFBZSxDQUFDLENBQUMsQ0FBQyxDQUFDO0lBRTlDLE1BQU0sVUFBVSxHQUFHLE1BQU0sQ0FBQyxJQUFJLEVBQUUsR0FBRyxPQUFPLENBQUM7SUFDM0MsNEVBQTRFO0lBQzVFLHVCQUF1QjtJQUN2QixPQUFPLFVBQVUsR0FBRyxJQUFJLEdBQUcsTUFBTSxDQUFDLEtBQUssRUFBRTtRQUN2QyxLQUFLLENBQUM7S0FDUDtJQUNELE1BQU0sQ0FBQyxJQUFJLENBQUMsVUFBVSxDQUFDLENBQUM7SUFFeEIsSUFBSSxLQUFLLEVBQUU7UUFDVCxLQUFLLENBQUMsQ0FBQyxLQUFLLENBQUM7S0FDZDtBQUNILENBQUMifQ==