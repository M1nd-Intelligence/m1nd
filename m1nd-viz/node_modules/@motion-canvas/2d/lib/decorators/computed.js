import { createComputed } from '@motion-canvas/core';
import { addInitializer } from './initializers';
/**
 * Create a computed method decorator.
 *
 * @remarks
 * This decorator turns the given method into a computed value.
 * See {@link createComputed} for more information.
 */
export function computed() {
    return (target, key) => {
        addInitializer(target, (instance) => {
            const method = Object.getPrototypeOf(instance)[key];
            instance[key] = createComputed(method.bind(instance), instance);
        });
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY29tcHV0ZWQuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2RlY29yYXRvcnMvY29tcHV0ZWQudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLGNBQWMsRUFBQyxNQUFNLHFCQUFxQixDQUFDO0FBQ25ELE9BQU8sRUFBQyxjQUFjLEVBQUMsTUFBTSxnQkFBZ0IsQ0FBQztBQUU5Qzs7Ozs7O0dBTUc7QUFDSCxNQUFNLFVBQVUsUUFBUTtJQUN0QixPQUFPLENBQUMsTUFBVyxFQUFFLEdBQUcsRUFBRSxFQUFFO1FBQzFCLGNBQWMsQ0FBQyxNQUFNLEVBQUUsQ0FBQyxRQUFhLEVBQUUsRUFBRTtZQUN2QyxNQUFNLE1BQU0sR0FBRyxNQUFNLENBQUMsY0FBYyxDQUFDLFFBQVEsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxDQUFDO1lBQ3BELFFBQVEsQ0FBQyxHQUFHLENBQUMsR0FBRyxjQUFjLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxRQUFRLENBQUMsRUFBRSxRQUFRLENBQUMsQ0FBQztRQUNsRSxDQUFDLENBQUMsQ0FBQztJQUNMLENBQUMsQ0FBQztBQUNKLENBQUMifQ==