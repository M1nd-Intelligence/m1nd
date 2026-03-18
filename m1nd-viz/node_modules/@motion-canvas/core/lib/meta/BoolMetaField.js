import { MetaField } from './MetaField';
/**
 * Represents a boolean value stored in a meta file.
 */
export class BoolMetaField extends MetaField {
    constructor() {
        super(...arguments);
        this.type = Boolean;
    }
    parse(value) {
        return !!value;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQm9vbE1ldGFGaWVsZC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9tZXRhL0Jvb2xNZXRhRmllbGQudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLFNBQVMsRUFBQyxNQUFNLGFBQWEsQ0FBQztBQUV0Qzs7R0FFRztBQUNILE1BQU0sT0FBTyxhQUFjLFNBQVEsU0FBdUI7SUFBMUQ7O1FBQ2tCLFNBQUksR0FBRyxPQUFPLENBQUM7SUFLakMsQ0FBQztJQUhRLEtBQUssQ0FBQyxLQUFVO1FBQ3JCLE9BQU8sQ0FBQyxDQUFDLEtBQUssQ0FBQztJQUNqQixDQUFDO0NBQ0YifQ==