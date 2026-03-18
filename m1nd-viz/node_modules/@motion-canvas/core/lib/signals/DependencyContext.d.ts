import { FlagDispatcher, Subscribable } from '../events';
import { Promisable } from '../threading';
export interface PromiseHandle<T> {
    promise: Promise<T>;
    value: T;
    stack?: string;
    owner?: any;
}
export declare class DependencyContext<TOwner = void> implements Promisable<DependencyContext<TOwner>> {
    protected owner: TOwner;
    protected static collectionSet: Set<DependencyContext<any>>;
    protected static collectionStack: DependencyContext<any>[];
    protected static promises: PromiseHandle<any>[];
    static collectPromise<T>(promise: Promise<T>): PromiseHandle<T | null>;
    static collectPromise<T>(promise: Promise<T>, initialValue: T): PromiseHandle<T>;
    static hasPromises(): boolean;
    static consumePromises(): Promise<PromiseHandle<any>[]>;
    protected readonly invokable: any;
    protected dependencies: Set<Subscribable<void, import("../events").EventHandler<void>>>;
    protected event: FlagDispatcher;
    protected markDirty: () => void;
    constructor(owner: TOwner);
    protected invoke(): void;
    protected startCollecting(): void;
    protected finishCollecting(): void;
    protected clearDependencies(): void;
    protected collect(): void;
    dispose(): void;
    toPromise(): Promise<this>;
}
//# sourceMappingURL=DependencyContext.d.ts.map