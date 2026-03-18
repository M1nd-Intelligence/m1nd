import { capitalize } from '@motion-canvas/core';
export function makeSignalExtensions(meta = {}, owner, name) {
    const extensions = {};
    if (name && owner) {
        const setter = meta.setter ?? owner?.[`set${capitalize(name)}`];
        if (setter) {
            extensions.setter = setter.bind(owner);
        }
        const getter = meta.getter ?? owner?.[`get${capitalize(name)}`];
        if (getter) {
            extensions.getter = getter.bind(owner);
        }
        const tweener = meta.tweener ?? owner?.[`tween${capitalize(name)}`];
        if (tweener) {
            extensions.tweener = tweener.bind(owner);
        }
    }
    return extensions;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWFrZVNpZ25hbEV4dGVuc2lvbnMuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL3V0aWxzL21ha2VTaWduYWxFeHRlbnNpb25zLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBbUIsVUFBVSxFQUFDLE1BQU0scUJBQXFCLENBQUM7QUFHakUsTUFBTSxVQUFVLG9CQUFvQixDQUNsQyxPQUEwQyxFQUFFLEVBQzVDLEtBQVcsRUFDWCxJQUFhO0lBRWIsTUFBTSxVQUFVLEdBQW9ELEVBQUUsQ0FBQztJQUV2RSxJQUFJLElBQUksSUFBSSxLQUFLLEVBQUU7UUFDakIsTUFBTSxNQUFNLEdBQUcsSUFBSSxDQUFDLE1BQU0sSUFBSSxLQUFLLEVBQUUsQ0FBQyxNQUFNLFVBQVUsQ0FBQyxJQUFJLENBQUMsRUFBRSxDQUFDLENBQUM7UUFDaEUsSUFBSSxNQUFNLEVBQUU7WUFDVixVQUFVLENBQUMsTUFBTSxHQUFHLE1BQU0sQ0FBQyxJQUFJLENBQUMsS0FBSyxDQUFDLENBQUM7U0FDeEM7UUFFRCxNQUFNLE1BQU0sR0FBRyxJQUFJLENBQUMsTUFBTSxJQUFJLEtBQUssRUFBRSxDQUFDLE1BQU0sVUFBVSxDQUFDLElBQUksQ0FBQyxFQUFFLENBQUMsQ0FBQztRQUNoRSxJQUFJLE1BQU0sRUFBRTtZQUNWLFVBQVUsQ0FBQyxNQUFNLEdBQUcsTUFBTSxDQUFDLElBQUksQ0FBQyxLQUFLLENBQUMsQ0FBQztTQUN4QztRQUVELE1BQU0sT0FBTyxHQUFHLElBQUksQ0FBQyxPQUFPLElBQUksS0FBSyxFQUFFLENBQUMsUUFBUSxVQUFVLENBQUMsSUFBSSxDQUFDLEVBQUUsQ0FBQyxDQUFDO1FBQ3BFLElBQUksT0FBTyxFQUFFO1lBQ1gsVUFBVSxDQUFDLE9BQU8sR0FBRyxPQUFPLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDO1NBQzFDO0tBQ0Y7SUFFRCxPQUFPLFVBQVUsQ0FBQztBQUNwQixDQUFDIn0=