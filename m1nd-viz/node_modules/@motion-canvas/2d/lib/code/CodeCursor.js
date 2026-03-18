import { clampRemap, Color, map, unwrap, Vector2, } from '@motion-canvas/core';
import { parseCodeFragment } from './CodeFragment';
import { isCodeScope } from './CodeScope';
import { isPointInCodeSelection } from './CodeSelection';
/**
 * A stateful class for recursively traversing a code scope.
 *
 * @internal
 */
export class CodeCursor {
    constructor(node) {
        this.node = node;
        this.cursor = new Vector2();
        this.beforeCursor = new Vector2();
        this.afterCursor = new Vector2();
        this.beforeIndex = 0;
        this.afterIndex = 0;
        this.context = {};
        this.monoWidth = 0;
        this.maxWidth = 0;
        this.lineHeight = 0;
        this.fallbackFill = new Color('white');
        this.caches = null;
        this.highlighter = null;
        this.selection = [];
        this.selectionProgress = null;
        this.globalProgress = [];
        this.fragmentDrawingInfo = [];
        this.fontHeight = 0;
        this.verticalOffset = 0;
    }
    /**
     * Prepare the cursor for the next traversal.
     *
     * @param context - The context used to measure and draw the code.
     */
    setupMeasure(context) {
        const metrics = context.measureText('X');
        this.monoWidth = metrics.width;
        this.fontHeight =
            metrics.fontBoundingBoxDescent + metrics.fontBoundingBoxAscent;
        this.verticalOffset = metrics.fontBoundingBoxAscent;
        this.context = context;
        this.lineHeight = parseFloat(this.node.styles.lineHeight);
        this.cursor = new Vector2();
        this.beforeCursor = new Vector2();
        this.afterCursor = new Vector2();
        this.beforeIndex = 0;
        this.afterIndex = 0;
        this.maxWidth = 0;
    }
    setupDraw(context) {
        this.setupMeasure(context);
        const fill = this.node.fill();
        this.fallbackFill =
            fill instanceof Color ? fill : new Color('white');
        this.caches = this.node.highlighterCache();
        this.highlighter = this.node.highlighter();
        this.selection = this.node.selection();
        this.selectionProgress = this.node.selectionProgress();
        this.fragmentDrawingInfo = [];
        this.globalProgress = [];
    }
    /**
     * Measure the desired size of the code scope.
     *
     * @remarks
     * The result can be retrieved with {@link getSize}.
     *
     * @param scope - The code scope to measure.
     */
    measureSize(scope) {
        const progress = unwrap(scope.progress);
        for (const wrapped of scope.fragments) {
            const possibleFragment = unwrap(wrapped);
            if (isCodeScope(possibleFragment)) {
                this.measureSize(possibleFragment);
                continue;
            }
            if (Array.isArray(possibleFragment)) {
                this.measureSize({
                    progress: scope.progress,
                    fragments: possibleFragment,
                });
                continue;
            }
            const fragment = parseCodeFragment(possibleFragment, this.context, this.monoWidth);
            const beforeMaxWidth = this.calculateMaxWidth(fragment.before);
            const afterMaxWidth = this.calculateMaxWidth(fragment.after);
            const maxWidth = map(beforeMaxWidth, afterMaxWidth, progress);
            if (maxWidth > this.maxWidth) {
                this.maxWidth = maxWidth;
            }
            const beforeEnd = this.calculateWidth(fragment.before);
            const afterEnd = this.calculateWidth(fragment.after);
            this.cursor.x = map(beforeEnd, afterEnd, progress);
            if (this.cursor.y === 0) {
                this.cursor.y = 1;
            }
            this.cursor.y += map(fragment.before.newRows, fragment.after.newRows, progress);
        }
    }
    /**
     * Get the size measured by the cursor.
     */
    getSize() {
        return {
            x: this.maxWidth * this.monoWidth,
            y: this.cursor.y * this.lineHeight + this.verticalOffset,
        };
    }
    /**
     * Get the drawing information created by the cursor.
     */
    getDrawingInfo() {
        return {
            fragments: this.fragmentDrawingInfo,
            verticalOffset: this.verticalOffset,
            fontHeight: this.fontHeight,
        };
    }
    /**
     * Draw the given code scope.
     *
     * @param scope - The code scope to draw.
     */
    drawScope(scope) {
        const progress = unwrap(scope.progress);
        for (const wrappedFragment of scope.fragments) {
            const possibleFragment = unwrap(wrappedFragment);
            if (isCodeScope(possibleFragment)) {
                this.drawScope(possibleFragment);
                continue;
            }
            if (Array.isArray(possibleFragment)) {
                this.drawScope({
                    progress: scope.progress,
                    fragments: possibleFragment,
                });
                continue;
            }
            const fragment = parseCodeFragment(possibleFragment, this.context, this.monoWidth);
            const timingOffset = 0.8;
            let alpha = 1;
            let offsetY = 0;
            if (fragment.before.content !== fragment.after.content) {
                const mirrored = Math.abs(progress - 0.5) * 2;
                alpha = clampRemap(1, 1 - timingOffset, 1, 0, mirrored);
                const isBigger = fragment.after.newRows > fragment.before.newRows ? 1 : -1;
                const isBefore = progress < 0.5 ? 1 : -1;
                const scale = isBigger * isBefore * 4;
                offsetY = map(Math.abs(fragment.after.newRows - fragment.before.newRows) / scale, 0, mirrored);
            }
            this.drawToken(fragment, scope, this.cursor.addY(offsetY), alpha);
            this.beforeCursor.x = this.calculateWidth(fragment.before, this.beforeCursor.x);
            this.afterCursor.x = this.calculateWidth(fragment.after, this.afterCursor.x);
            this.beforeCursor.y += fragment.before.newRows;
            this.afterCursor.y += fragment.after.newRows;
            this.beforeIndex += fragment.before.content.length;
            this.afterIndex += fragment.after.content.length;
            this.cursor.y += map(fragment.before.newRows, fragment.after.newRows, progress);
            const beforeEnd = this.calculateWidth(fragment.before);
            const afterEnd = this.calculateWidth(fragment.after);
            this.cursor.x = map(beforeEnd, afterEnd, progress);
        }
    }
    drawToken(fragment, scope, offset, alpha) {
        const progress = unwrap(scope.progress);
        const currentProgress = this.currentProgress();
        if (progress > 0) {
            this.globalProgress.push(progress);
        }
        const code = progress < 0.5 ? fragment.before : fragment.after;
        let hasOffset = true;
        let width = 0;
        let stringLength = 0;
        let y = 0;
        for (let i = 0; i < code.content.length; i++) {
            let color = this.fallbackFill.serialize();
            let char = code.content.charAt(i);
            const selection = {
                before: null,
                after: null,
            };
            if (char === '\n') {
                y++;
                hasOffset = false;
                width = 0;
                stringLength = 0;
                selection.before = null;
                selection.after = null;
                continue;
            }
            const beforeHighlight = this.caches &&
                this.highlighter?.highlight(this.beforeIndex + i, this.caches.before);
            const afterHighlight = this.caches &&
                this.highlighter?.highlight(this.afterIndex + i, this.caches.after);
            const highlight = progress < 0.5 ? beforeHighlight : afterHighlight;
            if (highlight) {
                // Handle edge cases where the highlight style changes despite the
                // content being the same. The code doesn't fade in and out so the color
                // has to be interpolated to avoid jarring changes.
                if (fragment.before.content === fragment.after.content &&
                    beforeHighlight?.color !== afterHighlight?.color) {
                    highlight.color = Color.lerp(beforeHighlight?.color ?? this.fallbackFill, afterHighlight?.color ?? this.fallbackFill, progress).serialize();
                }
                if (highlight.color) {
                    color = highlight.color;
                }
                let skipAhead = 0;
                do {
                    if (this.processSelection(selection, skipAhead, hasOffset, stringLength, y)) {
                        break;
                    }
                    skipAhead++;
                } while (skipAhead < highlight.skipAhead &&
                    code.content.charAt(i + skipAhead) !== '\n');
                if (skipAhead > 1) {
                    char = code.content.slice(i, i + skipAhead);
                }
                i += char.length - 1;
            }
            else {
                this.processSelection(selection, 0, hasOffset, stringLength, y);
                let skipAhead = 1;
                while (i < code.content.length - 1 &&
                    code.content.charAt(i + 1) !== '\n') {
                    if (this.processSelection(selection, skipAhead, hasOffset, stringLength, y)) {
                        break;
                    }
                    skipAhead++;
                    char += code.content.charAt(++i);
                }
            }
            let time;
            const selectionAfter = selection.after ?? 0;
            const selectionBefore = selection.before ?? 0;
            if (fragment.before.content === '') {
                time = selectionAfter;
            }
            else if (fragment.after.content === '') {
                time = selectionBefore;
            }
            else {
                time = map(selectionBefore, selectionAfter, this.selectionProgress ?? currentProgress);
            }
            const measure = this.context.measureText(char);
            this.fragmentDrawingInfo.push({
                text: char,
                position: new Vector2((hasOffset ? offset.x + width : width) * this.monoWidth, (offset.y + y) * this.lineHeight),
                cursor: new Vector2(hasOffset ? this.beforeCursor.x + stringLength : stringLength, this.beforeCursor.y + y),
                alpha,
                characterSize: new Vector2(measure.width / char.length, this.fontHeight),
                fill: color,
                time,
            });
            stringLength += char.length;
            width += Math.round(measure.width / this.monoWidth);
        }
    }
    calculateWidth(metrics, x = this.cursor.x) {
        return metrics.newRows === 0 ? x + metrics.lastWidth : metrics.lastWidth;
    }
    calculateMaxWidth(metrics, x = this.cursor.x) {
        return Math.max(this.maxWidth, metrics.maxWidth, x + metrics.firstWidth);
    }
    currentProgress() {
        if (this.globalProgress.length === 0) {
            return 0;
        }
        let sum = 0;
        for (const progress of this.globalProgress) {
            sum += progress;
        }
        return sum / this.globalProgress.length;
    }
    processSelection(selection, skipAhead, hasOffset, stringLength, y) {
        let shouldBreak = false;
        let currentSelected = this.isSelected((hasOffset ? this.beforeCursor.x + stringLength : stringLength) +
            skipAhead, this.beforeCursor.y + y);
        if (selection.before !== null && selection.before !== currentSelected) {
            shouldBreak = true;
        }
        else {
            selection.before = currentSelected;
        }
        currentSelected = this.isSelected((hasOffset ? this.afterCursor.x + stringLength : stringLength) +
            skipAhead, this.afterCursor.y + y, true);
        if (selection.after !== null && selection.after !== currentSelected) {
            shouldBreak = true;
        }
        else {
            selection.after = currentSelected;
        }
        return shouldBreak;
    }
    isSelected(x, y, isAfter) {
        const point = [y, x];
        const maxSelection = isPointInCodeSelection(point, this.selection) ? 1 : 0;
        if (this.node.oldSelection === null || this.selectionProgress === null) {
            return maxSelection;
        }
        if (isAfter) {
            return maxSelection;
        }
        return isPointInCodeSelection(point, this.node.oldSelection) ? 1 : 0;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZUN1cnNvci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29kZS9Db2RlQ3Vyc29yLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFDTCxVQUFVLEVBQ1YsS0FBSyxFQUNMLEdBQUcsRUFFSCxNQUFNLEVBQ04sT0FBTyxHQUNSLE1BQU0scUJBQXFCLENBQUM7QUFFN0IsT0FBTyxFQUFlLGlCQUFpQixFQUFDLE1BQU0sZ0JBQWdCLENBQUM7QUFJL0QsT0FBTyxFQUFZLFdBQVcsRUFBQyxNQUFNLGFBQWEsQ0FBQztBQUNuRCxPQUFPLEVBQUMsc0JBQXNCLEVBQUMsTUFBTSxpQkFBaUIsQ0FBQztBQVl2RDs7OztHQUlHO0FBQ0gsTUFBTSxPQUFPLFVBQVU7SUFvQnJCLFlBQW9DLElBQVU7UUFBVixTQUFJLEdBQUosSUFBSSxDQUFNO1FBbkJ2QyxXQUFNLEdBQUcsSUFBSSxPQUFPLEVBQUUsQ0FBQztRQUN2QixpQkFBWSxHQUFHLElBQUksT0FBTyxFQUFFLENBQUM7UUFDN0IsZ0JBQVcsR0FBRyxJQUFJLE9BQU8sRUFBRSxDQUFDO1FBQzVCLGdCQUFXLEdBQUcsQ0FBQyxDQUFDO1FBQ2hCLGVBQVUsR0FBRyxDQUFDLENBQUM7UUFDZCxZQUFPLEdBQUcsRUFBOEIsQ0FBQztRQUN6QyxjQUFTLEdBQUcsQ0FBQyxDQUFDO1FBQ2QsYUFBUSxHQUFHLENBQUMsQ0FBQztRQUNiLGVBQVUsR0FBRyxDQUFDLENBQUM7UUFDZixpQkFBWSxHQUFHLElBQUksS0FBSyxDQUFDLE9BQU8sQ0FBQyxDQUFDO1FBQ2xDLFdBQU0sR0FBNkMsSUFBSSxDQUFDO1FBQ3hELGdCQUFXLEdBQTJCLElBQUksQ0FBQztRQUMzQyxjQUFTLEdBQWdCLEVBQUUsQ0FBQztRQUM1QixzQkFBaUIsR0FBa0IsSUFBSSxDQUFDO1FBQ3hDLG1CQUFjLEdBQWEsRUFBRSxDQUFDO1FBQzlCLHdCQUFtQixHQUE4QixFQUFFLENBQUM7UUFDcEQsZUFB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