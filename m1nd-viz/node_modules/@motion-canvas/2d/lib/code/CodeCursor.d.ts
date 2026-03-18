import { Vector2 } from '@motion-canvas/core';
import { Code } from '../components';
import { CodeScope } from './CodeScope';
export interface CodeFragmentDrawingInfo {
    text: string;
    position: Vector2;
    characterSize: Vector2;
    cursor: Vector2;
    fill: string;
    time: number;
    alpha: number;
}
/**
 * A stateful class for recursively traversing a code scope.
 *
 * @internal
 */
export declare class CodeCursor {
    private readonly node;
    cursor: Vector2;
    beforeCursor: Vector2;
    afterCursor: Vector2;
    beforeIndex: number;
    afterIndex: number;
    private context;
    private monoWidth;
    private maxWidth;
    private lineHeight;
    private fallbackFill;
    private caches;
    private highlighter;
    private selection;
    private selectionProgress;
    private globalProgress;
    private fragmentDrawingInfo;
    private fontHeight;
    private verticalOffset;
    constructor(node: Code);
    /**
     * Prepare the cursor for the next traversal.
     *
     * @param context - The context used to measure and draw the code.
     */
    setupMeasure(context: CanvasRenderingContext2D): void;
    setupDraw(context: CanvasRenderingContext2D): void;
    /**
     * Measure the desired size of the code scope.
     *
     * @remarks
     * The result can be retrieved with {@link getSize}.
     *
     * @param scope - The code scope to measure.
     */
    measureSize(scope: CodeScope): void;
    /**
     * Get the size measured by the cursor.
     */
    getSize(): {
        x: number;
        y: number;
    };
    /**
     * Get the drawing information created by the cursor.
     */
    getDrawingInfo(): {
        fragments: CodeFragmentDrawingInfo[];
        verticalOffset: number;
        fontHeight: number;
    };
    /**
     * Draw the given code scope.
     *
     * @param scope - The code scope to draw.
     */
    drawScope(scope: CodeScope): void;
    private drawToken;
    private calculateWidth;
    private calculateMaxWidth;
    private currentProgress;
    private processSelection;
    private isSelected;
}
//# sourceMappingURL=CodeCursor.d.ts.map