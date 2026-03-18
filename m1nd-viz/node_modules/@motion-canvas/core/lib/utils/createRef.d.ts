export interface ReferenceReceiver<T> {
    (reference: T): void;
}
export interface Reference<T> extends ReferenceReceiver<T> {
    (): T;
}
export declare function createRef<T>(): Reference<T>;
export declare function makeRef<TObject, TKey extends keyof TObject>(object: TObject, key: TKey): ReferenceReceiver<TObject[TKey]>;
export type RefsProperty<TValue> = TValue extends (config: {
    refs?: infer TReference;
}) => void ? TReference : never;
export declare function makeRefs<T extends (config: {
    refs?: any;
}) => void>(): RefsProperty<T>;
//# sourceMappingURL=createRef.d.ts.map