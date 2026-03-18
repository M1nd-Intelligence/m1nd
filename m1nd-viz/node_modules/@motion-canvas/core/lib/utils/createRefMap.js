import { createRef } from './createRef';
/**
 * Create a group of references.
 *
 * @remarks
 * The returned object lets you easily create multiple references to the same
 * type without initializing them individually.
 *
 * You can retrieve references by accessing the object's properties. If the
 * reference for a given property does not exist, it will be created
 * automatically.
 *
 * @example
 * ```tsx
 * const labels = createRefMap<Txt>();
 *
 * view.add(
 *   <>
 *     <Txt ref={labels.a}>A</Txt>
 *     <Txt ref={labels.b}>B</Txt>
 *     <Txt ref={labels.c}>C</Txt>
 *   </>,
 * );
 *
 * // accessing the references individually:
 * yield* labels.a().text('A changes', 0.3);
 * yield* labels.b().text('B changes', 0.3);
 * yield* labels.c().text('C changes', 0.3);
 *
 * // checking if the given reference exists:
 * if ('d' in labels) {
 *   yield* labels.d().text('D changes', 0.3);
 * }
 *
 * // accessing all references at once:
 * yield* all(...labels.mapRefs(label => label.fill('white', 0.3)));
 * ```
 */
export function createRefMap() {
    const group = new Map();
    group.entries();
    return new Proxy(group, Handler);
}
const Handler = {
    get(target, property) {
        if (Reflect.has(target, property)) {
            return Reflect.get(target, property);
        }
        if (property === 'mapRefs') {
            return function (callback) {
                const result = [];
                for (const value of target.values()) {
                    result.push(callback(value(), result.length));
                }
                return result;
            };
        }
        if (typeof property === 'string') {
            let value = target.get(property);
            if (!value) {
                value = createRef();
                target.set(property, value);
            }
            return value;
        }
    },
    has(target, property) {
        if (Reflect.has(target, property)) {
            return true;
        }
        if (typeof property === 'string') {
            return target.has(property);
        }
        return false;
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY3JlYXRlUmVmTWFwLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3V0aWxzL2NyZWF0ZVJlZk1hcC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUMsU0FBUyxFQUFZLE1BQU0sYUFBYSxDQUFDO0FBY2pEOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7R0FvQ0c7QUFDSCxNQUFNLFVBQVUsWUFBWTtJQUMxQixNQUFNLEtBQUssR0FBRyxJQUFJLEdBQUcsRUFBd0IsQ0FBQztJQUM5QyxLQUFLLENBQUMsT0FBTyxFQUFFLENBQUM7SUFDaEIsT0FBTyxJQUFJLEtBQUssQ0FBQyxLQUFLLEVBQUUsT0FBTyxDQUFvQixDQUFDO0FBQ3RELENBQUM7QUFFRCxNQUFNLE9BQU8sR0FBOEM7SUFDekQsR0FBRyxDQUFDLE1BQU0sRUFBRSxRQUFRO1FBQ2xCLElBQUksT0FBTyxDQUFDLEdBQUcsQ0FBQyxNQUFNLEVBQUUsUUFBUSxDQUFDLEVBQUU7WUFDakMsT0FBTyxPQUFPLENBQUMsR0FBRyxDQUFDLE1BQU0sRUFBRSxRQUFRLENBQUMsQ0FBQztTQUN0QztRQUVELElBQUksUUFBUSxLQUFLLFNBQVMsRUFBRTtZQUMxQixPQUFPLFVBQ0wsUUFBK0M7Z0JBRS9DLE1BQU0sTUFBTSxHQUFhLEVBQUUsQ0FBQztnQkFDNUIsS0FBSyxNQUFNLEtBQUssSUFBSSxNQUFNLENBQUMsTUFBTSxFQUFFLEVBQUU7b0JBQ25DLE1BQU0sQ0FBQyxJQUFJLENBQUMsUUFBUSxDQUFDLEtBQUssRUFBRSxFQUFFLE1BQU0sQ0FBQyxNQUFNLENBQUMsQ0FBQyxDQUFDO2lCQUMvQztnQkFDRCxPQUFPLE1BQU0sQ0FBQztZQUNoQixDQUFDLENBQUM7U0FDSDtRQUVELElBQUksT0FBTyxRQUFRLEtBQUssUUFBUSxFQUFFO1lBQ2hDLElBQUksS0FBSyxHQUFHLE1BQU0sQ0FBQyxHQUFHLENBQUMsUUFBUSxDQUFDLENBQUM7WUFDakMsSUFBSSxDQUFDLEtBQUssRUFBRTtnQkFDVixLQUFLLEdBQUcsU0FBUyxFQUFFLENBQUM7Z0JBQ3BCLE1BQU0sQ0FBQyxHQUFHLENBQUMsUUFBUSxFQUFFLEtBQUssQ0FBQyxDQUFDO2FBQzdCO1lBQ0QsT0FBTyxLQUFLLENBQUM7U0FDZDtJQUNILENBQUM7SUFFRCxHQUFHLENBQUMsTUFBTSxFQUFFLFFBQVE7UUFDbEIsSUFBSSxPQUFPLENBQUMsR0FBRyxDQUFDLE1BQU0sRUFBRSxRQUFRLENBQUMsRUFBRTtZQUNqQyxPQUFPLElBQUksQ0FBQztTQUNiO1FBRUQsSUFBSSxPQUFPLFFBQVEsS0FBSyxRQUFRLEVBQUU7WUFDaEMsT0FBTyxNQUFNLENBQUMsR0FBRyxDQUFDLFFBQVEsQ0FBQyxDQUFDO1NBQzdCO1FBRUQsT0FBTyxLQUFLLENBQUM7SUFDZixDQUFDO0NBQ0YsQ0FBQyJ9