/**
 * A base for dispatching {@link Subscribable}s.
 *
 * @typeParam TValue - The type of the argument passed to subscribers.
 * @typeParam THandler - The type of the callback function.
 */
export class EventDispatcherBase {
    constructor() {
        this.subscribable = new Subscribable(this);
        this.subscribers = new Set();
    }
    /**
     * {@inheritDoc Subscribable.subscribe}
     */
    subscribe(handler) {
        this.subscribers.add(handler);
        return () => this.unsubscribe(handler);
    }
    /**
     * {@inheritDoc Subscribable.unsubscribe}
     */
    unsubscribe(handler) {
        this.subscribers.delete(handler);
    }
    /**
     * Unsubscribe all subscribers from the event.
     */
    clear() {
        this.subscribers.clear();
    }
    notifySubscribers(value) {
        return [...this.subscribers].map(handler => handler(value));
    }
}
/**
 * Provides safe access to the public interface of {@link EventDispatcherBase}.
 *
 * @remarks
 * External classes can use it to subscribe to an event without being able to
 * dispatch it.
 *
 * @typeParam TValue - The type of the argument passed to subscribers.
 * @typeParam THandler - The type of the callback function.
 */
export class Subscribable {
    constructor(dispatcher) {
        this.dispatcher = dispatcher;
    }
    /**
     * Subscribe to the event.
     *
     * @param handler - The handler to invoke when the event occurs.
     *
     * @returns A callback function that cancels the subscription.
     */
    subscribe(handler) {
        return this.dispatcher.subscribe(handler);
    }
    /**
     * Unsubscribe from the event.
     *
     * @param handler - The handler to unsubscribe.
     */
    unsubscribe(handler) {
        this.dispatcher.unsubscribe(handler);
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiRXZlbnREaXNwYXRjaGVyQmFzZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9ldmVudHMvRXZlbnREaXNwYXRjaGVyQmFzZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFJQTs7Ozs7R0FLRztBQUNILE1BQU0sT0FBZ0IsbUJBQW1CO0lBQXpDO1FBSWtCLGlCQUFZLEdBQzFCLElBQUksWUFBWSxDQUFDLElBQUksQ0FBQyxDQUFDO1FBRWpCLGdCQUFXLEdBQUcsSUFBSSxHQUFHLEVBQVksQ0FBQztJQTJCNUMsQ0FBQztJQXpCQzs7T0FFRztJQUNJLFNBQVMsQ0FBQyxPQUFpQjtRQUNoQyxJQUFJLENBQUMsV0FBVyxDQUFDLEdBQUcsQ0FBQyxPQUFPLENBQUMsQ0FBQztRQUM5QixPQUFPLEdBQUcsRUFBRSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQUMsT0FBTyxDQUFDLENBQUM7SUFDekMsQ0FBQztJQUVEOztPQUVHO0lBQ0ksV0FBVyxDQUFDLE9BQWlCO1FBQ2xDLElBQUksQ0FBQyxXQUFXLENBQUMsTUFBTSxDQUFDLE9BQU8sQ0FBQyxDQUFDO0lBQ25DLENBQUM7SUFFRDs7T0FFRztJQUNJLEtBQUs7UUFDVixJQUFJLENBQUMsV0FBVyxDQUFDLEtBQUssRUFBRSxDQUFDO0lBQzNCLENBQUM7SUFFUyxpQkFBaUIsQ0FBQyxLQUFhO1FBQ3ZDLE9BQU8sQ0FBQyxHQUFHLElBQUksQ0FBQyxXQUFXLENBQUMsQ0FBQyxHQUFHLENBQUMsT0FBTyxDQUFDLEVBQUUsQ0FBQyxPQUFPLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQztJQUM5RCxDQUFDO0NBQ0Y7QUFFRDs7Ozs7Ozs7O0dBU0c7QUFDSCxNQUFNLE9BQU8sWUFBWTtJQUl2QixZQUNZLFVBQWlEO1FBQWpELGVBQVUsR0FBVixVQUFVLENBQXVDO0lBQzFELENBQUM7SUFFSjs7Ozs7O09BTUc7SUFDSSxTQUFTLENBQUMsT0FBaUI7UUFDaEMsT0FBTyxJQUFJLENBQUMsVUFBVSxDQUFDLFNBQVMsQ0FBQyxPQUFPLENBQUMsQ0FBQztJQUM1QyxDQUFDO0lBRUQ7Ozs7T0FJRztJQUNJLFdBQVcsQ0FBQyxPQUFpQjtRQUNsQyxJQUFJLENBQUMsVUFBVSxDQUFDLFdBQVcsQ0FBQyxPQUFPLENBQUMsQ0FBQztJQUN2QyxDQUFDO0NBQ0YifQ==