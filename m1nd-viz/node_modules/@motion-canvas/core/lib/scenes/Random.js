import { map } from '../tweening';
import { range } from '../utils';
/**
 * A random number generator based on
 * {@link https://gist.github.com/tommyettinger/46a874533244883189143505d203312c | Mulberry32}.
 */
export class Random {
    constructor(state) {
        this.state = state;
        /**
         * Previously generated Gaussian random number.
         *
         * @remarks
         * This is an optimization.
         * Since {@link gauss} generates a pair of independent Gaussian random
         * numbers, it returns one immediately and stores the other for the next call
         * to {@link gauss}.
         */
        this.nextGauss = null;
    }
    /**
     * @internal
     */
    static createSeed() {
        return Math.floor(Math.random() * 4294967296);
    }
    /**
     * Get the next random float in the given range.
     *
     * @param from - The start of the range.
     * @param to - The end of the range.
     */
    nextFloat(from = 0, to = 1) {
        return map(from, to, this.next());
    }
    /**
     * Get the next random integer in the given range.
     *
     * @param from - The start of the range.
     * @param to - The end of the range. Exclusive.
     */
    nextInt(from = 0, to = 4294967296) {
        let value = Math.floor(map(from, to, this.next()));
        if (value === to) {
            value = from;
        }
        return value;
    }
    /**
     * Get a random float from a gaussian distribution.
     * @param mean - The mean of the distribution.
     * @param stdev - The standard deviation of the distribution.
     */
    gauss(mean = 0, stdev = 1) {
        let z = this.nextGauss;
        this.nextGauss = null;
        if (z === null) {
            const x2pi = this.next() * 2 * Math.PI;
            const g2rad = Math.sqrt(-2 * Math.log(1 - this.next()));
            z = Math.cos(x2pi) * g2rad;
            this.nextGauss = Math.sin(x2pi) * g2rad;
        }
        return mean + z * stdev;
    }
    /**
     * Get an array filled with random floats in the given range.
     *
     * @param size - The size of the array.
     * @param from - The start of the range.
     * @param to - The end of the range.
     */
    floatArray(size, from = 0, to = 1) {
        return range(size).map(() => this.nextFloat(from, to));
    }
    /**
     Get an array filled with random integers in the given range.
     *
     * @param size - The size of the array.
     * @param from - The start of the range.
     * @param to - The end of the range. Exclusive.
     */
    intArray(size, from = 0, to = 4294967296) {
        return range(size).map(() => this.nextInt(from, to));
    }
    /**
     * Create a new independent generator.
     */
    spawn() {
        return new Random(this.nextInt());
    }
    next() {
        this.state |= 0;
        this.state = (this.state + 0x6d2b79f5) | 0;
        let t = Math.imul(this.state ^ (this.state >>> 15), 1 | this.state);
        t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiUmFuZG9tLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3NjZW5lcy9SYW5kb20udHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFDLEdBQUcsRUFBQyxNQUFNLGFBQWEsQ0FBQztBQUNoQyxPQUFPLEVBQUMsS0FBSyxFQUFDLE1BQU0sVUFBVSxDQUFDO0FBRS9COzs7R0FHRztBQUNILE1BQU0sT0FBTyxNQUFNO0lBWWpCLFlBQTJCLEtBQWE7UUFBYixVQUFLLEdBQUwsS0FBSyxDQUFRO1FBWHhDOzs7Ozs7OztXQVFHO1FBQ0ssY0FBUyxHQUFrQixJQUFJLENBQUM7SUFFRyxDQUFDO0lBRTVDOztPQUVHO0lBQ0ksTUFBTSxDQUFDLFVBQVU7UUFDdEIsT0FBTyxJQUFJLENBQUMsS0FBSyxDQUFDLElBQUksQ0FBQyxNQUFNLEVBQUUsR0FBRyxVQUFVLENBQUMsQ0FBQztJQUNoRCxDQUFDO0lBRUQ7Ozs7O09BS0c7SUFDSSxTQUFTLENBQUMsSUFBSSxHQUFHLENBQUMsRUFBRSxFQUFFLEdBQUcsQ0FBQztRQUMvQixPQUFPLEdBQUcsQ0FBQyxJQUFJLEVBQUUsRUFBRSxFQUFFLElBQUksQ0FBQyxJQUFJLEVBQUUsQ0FBQyxDQUFDO0lBQ3BDLENBQUM7SUFFRDs7Ozs7T0FLRztJQUNJLE9BQU8sQ0FBQyxJQUFJLEdBQUcsQ0FBQyxFQUFFLEVBQUUsR0FBRyxVQUFVO1FBQ3RDLElBQUksS0FBSyxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsR0FBRyxDQUFDLElBQUksRUFBRSxFQUFFLEVBQUUsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDLENBQUMsQ0FBQztRQUNuRCxJQUFJLEtBQUssS0FBSyxFQUFFLEVBQUU7WUFDaEIsS0FBSyxHQUFHLElBQUksQ0FBQztTQUNkO1FBRUQsT0FBTyxLQUFLLENBQUM7SUFDZixDQUFDO0lBRUQ7Ozs7T0FJRztJQUNJLEtBQUssQ0FBQyxJQUFJLEdBQUcsQ0FBQyxFQUFFLEtBQUssR0FBRyxDQUFDO1FBQzlCLElBQUksQ0FBQyxHQUFHLElBQUksQ0FBQyxTQUFTLENBQUM7UUFDdkIsSUFBSSxDQUFDLFNBQVMsR0FBRyxJQUFJLENBQUM7UUFDdEIsSUFBSSxDQUFDLEtBQUssSUFBSSxFQUFFO1lBQ2QsTUFBTSxJQUFJLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxHQUFHLENBQUMsR0FBRyxJQUFJLENBQUMsRUFBRSxDQUFDO1lBQ3ZDLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxJQUFJLENBQUMsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksRUFBRSxDQUFDLENBQUMsQ0FBQztZQUN4RCxDQUFDLEdBQUcsSUFBSSxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsR0FBRyxLQUFLLENBQUM7WUFDM0IsSUFBSSxDQUFDLFNBQVMsR0FBRyxJQUFJLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxHQUFHLEtBQUssQ0FBQztTQUN6QztRQUNELE9BQU8sSUFBSSxHQUFHLENBQUMsR0FBRyxLQUFLLENBQUM7SUFDMUIsQ0FBQztJQUVEOzs7Ozs7T0FNRztJQUNJLFVBQVUsQ0FBQyxJQUFZLEVBQUUsSUFBSSxHQUFHLENBQUMsRUFBRSxFQUFFLEdBQUcsQ0FBQztRQUM5QyxPQUFPLEtBQUssQ0FBQyxJQUFJLENBQUMsQ0FBQyxHQUFHLENBQUMsR0FBRyxFQUFFLENBQUMsSUFBSSxDQUFDLFNBQVMsQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUFDLENBQUMsQ0FBQztJQUN6RCxDQUFDO0lBRUQ7Ozs7OztPQU1HO0lBQ0ksUUFBUSxDQUFDLElBQVksRUFBRSxJQUFJLEdBQUcsQ0FBQyxFQUFFLEVBQUUsR0FBRyxVQUFVO1FBQ3JELE9BQU8sS0FBSyxDQUFDLElBQUksQ0FBQyxDQUFDLEdBQUcsQ0FBQyxHQUFHLEVBQUUsQ0FBQyxJQUFJLENBQUMsT0FBTyxDQUFDLElBQUksRUFBRSxFQUFFLENBQUMsQ0FBQyxDQUFDO0lBQ3ZELENBQUM7SUFFRDs7T0FFRztJQUNJLEtBQUs7UUFDVixPQUFPLElBQUksTUFBTSxDQUFDLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxDQUFDO0lBQ3BDLENBQUM7SUFFTyxJQUFJO1FBQ1YsSUFBSSxDQUFDLEtBQUssSUFBSSxDQUFDLENBQUM7UUFDaEIsSUFBSSxDQUFDLEtBQUssR0FBRyxDQUFDLElBQUksQ0FBQyxLQUFLLEdBQUcsVUFBVSxDQUFDLEdBQUcsQ0FBQyxDQUFDO1FBQzNDLElBQUksQ0FBQyxHQUFHLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssR0FBRyxDQUFDLElBQUksQ0FBQyxLQUFLLEtBQUssRUFBRSxDQUFDLEVBQUUsQ0FBQyxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsQ0FBQztRQUNwRSxDQUFDLEdBQUcsQ0FBQyxDQUFDLEdBQUcsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDLEdBQUcsQ0FBQyxDQUFDLEtBQUssQ0FBQyxDQUFDLEVBQUUsRUFBRSxHQUFHLENBQUMsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxDQUFDO1FBQy9DLE9BQU8sQ0FBQyxDQUFDLENBQUMsR0FBRyxDQUFDLENBQUMsS0FBSyxFQUFFLENBQUMsQ0FBQyxLQUFLLENBQUMsQ0FBQyxHQUFHLFVBQVUsQ0FBQztJQUMvQyxDQUFDO0NBQ0YifQ==