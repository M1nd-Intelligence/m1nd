import { EventDispatcherBase } from './EventDispatcherBase';
/**
 * Dispatches an asynchronous {@link SubscribableEvent}.
 *
 * @remarks
 * The {@link dispatch} method returns a promise that resolves when all the
 * handlers resolve.
 *
 * @example
 * ```ts
 * class Example {
 *   // expose the event to external classes
 *   public get onValueChanged {
 *     return this.value.subscribable;
 *   }
 *   // create a private dispatcher
 *   private value = new AsyncEventDispatcher<number>();
 *
 *   private async dispatchExample() {
 *     // dispatching returns a Promise.
 *     await this.value.dispatch(0);
 *   }
 * }
 * ```
 *
 * @typeParam T - The type of the argument passed to subscribers.
 */
export class AsyncEventDispatcher extends EventDispatcherBase {
    async dispatch(value) {
        await Promise.all(this.notifySubscribers(value));
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQXN5bmNFdmVudERpc3BhdGNoZXIuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvZXZlbnRzL0FzeW5jRXZlbnREaXNwYXRjaGVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBQyxtQkFBbUIsRUFBZSxNQUFNLHVCQUF1QixDQUFDO0FBTXhFOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0dBeUJHO0FBQ0gsTUFBTSxPQUFPLG9CQUF3QixTQUFRLG1CQUc1QztJQUNRLEtBQUssQ0FBQyxRQUFRLENBQUMsS0FBUTtRQUM1QixNQUFNLE9BQU8sQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLGlCQUFpQixDQUFDLEtBQUssQ0FBQyxDQUFDLENBQUM7SUFDbkQsQ0FBQztDQUNGIn0=