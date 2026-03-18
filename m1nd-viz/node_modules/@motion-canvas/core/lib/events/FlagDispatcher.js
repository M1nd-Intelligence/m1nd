import { EventDispatcherBase, } from './EventDispatcherBase';
/**
 * Dispatches a {@link SubscribableFlagEvent}.
 *
 * @remarks
 * Subscribers are notified only when the flag is set.
 * Subsequent calls to {@link raise} don't trigger anything.
 * Any handlers added while the flag is raised are immediately invoked.
 *
 * Resetting the flag doesn't notify the subscribers, but raising it again does.
 *
 * @example
 * ```ts
 * class Example {
 *   // expose the event to external classes
 *   public get onChanged {
 *     return this.flag.subscribable;
 *   }
 *   // create a private dispatcher
 *   private flag = new FlagDispatcher();
 *
 *   private dispatchExample() {
 *     // setting the flag will notify all subscribers
 *     this.flag.raise();
 *   }
 * }
 * ```
 */
export class FlagDispatcher extends EventDispatcherBase {
    constructor() {
        super(...arguments);
        this.value = false;
    }
    /**
     * Notify all current and future subscribers.
     */
    raise() {
        if (!this.value) {
            this.value = true;
            this.notifySubscribers();
        }
    }
    /**
     * Stop notifying future subscribers.
     */
    reset() {
        this.value = false;
    }
    /**
     * Are subscribers being notified?
     */
    isRaised() {
        return this.value;
    }
    subscribe(handler) {
        const unsubscribe = super.subscribe(handler);
        if (this.value) {
            handler();
        }
        return unsubscribe;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiRmxhZ0Rpc3BhdGNoZXIuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvZXZlbnRzL0ZsYWdEaXNwYXRjaGVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFDTCxtQkFBbUIsR0FHcEIsTUFBTSx1QkFBdUIsQ0FBQztBQUUvQjs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7R0EwQkc7QUFDSCxNQUFNLE9BQU8sY0FBZSxTQUFRLG1CQUF5QjtJQUE3RDs7UUFDVSxVQUFLLEdBQUcsS0FBSyxDQUFDO0lBaUN4QixDQUFDO0lBL0JDOztPQUVHO0lBQ0ksS0FBSztRQUNWLElBQUksQ0FBQyxJQUFJLENBQUMsS0FBSyxFQUFFO1lBQ2YsSUFBSSxDQUFDLEtBQUssR0FBRyxJQUFJLENBQUM7WUFDbEIsSUFBSSxDQUFDLGlCQUFpQixFQUFFLENBQUM7U0FDMUI7SUFDSCxDQUFDO0lBRUQ7O09BRUc7SUFDSSxLQUFLO1FBQ1YsSUFBSSxDQUFDLEtBQUssR0FBRyxLQUFLLENBQUM7SUFDckIsQ0FBQztJQUVEOztPQUVHO0lBQ0ksUUFBUTtRQUNiLE9BQU8sSUFBSSxDQUFDLEtBQUssQ0FBQztJQUNwQixDQUFDO0lBRU0sU0FBUyxDQUFDLE9BQTJCO1FBQzFDLE1BQU0sV0FBVyxHQUFHLEtBQUssQ0FBQyxTQUFTLENBQUMsT0FBTyxDQUFDLENBQUM7UUFDN0MsSUFBSSxJQUFJLENBQUMsS0FBSyxFQUFFO1lBQ2QsT0FBTyxFQUFFLENBQUM7U0FDWDtRQUNELE9BQU8sV0FBVyxDQUFDO0lBQ3JCLENBQUM7Q0FDRiJ9