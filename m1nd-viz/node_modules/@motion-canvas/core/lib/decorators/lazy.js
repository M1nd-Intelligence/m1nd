const UNINITIALIZED = Symbol.for('@motion-canvas/core/decorators/UNINITIALIZED');
/**
 * Create a lazy decorator.
 *
 * @remarks
 * A property marked as lazy will not be initialized until it's requested for
 * the first time. Lazy properties are read-only.
 *
 * Must be used for any static properties that require the DOM API to be
 * initialized.
 *
 * @param factory - A function that returns the value of this property.
 */
export function lazy(factory) {
    return (target, propertyKey) => {
        let value = UNINITIALIZED;
        Object.defineProperty(target, propertyKey, {
            get() {
                if (value === UNINITIALIZED) {
                    value = factory.call(this);
                }
                return value;
            },
        });
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibGF6eS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9kZWNvcmF0b3JzL2xhenkudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsTUFBTSxhQUFhLEdBQUcsTUFBTSxDQUFDLEdBQUcsQ0FDOUIsOENBQThDLENBQy9DLENBQUM7QUFFRjs7Ozs7Ozs7Ozs7R0FXRztBQUNILE1BQU0sVUFBVSxJQUFJLENBQUMsT0FBc0I7SUFDekMsT0FBTyxDQUFDLE1BQU0sRUFBRSxXQUFXLEVBQUUsRUFBRTtRQUM3QixJQUFJLEtBQUssR0FBWSxhQUFhLENBQUM7UUFDbkMsTUFBTSxDQUFDLGNBQWMsQ0FBQyxNQUFNLEVBQUUsV0FBVyxFQUFFO1lBQ3pDLEdBQUc7Z0JBQ0QsSUFBSSxLQUFLLEtBQUssYUFBYSxFQUFFO29CQUMzQixLQUFLLEdBQUcsT0FBTyxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQztpQkFDNUI7Z0JBQ0QsT0FBTyxLQUFLLENBQUM7WUFDZixDQUFDO1NBQ0YsQ0FBQyxDQUFDO0lBQ0wsQ0FBQyxDQUFDO0FBQ0osQ0FBQyJ9