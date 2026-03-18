import { ValueDispatcher } from '../events';
/**
 * Represents an entry in the meta file.
 *
 * @typeParam TSerializedValue - The type used to store this field in the meta
 *                               file.
 * @typeParam TValue - The runtime type of this field.
 */
export class MetaField {
    /**
     * Triggered when the data of this field changes.
     *
     * @eventProperty
     */
    get onChanged() {
        return this.value.subscribable;
    }
    /**
     * Triggered when the field becomes disabled or enabled.
     *
     * @eventProperty
     */
    get onDisabled() {
        return this.disabled.subscribable;
    }
    /**
     * @param name - The name of this field displayed in the editor.
     * @param initial - The initial value of this field.
     */
    constructor(name, initial) {
        this.name = name;
        this.initial = initial;
        /**
         * The type of this field used by the editor to display the correct input.
         */
        this.type = undefined;
        this.spacing = false;
        this.description = '';
        this.disabled = new ValueDispatcher(false);
        this.value = new ValueDispatcher(initial);
    }
    /**
     * Get the current value.
     */
    get() {
        return this.value.current;
    }
    /**
     * Set the current value.
     *
     * @param value - The new value.
     */
    set(value) {
        this.value.current = this.parse(value);
    }
    /**
     * Convert a serialized value into a runtime type.
     *
     * @param value - The serialized value.
     */
    parse(value) {
        return value;
    }
    /**
     * Serialize the value of this field.
     */
    serialize() {
        return this.value.current;
    }
    /**
     * Create a clone of this field.
     */
    clone() {
        return new this.constructor(this.name, this.get());
    }
    /**
     * Disable or enable the field in the editor.
     *
     * @param value - Whether the field should be disabled.
     */
    disable(value = true) {
        this.disabled.current = value;
        return this;
    }
    /**
     * Add or remove spacing at the beginning of this field.
     *
     * @param value - Whether to include the spacing.
     */
    space(value = true) {
        this.spacing = value;
        return this;
    }
    /**
     * Set the description of this field.
     *
     * @param description - The description.
     */
    describe(description) {
        this.description = description;
        return this;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiTWV0YUZpZWxkLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL21ldGEvTWV0YUZpZWxkLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBQyxlQUFlLEVBQUMsTUFBTSxXQUFXLENBQUM7QUFFMUM7Ozs7OztHQU1HO0FBQ0gsTUFBTSxPQUFPLFNBQVM7SUFXcEI7Ozs7T0FJRztJQUNILElBQVcsU0FBUztRQUNsQixPQUFPLElBQUksQ0FBQyxLQUFLLENBQUMsWUFBWSxDQUFDO0lBQ2pDLENBQUM7SUFJRDs7OztPQUlHO0lBQ0gsSUFBVyxVQUFVO1FBQ25CLE9BQU8sSUFBSSxDQUFDLFFBQVEsQ0FBQyxZQUFZLENBQUM7SUFDcEMsQ0FBQztJQUlEOzs7T0FHRztJQUNILFlBQ2tCLElBQVksRUFDWixPQUFlO1FBRGYsU0FBSSxHQUFKLElBQUksQ0FBUTtRQUNaLFlBQU8sR0FBUCxPQUFPLENBQVE7UUFuQ2pDOztXQUVHO1FBQ2EsU0FBSSxHQUFRLFNBQVMsQ0FBQztRQUMvQixZQUFPLEdBQUcsS0FBSyxDQUFDO1FBQ2hCLGdCQUFXLEdBQUcsRUFBRSxDQUFDO1FBc0JMLGFBQVEsR0FBRyxJQUFJLGVBQWUsQ0FBQyxLQUFLLENBQUMsQ0FBQztRQVV2RCxJQUFJLENBQUMsS0FBSyxHQUFHLElBQUksZUFBZSxDQUFDLE9BQU8sQ0FBQyxDQUFDO0lBQzVDLENBQUM7SUFFRDs7T0FFRztJQUNJLEdBQUc7UUFDUixPQUFPLElBQUksQ0FBQyxLQUFLLENBQUMsT0FBTyxDQUFDO0lBQzVCLENBQUM7SUFFRDs7OztPQUlHO0lBQ0ksR0FBRyxDQUFDLEtBQXVCO1FBQ2hDLElBQUksQ0FBQyxLQUFLLENBQUMsT0FBTyxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsS0FBSyxDQUFDLENBQUM7SUFDekMsQ0FBQztJQUVEOzs7O09BSUc7SUFDSSxLQUFLLENBQUMsS0FBdUI7UUFDbEMsT0FBTyxLQUFlLENBQUM7SUFDekIsQ0FBQztJQUVEOztPQUVHO0lBQ0ksU0FBUztRQUNkLE9BQU8sSUFBSSxDQUFDLEtBQUssQ0FBQyxPQUFPLENBQUM7SUFDNUIsQ0FBQztJQUVEOztPQUVHO0lBQ0ksS0FBSztRQUNWLE9BQU8sSUFBVSxJQUFJLENBQUMsV0FBWSxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsSUFBSSxDQUFDLEdBQUcsRUFBRSxDQUFDLENBQUM7SUFDNUQsQ0FBQztJQUVEOzs7O09BSUc7SUFDSSxPQUFPLENBQUMsS0FBSyxHQUFHLElBQUk7UUFDekIsSUFBSSxDQUFDLFFBQVEsQ0FBQyxPQUFPLEdBQUcsS0FBSyxDQUFDO1FBQzlCLE9BQU8sSUFBSSxDQUFDO0lBQ2QsQ0FBQztJQUVEOzs7O09BSUc7SUFDSSxLQUFLLENBQUMsS0FBSyxHQUFHLElBQUk7UUFDdkIsSUFBSSxDQUFDLE9BQU8sR0FBRyxLQUFLLENBQUM7UUFDckIsT0FBTyxJQUFJLENBQUM7SUFDZCxDQUFDO0lBRUQ7Ozs7T0FJRztJQUNJLFFBQVEsQ0FBQyxXQUFtQjtRQUNqQyxJQUFJLENBQUMsV0FBVyxHQUFHLFdBQVcsQ0FBQztRQUMvQixPQUFPLElBQUksQ0FBQztJQUNkLENBQUM7Q0FDRiJ9