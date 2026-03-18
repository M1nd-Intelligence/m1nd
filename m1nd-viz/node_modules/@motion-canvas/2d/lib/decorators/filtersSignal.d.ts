import { Signal, SignalContext, SignalValue, SimpleSignal, ThreadGenerator, TimingFunction } from '@motion-canvas/core';
import { Filter, FilterName } from '../partials';
export type FiltersSignal<TOwner> = Signal<Filter[], Filter[], TOwner, FiltersSignalContext<TOwner>> & {
    [K in FilterName]: SimpleSignal<number, TOwner>;
};
export declare class FiltersSignalContext<TOwner> extends SignalContext<Filter[], Filter[], TOwner> {
    constructor(initial: Filter[], owner: TOwner);
    tweener(value: SignalValue<Filter[]>, duration: number, timingFunction: TimingFunction): ThreadGenerator;
}
export declare function filtersSignal(): PropertyDecorator;
//# sourceMappingURL=filtersSignal.d.ts.map