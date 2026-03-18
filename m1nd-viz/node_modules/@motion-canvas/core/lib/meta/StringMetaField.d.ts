import { MetaField } from './MetaField';
import { MetaOption } from './MetaOption';
/**
 * Represents a string stored in a meta file.
 */
export declare class StringMetaField<T extends string = string> extends MetaField<T> {
    readonly type: StringConstructor;
    protected presets: MetaOption<T>[];
    getPresets(): MetaOption<T>[];
    setPresets(options: MetaOption<T>[]): this;
}
//# sourceMappingURL=StringMetaField.d.ts.map