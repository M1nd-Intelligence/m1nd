/**
 * A helper function for exporting Motion Canvas plugins.
 *
 * @param plugin - The plugin configuration.
 *
 * @example
 * ```ts
 * export default makePlugin({
 *   name: 'my-custom-plugin',
 * });
 * ```
 */
export function makePlugin(plugin) {
    return typeof plugin === 'function' ? plugin : () => plugin;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWFrZVBsdWdpbi5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wbHVnaW4vbWFrZVBsdWdpbi50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFFQTs7Ozs7Ozs7Ozs7R0FXRztBQUNILE1BQU0sVUFBVSxVQUFVLENBQUMsTUFBK0I7SUFDeEQsT0FBTyxPQUFPLE1BQU0sS0FBSyxVQUFVLENBQUMsQ0FBQyxDQUFDLE1BQU0sQ0FBQyxDQUFDLENBQUMsR0FBRyxFQUFFLENBQUMsTUFBTSxDQUFDO0FBQzlELENBQUMifQ==