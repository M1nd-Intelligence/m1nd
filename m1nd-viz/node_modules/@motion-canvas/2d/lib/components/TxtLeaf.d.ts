import { BBox, SignalValue, SimpleSignal } from '@motion-canvas/core';
import { Shape, ShapeProps } from './Shape';
import { Txt } from './Txt';
export interface TxtLeafProps extends ShapeProps {
    children?: string;
    text?: SignalValue<string>;
}
export declare class TxtLeaf extends Shape {
    protected static formatter: HTMLDivElement;
    protected static readonly segmenter: any;
    readonly text: SimpleSignal<string, this>;
    constructor({ children, ...rest }: TxtLeafProps);
    protected parentTxt(): Txt | null;
    protected draw(context: CanvasRenderingContext2D): void;
    protected drawText(context: CanvasRenderingContext2D, text: string, box: BBox): void;
    protected getCacheBBox(): BBox;
    protected applyFlex(): void;
    protected updateLayout(): void;
}
//# sourceMappingURL=TxtLeaf.d.ts.map