/**
 * Create an array of references.
 *
 * @remarks
 * The returned object is both an array and a reference that can be passed
 * directly to the `ref` property of a node.
 *
 * @example
 * ```tsx
 * const labels = createRefArray<Txt>();
 *
 * view.add(['A', 'B'].map(text => <Txt ref={labels}>{text}</Txt>));
 * view.add(<Txt ref={labels}>C</Txt>);
 *
 * // accessing the references individually:
 * yield* labels[0].text('A changes', 0.3);
 * yield* labels[1].text('B changes', 0.3);
 * yield* labels[2].text('C changes', 0.3);
 *
 * // accessing all references at once:
 * yield* all(...labels.map(label => label.fill('white', 0.3)));
 * ```
 */
export function createRefArray() {
    const target = function () {
        // do nothing
    };
    target.array = [];
    return new Proxy(target, Handler);
}
const Handler = {
    apply(target, _, argArray) {
        if (argArray.length === 0) {
            return target.array[0];
        }
        target.array.push(...argArray);
    },
    get(target, property, receiver) {
        const value = Reflect.get(target.array, property, receiver);
        if (typeof value === 'function') {
            return value.bind(target.array);
        }
        return value;
    },
    set(target, property, value, receiver) {
        return Reflect.set(target.array, property, value, receiver);
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY3JlYXRlUmVmQXJyYXkuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvdXRpbHMvY3JlYXRlUmVmQXJyYXkudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBU0E7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7R0FzQkc7QUFDSCxNQUFNLFVBQVUsY0FBYztJQUM1QixNQUFNLE1BQU0sR0FBRztRQUNiLGFBQWE7SUFDZixDQUFtQixDQUFDO0lBQ3BCLE1BQU0sQ0FBQyxLQUFLLEdBQUcsRUFBRSxDQUFDO0lBQ2xCLE9BQU8sSUFBSSxLQUFLLENBQUMsTUFBTSxFQUFFLE9BQU8sQ0FBaUMsQ0FBQztBQUNwRSxDQUFDO0FBRUQsTUFBTSxPQUFPLEdBQW1DO0lBQzlDLEtBQUssQ0FBQyxNQUFNLEVBQUUsQ0FBQyxFQUFFLFFBQVE7UUFDdkIsSUFBSSxRQUFRLENBQUMsTUFBTSxLQUFLLENBQUMsRUFBRTtZQUN6QixPQUFPLE1BQU0sQ0FBQyxLQUFLLENBQUMsQ0FBQyxDQUFDLENBQUM7U0FDeEI7UUFFRCxNQUFNLENBQUMsS0FBSyxDQUFDLElBQUksQ0FBQyxHQUFHLFFBQVEsQ0FBQyxDQUFDO0lBQ2pDLENBQUM7SUFFRCxHQUFHLENBQUMsTUFBTSxFQUFFLFFBQVEsRUFBRSxRQUFRO1FBQzVCLE1BQU0sS0FBSyxHQUFHLE9BQU8sQ0FBQyxHQUFHLENBQUMsTUFBTSxDQUFDLEtBQUssRUFBRSxRQUFRLEVBQUUsUUFBUSxDQUFDLENBQUM7UUFDNUQsSUFBSSxPQUFPLEtBQUssS0FBSyxVQUFVLEVBQUU7WUFDL0IsT0FBTyxLQUFLLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxLQUFLLENBQUMsQ0FBQztTQUNqQztRQUVELE9BQU8sS0FBSyxDQUFDO0lBQ2YsQ0FBQztJQUVELEdBQUcsQ0FBQyxNQUFNLEVBQUUsUUFBUSxFQUFFLEtBQUssRUFBRSxRQUFRO1FBQ25DLE9BQU8sT0FBTyxDQUFDLEdBQUcsQ0FBQyxNQUFNLENBQUMsS0FBSyxFQUFFLFFBQVEsRUFBRSxLQUFLLEVBQUUsUUFBUSxDQUFDLENBQUM7SUFDOUQsQ0FBQztDQUNGLENBQUMifQ==