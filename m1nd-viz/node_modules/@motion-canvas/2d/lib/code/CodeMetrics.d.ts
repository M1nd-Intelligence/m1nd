export interface CodeMetrics {
    content: string;
    newRows: number;
    endColumn: number;
    firstWidth: number;
    maxWidth: number;
    lastWidth: number;
}
export declare function measureString(context: CanvasRenderingContext2D, monoWidth: number, value: string): CodeMetrics;
export declare function isCodeMetrics(value: any): value is CodeMetrics;
//# sourceMappingURL=CodeMetrics.d.ts.map