import { Random } from '../scenes';
/**
 * Get the random number generator for the current scene.
 **/
export declare function useRandom(): Random;
/**
 * Get the random number generator for the given seed.
 *
 * @param seed - The seed for the generator.
 * @param fixed - Whether the seed should be fixed. Fixed seeds remain
 *                the same even when the main scene seed changes.
 */
export declare function useRandom(seed: number, fixed?: boolean): Random;
//# sourceMappingURL=useRandom.d.ts.map