import { DependencyContext } from './DependencyContext';
export interface Computed<TValue> {
    (...args: any[]): TValue;
    context: ComputedContext<TValue>;
}
export declare class ComputedContext<TValue> extends DependencyContext<any> {
    private readonly factory;
    private last;
    constructor(factory: (...args: any[]) => TValue, owner?: any);
    toSignal(): Computed<TValue>;
    dispose(): void;
    protected invoke(...args: any[]): TValue;
}
//# sourceMappingURL=ComputedContext.d.ts.map