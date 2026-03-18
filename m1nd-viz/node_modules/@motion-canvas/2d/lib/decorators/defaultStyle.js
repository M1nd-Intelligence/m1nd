import { capitalize } from '@motion-canvas/core';
export function defaultStyle(styleName, parse = value => value) {
    return (target, key) => {
        target[`getDefault${capitalize(key)}`] = function () {
            this.requestLayoutUpdate();
            const old = this.element.style[styleName];
            this.element.style[styleName] = '';
            const ret = parse.call(this, this.styles.getPropertyValue(styleName));
            this.element.style[styleName] = old;
            return ret;
        };
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZGVmYXVsdFN0eWxlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9kZWNvcmF0b3JzL2RlZmF1bHRTdHlsZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUMsVUFBVSxFQUFDLE1BQU0scUJBQXFCLENBQUM7QUFHL0MsTUFBTSxVQUFVLFlBQVksQ0FDMUIsU0FBaUIsRUFDakIsUUFBOEIsS0FBSyxDQUFDLEVBQUUsQ0FBQyxLQUFVO0lBRWpELE9BQU8sQ0FBQyxNQUFXLEVBQUUsR0FBRyxFQUFFLEVBQUU7UUFDMUIsTUFBTSxDQUFDLGFBQWEsVUFBVSxDQUFTLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRztZQUMvQyxJQUFJLENBQUMsbUJBQW1CLEVBQUUsQ0FBQztZQUMzQixNQUFNLEdBQUcsR0FBUyxJQUFJLENBQUMsT0FBTyxDQUFDLEtBQU0sQ0FBQyxTQUFTLENBQUMsQ0FBQztZQUMzQyxJQUFJLENBQUMsT0FBTyxDQUFDLEtBQU0sQ0FBQyxTQUFTLENBQUMsR0FBRyxFQUFFLENBQUM7WUFDMUMsTUFBTSxHQUFHLEdBQUcsS0FBSyxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsSUFBSSxDQUFDLE1BQU0sQ0FBQyxnQkFBZ0IsQ0FBQyxTQUFTLENBQUMsQ0FBQyxDQUFDO1lBQ2hFLElBQUksQ0FBQyxPQUFPLENBQUMsS0FBTSxDQUFDLFNBQVMsQ0FBQyxHQUFHLEdBQUcsQ0FBQztZQUMzQyxPQUFPLEdBQUcsQ0FBQztRQUNiLENBQUMsQ0FBQztJQUNKLENBQUMsQ0FBQztBQUNKLENBQUMifQ==