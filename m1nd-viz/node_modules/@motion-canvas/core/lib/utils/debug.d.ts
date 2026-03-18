/**
 * Logs a debug message with an arbitrary payload.
 *
 * @remarks
 * This method is a shortcut for calling `useLogger().debug()` which allows
 * you to more easily log non-string values as well.
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
 *   debug(circle().position());
 * });
 * ```
 *
 * @param payload - The payload to log
 */
export declare function debug(payload: any): void;
//# sourceMappingURL=debug.d.ts.map