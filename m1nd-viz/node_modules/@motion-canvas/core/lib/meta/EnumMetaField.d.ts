import { MetaField } from './MetaField';
import { MetaOption } from './MetaOption';
/**
 * Represents an enum value stored in a meta file.
 */
export declare class EnumMetaField<T> extends MetaField<T> {
    readonly options: MetaOption<T>[];
    static readonly symbol: unique symbol;
    readonly type: symbol;
    constructor(name: string, options: MetaOption<T>[], initial?: T);
    set(value: T): void;
    parse(value: T): T;
    getOption(value: T): MetaOption<T>;
}
//# sourceMappingURL=EnumMetaField.d.ts.map