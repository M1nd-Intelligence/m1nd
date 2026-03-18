import type { MetaField } from './MetaField';
/**
 * Represents the meta file of a given entity.
 *
 * @remarks
 * This class is used exclusively by our Vite plugin as a bridge between
 * physical files and their runtime representation.
 *
 * @typeParam T - The type of the data stored in the meta file.
 *
 * @internal
 */
export declare class MetaFile<T> {
    private readonly name;
    private source;
    private readonly lock;
    private ignoreChange;
    private cache;
    private metaField;
    constructor(name: string, source?: string | false);
    attach(field: MetaField<T>): void;
    protected handleChanged: () => Promise<void>;
    private saveData;
    /**
     * Load new metadata from a file.
     *
     * @remarks
     * This method is called during hot module replacement.
     *
     * @param data - New metadata.
     */
    loadData(data: T): void;
    private static sourceLookup;
}
//# sourceMappingURL=MetaFile.d.ts.map