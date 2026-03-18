import type { ComponentChildren, FunctionComponent, JSXProps, Node, NodeConstructor } from './components';
export declare namespace JSX {
    type Element = Node;
    type ElementClass = Node;
    interface ElementChildrenAttribute {
        children: any;
    }
}
export declare const Fragment: unique symbol;
export declare function jsx(type: NodeConstructor | FunctionComponent | typeof Fragment, config: JSXProps, key?: any): ComponentChildren;
export { jsx as jsxs };
//# sourceMappingURL=jsx-runtime.d.ts.map