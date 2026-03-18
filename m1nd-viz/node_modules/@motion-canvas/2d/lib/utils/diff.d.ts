interface TransformDiff<T> {
    inserted: TransformDiffItem<T>[];
    deleted: TransformDiffItem<T>[];
    transformed: TransformDiffItemTransformed<T>[];
}
interface TransformDiffItem<T> {
    before?: T;
    beforeIdIndex: number;
    current: T;
    currentIndex: number;
}
interface TransformDiffItemTransformed<T> {
    insert: boolean;
    remove: boolean;
    from: TransformDiffItem<T>;
    to: TransformDiffItem<T>;
}
interface ApplyTransformInserted<T> {
    item: TransformDiffItem<T>;
    order: number;
}
interface ApplyTransformResult<T> {
    inserted: ApplyTransformInserted<T>[];
}
interface Idable {
    id: string;
}
export declare function getTransformDiff<T extends Idable>(from: T[], to: T[]): TransformDiff<T>;
export declare function applyTransformDiff<T extends Idable>(current: T[], diff: TransformDiff<T>, cloner: (original: T) => T): ApplyTransformResult<T>;
export {};
//# sourceMappingURL=diff.d.ts.map