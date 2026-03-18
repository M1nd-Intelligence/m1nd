export function getContext(options, canvas = document.createElement('canvas')) {
    const context = canvas.getContext('2d', options);
    if (!context) {
        throw new Error('Could not create a 2D context.');
    }
    return context;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZ2V0Q29udGV4dC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy91dGlscy9nZXRDb250ZXh0LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE1BQU0sVUFBVSxVQUFVLENBQ3hCLE9BQTBDLEVBQzFDLFNBQTRCLFFBQVEsQ0FBQyxhQUFhLENBQUMsUUFBUSxDQUFDO0lBRTVELE1BQU0sT0FBTyxHQUFHLE1BQU0sQ0FBQyxVQUFVLENBQUMsSUFBSSxFQUFFLE9BQU8sQ0FBQyxDQUFDO0lBQ2pELElBQUksQ0FBQyxPQUFPLEVBQUU7UUFDWixNQUFNLElBQUksS0FBSyxDQUFDLGdDQUFnQyxDQUFDLENBQUM7S0FDbkQ7SUFDRCxPQUFPLE9BQU8sQ0FBQztBQUNqQixDQUFDIn0=