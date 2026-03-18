import { SimpleSignal } from '@motion-canvas/core';
export type CanvasRepetition = null | 'repeat' | 'repeat-x' | 'repeat-y' | 'no-repeat';
export interface PatternProps {
    image: CanvasImageSource;
    repetition?: CanvasRepetition;
}
export declare class Pattern {
    readonly image: SimpleSignal<CanvasImageSource, this>;
    readonly repetition: SimpleSignal<CanvasRepetition, this>;
    constructor(props: PatternProps);
    canvasPattern(context: CanvasRenderingContext2D): CanvasPattern | null;
}
//# sourceMappingURL=Pattern.d.ts.map