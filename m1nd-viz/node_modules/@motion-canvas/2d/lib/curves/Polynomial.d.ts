/**
 * A polynomial in the form ax^3 + bx^2 + cx + d up to a cubic polynomial.
 *
 * Source code liberally taken from:
 * https://github.com/FreyaHolmer/Mathfs/blob/master/Runtime/Curves/Polynomial.cs
 */
export declare class Polynomial {
    readonly c0: number;
    readonly c1: number;
    readonly c2: number;
    readonly c3: number;
    /**
     * Constructs a constant polynomial
     *
     * @param c0 - The constant coefficient
     */
    static constant(c0: number): Polynomial;
    /**
     * Constructs a linear polynomial
     *
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     */
    static linear(c0: number, c1: number): Polynomial;
    /**
     * Constructs a quadratic polynomial
     *
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     * @param c2 - The quadratic coefficient
     */
    static quadratic(c0: number, c1: number, c2: number): Polynomial;
    /**
     * Constructs a cubic polynomial
     *
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     * @param c2 - The quadratic coefficient
     * @param c3 - The cubic coefficient
     */
    static cubic(c0: number, c1: number, c2: number, c3: number): Polynomial;
    /**
     * The degree of the polynomial
     */
    get degree(): number;
    /**
     * @param c0 - The constant coefficient
     */
    constructor(c0: number);
    /**
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     */
    constructor(c0: number, c1: number);
    /**
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     * @param c2 - The quadratic coefficient
     */
    constructor(c0: number, c1: number, c2: number);
    /**
     * @param c0 - The constant coefficient
     * @param c1 - The linear coefficient
     * @param c2 - The quadratic coefficient
     * @param c3 - The cubic coefficient
     */
    constructor(c0: number, c1: number, c2: number, c3: number);
    /**
     * Return the nth derivative of the polynomial.
     *
     * @param n - The number of times to differentiate the polynomial.
     */
    differentiate(n?: number): Polynomial;
    /**
     * Evaluate the polynomial at the given value t.
     *
     * @param t - The value to sample at
     */
    eval(t: number): number;
    /**
     * Evaluate the nth derivative of the polynomial at the given value t.
     *
     * @param t - The value to sample at
     * @param derivative - The derivative of the polynomial to sample from
     */
    eval(t: number, derivative: number): number;
    /**
     * Split the polynomial into two polynomials of the same overall shape.
     *
     * @param u - The point at which to split the polynomial.
     */
    split(u: number): [Polynomial, Polynomial];
    /**
     * Calculate the roots (values where this polynomial = 0).
     *
     * @remarks
     * Depending on the degree of the polynomial, returns between 0 and 3 results.
     */
    roots(): number[];
    /**
     * Calculate the local extrema of the polynomial.
     */
    localExtrema(): number[];
    /**
     * Calculate the local extrema of the polynomial in the unit interval.
     */
    localExtrema01(): number[];
    /**
     * Return the output value range within the unit interval.
     */
    outputRange01(): number[];
    private solveCubicRoots;
    private solveDepressedCubicRoots;
    private solveQuadraticRoots;
    private solveLinearRoot;
    private almostZero;
}
//# sourceMappingURL=Polynomial.d.ts.map