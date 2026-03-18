export function threadable(customName) {
    return function (_, propertyKey, descriptor) {
        descriptor.value.prototype.name = customName ?? propertyKey;
        descriptor.value.prototype.threadable = true;
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidGhyZWFkYWJsZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9kZWNvcmF0b3JzL3RocmVhZGFibGUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsTUFBTSxVQUFVLFVBQVUsQ0FBQyxVQUFtQjtJQUM1QyxPQUFPLFVBQ0wsQ0FBVSxFQUNWLFdBQTRCLEVBQzVCLFVBQThCO1FBRTlCLFVBQVUsQ0FBQyxLQUFLLENBQUMsU0FBUyxDQUFDLElBQUksR0FBRyxVQUFVLElBQUksV0FBVyxDQUFDO1FBQzVELFVBQVUsQ0FBQyxLQUFLLENBQUMsU0FBUyxDQUFDLFVBQVUsR0FBRyxJQUFJLENBQUM7SUFDL0MsQ0FBQyxDQUFDO0FBQ0osQ0FBQyJ9