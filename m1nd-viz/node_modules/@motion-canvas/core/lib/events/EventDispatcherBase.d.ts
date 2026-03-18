export interface EventHandler<T> {
    (value: T): any;
}
/**
 * A base for dispatching {@link Subscribable}s.
 *
 * @typeParam TValue - The type of the argument passed to subscribers.
 * @typeParam THandler - The type of the callback function.
 */
export declare abstract class EventDispatcherBase<TValue, THandler extends EventHandler<TValue> = EventHandler<TValue>> {
    readonly subscribable: Subscribable<TValue, THandler>;
    private subscribers;
    /**
     * {@inheritDoc Subscribable.subscribe}
     */
    subscribe(handler: THandler): () => void;
    /**
     * {@inheritDoc Subscribable.unsubscribe}
     */
    unsubscribe(handler: THandler): void;
    /**
     * Unsubscribe all subscribers from the event.
     */
    clear(): void;
    protected notifySubscribers(value: TValue): ReturnType<THandler>[];
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
export declare class Subscribable<TValue, THandler extends EventHandler<TValue> = EventHandler<TValue>> {
    protected dispatcher: EventDispatcherBase<TValue, THandler>;
    constructor(dispatcher: EventDispatcherBase<TValue, THandler>);
    /**
     * Subscribe to the event.
     *
     * @param handler - The handler to invoke when the event occurs.
     *
     * @returns A callback function that cancels the subscription.
     */
    subscribe(handler: THandler): () => void;
    /**
     * Unsubscribe from the event.
     *
     * @param handler - The handler to unsubscribe.
     */
    unsubscribe(handler: THandler): void;
}
//# sourceMappingURL=EventDispatcherBase.d.ts.map