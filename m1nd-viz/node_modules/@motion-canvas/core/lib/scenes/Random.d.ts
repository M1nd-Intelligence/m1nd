/**
 * A random number generator based on
 * {@link https://gist.github.com/tommyettinger/46a874533244883189143505d203312c | Mulberry32}.
 */
export declare class Random {
    private state;
    /**
     * Previously generated Gaussian random number.
     *
     * @remarks
     * This is an optimization.
     * Since {@link gauss} generates a pair of independent Gaussian random
     * numbers, it returns one immediately and stores the other for the next call
     * to {@link gauss}.
     */
    private nextGauss;
    constructor(state: number);
    /**
     * @internal
     */
    static createSeed(): number;
    /**
     * Get the next random float in the given range.
     *
     * @param from - The start of the range.
     * @param to - The end of the range.
     */
    nextFloat(from?: number, to?: number): number;
    /**
     * Get the next random integer in the given range.
     *
     * @param from - The start of the range.
     * @param to - The end of the range. Exclusive.
     */
    nextInt(from?: number, to?: number): number;
    /**
     * Get a random float from a gaussian distribution.
     * @param mean - The mean of the distribution.
     * @param stdev - The standard deviation of the distribution.
     */
    gauss(mean?: number, stdev?: number): number;
    /**
     * Get an array filled with random floats in the given range.
     *
     * @param size - The size of the array.
     * @param from - The start of the range.
     * @param to - The end of the range.
     */
    floatArray(size: number, from?: number, to?: number): number[];
    /**
     Get an array filled with random integers in the given range.
     *
     * @param size - The size of the array.
     * @param from - The start of the range.
     * @param to - The end of the range. Exclusive.
     */
    intArray(size: number, from?: number, to?: number): number[];
    /**
     * Create a new independent generator.
     */
    spawn(): Random;
    private next;
}
//# sourceMappingURL=Random.d.ts.map