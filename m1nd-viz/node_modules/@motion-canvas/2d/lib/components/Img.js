var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var Img_1;
import { BBox, Color, DependencyContext, DetailedError, Vector2, useLogger, viaProxy, } from '@motion-canvas/core';
import { computed, initial, nodeName, signal } from '../decorators';
import { drawImage } from '../utils';
import { Rect } from './Rect';
/**
 * A node for displaying images.
 *
 * @preview
 * ```tsx editor
 * import {Img} from '@motion-canvas/2d';
 * import {all, waitFor} from '@motion-canvas/core';
 * import {createRef} from '@motion-canvas/core';
 * import {makeScene2D} from '@motion-canvas/2d';
 *
 * export default makeScene2D(function* (view) {
 *   const ref = createRef<Img>();
 *   yield view.add(
 *     <Img
 *       ref={ref}
 *       src="https://images.unsplash.com/photo-1679218407381-a6f1660d60e9"
 *       width={300}
 *       radius={20}
 *     />,
 *   );
 *
 *   // set the background using the color sampled from the image:
 *   ref().fill(ref().getColorAtPoint(0));
 *
 *   yield* all(
 *     ref().size([100, 100], 1).to([300, null], 1),
 *     ref().radius(50, 1).to(20, 1),
 *     ref().alpha(0, 1).to(1, 1),
 *   );
 *   yield* waitFor(0.5);
 * });
 * ```
 */
let Img = Img_1 = class Img extends Rect {
    constructor(props) {
        super(props);
        if (!('src' in props)) {
            useLogger().warn({
                message: 'No source specified for the image',
                remarks: "<p>The image won&#39;t be visible unless you specify a source:</p>\n<pre class=\"\"><code class=\"language-tsx\"><span class=\"hljs-keyword\">import</span> myImage <span class=\"hljs-keyword\">from</span> <span class=\"hljs-string\">&#x27;./example.png&#x27;</span>;\n<span class=\"hljs-comment\">// ...</span>\n<span class=\"language-xml\"><span class=\"hljs-tag\">&lt;<span class=\"hljs-name\">Img</span> <span class=\"hljs-attr\">src</span>=<span class=\"hljs-string\">{myImage}</span> /&gt;</span></span>;</code></pre><p>If you did this intentionally, and don&#39;t want to see this warning, set the <code>src</code>\nproperty to <code>null</code>:</p>\n<pre class=\"\"><code class=\"language-tsx\">&lt;<span class=\"hljs-title class_\">Img</span> src={<span class=\"hljs-literal\">null</span>} /&gt;</code></pre><p><a href='https://motioncanvas.io/docs/media#images' target='_blank'>Learn more</a> about working with\nimages.</p>\n",
                inspect: this.key,
            });
        }
    }
    desiredSize() {
        const custom = super.desiredSize();
        if (custom.x === null && custom.y === null) {
            const image = this.image();
            return {
                x: image.naturalWidth,
                y: image.naturalHeight,
            };
        }
        return custom;
    }
    image() {
        const rawSrc = this.src();
        let src = '';
        let key = '';
        if (rawSrc) {
            key = viaProxy(rawSrc);
            const url = new URL(key, window.location.origin);
            if (url.origin === window.location.origin) {
                const hash = this.view().assetHash();
                url.searchParams.set('asset-hash', hash);
            }
            src = url.toString();
        }
        let image = Img_1.pool[key];
        if (!image) {
            image = document.createElement('img');
            image.crossOrigin = 'anonymous';
            image.src = src;
            Img_1.pool[key] = image;
        }
        if (!image.complete) {
            DependencyContext.collectPromise(new Promise((resolve, reject) => {
                image.addEventListener('load', resolve);
                image.addEventListener('error', () => reject(new DetailedError({
                    message: `Failed to load an image`,
                    remarks: `\
The <code>src</code> property was set to:
<pre><code>${rawSrc}</code></pre>
...which resolved to the following url:
<pre><code>${src}</code></pre>
Make sure that source is correct and that the image exists.<br/>
<a target='_blank' href='https://motioncanvas.io/docs/media#images'>Learn more</a>
about working with images.`,
                    inspect: this.key,
                })));
            }));
        }
        return image;
    }
    imageCanvas() {
        const canvas = document
            .createElement('canvas')
            .getContext('2d', { willReadFrequently: true });
        if (!canvas) {
            throw new Error('Could not create an image canvas');
        }
        return canvas;
    }
    filledImageCanvas() {
        const context = this.imageCanvas();
        const image = this.image();
        context.canvas.width = image.naturalWidth;
        context.canvas.height = image.naturalHeight;
        context.imageSmoothingEnabled = this.smoothing();
        context.drawImage(image, 0, 0);
        return context;
    }
    draw(context) {
        this.drawShape(context);
        const alpha = this.alpha();
        if (alpha > 0) {
            const box = BBox.fromSizeCentered(this.computedSize());
            context.save();
            context.clip(this.getPath());
            if (alpha < 1) {
                context.globalAlpha *= alpha;
            }
            context.imageSmoothingEnabled = this.smoothing();
            drawImage(context, this.image(), box);
            context.restore();
        }
        if (this.clip()) {
            context.clip(this.getPath());
        }
        this.drawChildren(context);
    }
    applyFlex() {
        super.applyFlex();
        const image = this.image();
        this.element.style.aspectRatio = (this.ratio() ?? image.naturalWidth / image.naturalHeight).toString();
    }
    /**
     * Get color of the image at the given position.
     *
     * @param position - The position in local space at which to sample the color.
     */
    getColorAtPoint(position) {
        const size = this.computedSize();
        const naturalSize = this.naturalSize();
        const pixelPosition = new Vector2(position)
            .add(this.computedSize().scale(0.5))
            .mul(naturalSize.div(size).safe);
        return this.getPixelColor(pixelPosition);
    }
    /**
     * The natural size of this image.
     *
     * @remarks
     * The natural size is the size of the source image unaffected by the size
     * and scale properties.
     */
    naturalSize() {
        const image = this.image();
        return new Vector2(image.naturalWidth, image.naturalHeight);
    }
    /**
     * Get color of the image at the given pixel.
     *
     * @param position - The pixel's position.
     */
    getPixelColor(position) {
        const context = this.filledImageCanvas();
        const vector = new Vector2(position);
        const data = context.getImageData(vector.x, vector.y, 1, 1).data;
        return new Color({
            r: data[0],
            g: data[1],
            b: data[2],
            a: data[3] / 255,
        });
    }
    collectAsyncResources() {
        super.collectAsyncResources();
        this.image();
    }
};
Img.pool = {};
(() => {
    if (import.meta.hot) {
        import.meta.hot.on('motion-canvas:assets', ({ urls }) => {
            for (const url of urls) {
                if (Img_1.pool[url]) {
                    delete Img_1.pool[url];
                }
            }
        });
    }
})();
__decorate([
    signal()
], Img.prototype, "src", void 0);
__decorate([
    initial(1),
    signal()
], Img.prototype, "alpha", void 0);
__decorate([
    initial(true),
    signal()
], Img.prototype, "smoothing", void 0);
__decorate([
    computed()
], Img.prototype, "image", null);
__decorate([
    computed()
], Img.prototype, "imageCanvas", null);
__decorate([
    computed()
], Img.prototype, "filledImageCanvas", null);
__decorate([
    computed()
], Img.prototype, "naturalSize", null);
Img = Img_1 = __decorate([
    nodeName('Img')
], Img);
export { Img };
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiSW1nLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL0ltZy50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQUEsT0FBTyxFQUNMLElBQUksRUFDSixLQUFLLEVBQ0wsaUJBQWlCLEVBQ2pCLGFBQWEsRUFLYixPQUFPLEVBQ1AsU0FBUyxFQUNULFFBQVEsR0FDVCxNQUFNLHFCQUFxQixDQUFDO0FBQzdCLE9BQU8sRUFBQyxRQUFRLEVBQUUsT0FBTyxFQUFFLFFBQVEsRUFBRSxNQUFNLEVBQUMsTUFBTSxlQUFlLENBQUM7QUFFbEUsT0FBTyxFQUFDLFNBQVMsRUFBQyxNQUFNLFVBQVUsQ0FBQztBQUNuQyxPQUFPLEVBQUMsSUFBSSxFQUFZLE1BQU0sUUFBUSxDQUFDO0FBa0J2Qzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7R0FnQ0c7QUFFSSxJQUFNLEdBQUcsV0FBVCxNQUFNLEdBQUksU0FBUSxJQUFJO0lBeUQzQixZQUFtQixLQUFlO1FBQ2hDLEtBQUssQ0FBQyxLQUFLLENBQUMsQ0FBQztRQUNiLElBQUksQ0FBQyxDQUFDLEtBQUssSUFBSSxLQUFLLENBQUMsRUFBRTtZQUNyQixTQUFTLEVBQUUsQ0FBQyxJQUFJLENBQUM7Z0JBQ2YsT0FBTyxFQUFFLG1DQUFtQztnQkFDNUMsT0FBTyw0NkJBQW9CO2dCQUMzQixPQUFPLEVBQUUsSUFBSSxDQUFDLEdBQUc7YUFDbEIsQ0FBQyxDQUFDO1NBQ0o7SUFDSCxDQUFDO0lBRWtCLFdBQVc7UUFDNUIsTUFBTSxNQUFNLEdBQUcsS0FBSyxDQUFDLFdBQVcsRUFBRSxDQUFDO1FBQ25DLElBQUksTUFBTSxDQUFDLENBQUMsS0FBSyxJQUFJLElBQUksTUFBTSxDQUFDLENBQUMsS0FBSyxJQUFJLEVBQUU7WUFDMUMsTUFBTSxLQUFLLEdBQUcsSUFBSSxDQUFDLEtBQUssRUFBRSxDQUFDO1lBQzNCLE9BQU87Z0JBQ0wsQ0FBQyxFQUFFLEtBQUssQ0FBQyxZQUFZO2dCQUNyQixDQUFDLEVBQUUsS0FBSyxDQUFDLGFBQWE7YUFDdkIsQ0FBQztTQUNIO1FBRUQsT0FBTyxNQUFNLENBQUM7SUFDaEIsQ0FBQztJQUdTLEtBQUs7UUFDYixNQUFNLE1BQU0sR0FBRyxJQUFJLENBQUMsR0FBRyxFQUFFLENBQUM7UUFDMUIsSUFBSSxHQUFHLEdBQUcsRUFBRSxDQUFDO1FBQ2IsSUFBSSxHQUFHLEdBQUcsRUFBRSxDQUFDO1FBQ2IsSUFBSSxNQUFNLEVBQUU7WUFDVixHQUFHLEdBQUcsUUFBUSxDQUFDLE1BQU0sQ0FBQyxDQUFDO1lBQ3ZCLE1BQU0sR0FBRyxHQUFHLElBQUksR0FBRyxDQUFDLEdBQUcsRUFBRSxNQUFNLENBQUMsUUFBUSxDQUFDLE1BQU0sQ0FBQyxDQUFDO1lBQ2pELElBQUksR0FBRyxDQUFDLE1BQU0sS0FBSyxNQUFNLENBQUMsUUFBUSxDQUFDLE1BQU0sRUFBRTtnQkFDekMsTUFBTSxJQUFJLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDLFNBQVMsRUFBRSxDQUFDO2dCQUNyQyxHQUFHLENBQUMsWUFBWSxDQUFDLEdBQUcsQ0FBQyxZQUFZLEVBQUUsSUFBSSxDQUFDLENBQUM7YUFDMUM7WUFDRCxHQUFHLEdBQUcsR0FBRyxDQUFDLFFBQVEsRUFBRSxDQUFDO1NBQ3RCO1FBRUQsSUFBSSxLQUFLLEdBQUcsS0FBRyxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsQ0FBQztRQUMxQixJQUFJLENBQUMsS0FBSyxFQUFFO1lBQ1YsS0FBSyxHQUFHLFFBQVEsQ0FBQyxhQUFhLENBQUMsS0FBSyxDQUFDLENBQUM7WUFDdEMsS0FBSyxDQUFDLFdBQVcsR0FBRyxXQUFXLENBQUM7WUFDaEMsS0FBSyxDQUFDLEdBQUcsR0FBRyxHQUFHLENBQUM7WUFDaEIsS0FBRyxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsR0FBRyxLQUFLLENBQUM7U0FDdkI7UUFFRCxJQUFJLENBQUMsS0FBSyxDQUFDLFFBQVEsRUFBRTtZQUNuQixpQkFBaUIsQ0FBQyxjQUFjLENBQzlCLElBQUksT0FBTyxDQUFDLENBQUMsT0FBTyxFQUFFLE1BQU0sRUFBRSxFQUFFO2dCQUM5QixLQUFLLENBQUMsZ0JBQWdCLENBQUMsTUFBTSxFQUFFLE9BQU8sQ0FBQyxDQUFDO2dCQUN4QyxLQUFLLENBQUMsZ0JBQWdCLENBQUMsT0FBTyxFQUFFLEdBQUcsRUFBRSxDQUNuQyxNQUFNLENBQ0osSUFBSSxhQUFhLENBQUM7b0JBQ2hCLE9BQU8sRUFBRSx5QkFBeUI7b0JBQ2xDLE9BQU8sRUFBRTs7YUFFWixNQUFNOzthQUVOLEdBQUc7OzsyQkFHVztvQkFDWCxPQUFPLEVBQUUsSUFBSSxDQUFDLEdBQUc7aUJBQ2xCLENBQUMsQ0FDSCxDQUNGLENBQUM7WUFDSixDQUFDLENBQUMsQ0FDSCxDQUFDO1NBQ0g7UUFFRCxPQUFPLEtBQUssQ0FBQztJQUNmLENBQUM7SUFHUyxXQUFXO1FBQ25CLE1BQU0sTUFBTSxHQUFHLFFBQVE7YUFDcEIsYUFBYSxDQUFDLFFBQVEsQ0FBQzthQUN2QixVQUFVLENBQUMsSUFBSSxFQUFFLEVBQUMsa0JBQWtCLEVBQUUsSUFBSSxFQUFDLENBQUMsQ0FBQztRQUNoRCxJQUFJLENBQUMsTUFBTSxFQUFFO1lBQ1gsTUFBTSxJQUFJLEtBQUssQ0FBQyxrQ0FBa0MsQ0FBQyxDQUFDO1NBQ3JEO1FBRUQsT0FBTyxNQUFNLENBQUM7SUFDaEIsQ0FBQztJQUdTLGlCQUFpQjtRQUN6QixNQUFNLE9BQU8sR0FBRyxJQUFJLENBQUMsV0FBVyxFQUFFLENBQUM7UUFDbkMsTUFBTSxLQUFLLEdBQUcsSUFBSSxDQUFDLEtBQUssRUFBRSxDQUFDO1FBQzNCLE9BQU8sQ0FBQyxNQUFNLENBQUMsS0FBSyxHQUFHLEtBQUssQ0FBQyxZQUFZLENBQUM7UUFDMUMsT0FBTyxDQUFDLE1BQU0sQ0FBQyxNQUFNLEdBQUcsS0FBSyxDQUFDLGFBQWEsQ0FBQztRQUM1QyxPQUFPLENBQUMscUJBQXFCLEdBQUcsSUFBSSxDQUFDLFNBQVMsRUFBRSxDQUFDO1FBQ2pELE9BQU8sQ0FBQyxTQUFTLENBQUMsS0FBSyxFQUFFLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQztRQUUvQixPQUFPLE9BQU8sQ0FBQztJQUNqQixDQUFDO0lBRWtCLElBQUksQ0FBQyxPQUFpQztRQUN2RCxJQUFJLENBQUMsU0FBUyxDQUFDLE9BQU8sQ0FBQyxDQUFDO1FBQ3hCLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztRQUMzQixJQUFJLEtBQUssR0FBRyxDQUFDLEVBQUU7WUFDYixNQUFNLEdBQUcsR0FBRyxJQUFJLENBQUMsZ0JBQWdCLENBQUMsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDLENBQUM7WUFDdkQsT0FBTyxDQUFDLElBQUksRUFBRSxDQUFDO1lBQ2YsT0FBTyxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsT0FBTyxFQUFFLENBQUMsQ0FBQztZQUM3QixJQUFJLEtBQUssR0FBRyxDQUFDLEVBQUU7Z0JBQ2IsT0FBTyxDQUFDLFdBQVcsSUFBSSxLQUFLLENBQUM7YUFDOUI7WUFDRCxPQUFPLENBQUMscUJBQXFCLEdBQUcsSUFBSSxDQUFDLFNBQVMsRUFBRSxDQUFDO1lBQ2pELFNBQVMsQ0FBQyxPQUFPLEVBQUUsSUFBSSxDQUFDLEtBQUssRUFBRSxFQUFFLEdBQUcsQ0FBQyxDQUFDO1lBQ3RDLE9BQU8sQ0FBQyxPQUFPLEVBQUUsQ0FBQztTQUNuQjtRQUVELElBQUksSUFBSSxDQUFDLElBQUksRUFBRSxFQUFFO1lBQ2YsT0FBTyxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsT0FBTyxFQUFFLENBQUMsQ0FBQztTQUM5QjtRQUVELElBQUksQ0FBQyxZQUFZLENBQUMsT0FBTyxDQUFDLENBQUM7SUFDN0IsQ0FBQztJQUVrQixTQUFTO1FBQzFCLEtBQUssQ0FBQyxTQUFTLEVBQUUsQ0FBQztRQUNsQixNQUFNLEtBQUssR0FBRyxJQUFJLENBQUMsS0FBSyxFQUFFLENBQUM7UUFDM0IsSUFBSSxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUMsV0FBVyxHQUFHLENBQy9CLElBQUksQ0FBQyxLQUFLLEVBQUUsSUFBSSxLQUFLLENBQUMsWUFBWSxHQUFHLEtBQUssQ0FBQyxhQUFhLENBQ3pELENBQUMsUUFBUSxFQUFFLENBQUM7SUFDZixDQUFDO0lBRUQ7Ozs7T0FJRztJQUNJLGVBQWUsQ0FBQyxRQUF5QjtRQUM5QyxNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsWUFBWSxFQUFFLENBQUM7UUFDakMsTUFBTSxXQUFXLEdBQUcsSUFBSSxDQUFDLFdBQVcsRUFBRSxDQUFDO1FBRXZDLE1BQU0sYUFBYSxHQUFHLElBQUksT0FBTyxDQUFDLFFBQVEsQ0FBQzthQUN4QyxHQUFHLENBQUMsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsQ0FBQzthQUNuQyxHQUFHLENBQUMsV0FBVyxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsQ0FBQyxJQUFJLENBQUMsQ0FBQztRQUVuQyxPQUFPLElBQUksQ0FBQyxhQUFhLENBQUMsYUFBYSxDQUFDLENBQUM7SUFDM0MsQ0FBQztJQUVEOzs7Ozs7T0FNRztJQUVJLFdBQVc7UUFDaEIsTUFBTSxLQUFLLEdBQUcsSUFBSSxDQUFDLEtBQUssRUFBRSxDQUFDO1FBQzNCLE9BQU8sSUFBSSxPQUFPLENBQUMsS0FBSyxDQUFDLFlBQVksRUFBRSxLQUFLLENBQUMsYUFBYSxDQUFDLENBQUM7SUFDOUQsQ0FBQztJQUVEOzs7O09BSUc7SUFDSSxhQUFhLENBQUMsUUFBeUI7UUFDNUMsTUFBTSxPQUFPLEdBQUcsSUFBSSxDQUFDLGlCQUFpQixFQUFFLENBQUM7UUFDekMsTUFBTSxNQUFNLEdBQUcsSUFBSSxPQUFPLENBQUMsUUFBUSxDQUFDLENBQUM7UUFDckMsTUFBTSxJQUFJLEdBQUcsT0FBTyxDQUFDLFlBQVksQ0FBQyxNQUFNLENBQUMsQ0FBQyxFQUFFLE1BQU0sQ0FBQyxDQUFDLEVBQUUsQ0FBQyxFQUFFLENBQUMsQ0FBQyxDQUFDLElBQUksQ0FBQztRQUVqRSxPQUFPLElBQUksS0FBSyxDQUFDO1lBQ2YsQ0FBQyxFQUFFLElBQUksQ0FBQyxDQUFDLENBQUM7WUFDVixDQUFDLEVBQUUsSUFBSSxDQUFDLENBQUMsQ0FBQztZQUNWLENBQUMsRUFBRSxJQUFJLENBQUMsQ0FBQyxDQUFDO1lBQ1YsQ0FBQyxFQUFFLElBQUksQ0FBQyxDQUFDLENBQUMsR0FBRyxHQUFHO1NBQ2pCLENBQUMsQ0FBQztJQUNMLENBQUM7SUFFa0IscUJBQXFCO1FBQ3RDLEtBQUssQ0FBQyxxQkFBcUIsRUFBRSxDQUFDO1FBQzlCLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztJQUNmLENBQUM7O0FBMU9jLFFBQUksR0FBcUMsRUFBRSxBQUF2QyxDQUF3QztBQUUzRDtJQUNFLElBQUksTUFBTSxDQUFDLElBQUksQ0FBQyxHQUFHLEVBQUU7UUFDbkIsTUFBTSxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsRUFBRSxDQUFDLHNCQUFzQixFQUFFLENBQUMsRUFBQyxJQUFJLEVBQUMsRUFBRSxFQUFFO1lBQ3BELEtBQUssTUFBTSxHQUFHLElBQUksSUFBSSxFQUFFO2dCQUN0QixJQUFJLEtBQUcsQ0FBQyxJQUFJLENBQUMsR0FBRyxDQUFDLEVBQUU7b0JBQ2pCLE9BQU8sS0FBRyxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsQ0FBQztpQkFDdEI7YUFDRjtRQUNILENBQUMsQ0FBQyxDQUFDO0tBQ0o7QUFDSCxDQUFDLEdBQUEsQ0FBQTtBQWtCdUI7SUFEdkIsTUFBTSxFQUFFO2dDQUMrQztBQVdoQztJQUZ2QixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFO2tDQUNpRDtBQWFsQztJQUZ2QixPQUFPLENBQUMsSUFBSSxDQUFDO0lBQ2IsTUFBTSxFQUFFO3NDQUNzRDtBQTJCckQ7SUFEVCxRQUFRLEVBQUU7Z0NBZ0RWO0FBR1M7SUFEVCxRQUFRLEVBQUU7c0NBVVY7QUFHUztJQURULFFBQVEsRUFBRTs0Q0FVVjtBQXdETTtJQUROLFFBQVEsRUFBRTtzQ0FJVjtBQXBOVSxHQUFHO0lBRGYsUUFBUSxDQUFDLEtBQUssQ0FBQztHQUNILEdBQUcsQ0E0T2YifQ==