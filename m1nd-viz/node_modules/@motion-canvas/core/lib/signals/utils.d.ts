import { SignalValue } from './types';
export declare function isReactive<T>(value: SignalValue<T>): value is () => T;
export declare function modify<TFrom, TTo>(value: SignalValue<TFrom>, modification: (value: TFrom) => TTo): SignalValue<TTo>;
export declare function unwrap<T>(value: SignalValue<T>): T;
//# sourceMappingURL=utils.d.ts.map