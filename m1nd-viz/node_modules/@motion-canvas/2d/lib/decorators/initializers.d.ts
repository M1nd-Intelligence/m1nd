export type Initializer<T> = (instance: T, context?: any) => void;
export declare function addInitializer<T>(target: any, initializer: Initializer<T>): void;
export declare function initialize(target: any, context?: any): void;
//# sourceMappingURL=initializers.d.ts.map