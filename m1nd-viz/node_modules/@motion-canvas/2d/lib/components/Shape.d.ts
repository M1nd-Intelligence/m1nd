import { BBox, SignalValue, SimpleSignal } from '@motion-canvas/core';
import { CanvasStyleSignal } from '../decorators/canvasStyleSignal';
import { PossibleCanvasStyle } from '../partials';
import { Layout, LayoutProps } from './Layout';
export interface ShapeProps extends LayoutProps {
    fill?: SignalValue<PossibleCanvasStyle>;
    stroke?: SignalValue<PossibleCanvasStyle>;
    strokeFirst?: SignalValue<boolean>;
    lineWidth?: SignalValue<number>;
    lineJoin?: SignalValue<CanvasLineJoin>;
    lineCap?: SignalValue<CanvasLineCap>;
    lineDash?: SignalValue<number[]>;
    lineDashOffset?: SignalValue<number>;
    antialiased?: SignalValue<boolean>;
}
export declare abstract class Shape extends Layout {
    readonly fill: CanvasStyleSignal<this>;
    readonly stroke: CanvasStyleSignal<this>;
    readonly strokeFirst: SimpleSignal<boolean, this>;
    readonly lineWidth: SimpleSignal<number, this>;
    readonly lineJoin: SimpleSignal<CanvasLineJoin, this>;
    readonly lineCap: SimpleSignal<CanvasLineCap, this>;
    readonly lineDash: SimpleSignal<number[], this>;
    readonly lineDashOffset: SimpleSignal<number, this>;
    readonly antialiased: SimpleSignal<boolean, this>;
    protected readonly rippleStrength: SimpleSignal<number, this>;
    protected rippleSize(): number;
    constructor(props: ShapeProps);
    protected applyText(context: CanvasRenderingContext2D): void;
    protected applyStyle(context: CanvasRenderingContext2D): void;
    protected draw(context: CanvasRenderingContext2D): void;
    protected drawShape(context: CanvasRenderingContext2D): void;
    protected getCacheBBox(): BBox;
    protected getPath(): Path2D;
    protected getRipplePath(): Path2D;
    protected drawRipple(context: CanvasRenderingContext2D): void;
    ripple(duration?: number): Generator<void | import("@motion-canvas/core").ThreadGenerator | Promise<any> | import("@motion-canvas/core").Promisable<any>, void, any>;
}
//# sourceMappingURL=Shape.d.ts.map