import { SerializedVector2, SignalValue, SimpleSignal } from '@motion-canvas/core';
import { DesiredLength } from '../partials';
import { Rect, RectProps } from './Rect';
export interface VideoProps extends RectProps {
    /**
     * {@inheritDoc Video.src}
     */
    src?: SignalValue<string>;
    /**
     * {@inheritDoc Video.alpha}
     */
    alpha?: SignalValue<number>;
    /**
     * {@inheritDoc Video.smoothing}
     */
    smoothing?: SignalValue<boolean>;
    /**
     * {@inheritDoc Video.loop}
     */
    loop?: SignalValue<boolean>;
    /**
     * {@inheritDoc Video.playbackRate}
     */
    playbackRate?: number;
    /**
     * The starting time for this video in seconds.
     */
    time?: SignalValue<number>;
    play?: boolean;
}
export declare class Video extends Rect {
    private static readonly pool;
    /**
     * The source of this video.
     *
     * @example
     * Using a local video:
     * ```tsx
     * import video from './example.mp4';
     * // ...
     * view.add(<Video src={video} />)
     * ```
     * Loading an image from the internet:
     * ```tsx
     * view.add(<Video src="https://example.com/video.mp4" />)
     * ```
     */
    readonly src: SimpleSignal<string, this>;
    /**
     * The alpha value of this video.
     *
     * @remarks
     * Unlike opacity, the alpha value affects only the video itself, leaving the
     * fill, stroke, and children intact.
     */
    readonly alpha: SimpleSignal<number, this>;
    /**
     * Whether the video should be smoothed.
     *
     * @remarks
     * When disabled, the video will be scaled using the nearest neighbor
     * interpolation with no smoothing. The resulting video will appear pixelated.
     *
     * @defaultValue true
     */
    readonly smoothing: SimpleSignal<boolean, this>;
    /**
     * Whether this video should loop upon reaching the end.
     */
    readonly loop: SimpleSignal<boolean, this>;
    /**
     * The rate at which the video plays, as multiples of the normal speed.
     *
     * @defaultValue 1
     */
    readonly playbackRate: SimpleSignal<number, this>;
    protected readonly time: SimpleSignal<number, this>;
    protected readonly playing: SimpleSignal<boolean, this>;
    private lastTime;
    constructor({ play, ...props }: VideoProps);
    isPlaying(): boolean;
    getCurrentTime(): number;
    getDuration(): number;
    protected desiredSize(): SerializedVector2<DesiredLength>;
    completion(): number;
    protected video(): HTMLVideoElement;
    protected seekedVideo(): HTMLVideoElement;
    protected fastSeekedVideo(): HTMLVideoElement;
    protected draw(context: CanvasRenderingContext2D): void;
    protected applyFlex(): void;
    protected setCurrentTime(value: number): void;
    protected setPlaybackRate(playbackRate: number): void;
    play(): void;
    pause(): void;
    seek(time: number): void;
    clampTime(time: number): number;
    protected collectAsyncResources(): void;
}
//# sourceMappingURL=Video.d.ts.map