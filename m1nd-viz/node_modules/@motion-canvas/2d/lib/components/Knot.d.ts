import { PossibleVector2, Signal, SignalValue, Vector2Signal } from '@motion-canvas/core';
import { KnotInfo } from '../curves';
import { Node, NodeProps } from './Node';
export interface KnotProps extends NodeProps {
    /**
     * {@inheritDoc Knot.startHandle}
     */
    startHandle?: SignalValue<PossibleVector2>;
    /**
     * {@inheritDoc Knot.endHandle}
     */
    endHandle?: SignalValue<PossibleVector2>;
    /**
     * {@inheritDoc Knot.auto}
     */
    auto?: SignalValue<PossibleKnotAuto>;
    startHandleAuto?: SignalValue<number>;
    endHandleAuto?: SignalValue<number>;
}
export type KnotAuto = {
    startHandle: number;
    endHandle: number;
};
export type PossibleKnotAuto = KnotAuto | number | [number, number];
export type KnotAutoSignal<TOwner> = Signal<PossibleKnotAuto, KnotAuto, TOwner> & {
    endHandle: Signal<number, number, TOwner>;
    startHandle: Signal<number, number, TOwner>;
};
/**
 * A node representing a knot of a {@link Spline}.
 */
export declare class Knot extends Node {
    /**
     * The position of the knot's start handle. The position is provided relative
     * to the knot's position.
     *
     * @remarks
     * By default, the position of the start handle will be the mirrored position
     * of the {@link endHandle}.
     *
     * If neither an end handle nor a start handle is provided, the positions of
     * the handles gets calculated automatically to create smooth curve through
     * the knot. The smoothness of the resulting curve can be controlled via the
     * {@link Spline.smoothness} property.
     *
     * It is also possible to blend between a user-defined position and the
     * auto-calculated position by using the {@link auto} property.
     *
     * @defaultValue Mirrored position of the endHandle.
     */
    readonly startHandle: Vector2Signal<this>;
    /**
     * The position of the knot's end handle. The position is provided relative
     * to the knot's position.
     *
     * @remarks
     * By default, the position of the end handle will be the mirrored position
     * of the {@link startHandle}.
     *
     * If neither an end handle nor a start handle is provided, the positions of
     * the handles gets calculated automatically to create smooth curve through
     * the knot. The smoothness of the resulting curve can be controlled via the
     * {@link Spline.smoothness} property.
     *
     * It is also possible to blend between a user-defined position and the
     * auto-calculated position by using the {@link auto} property.
     *
     * @defaultValue Mirrored position of the startHandle.
     */
    readonly endHandle: Vector2Signal<this>;
    /**
     * How much to blend between the user-provided handles and the auto-calculated
     * handles.
     *
     * @remarks
     * This property has no effect if no explicit handles are provided for the
     * knot.
     *
     * @defaultValue 0
     */
    readonly auto: KnotAutoSignal<this>;
    get startHandleAuto(): Signal<number, number, this, import("@motion-canvas/core").SignalContext<number, number, this>>;
    get endHandleAuto(): Signal<number, number, this, import("@motion-canvas/core").SignalContext<number, number, this>>;
    constructor(props: KnotProps);
    points(): KnotInfo;
    private getDefaultEndHandle;
    private getDefaultStartHandle;
}
//# sourceMappingURL=Knot.d.ts.map