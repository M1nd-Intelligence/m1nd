/**
 * @internal
 */
export const NODE_NAME = Symbol.for('@motion-canvas/2d/nodeName');
/**
 * @internal
 */
export function nodeName(name) {
    return function (target) {
        target.prototype[NODE_NAME] = name;
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZU5hbWUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2RlY29yYXRvcnMvbm9kZU5hbWUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUE7O0dBRUc7QUFDSCxNQUFNLENBQUMsTUFBTSxTQUFTLEdBQUcsTUFBTSxDQUFDLEdBQUcsQ0FBQyw0QkFBNEIsQ0FBQyxDQUFDO0FBRWxFOztHQUVHO0FBQ0gsTUFBTSxVQUFVLFFBQVEsQ0FBQyxJQUFZO0lBQ25DLE9BQU8sVUFBVSxNQUFXO1FBQzFCLE1BQU0sQ0FBQyxTQUFTLENBQUMsU0FBUyxDQUFDLEdBQUcsSUFBSSxDQUFDO0lBQ3JDLENBQUMsQ0FBQztBQUNKLENBQUMifQ==