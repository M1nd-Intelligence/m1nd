import { ThreadGenerator } from '../threading';
type ProgressFunction = (value: number, time: number) => void;
export declare function spring(spring: Spring | null, from: number, to: number, settleTolerance: number, onProgress: ProgressFunction, onEnd?: ProgressFunction): ThreadGenerator;
export declare function spring(spring: Spring | null, from: number, to: number, onProgress: ProgressFunction, onEnd?: ProgressFunction): ThreadGenerator;
export interface Spring {
    mass: number;
    stiffness: number;
    damping: number;
    initialVelocity?: number;
}
export declare function makeSpring(mass: number, stiffness: number, damping: number, initialVelocity?: number): Spring;
export declare const BeatSpring: Spring;
export declare const PlopSpring: Spring;
export declare const BounceSpring: Spring;
export declare const SwingSpring: Spring;
export declare const JumpSpring: Spring;
export declare const StrikeSpring: Spring;
export declare const SmoothSpring: Spring;
export {};
//# sourceMappingURL=spring.d.ts.map