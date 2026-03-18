import { BBox, Vector2 } from '@motion-canvas/core';
import { Polynomial } from './Polynomial';
export declare class Polynomial2D {
    readonly c0: Vector2 | Polynomial;
    readonly c1: Vector2 | Polynomial;
    readonly c2?: Vector2 | undefined;
    readonly c3?: Vector2 | undefined;
    readonly x: Polynomial;
    readonly y: Polynomial;
    constructor(c0: Vector2, c1: Vector2, c2: Vector2, c3: Vector2);
    constructor(c0: Vector2, c1: Vector2, c2: Vector2);
    constructor(x: Polynomial, y: Polynomial);
    eval(t: number, derivative?: number): Vector2;
    split(u: number): [Polynomial2D, Polynomial2D];
    differentiate(n?: number): Polynomial2D;
    evalDerivative(t: number): Vector2;
    /**
     * Calculate the tight axis-aligned bounds of the curve in the unit interval.
     */
    getBounds(): BBox;
}
//# sourceMappingURL=Polynomial2D.d.ts.map