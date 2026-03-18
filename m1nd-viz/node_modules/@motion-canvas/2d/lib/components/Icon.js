var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
import { useLogger, } from '@motion-canvas/core';
import { colorSignal, computed, initial, signal } from '../decorators';
import { Img } from './Img';
/**
 * An Icon Component that provides easy access to over 150k icons.
 * See https://icones.js.org/collection/all for all available Icons.
 */
export class Icon extends Img {
    constructor(props) {
        super({
            ...props,
            src: null,
        });
    }
    /**
     * Create the URL that will be used as the Image source
     * @returns Address to Iconify API for the requested Icon.
     */
    svgUrl() {
        const iconPathSegment = this.icon().replace(':', '/');
        const encodedColorValue = encodeURIComponent(this.color().hex());
        // Iconify API is documented here: https://docs.iconify.design/api/svg.html#color
        return `https://api.iconify.design/${iconPathSegment}.svg?color=${encodedColorValue}`;
    }
    /**
     * overrides `Image.src` getter
     */
    getSrc() {
        return this.svgUrl();
    }
    /**
     * overrides `Image.src` setter to warn the user that the value
     * is not used
     */
    setSrc(src) {
        if (src === null) {
            return;
        }
        useLogger().warn("The Icon Component does not accept setting the `src`. If you need access to `src`, use '<Img/>` instead.");
    }
}
__decorate([
    signal()
], Icon.prototype, "icon", void 0);
__decorate([
    initial('white'),
    colorSignal()
], Icon.prototype, "color", void 0);
__decorate([
    computed()
], Icon.prototype, "svgUrl", null);
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiSWNvbi5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29tcG9uZW50cy9JY29uLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7Ozs7OztBQUFBLE9BQU8sRUFLTCxTQUFTLEdBQ1YsTUFBTSxxQkFBcUIsQ0FBQztBQUM3QixPQUFPLEVBQUMsV0FBVyxFQUFFLFFBQVEsRUFBRSxPQUFPLEVBQUUsTUFBTSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBQ3JFLE9BQU8sRUFBQyxHQUFHLEVBQVcsTUFBTSxPQUFPLENBQUM7QUFjcEM7OztHQUdHO0FBQ0gsTUFBTSxPQUFPLElBQUssU0FBUSxHQUFHO0lBOEIzQixZQUFtQixLQUFnQjtRQUNqQyxLQUFLLENBQUM7WUFDSixHQUFHLEtBQUs7WUFDUixHQUFHLEVBQUUsSUFBSTtTQUNWLENBQUMsQ0FBQztJQUNMLENBQUM7SUFFRDs7O09BR0c7SUFFTyxNQUFNO1FBQ2QsTUFBTSxlQUFlLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDLE9BQU8sQ0FBQyxHQUFHLEVBQUUsR0FBRyxDQUFDLENBQUM7UUFDdEQsTUFBTSxpQkFBaUIsR0FBRyxrQkFBa0IsQ0FBQyxJQUFJLENBQUMsS0FBSyxFQUFFLENBQUMsR0FBRyxFQUFFLENBQUMsQ0FBQztRQUNqRSxpRkFBaUY7UUFDakYsT0FBTyw4QkFBOEIsZUFBZSxjQUFjLGlCQUFpQixFQUFFLENBQUM7SUFDeEYsQ0FBQztJQUVEOztPQUVHO0lBQ08sTUFBTTtRQUNkLE9BQU8sSUFBSSxDQUFDLE1BQU0sRUFBRSxDQUFDO0lBQ3ZCLENBQUM7SUFFRDs7O09BR0c7SUFDTyxNQUFNLENBQUMsR0FBa0I7UUFDakMsSUFBSSxHQUFHLEtBQUssSUFBSSxFQUFFO1lBQ2hCLE9BQU87U0FDUjtRQUNELFNBQVMsRUFBRSxDQUFDLElBQUksQ0FDZCwwR0FBMEcsQ0FDM0csQ0FBQztJQUNKLENBQUM7Q0FDRjtBQXhEZ0I7SUFEZCxNQUFNLEVBQUU7a0NBQ3VDO0FBZ0JqQztJQUZkLE9BQU8sQ0FBQyxPQUFPLENBQUM7SUFDaEIsV0FBVyxFQUFFO21DQUMwQjtBQWM5QjtJQURULFFBQVEsRUFBRTtrQ0FNViJ9