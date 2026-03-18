import { PossibleVector2, Signal, Vector2 } from '@motion-canvas/core';
import type { Length } from '../partials';
export type Vector2LengthSignal<TOwner> = Signal<PossibleVector2<Length>, Vector2, TOwner> & {
    x: Signal<Length, number, TOwner>;
    y: Signal<Length, number, TOwner>;
};
export declare function vector2Signal(prefix?: string | Record<string, string>): PropertyDecorator;
//# sourceMappingURL=vector2Signal.d.ts.map