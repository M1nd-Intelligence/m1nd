import type { Scene } from '../Scene';
import type { TimeEvent } from './TimeEvent';
import type { TimeEvents } from './TimeEvents';
/**
 * Manages time events during rendering and presentation.
 */
export declare class ReadOnlyTimeEvents implements TimeEvents {
    private readonly scene;
    get onChanged(): import("../../events").SubscribableValueEvent<TimeEvent[]>;
    private readonly events;
    private lookup;
    constructor(scene: Scene);
    set(): void;
    register(name: string, initialTime: number): number;
    /**
     * Called when the parent scene gets reloaded.
     */
    private handleReload;
}
//# sourceMappingURL=ReadOnlyTimeEvents.d.ts.map