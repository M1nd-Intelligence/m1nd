import { Vector2 } from './Vector';
export declare enum Center {
    Vertical = 1,
    Horizontal = 2
}
export declare enum Direction {
    Top = 4,
    Bottom = 8,
    Left = 16,
    Right = 32
}
export declare enum Origin {
    Middle = 3,
    Top = 5,
    Bottom = 9,
    Left = 18,
    Right = 34,
    TopLeft = 20,
    TopRight = 36,
    BottomLeft = 24,
    BottomRight = 40
}
export declare function flipOrigin(origin: Direction, axis?: Center): Direction;
export declare function flipOrigin(origin: Origin, axis?: Center): Origin;
/**
 * Convert the given origin to a vector representing its offset.
 *
 * @example
 * ```ts
 * const bottomRight = originToOffset(Origin.TopRight);
 * // bottomRight = {x: 1, y: -1}
 * ```
 *
 * @param origin - The origin to convert.
 */
export declare function originToOffset(origin: Origin | Direction): Vector2;
//# sourceMappingURL=Origin.d.ts.map