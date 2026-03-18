import { EventDispatcherBase } from './EventDispatcherBase';
/**
 * Dispatches a {@link SubscribableEvent}.
 *
 * @example
 * ```ts
 * class Example {
 *   // expose the event to external classes
 *   public get onValueChanged {
 *     return this.value.subscribable;
 *   }
 *   // create a private dispatcher
 *   private value = new EventDispatcher<number>();
 *
 *   private dispatchExample() {
 *     // dispatching will notify all subscribers.
 *     this.value.dispatch(0);
 *   }
 * }
 * ```
 *
 * @typeParam T - The type of the value argument to subscribers.
 */
export class EventDispatcher extends EventDispatcherBase {
    dispatch(value) {
        this.notifySubscribers(value);
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiRXZlbnREaXNwYXRjaGVyLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2V2ZW50cy9FdmVudERpc3BhdGNoZXIudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLG1CQUFtQixFQUFlLE1BQU0sdUJBQXVCLENBQUM7QUFFeEU7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztHQXFCRztBQUNILE1BQU0sT0FBTyxlQUFtQixTQUFRLG1CQUFzQjtJQUNyRCxRQUFRLENBQUMsS0FBUTtRQUN0QixJQUFJLENBQUMsaUJBQWlCLENBQUMsS0FBSyxDQUFDLENBQUM7SUFDaEMsQ0FBQztDQUNGIn0=