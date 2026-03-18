import type { Scene } from './Scene';
export interface Slide {
    id: string;
    name: string;
    time: number;
    scene: Scene;
    stack?: string;
}
export declare class Slides {
    private readonly scene;
    get onChanged(): import("../events").SubscribableValueEvent<Slide[]>;
    private readonly slides;
    private readonly lookup;
    private readonly collisionLookup;
    private current;
    private canResume;
    private waitsForId;
    private targetId;
    constructor(scene: Scene);
    setTarget(target: string | null): void;
    resume(): void;
    isWaitingFor(slide: string): boolean;
    isWaiting(): boolean;
    didHappen(slide: string): boolean;
    getCurrent(): Slide | null;
    register(name: string, initialTime: number): void;
    shouldWait(name: string): boolean;
    private handleReload;
    private handleReset;
    private handleRecalculated;
    private toId;
}
//# sourceMappingURL=Slides.d.ts.map