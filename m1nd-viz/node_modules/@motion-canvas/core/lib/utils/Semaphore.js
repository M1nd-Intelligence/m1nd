/**
 * A simple semaphore implementation with a capacity of 1.
 *
 * @internal
 */
export class Semaphore {
    constructor() {
        this.resolveCurrent = null;
        this.current = null;
    }
    async acquire() {
        while (this.current) {
            await this.current;
        }
        this.current = new Promise(resolve => {
            this.resolveCurrent = resolve;
        });
    }
    release() {
        this.current = null;
        this.resolveCurrent?.();
        this.resolveCurrent = null;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiU2VtYXBob3JlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3V0aWxzL1NlbWFwaG9yZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQTs7OztHQUlHO0FBQ0gsTUFBTSxPQUFPLFNBQVM7SUFBdEI7UUFDVSxtQkFBYyxHQUF3QixJQUFJLENBQUM7UUFDM0MsWUFBTyxHQUF5QixJQUFJLENBQUM7SUFnQi9DLENBQUM7SUFkUSxLQUFLLENBQUMsT0FBTztRQUNsQixPQUFPLElBQUksQ0FBQyxPQUFPLEVBQUU7WUFDbkIsTUFBTSxJQUFJLENBQUMsT0FBTyxDQUFDO1NBQ3BCO1FBQ0QsSUFBSSxDQUFDLE9BQU8sR0FBRyxJQUFJLE9BQU8sQ0FBQyxPQUFPLENBQUMsRUFBRTtZQUNuQyxJQUFJLENBQUMsY0FBYyxHQUFHLE9BQU8sQ0FBQztRQUNoQyxDQUFDLENBQUMsQ0FBQztJQUNMLENBQUM7SUFFTSxPQUFPO1FBQ1osSUFBSSxDQUFDLE9BQU8sR0FBRyxJQUFJLENBQUM7UUFDcEIsSUFBSSxDQUFDLGNBQWMsRUFBRSxFQUFFLENBQUM7UUFDeEIsSUFBSSxDQUFDLGNBQWMsR0FBRyxJQUFJLENBQUM7SUFDN0IsQ0FBQztDQUNGIn0=