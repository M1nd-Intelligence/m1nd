var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var Video_1;
import { BBox, DependencyContext, PlaybackState, clamp, isReactive, useLogger, useThread, } from '@motion-canvas/core';
import { computed, initial, nodeName, signal } from '../decorators';
import { drawImage } from '../utils';
import { Rect } from './Rect';
let Video = Video_1 = class Video extends Rect {
    constructor({ play, ...props }) {
        super(props);
        this.lastTime = -1;
        if (play) {
            this.play();
        }
    }
    isPlaying() {
        return this.playing();
    }
    getCurrentTime() {
        return this.clampTime(this.time());
    }
    getDuration() {
        return this.video().duration;
    }
    desiredSize() {
        const custom = super.desiredSize();
        if (custom.x === null && custom.y === null) {
            const image = this.video();
            return {
                x: image.videoWidth,
                y: image.videoHeight,
            };
        }
        return custom;
    }
    completion() {
        return this.clampTime(this.time()) / this.video().duration;
    }
    video() {
        const src = this.src();
        const key = `${this.key}/${src}`;
        let video = Video_1.pool[key];
        if (!video) {
            video = document.createElement('video');
            video.src = src;
            Video_1.pool[key] = video;
        }
        if (video.readyState < 2) {
            DependencyContext.collectPromise(new Promise(resolve => {
                const listener = () => {
                    resolve();
                    video.removeEventListener('canplay', listener);
                };
                video.addEventListener('canplay', listener);
            }));
        }
        return video;
    }
    seekedVideo() {
        const video = this.video();
        const time = this.clampTime(this.time());
        video.playbackRate = this.playbackRate();
        if (!video.paused) {
            video.pause();
        }
        if (this.lastTime === time) {
            return video;
        }
        this.setCurrentTime(time);
        return video;
    }
    fastSeekedVideo() {
        const video = this.video();
        const time = this.clampTime(this.time());
        video.playbackRate = this.playbackRate();
        if (this.lastTime === time) {
            return video;
        }
        const playing = this.playing() && time < video.duration && video.playbackRate > 0;
        if (playing) {
            if (video.paused) {
                DependencyContext.collectPromise(video.play());
            }
        }
        else {
            if (!video.paused) {
                video.pause();
            }
        }
        if (Math.abs(video.currentTime - time) > 0.2) {
            this.setCurrentTime(time);
        }
        else if (!playing) {
            video.currentTime = time;
        }
        return video;
    }
    draw(context) {
        this.drawShape(context);
        const alpha = this.alpha();
        if (alpha > 0) {
            const playbackState = this.view().playbackState();
            const video = playbackState === PlaybackState.Playing ||
                playbackState === PlaybackState.Presenting
                ? this.fastSeekedVideo()
                : this.seekedVideo();
            const box = BBox.fromSizeCentered(this.computedSize());
            context.save();
            context.clip(this.getPath());
            if (alpha < 1) {
                context.globalAlpha *= alpha;
            }
            context.imageSmoothingEnabled = this.smoothing();
            drawImage(context, video, box);
            context.restore();
        }
        if (this.clip()) {
            context.clip(this.getPath());
        }
        this.drawChildren(context);
    }
    applyFlex() {
        super.applyFlex();
        const video = this.video();
        this.element.style.aspectRatio = (this.ratio() ?? video.videoWidth / video.videoHeight).toString();
    }
    setCurrentTime(value) {
        const video = this.video();
        if (video.readyState < 2)
            return;
        video.currentTime = value;
        this.lastTime = value;
        if (video.seeking) {
            DependencyContext.collectPromise(new Promise(resolve => {
                const listener = () => {
                    resolve();
                    video.removeEventListener('seeked', listener);
                };
                video.addEventListener('seeked', listener);
            }));
        }
    }
    setPlaybackRate(playbackRate) {
        let value;
        if (isReactive(playbackRate)) {
            value = playbackRate();
            useLogger().warn({
                message: 'Invalid value set as the playback rate',
                remarks: "<p>The <code>playbackRate</code> of a <code>Video</code> cannot be reactive.</p>\n<p>Make sure to use a concrete value and not a function:</p>\n<pre class=\"wrong\"><code class=\"language-ts\">video.<span class=\"hljs-title function_\">playbackRate</span>(<span class=\"hljs-function\">() =&gt;</span> <span class=\"hljs-number\">7</span>);</code></pre><pre class=\"correct\"><code class=\"language-ts\">video.<span class=\"hljs-title function_\">playbackRate</span>(<span class=\"hljs-number\">7</span>);</code></pre><p>If you&#39;re using a signal, extract its value before passing it to the property:</p>\n<pre class=\"wrong\"><code class=\"language-ts\">video.<span class=\"hljs-title function_\">playbackRate</span>(mySignal);</code></pre><pre class=\"correct\"><code class=\"language-ts\">video.<span class=\"hljs-title function_\">playbackRate</span>(<span class=\"hljs-title function_\">mySignal</span>());</code></pre>",
                inspect: this.key,
                stack: new Error().stack,
            });
        }
        else {
            value = playbackRate;
        }
        this.playbackRate.context.setter(value);
        if (this.playing()) {
            if (value === 0) {
                this.pause();
            }
            else {
                const time = useThread().time;
                const start = time();
                const offset = this.time();
                this.time(() => this.clampTime(offset + (time() - start) * value));
            }
        }
    }
    play() {
        const time = useThread().time;
        const start = time();
        const offset = this.time();
        const playbackRate = this.playbackRate();
        this.playing(true);
        this.time(() => this.clampTime(offset + (time() - start) * playbackRate));
    }
    pause() {
        this.playing(false);
        this.time.save();
        this.video().pause();
    }
    seek(time) {
        const playing = this.playing();
        this.time(this.clampTime(time));
        if (playing) {
            this.play();
        }
        else {
            this.pause();
        }
    }
    clampTime(time) {
        const duration = this.video().duration;
        if (this.loop()) {
            time %= duration;
        }
        return clamp(0, duration, time);
    }
    collectAsyncResources() {
        super.collectAsyncResources();
        this.seekedVideo();
    }
};
Video.pool = {};
__decorate([
    signal()
], Video.prototype, "src", void 0);
__decorate([
    initial(1),
    signal()
], Video.prototype, "alpha", void 0);
__decorate([
    initial(true),
    signal()
], Video.prototype, "smoothing", void 0);
__decorate([
    initial(false),
    signal()
], Video.prototype, "loop", void 0);
__decorate([
    initial(1),
    signal()
], Video.prototype, "playbackRate", void 0);
__decorate([
    initial(0),
    signal()
], Video.prototype, "time", void 0);
__decorate([
    initial(false),
    signal()
], Video.prototype, "playing", void 0);
__decorate([
    computed()
], Video.prototype, "completion", null);
__decorate([
    computed()
], Video.prototype, "video", null);
__decorate([
    computed()
], Video.prototype, "seekedVideo", null);
__decorate([
    computed()
], Video.prototype, "fastSeekedVideo", null);
Video = Video_1 = __decorate([
    nodeName('Video')
], Video);
export { Video };
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiVmlkZW8uanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL2NvbXBvbmVudHMvVmlkZW8udHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7Ozs7OztBQUFBLE9BQU8sRUFDTCxJQUFJLEVBQ0osaUJBQWlCLEVBQ2pCLGFBQWEsRUFJYixLQUFLLEVBQ0wsVUFBVSxFQUNWLFNBQVMsRUFDVCxTQUFTLEdBQ1YsTUFBTSxxQkFBcUIsQ0FBQztBQUM3QixPQUFPLEVBQUMsUUFBUSxFQUFFLE9BQU8sRUFBRSxRQUFRLEVBQUUsTUFBTSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBRWxFLE9BQU8sRUFBQyxTQUFTLEVBQUMsTUFBTSxVQUFVLENBQUM7QUFDbkMsT0FBTyxFQUFDLElBQUksRUFBWSxNQUFNLFFBQVEsQ0FBQztBQWdDaEMsSUFBTSxLQUFLLGFBQVgsTUFBTSxLQUFNLFNBQVEsSUFBSTtJQXVFN0IsWUFBbUIsRUFBQyxJQUFJLEVBQUUsR0FBRyxLQUFLLEVBQWE7UUFDN0MsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBSFAsYUFBUSxHQUFHLENBQUMsQ0FBQyxDQUFDO1FBSXBCLElBQUksSUFBSSxFQUFFO1lBQ1IsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1NBQ2I7SUFDSCxDQUFDO0lBRU0sU0FBUztRQUNkLE9BQU8sSUFBSSxDQUFDLE9BQU8sRUFBRSxDQUFDO0lBQ3hCLENBQUM7SUFFTSxjQUFjO1FBQ25CLE9BQU8sSUFBSSxDQUFDLFNBQVMsQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLENBQUMsQ0FBQztJQUNyQyxDQUFDO0lBRU0sV0FBVztRQUNoQixPQUFPLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQyxRQUFRLENBQUM7SUFDL0IsQ0FBQztJQUVrQixXQUFXO1FBQzVCLE1BQU0sTUFBTSxHQUFHLEtBQUssQ0FBQyxXQUFXLEVBQUUsQ0FBQztRQUNuQyxJQUFJLE1BQU0sQ0FBQyxDQUFDLEtBQUssSUFBSSxJQUFJLE1BQU0sQ0FBQyxDQUFDLEtBQUssSUFBSSxFQUFFO1lBQzFDLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztZQUMzQixPQUFPO2dCQUNMLENBQUMsRUFBRSxLQUFLLENBQUMsVUFBVTtnQkFDbkIsQ0FBQyxFQUFFLEtBQUssQ0FBQyxXQUFXO2FBQ3JCLENBQUM7U0FDSDtRQUVELE9BQU8sTUFBTSxDQUFDO0lBQ2hCLENBQUM7SUFHZSxVQUFVO1FBQ3hCLE9BQU8sSUFBSSxDQUFDLFNBQVMsQ0FBQyxJQUFJLENBQUMsSUFBSSxFQUFFLENBQUMsR0FBRyxJQUFJLENBQUMsS0FBSyxFQUFFLENBQUMsUUFBUSxDQUFDO0lBQzdELENBQUM7SUFHUyxLQUFLO1FBQ2IsTUFBTSxHQUFHLEdBQUcsSUFBSSxDQUFDLEdBQUcsRUFBRSxDQUFDO1FBQ3ZCLE1BQU0sR0FBRyxHQUFHLEdBQUcsSUFBSSxDQUFDLEdBQUcsSUFBSSxHQUFHLEVBQUUsQ0FBQztRQUNqQyxJQUFJLEtBQUssR0FBRyxPQUFLLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDO1FBQzVCLElBQUksQ0FBQyxLQUFLLEVBQUU7WUFDVixLQUFLLEdBQUcsUUFBUSxDQUFDLGFBQWEsQ0FBQyxPQUFPLENBQUMsQ0FBQztZQUN4QyxLQUFLLENBQUMsR0FBRyxHQUFHLEdBQUcsQ0FBQztZQUNoQixPQUFLLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxHQUFHLEtBQUssQ0FBQztTQUN6QjtRQUVELElBQUksS0FBSyxDQUFDLFVBQVUsR0FBRyxDQUFDLEVBQUU7WUFDeEIsaUJBQWlCLENBQUMsY0FBYyxDQUM5QixJQUFJLE9BQU8sQ0FBTyxPQUFPLENBQUMsRUFBRTtnQkFDMUIsTUFBTSxRQUFRLEdBQUcsR0FBRyxFQUFFO29CQUNwQixPQUFPLEVBQUUsQ0FBQztvQkFDVixLQUFLLENBQUMsbUJBQW1CLENBQUMsU0FBUyxFQUFFLFFBQVEsQ0FBQyxDQUFDO2dCQUNqRCxDQUFDLENBQUM7Z0JBQ0YsS0FBSyxDQUFDLGdCQUFnQixDQUFDLFNBQVMsRUFBRSxRQUFRLENBQUMsQ0FBQztZQUM5QyxDQUFDLENBQUMsQ0FDSCxDQUFDO1NBQ0g7UUFFRCxPQUFPLEtBQUssQ0FBQztJQUNmLENBQUM7SUFHUyxXQUFXO1FBQ25CLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztRQUMzQixNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsU0FBUyxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsQ0FBQyxDQUFDO1FBRXpDLEtBQUssQ0FBQyxZQUFZLEdBQUcsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDO1FBRXpDLElBQUksQ0FBQyxLQUFLLENBQUMsTUFBTSxFQUFFO1lBQ2pCLEtBQUssQ0FBQyxLQUFLLEVBQUUsQ0FBQztTQUNmO1FBRUQsSUFBSSxJQUFJLENBQUMsUUFBUSxLQUFLLElBQUksRUFBRTtZQUMxQixPQUFPLEtBQUssQ0FBQztTQUNkO1FBRUQsSUFBSSxDQUFDLGNBQWMsQ0FBQyxJQUFJLENBQUMsQ0FBQztRQUUxQixPQUFPLEtBQUssQ0FBQztJQUNmLENBQUM7SUFHUyxlQUFlO1FBQ3ZCLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztRQUMzQixNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsU0FBUyxDQUFDLElBQUksQ0FBQyxJQUFJLEVBQUUsQ0FBQyxDQUFDO1FBRXpDLEtBQUssQ0FBQyxZQUFZLEdBQUcsSUFBSSxDQUFDLFlBQVksRUFBRSxDQUFDO1FBRXpDLElBQUksSUFBSSxDQUFDLFFBQVEsS0FBSyxJQUFJLEVBQUU7WUFDMUIsT0FBTyxLQUFLLENBQUM7U0FDZDtRQUVELE1BQU0sT0FBTyxHQUNYLElBQUksQ0FBQyxPQUFPLEVBQUUsSUFBSSxJQUFJLEdBQUcsS0FBSyxDQUFDLFFBQVEsSUFBSSxLQUFLLENBQUMsWUFBWSxHQUFHLENBQUMsQ0FBQztRQUNwRSxJQUFJLE9BQU8sRUFBRTtZQUNYLElBQUksS0FBSyxDQUFDLE1BQU0sRUFBRTtnQkFDaEIsaUJBQWlCLENBQUMsY0FBYyxDQUFDLEtBQUssQ0FBQyxJQUFJLEVBQUUsQ0FBQyxDQUFDO2FBQ2hEO1NBQ0Y7YUFBTTtZQUNMLElBQUksQ0FBQyxLQUFLLENBQUMsTUFBTSxFQUFFO2dCQUNqQixLQUFLLENBQUMsS0FBSyxFQUFFLENBQUM7YUFDZjtTQUNGO1FBRUQsSUFBSSxJQUFJLENBQUMsR0FBRyxDQUFDLEtBQUssQ0FBQyxXQUFXLEdBQUcsSUFBSSxDQUFDLEdBQUcsR0FBRyxFQUFFO1lBQzVDLElBQUksQ0FBQyxjQUFjLENBQUMsSUFBSSxDQUFDLENBQUM7U0FDM0I7YUFBTSxJQUFJLENBQUMsT0FBTyxFQUFFO1lBQ25CLEtBQUssQ0FBQyxXQUFXLEdBQUcsSUFBSSxDQUFDO1NBQzFCO1FBRUQsT0FBTyxLQUFLLENBQUM7SUFDZixDQUFDO0lBRWtCLElBQUksQ0FBQyxPQUFpQztRQUN2RCxJQUFJLENBQUMsU0FBUyxDQUFDLE9BQU8sQ0FBQyxDQUFDO1FBQ3hCLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztRQUMzQixJQUFJLEtBQUssR0FBRyxDQUFDLEVBQUU7WUFDYixNQUFNLGFBQWEsR0FBRyxJQUFJLENBQUMsSUFBSSxFQUFFLENBQUMsYUFBYSxFQUFFLENBQUM7WUFDbEQsTUFBTSxLQUFLLEdBQ1QsYUFBYSxLQUFLLGFBQWEsQ0FBQyxPQUFPO2dCQUN2QyxhQUFhLEtBQUssYUFBYSxDQUFDLFVBQVU7Z0JBQ3hDLENBQUMsQ0FBQyxJQUFJLENBQUMsZUFBZSxFQUFFO2dCQUN4QixDQUFDLENBQUMsSUFBSSxDQUFDLFdBQVcsRUFBRSxDQUFDO1lBRXpCLE1BQU0sR0FBRyxHQUFHLElBQUksQ0FBQyxnQkFBZ0IsQ0FBQyxJQUFJLENBQUMsWUFBWSxFQUFFLENBQUMsQ0FBQztZQUN2RCxPQUFPLENBQUMsSUFBSSxFQUFFLENBQUM7WUFDZixPQUFPLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxDQUFDO1lBQzdCLElBQUksS0FBSyxHQUFHLENBQUMsRUFBRTtnQkFDYixPQUFPLENBQUMsV0FBVyxJQUFJLEtBQUssQ0FBQzthQUM5QjtZQUNELE9BQU8sQ0FBQyxxQkFBcUIsR0FBRyxJQUFJLENBQUMsU0FBUyxFQUFFLENBQUM7WUFDakQsU0FBUyxDQUFDLE9BQU8sRUFBRSxLQUFLLEVBQUUsR0FBRyxDQUFDLENBQUM7WUFDL0IsT0FBTyxDQUFDLE9BQU8sRUFBRSxDQUFDO1NBQ25CO1FBRUQsSUFBSSxJQUFJLENBQUMsSUFBSSxFQUFFLEVBQUU7WUFDZixPQUFPLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxDQUFDO1NBQzlCO1FBRUQsSUFBSSxDQUFDLFlBQVksQ0FBQyxPQUFPLENBQUMsQ0FBQztJQUM3QixDQUFDO0lBRWtCLFNBQVM7UUFDMUIsS0FBSyxDQUFDLFNBQVMsRUFBRSxDQUFDO1FBQ2xCLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztRQUMzQixJQUFJLENBQUMsT0FBTyxDQUFDLEtBQUssQ0FBQyxXQUFXLEdBQUcsQ0FDL0IsSUFBSSxDQUFDLEtBQUssRUFBRSxJQUFJLEtBQUssQ0FBQyxVQUFVLEdBQUcsS0FBSyxDQUFDLFdBQVcsQ0FDckQsQ0FBQyxRQUFRLEVBQUUsQ0FBQztJQUNmLENBQUM7SUFFUyxjQUFjLENBQUMsS0FBYTtRQUNwQyxNQUFNLEtBQUssR0FBRyxJQUFJLENBQUMsS0FBSyxFQUFFLENBQUM7UUFDM0IsSUFBSSxLQUFLLENBQUMsVUFBVSxHQUFHLENBQUM7WUFBRSxPQUFPO1FBRWpDLEtBQUssQ0FBQyxXQUFXLEdBQUcsS0FBSyxDQUFDO1FBQzFCLElBQUksQ0FBQyxRQUFRLEdBQUcsS0FBSyxDQUFDO1FBQ3RCLElBQUksS0FBSyxDQUFDLE9BQU8sRUFBRTtZQUNqQixpQkFBaUIsQ0FBQyxjQUFjLENBQzlCLElBQUksT0FBTyxDQUFPLE9BQU8sQ0FBQyxFQUFFO2dCQUMxQixNQUFNLFFBQVEsR0FBRyxHQUFHLEVBQUU7b0JBQ3BCLE9BQU8sRUFBRSxDQUFDO29CQUNWLEtBQUssQ0FBQyxtQkFBbUIsQ0FBQyxRQUFRLEVBQUUsUUFBUSxDQUFDLENBQUM7Z0JBQ2hELENBQUMsQ0FBQztnQkFDRixLQUFLLENBQUMsZ0JBQWdCLENBQUMsUUFBUSxFQUFFLFFBQVEsQ0FBQyxDQUFDO1lBQzdDLENBQUMsQ0FBQyxDQUNILENBQUM7U0FDSDtJQUNILENBQUM7SUFFUyxlQUFlLENBQUMsWUFBb0I7UUFDNUMsSUFBSSxLQUFhLENBQUM7UUFDbEIsSUFBSSxVQUFVLENBQUMsWUFBWSxDQUFDLEVBQUU7WUFDNUIsS0FBSyxHQUFHLFlBQVksRUFBRSxDQUFDO1lBQ3ZCLFNBQVMsRUFBRSxDQUFDLElBQUksQ0FBQztnQkFDZixPQUFPLEVBQUUsd0NBQXdDO2dCQUNqRCxPQUFPLG02QkFBc0I7Z0JBQzdCLE9BQU8sRUFBRSxJQUFJLENBQUMsR0FBRztnQkFDakIsS0FBSyxFQUFFLElBQUksS0FBSyxFQUFFLENBQUMsS0FBSzthQUN6QixDQUFDLENBQUM7U0FDSjthQUFNO1lBQ0wsS0FBSyxHQUFHLFlBQVksQ0FBQztTQUN0QjtRQUNELElBQUksQ0FBQyxZQUFZLENBQUMsT0FBTyxDQUFDLE1BQU0sQ0FBQyxLQUFLLENBQUMsQ0FBQztRQUV4QyxJQUFJLElBQUksQ0FBQyxPQUFPLEVBQUUsRUFBRTtZQUNsQixJQUFJLEtBQUssS0FBSyxDQUFDLEVBQUU7Z0JBQ2YsSUFBSSxDQUFDLEtBQUssRUFBRSxDQUFDO2FBQ2Q7aUJBQU07Z0JBQ0wsTUFBTSxJQUFJLEdBQUcsU0FBUyxFQUFFLENBQUMsSUFBSSxDQUFDO2dCQUM5QixNQUFNLEtBQUssR0FBRyxJQUFJLEVBQUUsQ0FBQztnQkFDckIsTUFBTSxNQUFNLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO2dCQUMzQixJQUFJLENBQUMsSUFBSSxDQUFDLEdBQUcsRUFBRSxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUMsTUFBTSxHQUFHLENBQUMsSUFBSSxFQUFFLEdBQUcsS0FBSyxDQUFDLEdBQUcsS0FBSyxDQUFDLENBQUMsQ0FBQzthQUNwRTtTQUNGO0lBQ0gsQ0FBQztJQUVNLElBQUk7UUFDVCxNQUFNLElBQUksR0FBRyxTQUFTLEVBQUUsQ0FBQyxJQUFJLENBQUM7UUFDOUIsTUFBTSxLQUFLLEdBQUcsSUFBSSxFQUFFLENBQUM7UUFDckIsTUFBTSxNQUFNLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1FBQzNCLE1BQU0sWUFBWSxHQUFHLElBQUksQ0FBQyxZQUFZLEVBQUUsQ0FBQztRQUN6QyxJQUFJLENBQUMsT0FBTyxDQUFDLElBQUksQ0FBQyxDQUFDO1FBQ25CLElBQUksQ0FBQyxJQUFJLENBQUMsR0FBRyxFQUFFLENBQUMsSUFBSSxDQUFDLFNBQVMsQ0FBQyxNQUFNLEdBQUcsQ0FBQyxJQUFJLEVBQUUsR0FBRyxLQUFLLENBQUMsR0FBRyxZQUFZLENBQUMsQ0FBQyxDQUFDO0lBQzVFLENBQUM7SUFFTSxLQUFLO1FBQ1YsSUFBSSxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUMsQ0FBQztRQUNwQixJQUFJLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1FBQ2pCLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQyxLQUFLLEVBQUUsQ0FBQztJQUN2QixDQUFDO0lBRU0sSUFBSSxDQUFDLElBQVk7UUFDdEIsTUFBTSxPQUFPLEdBQUcsSUFBSSxDQUFDLE9BQU8sRUFBRSxDQUFDO1FBQy9CLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLFNBQVMsQ0FBQyxJQUFJLENBQUMsQ0FBQyxDQUFDO1FBQ2hDLElBQUksT0FBTyxFQUFFO1lBQ1gsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDO1NBQ2I7YUFBTTtZQUNMLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQztTQUNkO0lBQ0gsQ0FBQztJQUVNLFNBQVMsQ0FBQyxJQUFZO1FBQzNCLE1BQU0sUUFBUSxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQUUsQ0FBQyxRQUFRLENBQUM7UUFDdkMsSUFBSSxJQUFJLENBQUMsSUFBSSxFQUFFLEVBQUU7WUFDZixJQUFJLElBQUksUUFBUSxDQUFDO1NBQ2xCO1FBQ0QsT0FBTyxLQUFLLENBQUMsQ0FBQyxFQUFFLFFBQVEsRUFBRSxJQUFJLENBQUMsQ0FBQztJQUNsQyxDQUFDO0lBRWtCLHFCQUFxQjtRQUN0QyxLQUFLLENBQUMscUJBQXFCLEVBQUUsQ0FBQztRQUM5QixJQUFJLENBQUMsV0FBVyxFQUFFLENBQUM7SUFDckIsQ0FBQzs7QUFoVHVCLFVBQUksR0FBcUMsRUFBRSxBQUF2QyxDQUF3QztBQWtCNUM7SUFEdkIsTUFBTSxFQUFFO2tDQUMrQztBQVdoQztJQUZ2QixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFO29DQUNpRDtBQWFsQztJQUZ2QixPQUFPLENBQUMsSUFBSSxDQUFDO0lBQ2IsTUFBTSxFQUFFO3dDQUNzRDtBQU92QztJQUZ2QixPQUFPLENBQUMsS0FBSyxDQUFDO0lBQ2QsTUFBTSxFQUFFO21DQUNpRDtBQVNsQztJQUZ2QixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFOzJDQUN3RDtBQUl0QztJQUYxQixPQUFPLENBQUMsQ0FBQyxDQUFDO0lBQ1YsTUFBTSxFQUFFO21DQUNtRDtBQUlqQztJQUYxQixPQUFPLENBQUMsS0FBSyxDQUFDO0lBQ2QsTUFBTSxFQUFFO3NDQUN1RDtBQXFDaEQ7SUFEZixRQUFRLEVBQUU7dUNBR1Y7QUFHUztJQURULFFBQVEsRUFBRTtrQ0F3QlY7QUFHUztJQURULFFBQVEsRUFBRTt3Q0FrQlY7QUFHUztJQURULFFBQVEsRUFBRTs0Q0E4QlY7QUF4TFUsS0FBSztJQURqQixRQUFRLENBQUMsT0FBTyxDQUFDO0dBQ0wsS0FBSyxDQWtUakIifQ==