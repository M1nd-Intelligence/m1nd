import { useThread } from './useThread';
/**
 * Get the real time since the start of the animation.
 *
 * @remarks
 * The returned value accounts for offsets caused by functions such as
 * {@link flow.waitFor}.
 *
 * @example
 * ```ts
 * // current time: 0s
 * yield* waitFor(0.02);
 *
 * // current time: 0.016(6)s
 * // real time: 0.02s
 * const realTime = useTime();
 * ```
 */
export function useTime() {
    return useThread().time();
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidXNlVGltZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy91dGlscy91c2VUaW1lLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBQyxTQUFTLEVBQUMsTUFBTSxhQUFhLENBQUM7QUFFdEM7Ozs7Ozs7Ozs7Ozs7Ozs7R0FnQkc7QUFDSCxNQUFNLFVBQVUsT0FBTztJQUNyQixPQUFPLFNBQVMsRUFBRSxDQUFDLElBQUksRUFBRSxDQUFDO0FBQzVCLENBQUMifQ==