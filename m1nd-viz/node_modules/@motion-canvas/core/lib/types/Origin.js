import { Vector2 } from './Vector';
export var Center;
(function (Center) {
    Center[Center["Vertical"] = 1] = "Vertical";
    Center[Center["Horizontal"] = 2] = "Horizontal";
})(Center || (Center = {}));
export var Direction;
(function (Direction) {
    Direction[Direction["Top"] = 4] = "Top";
    Direction[Direction["Bottom"] = 8] = "Bottom";
    Direction[Direction["Left"] = 16] = "Left";
    Direction[Direction["Right"] = 32] = "Right";
})(Direction || (Direction = {}));
export var Origin;
(function (Origin) {
    Origin[Origin["Middle"] = 3] = "Middle";
    Origin[Origin["Top"] = 5] = "Top";
    Origin[Origin["Bottom"] = 9] = "Bottom";
    Origin[Origin["Left"] = 18] = "Left";
    Origin[Origin["Right"] = 34] = "Right";
    Origin[Origin["TopLeft"] = 20] = "TopLeft";
    Origin[Origin["TopRight"] = 36] = "TopRight";
    Origin[Origin["BottomLeft"] = 24] = "BottomLeft";
    Origin[Origin["BottomRight"] = 40] = "BottomRight";
})(Origin || (Origin = {}));
export function flipOrigin(origin, axis = Center.Horizontal | Center.Vertical) {
    if (axis & Center.Vertical) {
        if (origin & Direction.Top) {
            origin = (origin & ~Direction.Top) | Direction.Bottom;
        }
        else if (origin & Direction.Bottom) {
            origin = (origin & ~Direction.Bottom) | Direction.Top;
        }
    }
    if (axis & Center.Horizontal) {
        if (origin & Direction.Left) {
            origin = (origin & ~Direction.Left) | Direction.Right;
        }
        else if (origin & Direction.Right) {
            origin = (origin & ~Direction.Right) | Direction.Left;
        }
    }
    return origin;
}
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
export function originToOffset(origin) {
    if (origin === Origin.Middle) {
        return Vector2.zero;
    }
    let x = 0;
    if (origin & Direction.Left) {
        x = -1;
    }
    else if (origin & Direction.Right) {
        x = 1;
    }
    let y = 0;
    if (origin & Direction.Top) {
        y = -1;
    }
    else if (origin & Direction.Bottom) {
        y = 1;
    }
    return new Vector2(x, y);
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiT3JpZ2luLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3R5cGVzL09yaWdpbi50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUMsT0FBTyxFQUFDLE1BQU0sVUFBVSxDQUFDO0FBRWpDLE1BQU0sQ0FBTixJQUFZLE1BR1g7QUFIRCxXQUFZLE1BQU07SUFDaEIsMkNBQVksQ0FBQTtJQUNaLCtDQUFjLENBQUE7QUFDaEIsQ0FBQyxFQUhXLE1BQU0sS0FBTixNQUFNLFFBR2pCO0FBRUQsTUFBTSxDQUFOLElBQVksU0FLWDtBQUxELFdBQVksU0FBUztJQUNuQix1Q0FBTyxDQUFBO0lBQ1AsNkNBQVUsQ0FBQTtJQUNWLDBDQUFTLENBQUE7SUFDVCw0Q0FBVSxDQUFBO0FBQ1osQ0FBQyxFQUxXLFNBQVMsS0FBVCxTQUFTLFFBS3BCO0FBRUQsTUFBTSxDQUFOLElBQVksTUFVWDtBQVZELFdBQVksTUFBTTtJQUNoQix1Q0FBVSxDQUFBO0lBQ1YsaUNBQU8sQ0FBQTtJQUNQLHVDQUFVLENBQUE7SUFDVixvQ0FBUyxDQUFBO0lBQ1Qsc0NBQVUsQ0FBQTtJQUNWLDBDQUFZLENBQUE7SUFDWiw0Q0FBYSxDQUFBO0lBQ2IsZ0RBQWUsQ0FBQTtJQUNmLGtEQUFnQixDQUFBO0FBQ2xCLENBQUMsRUFWVyxNQUFNLEtBQU4sTUFBTSxRQVVqQjtBQUlELE1BQU0sVUFBVSxVQUFVLENBQ3hCLE1BQTBCLEVBQzFCLE9BQWUsTUFBTSxDQUFDLFVBQVUsR0FBRyxNQUFNLENBQUMsUUFBUTtJQUVsRCxJQUFJLElBQUksR0FBRyxNQUFNLENBQUMsUUFBUSxFQUFFO1FBQzFCLElBQUksTUFBTSxHQUFHLFNBQVMsQ0FBQyxHQUFHLEVBQUU7WUFDMUIsTUFBTSxHQUFHLENBQUMsTUFBTSxHQUFHLENBQUMsU0FBUyxDQUFDLEdBQUcsQ0FBQyxHQUFHLFNBQVMsQ0FBQyxNQUFNLENBQUM7U0FDdkQ7YUFBTSxJQUFJLE1BQU0sR0FBRyxTQUFTLENBQUMsTUFBTSxFQUFFO1lBQ3BDLE1BQU0sR0FBRyxDQUFDLE1BQU0sR0FBRyxDQUFDLFNBQVMsQ0FBQyxNQUFNLENBQUMsR0FBRyxTQUFTLENBQUMsR0FBRyxDQUFDO1NBQ3ZEO0tBQ0Y7SUFFRCxJQUFJLElBQUksR0FBRyxNQUFNLENBQUMsVUFBVSxFQUFFO1FBQzVCLElBQUksTUFBTSxHQUFHLFNBQVMsQ0FBQyxJQUFJLEVBQUU7WUFDM0IsTUFBTSxHQUFHLENBQUMsTUFBTSxHQUFHLENBQUMsU0FBUyxDQUFDLElBQUksQ0FBQyxHQUFHLFNBQVMsQ0FBQyxLQUFLLENBQUM7U0FDdkQ7YUFBTSxJQUFJLE1BQU0sR0FBRyxTQUFTLENBQUMsS0FBSyxFQUFFO1lBQ25DLE1BQU0sR0FBRyxDQUFDLE1BQU0sR0FBRyxDQUFDLFNBQVMsQ0FBQyxLQUFLLENBQUMsR0FBRyxTQUFTLENBQUMsSUFBSSxDQUFDO1NBQ3ZEO0tBQ0Y7SUFFRCxPQUFPLE1BQU0sQ0FBQztBQUNoQixDQUFDO0FBRUQ7Ozs7Ozs7Ozs7R0FVRztBQUNILE1BQU0sVUFBVSxjQUFjLENBQUMsTUFBMEI7SUFDdkQsSUFBSSxNQUFNLEtBQUssTUFBTSxDQUFDLE1BQU0sRUFBRTtRQUM1QixPQUFPLE9BQU8sQ0FBQyxJQUFJLENBQUM7S0FDckI7SUFFRCxJQUFJLENBQUMsR0FBRyxDQUFDLENBQUM7SUFDVixJQUFJLE1BQU0sR0FBRyxTQUFTLENBQUMsSUFBSSxFQUFFO1FBQzNCLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztLQUNSO1NBQU0sSUFBSSxNQUFNLEdBQUcsU0FBUyxDQUFDLEtBQUssRUFBRTtRQUNuQyxDQUFDLEdBQUcsQ0FBQyxDQUFDO0tBQ1A7SUFFRCxJQUFJLENBQUMsR0FBRyxDQUFDLENBQUM7SUFDVixJQUFJLE1BQU0sR0FBRyxTQUFTLENBQUMsR0FBRyxFQUFFO1FBQzFCLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztLQUNSO1NBQU0sSUFBSSxNQUFNLEdBQUcsU0FBUyxDQUFDLE1BQU0sRUFBRTtRQUNwQyxDQUFDLEdBQUcsQ0FBQyxDQUFDO0tBQ1A7SUFFRCxPQUFPLElBQUksT0FBTyxDQUFDLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQztBQUMzQixDQUFDIn0=