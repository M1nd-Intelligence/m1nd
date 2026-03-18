var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var CodeBlock_1;
import { Vector2, clampRemap, createComputedAsync, createSignal, easeInOutSine, join, map, threadable, tween, useLogger, waitFor, } from '@motion-canvas/core';
import { diff, parse, ready, } from 'code-fns';
import { computed, initial, nodeName, parser, signal } from '../decorators';
import { Shape } from './Shape';
/**
 * @deprecated Use {@link Code} instead.
 */
let CodeBlock = CodeBlock_1 = class CodeBlock extends Shape {
    *tweenSelection(value, duration, timingFunction) {
        this.oldSelection = this.selection();
        this.selection(value);
        this.selectionProgress(0);
        yield* this.selectionProgress(1, duration, timingFunction);
        this.selectionProgress(null);
        this.oldSelection = null;
    }
    getLineCountOfTokenArray(tokens) {
        let count = 0;
        for (const token of tokens) {
            for (let i = 0; i < token.code.length; i++) {
                if (token.code[i] === '\n') {
                    count++;
                }
            }
        }
        if (tokens.length > 0) {
            count++;
        }
        return count;
    }
    lineCount() {
        const progress = this.codeProgress();
        if (progress !== null) {
            return Math.round(map(this.currentLineCount, this.newLineCount, progress));
        }
        return this.getLineCountOfTokenArray(this.parsed());
    }
    parsed() {
        if (!CodeBlock_1.initialized()) {
            return [];
        }
        return parse(this.code(), { codeStyle: this.theme() });
    }
    constructor({ children, ...rest }) {
        super({
            fontFamily: 'monospace',
            ...rest,
        });
        this.codeProgress = createSignal(null);
        this.selectionProgress = createSignal(null);
        this.oldSelection = null;
        this.diffed = null;
        this.currentLineCount = 0;
        this.newLineCount = 0;
        if (children) {
            this.code(children);
        }
    }
    characterSize() {
        this.requestFontUpdate();
        const context = this.cacheCanvas();
        context.save();
        this.applyStyle(context);
        context.font = this.styles.font;
        const width = context.measureText('X').width;
        context.restore();
        return new Vector2(width, parseFloat(this.styles.lineHeight));
    }
    desiredSize() {
        const custom = super.desiredSize();
        const tokensSize = this.getTokensSize(this.parsed());
        return {
            x: custom.x ?? tokensSize.x,
            y: custom.y ?? tokensSize.y,
        };
    }
    getTokensSize(tokens) {
        const size = this.characterSize();
        let maxWidth = 0;
        let height = size.height;
        let width = 0;
        for (const token of tokens) {
            for (let i = 0; i < token.code.length; i++) {
                if (token.code[i] === '\n') {
                    if (width > maxWidth) {
                        maxWidth = width;
                    }
                    width = 0;
                    height += size.height;
                }
                else {
                    width += size.width;
                }
            }
        }
        if (width > maxWidth) {
            maxWidth = width;
        }
        return { x: maxWidth, y: height };
    }
    collectAsyncResources() {
        super.collectAsyncResources();
        CodeBlock_1.initialized();
    }
    set(strings, ...rest) {
        this.code({
            language: this.language(),
            spans: strings,
            nodes: rest,
        });
    }
    /**
     * Smoothly edit the code.
     *
     * @remarks
     * This method returns a tag function that should be used together with a
     * template literal to define what to edit. Expressions can be used to either
     * {@link insert}, {@link remove}, or {@link edit} the code.
     *
     * @example
     * ```ts
     * yield* codeBlock().edit()`
     *   const ${edit('a', 'b')} = [${insert('1, 2, 3')}];${remove(`
     *   // this comment will be removed`)}
     * `;
     * ```
     *
     * @param duration - The duration of the transition.
     * @param changeSelection - When set to `true`, the selection will be modified
     *                          to highlight the newly inserted code. Setting it
     *                          to `false` leaves the selection untouched.
     *                          Providing a custom {@link CodeRange} will select
     *                          it instead.
     */
    edit(duration = 0.6, changeSelection = true) {
        function* generator(strings, ...rest) {
            const from = {
                language: this.language(),
                spans: [...strings],
                nodes: rest.map(modification => isCodeModification(modification) ? modification.from : modification),
            };
            const to = {
                language: this.language(),
                spans: [...strings],
                nodes: rest.map(modification => isCodeModification(modification) ? modification.to : modification),
            };
            this.code(from);
            if (changeSelection) {
                const task = yield this.code(to, duration);
                yield* waitFor(duration * 0.2);
                yield* this.selection([], duration * 0.3);
                const newSelection = changeSelection === true
                    ? diff(from, to)
                        .filter(token => token.morph === 'create')
                        .map(token => [
                        [token.to[1], token.to[0]],
                        [token.to[1], token.to[0] + token.code.length],
                    ])
                    : changeSelection;
                yield* this.selection(newSelection, duration * 0.3);
                yield* join(task);
            }
            else {
                yield* this.code(to, duration);
            }
        }
        return generator.bind(this);
    }
    *tweenCode(code, time, timingFunction) {
        if (typeof code === 'function')
            throw new Error();
        if (!CodeBlock_1.initialized())
            return;
        const currentParsedCode = parse(this.code(), { codeStyle: this.theme() });
        const newParsedCode = parse(code, { codeStyle: this.theme() });
        this.currentLineCount = this.getLineCountOfTokenArray(currentParsedCode);
        this.newLineCount = this.getLineCountOfTokenArray(newParsedCode);
        const autoWidth = this.width.isInitial();
        const autoHeight = this.height.isInitial();
        const fromSize = this.size();
        const toSize = this.getTokensSize(newParsedCode);
        const beginning = 0.2;
        const ending = 0.8;
        this.codeProgress(0);
        this.diffed = diff(this.code(), code, { codeStyle: this.theme() });
        yield* tween(time, value => {
            const progress = timingFunction(value);
            const remapped = clampRemap(beginning, ending, 0, 1, progress);
            this.codeProgress(progress);
            if (autoWidth) {
                this.width(easeInOutSine(remapped, fromSize.x, toSize.x));
            }
            if (autoHeight) {
                this.height(easeInOutSine(remapped, fromSize.y, toSize.y));
            }
        }, () => {
            this.codeProgress(null);
            this.diffed = null;
            if (autoWidth) {
                this.width.reset();
            }
            if (autoHeight) {
                this.height.reset();
            }
            this.code(code);
        });
    }
    draw(context) {
        if (!CodeBlock_1.initialized())
            return;
        this.requestFontUpdate();
        this.applyStyle(context);
        context.font = this.styles.font;
        context.textBaseline = 'top';
        const lh = parseFloat(this.styles.lineHeight);
        const w = context.measureText('X').width;
        const size = this.computedSize();
        const progress = this.codeProgress();
        const unselectedOpacity = this.unselectedOpacity();
        const globalAlpha = context.globalAlpha;
        const getSelectionAlpha = (x, y) => map(unselectedOpacity, 1, this.selectionStrength(x, y));
        const drawToken = (code, position, alpha = 1) => {
            for (let i = 0; i < code.length; i++) {
                const char = code.charAt(i);
                if (char === '\n') {
                    position.y++;
                    position.x = 0;
                    continue;
                }
                context.globalAlpha =
                    globalAlpha * alpha * getSelectionAlpha(position.x, position.y);
                context.fillText(char, position.x * w, position.y * lh);
                position.x++;
            }
        };
        context.translate(size.x / -2, size.y / -2);
        if (progress == null) {
            const parsed = this.parsed();
            const position = { x: 0, y: 0 };
            for (const token of parsed) {
                context.save();
                context.fillStyle = token.color ?? '#c9d1d9';
                drawToken(token.code, position);
                context.restore();
            }
        }
        else {
            const diffed = this.diffed;
            const beginning = 0.2;
            const ending = 0.8;
            const overlap = 0.15;
            for (const token of diffed) {
                context.save();
                context.fillStyle = token.color ?? '#c9d1d9';
                if (token.morph === 'delete') {
                    drawToken(token.code, { x: token.from[0], y: token.from[1] }, clampRemap(0, beginning + overlap, 1, 0, progress));
                }
                else if (token.morph === 'create') {
                    drawToken(token.code, { x: token.to[0], y: token.to[1] }, clampRemap(ending - overlap, 1, 0, 1, progress));
                }
                else if (token.morph === 'retain') {
                    const remapped = clampRemap(beginning, ending, 0, 1, progress);
                    const x = easeInOutSine(remapped, token.from[0], token.to[0]);
                    const y = easeInOutSine(remapped, token.from[1], token.to[1]);
                    const point = remapped > 0.5 ? token.to : token.from;
                    let offsetX = 0;
                    let offsetY = 0;
                    for (let i = 0; i < token.code.length; i++) {
                        const char = token.code.charAt(i);
                        if (char === '\n') {
                            offsetY++;
                            offsetX = 0;
                            continue;
                        }
                        context.globalAlpha =
                            globalAlpha *
                                getSelectionAlpha(point[0] + offsetX, point[1] + offsetY);
                        context.fillText(char, (x + offsetX) * w, (y + offsetY) * lh);
                        offsetX++;
                    }
                }
                else {
                    useLogger().warn({
                        message: 'Invalid token',
                        object: token,
                    });
                }
                context.restore();
            }
        }
    }
    selectionStrength(x, y) {
        const selection = this.selection();
        const selectionProgress = this.selectionProgress();
        const isSelected = CodeBlock_1.selectionStrength(selection, x, y);
        if (selectionProgress === null || this.oldSelection === null) {
            return isSelected ? 1 : 0;
        }
        const wasSelected = CodeBlock_1.selectionStrength(this.oldSelection, x, y);
        if (isSelected === wasSelected) {
            return isSelected;
        }
        return map(wasSelected, isSelected, selectionProgress);
    }
    static selectionStrength(selection, x, y) {
        return selection.length > 0 &&
            !!selection.find(([[startLine, startColumn], [endLine, endColumn]]) => {
                return (((y === startLine && x >= startColumn) || y > startLine) &&
                    ((y === endLine && x < endColumn) || y < endLine));
            })
            ? 1
            : 0;
    }
};
CodeBlock.initialized = createComputedAsync(() => ready().then(() => true), false);
__decorate([
    initial('tsx'),
    signal()
], CodeBlock.prototype, "language", void 0);
__decorate([
    initial(''),
    parser(function (value) {
        return typeof value === 'string'
            ? {
                language: this.language(),
                spans: [value],
                nodes: [],
            }
            : value;
    }),
    signal()
], CodeBlock.prototype, "code", void 0);
__decorate([
    initial(undefined),
    signal()
], CodeBlock.prototype, "theme", void 0);
__decorate([
    initial(lines(0, Infinity)),
    signal()
], CodeBlock.prototype, "selection", void 0);
__decorate([
    initial(0.32),
    signal()
], CodeBlock.prototype, "unselectedOpacity", void 0);
__decorate([
    computed()
], CodeBlock.prototype, "lineCount", null);
__decorate([
    computed()
], CodeBlock.prototype, "parsed", null);
__decorate([
    computed()
], CodeBlock.prototype, "characterSize", null);
__decorate([
    threadable()
], CodeBlock.prototype, "tweenCode", null);
CodeBlock = CodeBlock_1 = __decorate([
    nodeName('CodeBlock')
], CodeBlock);
export { CodeBlock };
function isCodeModification(value) {
    return (value &&
        typeof value === 'object' &&
        value.from !== undefined &&
        value.to !== undefined);
}
/**
 * Create a code modification that inserts a piece of code.
 *
 * @remarks
 * Should be used in conjunction with {@link CodeBlock.edit}.
 *
 * @param content - The code to insert.
 */
export function insert(content) {
    return {
        from: '',
        to: content,
    };
}
/**
 * Create a code modification that removes a piece of code.
 *
 * @remarks
 * Should be used in conjunction with {@link CodeBlock.edit}.
 *
 * @param content - The code to remove.
 */
export function remove(content) {
    return {
        from: content,
        to: '',
    };
}
/**
 * Create a code modification that changes one piece of code into another.
 *
 * @remarks
 * Should be used in conjunction with {@link CodeBlock.edit}.
 *
 * @param from - The code to change from.
 * @param to - The code to change to.
 */
export function edit(from, to) {
    return { from, to };
}
/**
 * Create a selection range that highlights the given lines.
 *
 * @param from - The line from which the selection starts.
 * @param to - The line at which the selection ends. If omitted, the selection
 *             will cover only one line.
 */
export function lines(from, to) {
    return [
        [
            [from, 0],
            [to ?? from, Infinity],
        ],
    ];
}
/**
 * Create a selection range that highlights the given word.
 *
 * @param line - The line at which the word appears.
 * @param from - The column at which the word starts.
 * @param length - The length of the word. If omitted, the selection will cover
 *                 the rest of the line.
 */
export function word(line, from, length) {
    return [
        [
            [line, from],
            [line, from + (length ?? Infinity)],
        ],
    ];
}
/**
 * Create a custom selection range.
 *
 * @param startLine - The line at which the selection starts.
 * @param startColumn - The column at which the selection starts.
 * @param endLine - The line at which the selection ends.
 * @param endColumn - The column at which the selection ends.
 */
export function range(startLine, startColumn, endLine, endColumn) {
    return [
        [
            [startLine, startColumn],
            [endLine, endColumn],
        ],
    ];
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZUJsb2NrLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9jb21wb25lbnRzL0NvZGVCbG9jay50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQUEsT0FBTyxFQU9MLE9BQU8sRUFDUCxVQUFVLEVBQ1YsbUJBQW1CLEVBQ25CLFlBQVksRUFDWixhQUFhLEVBQ2IsSUFBSSxFQUNKLEdBQUcsRUFDSCxVQUFVLEVBQ1YsS0FBSyxFQUNMLFNBQVMsRUFDVCxPQUFPLEdBQ1IsTUFBTSxxQkFBcUIsQ0FBQztBQUM3QixPQUFPLEVBTUwsSUFBSSxFQUNKLEtBQUssRUFDTCxLQUFLLEdBQ04sTUFBTSxVQUFVLENBQUM7QUFDbEIsT0FBTyxFQUFDLFFBQVEsRUFBRSxPQUFPLEVBQUUsUUFBUSxFQUFFLE1BQU0sRUFBRSxNQUFNLEVBQUMsTUFBTSxlQUFlLENBQUM7QUFFMUUsT0FBTyxFQUFDLEtBQUssRUFBYSxNQUFNLFNBQVMsQ0FBQztBQWtCMUM7O0dBRUc7QUFFSSxJQUFNLFNBQVMsaUJBQWYsTUFBTSxTQUFVLFNBQVEsS0FBSztJQStCeEIsQ0FBQyxjQUFjLENBQ3ZCLEtBQWtCLEVBQ2xCLFFBQWdCLEVBQ2hCLGNBQThCO1FBRTlCLElBQUksQ0FBQyxZQUFZLEdBQUcsSUFBSSxDQUFDLFNBQVMsRUFBRSxDQUFDO1FBQ3JDLElBQUksQ0FBQyxTQUFTLENBQUMsS0FBSyxDQUFDLENBQUM7UUFDdEIsSUFBSSxDQUFDLGlCQUFpQixDQUFDLENBQUMsQ0FBQyxDQUFDO1FBQzFCLEtBQUssQ0FBQyxDQUFDLElBQUksQ0FBQyxpQkFBaUIsQ0FBQyxDQUFDLEVBQUUsUUFBUSxFQUFFLGNBQWMsQ0FBQyxDQUFDO1FBQzNELElBQUksQ0FBQyxpQkFBaUIsQ0FBQyxJQUFJLENBQUMsQ0FBQztRQUM3QixJQUFJLENBQUMsWUFBWSxHQUFHLElBQUksQ0FBQztJQUMzQixDQUFDO0lBYVMsd0JBQXdCLENBQUMsTUFBZTtRQUNoRCxJ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