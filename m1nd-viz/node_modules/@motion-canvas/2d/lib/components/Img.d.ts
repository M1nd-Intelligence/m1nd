import { Color, PossibleVector2, SerializedVector2, SignalValue, SimpleSignal, Vector2 } from '@motion-canvas/core';
import { DesiredLength } from '../partials';
import { Rect, RectProps } from './Rect';
export interface ImgProps extends RectProps {
    /**
     * {@inheritDoc Img.src}
     */
    src?: SignalValue<string | null>;
    /**
     * {@inheritDoc Img.alpha}
     */
    alpha?: SignalValue<number>;
    /**
     * {@inheritDoc Img.smoothing}
     */
    smoothing?: SignalValue<boolean>;
}
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
export declare class Img extends Rect {
    private static pool;
    /**
     * The source of this image.
     *
     * @example
     * Using a local image:
     * ```tsx
     * import image from './example.png';
     * // ...
     * view.add(<Img src={image} />)
     * ```
     * Loading an image from the internet:
     * ```tsx
     * view.add(<Img src="https://example.com/image.png" />)
     * ```
     */
    readonly src: SimpleSignal<string, this>;
    /**
     * The alpha value of this image.
     *
     * @remarks
     * Unlike opacity, the alpha value affects only the image itself, leaving the
     * fill, stroke, and children intact.
     */
    readonly alpha: SimpleSignal<number, this>;
    /**
     * Whether the image should be smoothed.
     *
     * @remarks
     * When disabled, the image will be scaled using the nearest neighbor
     * interpolation with no smoothing. The resulting image will appear pixelated.
     *
     * @defaultValue true
     */
    readonly smoothing: SimpleSignal<boolean, this>;
    constructor(props: ImgProps);
    protected desiredSize(): SerializedVector2<DesiredLength>;
    protected image(): HTMLImageElement;
    protected imageCanvas(): CanvasRenderingContext2D;
    protected filledImageCanvas(): CanvasRenderingContext2D;
    protected draw(context: CanvasRenderingContext2D): void;
    protected applyFlex(): void;
    /**
     * Get color of the image at the given position.
     *
     * @param position - The position in local space at which to sample the color.
     */
    getColorAtPoint(position: PossibleVector2): Color;
    /**
     * The natural size of this image.
     *
     * @remarks
     * The natural size is the size of the source image unaffected by the size
     * and scale properties.
     */
    naturalSize(): Vector2;
    /**
     * Get color of the image at the given pixel.
     *
     * @param position - The pixel's position.
     */
    getPixelColor(position: PossibleVector2): Color;
    protected collectAsyncResources(): void;
}
//# sourceMappingURL=Img.d.ts.map