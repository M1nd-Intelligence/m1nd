import { Computed } from '../signals';
export declare function createComputedAsync<T>(factory: () => Promise<T>): Computed<T | null>;
export declare function createComputedAsync<T>(factory: () => Promise<T>, initial: T): Computed<T>;
//# sourceMappingURL=createComputedAsync.d.ts.map