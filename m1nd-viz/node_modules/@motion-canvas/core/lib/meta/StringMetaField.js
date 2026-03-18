import { MetaField } from './MetaField';
/**
 * Represents a string stored in a meta file.
 */
export class StringMetaField extends MetaField {
    constructor() {
        super(...arguments);
        this.type = String;
        this.presets = [];
    }
    getPresets() {
        return this.presets;
    }
    setPresets(options) {
        this.presets = options;
        return this;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiU3RyaW5nTWV0YUZpZWxkLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL21ldGEvU3RyaW5nTWV0YUZpZWxkLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBQyxTQUFTLEVBQUMsTUFBTSxhQUFhLENBQUM7QUFHdEM7O0dBRUc7QUFDSCxNQUFNLE9BQU8sZUFBMkMsU0FBUSxTQUFZO0lBQTVFOztRQUNrQixTQUFJLEdBQUcsTUFBTSxDQUFDO1FBQ3BCLFlBQU8sR0FBb0IsRUFBRSxDQUFDO0lBVTFDLENBQUM7SUFSUSxVQUFVO1FBQ2YsT0FBTyxJQUFJLENBQUMsT0FBTyxDQUFDO0lBQ3RCLENBQUM7SUFFTSxVQUFVLENBQUMsT0FBd0I7UUFDeEMsSUFBSSxDQUFDLE9BQU8sR0FBRyxPQUFPLENBQUM7UUFDdkIsT0FBTyxJQUFJLENBQUM7SUFDZCxDQUFDO0NBQ0YifQ==