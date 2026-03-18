import { clamp } from '@motion-canvas/core';
/**
 * A polynomial in the form ax^3 + bx^2 + cx + d up to a cubic polynomial.
 *
 * Source code liberally taken from:
 * https://github.com/FreyaHolmer/Mathfs/blob/master/Runtime/Curves/Polynomial.cs
 */
export class Polynomial {
    /**
     * Constructs a constant polynomial
     *
     * @param c0 - The constant coefficient
     */
    static constant(c0) {
        return new Polynomial(c0);
    }
    /**
     * Constructs a linear polynomial
     *
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     */
    static linear(c0, c1) {
        return new Polynomial(c0, c1);
    }
    /**
     * Constructs a quadratic polynomial
     *
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     * @param c2 - The quadratic coefficient
     */
    static quadratic(c0, c1, c2) {
        return new Polynomial(c0, c1, c2);
    }
    /**
     * Constructs a cubic polynomial
     *
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     * @param c2 - The quadratic coefficient
     * @param c3 - The cubic coefficient
     */
    static cubic(c0, c1, c2, c3) {
        return new Polynomial(c0, c1, c2, c3);
    }
    /**
     * The degree of the polynomial
     */
    get degree() {
        if (this.c3 !== 0) {
            return 3;
        }
        else if (this.c2 !== 0) {
            return 2;
        }
        else if (this.c1 !== 0) {
            return 1;
        }
        return 0;
    }
    constructor(c0, c1, c2, c3) {
        this.c0 = c0;
        this.c1 = c1 ?? 0;
        this.c2 = c2 ?? 0;
        this.c3 = c3 ?? 0;
    }
    /**
     * Return the nth derivative of the polynomial.
     *
     * @param n - The number of times to differentiate the polynomial.
     */
    differentiate(n = 1) {
        switch (n) {
            case 0:
                return this;
            case 1:
                return new Polynomial(this.c1, 2 * this.c2, 3 * this.c3, 0);
            case 2:
                return new Polynomial(2 * this.c2, 6 * this.c3, 0, 0);
            case 3:
                return new Polynomial(6 * this.c3, 0, 0, 0);
            default:
                throw new Error('Unsupported derivative');
        }
    }
    eval(t, derivative = 0) {
        if (derivative !== 0) {
            return this.differentiate(derivative).eval(t);
        }
        return this.c3 * (t * t * t) + this.c2 * (t * t) + this.c1 * t + this.c0;
    }
    /**
     * Split the polynomial into two polynomials of the same overall shape.
     *
     * @param u - The point at which to split the polynomial.
     */
    split(u) {
        const d = 1 - u;
        const pre = new Polynomial(this.c0, this.c1 * u, this.c2 * u * u, this.c3 * u * u * u);
        const post = new Polynomial(this.eval(0), d * this.differentiate(1).eval(u), ((d * d) / 2) * this.differentiate(2).eval(u), ((d * d * d) / 6) * this.differentiate(3).eval(u));
        return [pre, post];
    }
    /**
     * Calculate the roots (values where this polynomial = 0).
     *
     * @remarks
     * Depending on the degree of the polynomial, returns between 0 and 3 results.
     */
    roots() {
        switch (this.degree) {
            case 3:
                return this.solveCubicRoots();
            case 2:
                return this.solveQuadraticRoots();
            case 1:
                return this.solveLinearRoot();
            case 0:
                return [];
            default:
                throw new Error(`Unsupported polynomial degree: ${this.degree}`);
        }
    }
    /**
     * Calculate the local extrema of the polynomial.
     */
    localExtrema() {
        return this.differentiate().roots();
    }
    /**
     * Calculate the local extrema of the polynomial in the unit interval.
     */
    localExtrema01() {
        const all = this.localExtrema();
        const valids = [];
        for (let i = 0; i < all.length; i++) {
            const t = all[i];
            if (t >= 0 && t <= 1) {
                valids.push(all[i]);
            }
        }
        return valids;
    }
    /**
     * Return the output value range within the unit interval.
     */
    outputRange01() {
        let range = [this.eval(0), this.eval(1)];
        // Expands the minimum or maximum value of the range to contain the given
        // value.
        const encapsulate = (value) => {
            if (range[1] > range[0]) {
                range = [Math.min(range[0], value), Math.max(range[1], value)];
            }
            else {
                range = [Math.min(range[1], value), Math.max(range[0], value)];
            }
        };
        this.localExtrema01().forEach(t => encapsulate(this.eval(t)));
        return range;
    }
    solveCubicRoots() {
        const a = this.c0;
        const b = this.c1;
        const c = this.c2;
        const d = this.c3;
        // First, depress the cubic to make it easier to solve
        const aa = a * a;
        const ac = a * c;
        const bb = b * b;
        const p = (3 * ac - bb) / (3 * aa);
        const q = (2 * bb * b - 9 * ac * b + 27 * aa * d) / (27 * aa * a);
        const dpr = this.solveDepressedCubicRoots(p, q);
        // We now have the roots of the depressed cubic, now convert back to the
        // normal cubic
        const undepressRoot = (r) => r - b / (3 * a);
        switch (dpr.length) {
            case 1:
                return [undepressRoot(dpr[0])];
            case 2:
                return [undepressRoot(dpr[0]), undepressRoot(dpr[1])];
            case 3:
                return [
                    undepressRoot(dpr[0]),
                    undepressRoot(dpr[1]),
                    undepressRoot(dpr[2]),
                ];
            default:
                return [];
        }
    }
    solveDepressedCubicRoots(p, q) {
        // t³+pt+q = 0
        // Triple root - one solution. solve x³+q = 0 => x = cr(-q)
        if (this.almostZero(p)) {
            return [Math.cbrt(-q)];
        }
        const TAU = Math.PI * 2;
        const discriminant = 4 * p * p * p + 27 * q * q;
        if (discriminant < 0.00001) {
            // Two or three roots guaranteed, use trig solution
            const pre = 2 * Math.sqrt(-p / 3);
            const acosInner = ((3 * q) / (2 * p)) * Math.sqrt(-3 / p);
            const getRoot = (k) => pre *
                Math.cos((1 / 3) * Math.acos(clamp(-1, 1, acosInner)) - (TAU / 3) * k);
            // If acos hits 0 or TAU/2, the offsets will have the same value,
            // which means we have a double root plus one regular root on our hands
            if (acosInner >= 0.9999) {
                // two roots - one single and one double root
                return [getRoot(0), getRoot(2)];
            }
            if (acosInner <= -0.9999) {
                // two roots - one single and one double root
                return [getRoot(1), getRoot(2)];
            }
            return [getRoot(0), getRoot(1), getRoot(2)];
        }
        if (discriminant > 0 && p < 0) {
            // one root
            const coshInner = (1 / 3) *
                Math.acosh(((-3 * Math.abs(q)) / (2 * p)) * Math.sqrt(-3 / p));
            const r = -2 * Math.sign(q) * Math.sqrt(-p / 3) * Math.cosh(coshInner);
            return [r];
        }
        if (p > 0) {
            // one root
            const sinhInner = (1 / 3) * Math.asinh(((3 * q) / (2 * p)) * Math.sqrt(3 / p));
            const r = -2 * Math.sqrt(p / 3) * Math.sinh(sinhInner);
            return [r];
        }
        // no roots
        return [];
    }
    solveQuadraticRoots() {
        const a = this.c2;
        const b = this.c1;
        const c = this.c0;
        const rootContent = b * b - 4 * a * c;
        if (this.almostZero(rootContent)) {
            // two equivalent solutions at one point
            return [-b / (2 * a)];
        }
        if (rootContent >= 0) {
            const root = Math.sqrt(rootContent);
            // crosses at two points
            const r0 = (-b - root) / (2 * a);
            const r1 = (-b + root) / (2 * a);
            return [Math.min(r0, r1), Math.max(r0, r1)];
        }
        return [];
    }
    solveLinearRoot() {
        return [-this.c0 / this.c1];
    }
    almostZero(value) {
        return Math.abs(0 - value) <= Number.EPSILON;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUG9seW5vbWlhbC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY3VydmVzL1BvbHlub21pYWwudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLEtBQUssRUFBQyxNQUFNLHFCQUFxQixDQUFDO0FBRTFDOzs7OztHQUtHO0FBQ0gsTUFBTSxPQUFPLFVBQVU7SUFLckI7Ozs7T0FJRztJQUNJLE1BQU0sQ0FBQyxRQUFRLENBQUMsRUFBVTtRQUMvQixPQUFPLElBQUksVUFBVSxDQUFDLEVBQUUsQ0FBQyxDQUFDO0lBQzVCLENBQUM7SUFFRDs7Ozs7T0FLRztJQUNJLE1BQU0sQ0FBQyxNQUFNLENBQUMsRUFBVSxFQUFFLEVBQVU7UUFDekMsT0FBTyxJQUFJLFVBQVUsQ0FBQyxFQUFFLEVBQUUsRUFBRSxDQUFDLENBQUM7SUFDaEMsQ0FBQztJQUVEOzs7Ozs7T0FNRztJQUNJLE1BQU0sQ0FBQyxTQUFTLENBQUMsRUFBVSxFQUFFLEVBQVUsRUFBRSxFQUFVO1FBQ3hELE9BQU8sSUFBSSxVQUFVLENBQUMsRUFBRSxFQUFFLEVBQUUsRUFBRSxFQUFFLENBQUMsQ0FBQztJQUNwQyxDQUFDO0lBRUQ7Ozs7Ozs7T0FPRztJQUNJLE1BQU0sQ0FBQyxLQUFLLENBQ2pCLEVBQVUsRUFDVixFQUFVLEVBQ1YsRUFBVSxFQUNWLEVBQVU7UUFFVixPQUFPLElBQUksVUFBVSxDQUFDLEVBQUUsRUFBRSxFQUFFLEVBQUUsRUFBRSxFQUFFLEVBQUUsQ0FBQyxDQUFDO0lBQ3hDLENBQUM7SUFFRDs7T0FFRztJQUNILElBQVcsTUFBTTtRQUNmLElBQUksSUFBSSxDQUFDLEVBQUUsS0FBSyxDQUFDLEVBQUU7WUFDakIsT0FBTyxDQUFDLENBQUM7U0FDVjthQUFNLElBQUksSUFBSSxDQUFDLEVBQUUsS0FBSyxDQUFDLEVBQUU7WUFDeEIsT0FBTyxDQUFDLENBQUM7U0FDVjthQUFNLElBQUksSUFBSSxDQUFDLEVBQUUsS0FBSyxDQUFD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