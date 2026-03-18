import { highlightTree } from '@lezer/highlight';
import { DefaultHighlightStyle } from './DefaultHighlightStyle';
export class LezerHighlighter {
    constructor(parser, style = DefaultHighlightStyle) {
        this.parser = parser;
        this.style = style;
        this.classLookup = new Map();
        for (const rule of this.style.module?.getRules().split('\n') ?? []) {
            const match = rule.match(LezerHighlighter.classRegex);
            if (!match) {
                continue;
            }
            const className = match[1];
            const color = match[2].trim();
            this.classLookup.set(className, color);
        }
    }
    initialize() {
        return true;
    }
    prepare(code) {
        const colorLookup = new Map();
        const tree = this.parser.parse(code);
        highlightTree(tree, this.style, (from, to, classes) => {
            const color = this.classLookup.get(classes);
            if (!color) {
                return;
            }
            const cursor = tree.cursorAt(from, 1);
            do {
                const id = this.getNodeId(cursor.node);
                colorLookup.set(id, color);
            } while (cursor.next() && cursor.to <= to);
        });
        return {
            tree,
            code,
            colorLookup,
        };
    }
    highlight(index, cache) {
        if (!cache) {
            return {
                color: null,
                skipAhead: 0,
            };
        }
        const node = cache.tree.resolveInner(index, 1);
        const id = this.getNodeId(node);
        const color = cache.colorLookup.get(id);
        if (color) {
            return {
                color,
                skipAhead: node.to - index,
            };
        }
        let skipAhead = 0;
        if (!node.firstChild) {
            skipAhead = node.to - index;
        }
        return {
            color: null,
            skipAhead,
        };
    }
    tokenize(code) {
        const tree = this.parser.parse(code);
        const cursor = tree.cursor();
        const tokens = [];
        let current = 0;
        do {
            if (!cursor.node.firstChild) {
                if (cursor.from > current) {
                    tokens.push(code.slice(current, cursor.from));
                }
                if (cursor.from < cursor.to) {
                    tokens.push(code.slice(cursor.from, cursor.to));
                }
                current = cursor.to;
            }
        } while (cursor.next());
        return tokens;
    }
    getNodeId(node) {
        return `${node.from}:${node.to}`;
    }
}
LezerHighlighter.classRegex = /\.(\S+).*color:([^;]+)/;
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiTGV6ZXJIaWdobGlnaHRlci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29kZS9MZXplckhpZ2hsaWdodGVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUVBLE9BQU8sRUFBQyxhQUFhLEVBQUMsTUFBTSxrQkFBa0IsQ0FBQztBQUUvQyxPQUFPLEVBQUMscUJBQXFCLEVBQUMsTUFBTSx5QkFBeUIsQ0FBQztBQVE5RCxNQUFNLE9BQU8sZ0JBQWdCO0lBSTNCLFlBQ21CLE1BQWMsRUFDZCxRQUF3QixxQkFBcUI7UUFEN0MsV0FBTSxHQUFOLE1BQU0sQ0FBUTtRQUNkLFVBQUssR0FBTCxLQUFLLENBQXdDO1FBSi9DLGdCQUFXLEdBQUcsSUFBSSxHQUFHLEVBQWtCLENBQUM7UUFNdkQsS0FBSyxNQUFNLElBQUksSUFBSSxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sRUFBRSxRQUFRLEVBQUUsQ0FBQyxLQUFLLENBQUMsSUFBSSxDQUFDLElBQUksRUFBRSxFQUFFO1lBQ2xFLE1BQU0sS0FBSyxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsZ0JBQWdCLENBQUMsVUFBVSxDQUFDLENBQUM7WUFDdEQsSUFBSSxDQUFDLEtBQUssRUFBRTtnQkFDVixTQUFTO2FBQ1Y7WUFFRCxNQUFNLFNBQVMsR0FBRyxLQUFLLENBQUMsQ0FBQyxDQUFDLENBQUM7WUFDM0IsTUFBTSxLQUFLLEdBQUcsS0FBSyxDQUFDLENBQUMsQ0FBQyxDQUFDLElBQUksRUFBRSxDQUFDO1lBQzlCLElBQUksQ0FBQyxXQUFXLENBQUMsR0FBRyxDQUFDLFNBQVMsRUFBRSxLQUFLLENBQUMsQ0FBQztTQUN4QztJQUNILENBQUM7SUFFTSxVQUFVO1FBQ2YsT0FBTyxJQUFJLENBQUM7SUFDZCxDQUFDO0lBRU0sT0FBTyxDQUFDLElBQVk7UUFDekIsTUFBTSxXQUFXLEdBQUcsSUFBSSxHQUFHLEVBQWtCLENBQUM7UUFDOUMsTUFBTSxJQUFJLEdBQUcsSUFBSSxDQUFD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