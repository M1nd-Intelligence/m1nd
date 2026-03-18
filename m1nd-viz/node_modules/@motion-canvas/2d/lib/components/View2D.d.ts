import { PlaybackState, SimpleSignal } from '@motion-canvas/core';
import type { Node } from './Node';
import { Rect, RectProps } from './Rect';
export interface View2DProps extends RectProps {
    assetHash: string;
}
export declare class View2D extends Rect {
    static shadowRoot: ShadowRoot;
    readonly playbackState: SimpleSignal<PlaybackState, this>;
    readonly globalTime: SimpleSignal<number, this>;
    readonly assetHash: SimpleSignal<string, this>;
    constructor(props: View2DProps);
    dispose(): void;
    render(context: CanvasRenderingContext2D): void;
    /**
     * Find a node by its key.
     *
     * @param key - The key of the node.
     */
    findKey<T extends Node = Node>(key: string): T | null;
    protected requestLayoutUpdate(): void;
    protected requestFontUpdate(): void;
    view(): View2D;
}
//# sourceMappingURL=View2D.d.ts.map