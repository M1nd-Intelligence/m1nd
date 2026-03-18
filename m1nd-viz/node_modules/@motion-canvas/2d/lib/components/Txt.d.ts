import { InterpolationFunction, SignalValue, SimpleSignal, ThreadGenerator, TimingFunction } from '@motion-canvas/core';
import { Node } from './Node';
import { Shape, ShapeProps } from './Shape';
import { TxtLeaf } from './TxtLeaf';
import { ComponentChildren } from './types';
type TxtChildren = string | Node | (string | Node)[];
type AnyTxt = Txt | TxtLeaf;
export interface TxtProps extends ShapeProps {
    children?: TxtChildren;
    text?: SignalValue<string>;
}
export declare class Txt extends Shape {
    /**
     * Create a bold text node.
     *
     * @remarks
     * This is a shortcut for
     * ```tsx
     * <Txt fontWeight={700} />
     * ```
     *
     * @param props - Additional text properties.
     */
    static b(props: TxtProps): Txt;
    /**
     * Create an italic text node.
     *
     * @remarks
     * This is a shortcut for
     * ```tsx
     * <Txt fontStyle={'italic'} />
     * ```
     *
     * @param props - Additional text properties.
     */
    static i(props: TxtProps): Txt;
    readonly text: SimpleSignal<string, this>;
    protected getText(): string;
    protected setText(value: SignalValue<string>): void;
    protected setChildren(value: SignalValue<ComponentChildren>): void;
    protected tweenText(value: SignalValue<string>, time: number, timingFunction: TimingFunction, interpolationFunction: InterpolationFunction<string>): ThreadGenerator;
    protected getLayout(): boolean;
    constructor({ children, text, ...props }: TxtProps);
    protected innerText(): string;
    protected parentTxt(): Txt | null;
    protected parseChildren(children: ComponentChildren): AnyTxt[];
    protected applyFlex(): void;
    protected draw(context: CanvasRenderingContext2D): void;
}
export {};
//# sourceMappingURL=Txt.d.ts.map