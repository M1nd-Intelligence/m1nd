import { Reference } from './createRef';
export type ReferenceArray<T> = T[] & Reference<T>;
/**
 * Create an array of references.
 *
 * @remarks
 * The returned object is both an array and a reference that can be passed
 * directly to the `ref` property of a node.
 *
 * @example
 * ```tsx
 * const labels = createRefArray<Txt>();
 *
 * view.add(['A', 'B'].map(text => <Txt ref={labels}>{text}</Txt>));
 * view.add(<Txt ref={labels}>C</Txt>);
 *
 * // accessing the references individually:
 * yield* labels[0].text('A changes', 0.3);
 * yield* labels[1].text('B changes', 0.3);
 * yield* labels[2].text('C changes', 0.3);
 *
 * // accessing all references at once:
 * yield* all(...labels.map(label => label.fill('white', 0.3)));
 * ```
 */
export declare function createRefArray<T>(): ReferenceArray<T>;
//# sourceMappingURL=createRefArray.d.ts.map