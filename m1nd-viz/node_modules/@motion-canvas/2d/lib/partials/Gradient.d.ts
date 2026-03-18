import { PossibleColor, PossibleVector2, SignalValue, SimpleSignal, Vector2Signal } from '@motion-canvas/core';
export type GradientType = 'linear' | 'conic' | 'radial';
export interface GradientStop {
    offset: SignalValue<number>;
    color: SignalValue<PossibleColor>;
}
export interface GradientProps {
    type?: SignalValue<GradientType>;
    fromX?: SignalValue<number>;
    fromY?: SignalValue<number>;
    from?: SignalValue<PossibleVector2>;
    toX?: SignalValue<number>;
    toY?: SignalValue<number>;
    to?: SignalValue<PossibleVector2>;
    angle?: SignalValue<number>;
    fromRadius?: SignalValue<number>;
    toRadius?: SignalValue<number>;
    stops?: GradientStop[];
}
export declare class Gradient {
    readonly type: SimpleSignal<GradientType, this>;
    readonly from: Vector2Signal<this>;
    readonly to: Vector2Signal<this>;
    readonly angle: SimpleSignal<number, this>;
    readonly fromRadius: SimpleSignal<number, this>;
    readonly toRadius: SimpleSignal<number, this>;
    readonly stops: SimpleSignal<GradientStop[], this>;
    constructor(props: GradientProps);
    canvasGradient(context: CanvasRenderingContext2D): CanvasGradient;
}
//# sourceMappingURL=Gradient.d.ts.map