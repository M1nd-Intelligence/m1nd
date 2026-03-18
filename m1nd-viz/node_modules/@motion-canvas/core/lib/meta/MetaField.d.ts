import { ValueDispatcher } from '../events';
/**
 * Represents an entry in the meta file.
 *
 * @typeParam TSerializedValue - The type used to store this field in the meta
 *                               file.
 * @typeParam TValue - The runtime type of this field.
 */
export declare class MetaField<TSerializedValue, TValue extends TSerializedValue = TSerializedValue> {
    readonly name: string;
    readonly initial: TValue;
    /**
     * The type of this field used by the editor to display the correct input.
     */
    readonly type: any;
    spacing: boolean;
    description: string;
    /**
     * Triggered when the data of this field changes.
     *
     * @eventProperty
     */
    get onChanged(): import("../events").SubscribableValueEvent<TValue>;
    protected readonly value: ValueDispatcher<TValue>;
    /**
     * Triggered when the field becomes disabled or enabled.
     *
     * @eventProperty
     */
    get onDisabled(): import("../events").SubscribableValueEvent<boolean>;
    protected readonly disabled: ValueDispatcher<boolean>;
    /**
     * @param name - The name of this field displayed in the editor.
     * @param initial - The initial value of this field.
     */
    constructor(name: string, initial: TValue);
    /**
     * Get the current value.
     */
    get(): TValue;
    /**
     * Set the current value.
     *
     * @param value - The new value.
     */
    set(value: TSerializedValue): void;
    /**
     * Convert a serialized value into a runtime type.
     *
     * @param value - The serialized value.
     */
    parse(value: TSerializedValue): TValue;
    /**
     * Serialize the value of this field.
     */
    serialize(): TSerializedValue;
    /**
     * Create a clone of this field.
     */
    clone(): this;
    /**
     * Disable or enable the field in the editor.
     *
     * @param value - Whether the field should be disabled.
     */
    disable(value?: boolean): this;
    /**
     * Add or remove spacing at the beginning of this field.
     *
     * @param value - Whether to include the spacing.
     */
    space(value?: boolean): this;
    /**
     * Set the description of this field.
     *
     * @param description - The description.
     */
    describe(description: string): this;
}
//# sourceMappingURL=MetaField.d.ts.map