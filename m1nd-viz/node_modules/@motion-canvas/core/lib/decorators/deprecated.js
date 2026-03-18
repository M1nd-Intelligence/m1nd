import { deprecate } from '../utils';
/**
 * Create a deprecated decorator that marks methods as deprecated.
 */
export function deprecated(remarks) {
    return function (target, propertyKey, descriptor) {
        const name = target.constructor.name
            ? `${target.constructor.name}.${propertyKey}`
            : propertyKey;
        const message = `${name}() has been deprecated.`;
        if (descriptor.value) {
            descriptor.value = deprecate(descriptor.value, message, remarks);
        }
        if (descriptor.get) {
            descriptor.get = deprecate(descriptor.get, message, remarks);
        }
        if (descriptor.set) {
            descriptor.set = deprecate(descriptor.set, message, remarks);
        }
        return descriptor;
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZGVwcmVjYXRlZC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9kZWNvcmF0b3JzL2RlcHJlY2F0ZWQudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLFNBQVMsRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUVuQzs7R0FFRztBQUNILE1BQU0sVUFBVSxVQUFVLENBQUMsT0FBZ0I7SUFDekMsT0FBTyxVQUNMLE1BQVcsRUFDWCxXQUE0QixFQUM1QixVQUE4QjtRQUU5QixNQUFNLElBQUksR0FBRyxNQUFNLENBQUMsV0FBVyxDQUFDLElBQUk7WUFDbEMsQ0FBQyxDQUFDLEdBQUcsTUFBTSxDQUFDLFdBQVcsQ0FBQyxJQUFJLElBQVksV0FBVyxFQUFFO1lBQ3JELENBQUMsQ0FBUyxXQUFXLENBQUM7UUFDeEIsTUFBTSxPQUFPLEdBQUcsR0FBRyxJQUFJLHlCQUF5QixDQUFDO1FBRWpELElBQUksVUFBVSxDQUFDLEtBQUssRUFBRTtZQUNwQixVQUFVLENBQUMsS0FBSyxHQUFHLFNBQVMsQ0FBQyxVQUFVLENBQUMsS0FBSyxFQUFFLE9BQU8sRUFBRSxPQUFPLENBQUMsQ0FBQztTQUNsRTtRQUNELElBQUksVUFBVSxDQUFDLEdBQUcsRUFBRTtZQUNsQixVQUFVLENBQUMsR0FBRyxHQUFHLFNBQVMsQ0FBQyxVQUFVLENBQUMsR0FBRyxFQUFFLE9BQU8sRUFBRSxPQUFPLENBQUMsQ0FBQztTQUM5RDtRQUNELElBQUksVUFBVSxDQUFDLEdBQUcsRUFBRTtZQUNsQixVQUFVLENBQUMsR0FBRyxHQUFHLFNBQVMsQ0FBQyxVQUFVLENBQUMsR0FBRyxFQUFFLE9BQU8sRUFBRSxPQUFPLENBQUMsQ0FBQztTQUM5RDtRQUVELE9BQU8sVUFBVSxDQUFDO0lBQ3BCLENBQUMsQ0FBQztBQUNKLENBQUMifQ==