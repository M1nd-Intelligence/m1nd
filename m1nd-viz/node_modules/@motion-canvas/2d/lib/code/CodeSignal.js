import { createSignal, deepLerp, DependencyContext, SignalContext, unwrap, } from '@motion-canvas/core';
import { Code } from '../components';
import { addInitializer, getPropertyMetaOrCreate } from '../decorators';
import { defaultDiffer } from './CodeDiffer';
import { insert, replace } from './CodeFragment';
import { CODE, parseCodeScope, resolveCodeTag, } from './CodeScope';
import { defaultTokenize } from './CodeTokenizer';
import { extractRange } from './extractRange';
export class CodeSignalContext extends SignalContext {
    constructor(initial, owner, highlighter) {
        super(initial, deepLerp, owner);
        this.highlighter = highlighter;
        this.progress = createSignal(0);
        if (owner instanceof Code) {
            this.highlighter ?? (this.highlighter = owner.highlighter);
        }
        Object.defineProperty(this.invokable, 'edit', {
            value: this.edit.bind(this),
        });
        Object.defineProperty(this.invokable, 'append', {
            value: this.append.bind(this),
        });
        Object.defineProperty(this.invokable, 'prepend', {
            value: this.prepend.bind(this),
        });
        Object.defineProperty(this.invokable, 'insert', {
            value: this.insert.bind(this),
        });
        Object.defineProperty(this.invokable, 'remove', {
            value: this.remove.bind(this),
        });
        Object.defineProperty(this.invokable, 'replace', {
            value: this.replace.bind(this),
        });
    }
    *tweener(value, duration, timingFunction) {
        let tokenize = defaultTokenize;
        const highlighter = unwrap(this.highlighter);
        if (highlighter) {
            yield (async () => {
                do {
                    await DependencyContext.consumePromises();
                    highlighter.initialize();
                } while (DependencyContext.hasPromises());
            })();
            tokenize = (input) => highlighter.tokenize(input);
        }
        this.progress(0);
        this.set({
            progress: this.progress,
            fragments: defaultDiffer(this.get(), this.parse(unwrap(value)), tokenize),
        });
        yield* this.progress(1, duration, timingFunction);
        this.set(value);
    }
    edit(duration = 0.6) {
        return (strings, ...tags) => this.editTween(CODE(strings, ...tags), duration);
    }
    append(first = 0.6, duration) {
        if (typeof first !== 'undefined' && typeof first !== 'number') {
            if (duration === undefined) {
                const current = this.get();
                return this.set({
                    progress: 0,
                    fragments: [...current.fragments, first],
                });
            }
            else {
                return this.appendTween(first, duration);
            }
        }
        const savedDuration = first;
        return (strings, ...tags) => this.append(CODE(strings, ...tags), savedDuration);
    }
    prepend(first = 0.6, duration) {
        if (typeof first !== 'undefined' && typeof first !== 'number') {
            if (duration === undefined) {
                const current = this.get();
                return this.set({
                    progress: 0,
                    fragments: [first, ...current.fragments],
                });
            }
            else {
                return this.prependTween(first, duration);
            }
        }
        const savedDuration = first;
        return (strings, ...tags) => this.prepend(CODE(strings, ...tags), savedDuration);
    }
    insert(point, first = 0.6, duration) {
        return this.replace([point, point], first, duration);
    }
    remove(range, duration) {
        return this.replace(range, '', duration);
    }
    replace(range, first = 0.6, duration) {
        if (typeof first !== 'undefined' && typeof first !== 'number') {
            if (duration === undefined) {
                const current = this.get();
                const [fragments, index] = extractRange(range, current.fragments);
                fragments[index] = first;
                return this.set({
                    progress: current.progress,
                    fragments,
                });
            }
            else {
                return this.replaceTween(range, first, duration);
            }
        }
        const savedDuration = first;
        return (strings, ...tags) => this.replaceTween(range, CODE(strings, ...tags), savedDuration);
    }
    *replaceTween(range, code, duration) {
        let current = this.get();
        const [fragments, index] = extractRange(range, current.fragments);
        const progress = createSignal(0);
        const resolved = resolveCodeTag(code, true);
        const scope = {
            progress,
            fragments: [replace(fragments[index], resolved)],
        };
        fragments[index] = scope;
        this.set({
            progress: current.progress,
            fragments,
        });
        yield* progress(1, duration);
        current = this.get();
        this.set({
            progress: current.progress,
            fragments: current.fragments.map(fragment => fragment === scope ? code : fragment),
        });
        progress.context.dispose();
    }
    *editTween(value, duration) {
        this.progress(0);
        this.set({
            progress: this.progress,
            fragments: value,
        });
        yield* this.progress(1, duration);
        const current = this.get();
        this.set({
            progress: 0,
            fragments: current.fragments.map(fragment => value.includes(fragment) ? resolveCodeTag(fragment, true) : fragment),
        });
    }
    *appendTween(value, duration) {
        let current = this.get();
        const progress = createSignal(0);
        const resolved = resolveCodeTag(value, true);
        const scope = {
            progress,
            fragments: [insert(resolved)],
        };
        this.set({
            progress: current.progress,
            fragments: [...current.fragments, scope],
        });
        yield* progress(1, duration);
        current = this.get();
        this.set({
            progress: current.progress,
            fragments: current.fragments.map(fragment => fragment === scope ? value : fragment),
        });
        progress.context.dispose();
    }
    *prependTween(value, duration) {
        let current = this.get();
        const progress = createSignal(0);
        const resolved = resolveCodeTag(value, true);
        const scope = {
            progress,
            fragments: [insert(resolved)],
        };
        this.set({
            progress: current.progress,
            fragments: [scope, ...current.fragments],
        });
        yield* progress(1, duration);
        current = this.get();
        this.set({
            progress: current.progress,
            fragments: current.fragments.map(fragment => fragment === scope ? value : fragment),
        });
        progress.context.dispose();
    }
    parse(value) {
        return parseCodeScope(value);
    }
    toSignal() {
        return this.invokable;
    }
}
export function codeSignal() {
    return (target, key) => {
        const meta = getPropertyMetaOrCreate(target, key);
        addInitializer(target, (instance) => {
            instance[key] = new CodeSignalContext(meta.default ?? [], instance).toSignal();
        });
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZVNpZ25hbC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29kZS9Db2RlU2lnbmFsLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFDTCxZQUFZLEVBQ1osUUFBUSxFQUNSLGlCQUFpQixFQUVqQixhQUFhLEVBSWIsTUFBTSxHQUNQLE1BQU0scUJBQXFCLENBQUM7QUFDN0IsT0FBTyxFQUFDLElBQUksRUFBQyxNQUFNLGVBQWUsQ0FBQztBQUNuQyxPQUFPLEVBQUMsY0FBYyxFQUFFLHVCQUF1QixFQUFDLE1BQU0sZUFBZSxDQUFDO0FBQ3RFLE9BQU8sRUFBQyxhQUFhLEVBQUMsTUFBTSxjQUFjLENBQUM7QUFDM0MsT0FBTyxFQUFDLE1BQU0sRUFBRSxPQUFPLEVBQUMsTUFBTSxnQkFBZ0IsQ0FBQztBQUcvQyxPQUFPLEVBQ0wsSUFBSSxFQUdKLGNBQWMsRUFFZCxjQUFjLEdBQ2YsTUFBTSxhQUFhLENBQUM7QUFDckIsT0FBTyxFQUFDLGVBQWUsRUFBQyxNQUFNLGlCQUFpQixDQUFDO0FBQ2hELE9BQU8sRUFBQyxZQUFZLEVBQUMsTUFBTSxnQkFBZ0IsQ0FBQztBQStDNUMsTUFBTSxPQUFPLGlCQUNYLFNBQVEsYUFBbUQ7SUFLM0QsWUFDRSxPQUF1QyxFQUN2QyxLQUFhLEVBQ0ksV0FBaUQ7UUFFbEUsS0FBSyxDQUFDLE9BQU8sRUFBRSxRQUFRLEVBQUUsS0FBSyxDQUFDLENBQUM7UUFGZixnQkFBVyxHQUFYLFdBQVcsQ0FBc0M7UUFMbkQsYUFBUSxHQUFHLFlBQVksQ0FBQyxDQUFDLENBQUMsQ0FBQztRQVExQyxJQUFJLEtBQUssWUFBWSxJQUFJLEVBQUU7WUFDekIsSUFBSSxDQUFDLFdBQVcsS0FBaEIsSUFBSSxDQUFDLFdBQVcsR0FBSyxLQUFLLENBQUMsV0FBVyxFQUFDO1NBQ3hDO1FBQ0QsTUFBTSxDQUFDLGNBQWMsQ0FBQyxJQUFJLENBQUMsU0FBUyxFQUFFLE1BQU0sRUFBRTtZQUM1QyxL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