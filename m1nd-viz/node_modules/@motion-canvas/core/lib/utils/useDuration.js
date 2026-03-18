import { useScene } from './useScene';
import { useThread } from './useThread';
/**
 * Register a time event and get its duration in seconds.
 *
 * @remarks
 * This can be used to better specify when an animation should start
 * as well as how long this animation should take
 *
 * @example
 * ```ts
 * export default makeScene2D(function* (view) {
 *   const circle = createRef<Circle>();
 *
 *   view.add(
 *     <Circle ref={circle} width={320} height={320} fill={'lightseagreen'} />,
 *   );
 *
 *   yield* circle().scale(2, useDuration('circleGrow'));
 * });
 * ```
 *
 * @param name - The name of the event.
 *
 * @returns The duration of the event in seconds.
 */
export function useDuration(name) {
    const scene = useScene();
    const thread = useThread();
    return scene.timeEvents.register(name, thread.time());
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidXNlRHVyYXRpb24uanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvdXRpbHMvdXNlRHVyYXRpb24udHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLFFBQVEsRUFBQyxNQUFNLFlBQVksQ0FBQztBQUNwQyxPQUFPLEVBQUMsU0FBUyxFQUFDLE1BQU0sYUFBYSxDQUFDO0FBRXRDOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztHQXVCRztBQUNILE1BQU0sVUFBVSxXQUFXLENBQUMsSUFBWTtJQUN0QyxNQUFNLEtBQUssR0FBRyxRQUFRLEVBQUUsQ0FBQztJQUN6QixNQUFNLE1BQU0sR0FBRyxTQUFTLEVBQUUsQ0FBQztJQUMzQixPQUFPLEtBQUssQ0FBQyxVQUFVLENBQUMsUUFBUSxDQUFDLElBQUksRUFBRSxNQUFNLENBQUMsSUFBSSxFQUFFLENBQUMsQ0FBQztBQUN4RCxDQUFDIn0=