import './index.css';
import { useApplication, useCurrentScene, Pane, Separator, Group, Label, Button, findAndOpenFirstUserFile, UnknownField, AutoField, useViewportContext, useViewportMatrix, OverlayWrapper, MouseButton, Toggle, Collapse, usePanels, useReducedMotion, emphasize, Tab, AccountTree, makeEditorPlugin } from '@motion-canvas/ui';
import { jsx, jsxs, Fragment } from 'preact/jsx-runtime';
import { signal, computed, useSignalEffect, useComputed, useSignal } from '@preact/signals';
import { SceneRenderEvent, Vector2 } from '@motion-canvas/core';
import { createContext } from 'preact';
import { useMemo, useContext, useRef, useEffect } from 'preact/hooks';
import { NODE_NAME } from '@motion-canvas/2d';

const PluginContext = createContext(null);
const NodeInspectorKey = '@motion-canvas/2d/node-inspector';
function usePluginState() {
    return useContext(PluginContext);
}
function Provider({ children }) {
    const { inspection } = useApplication();
    const currentScene = useCurrentScene();
    const state = useMemo(() => {
        const scene = signal(currentScene);
        const selectedKey = signal(null);
        const afterRender = signal(0);
        const hoveredKey = signal(null);
        const openNodes = new Map();
        const selectedChain = computed(() => {
            const chain = new Set();
            const key = selectedKey.value;
            const selectedNode = scene.value?.getNode(key);
            if (selectedNode) {
                let node = selectedNode.parent() ?? null;
                while (node) {
                    chain.add(node.key);
                    node = node.parent();
                }
            }
            return chain;
        });
        return {
            selectedKey,
            hoveredKey,
            afterRender,
            openNodes,
            selectedChain,
            scene,
        };
    }, []);
    state.scene.value = currentScene;
    useSignalEffect(() => state.scene.value?.onRenderLifecycle.subscribe(([event]) => {
        if (event === SceneRenderEvent.AfterRender) {
            state.afterRender.value++;
        }
    }));
    useSignalEffect(() => {
        const { key, payload } = inspection.value;
        if (key === NodeInspectorKey) {
            state.selectedKey.value = payload;
        }
    });
    useSignalEffect(() => {
        const nodeKey = state.selectedKey.value;
        const { key, payload } = inspection.peek();
        if (key === NodeInspectorKey && !nodeKey) {
            inspection.value = { key: '', payload: null };
        }
        else if (payload !== nodeKey) {
            inspection.value = { key: NodeInspectorKey, payload: nodeKey };
        }
    });
    return (jsx(PluginContext.Provider, { value: state, children: children }));
}

function Component$1() {
    const { inspection } = useApplication();
    const { scene, afterRender } = usePluginState();
    const node = useComputed(() => {
        afterRender.value;
        const { payload } = inspection.value;
        return scene.value?.getNode(payload);
    });
    const attributes = useComputed(() => {
        afterRender.value;
        const currentNode = node.value;
        const attributes = [];
        if (currentNode) {
            for (const { key, meta, signal } of currentNode) {
                if (!meta.inspectable)
                    continue;
                attributes.push([key, signal()]);
            }
        }
        return attributes;
    });
    const stack = node.value?.creationStack;
    return (jsxs(Pane, { title: "Node Inspector", id: "node-inspector-pane", children: [jsx(Separator, { size: 1 }), stack && (jsxs(Group, { children: [jsx(Label, {}), jsx(Button, { onClick: () => findAndOpenFirstUserFile(stack), main: true, children: "GO TO SOURCE" })] })), jsxs(Group, { children: [jsx(Label, { children: "key" }), jsx(UnknownField, { value: inspection.value.payload })] }), !node.value && (jsxs(Group, { children: [jsx(Label, {}), "Couldn't find the node. It may have been deleted or doesn't exist yet."] })), attributes.value.map(([key, value]) => (jsxs(Group, { children: [jsx(Label, { children: key }), jsx(AutoField, { value: value })] }, key)))] }));
}
const NodeInspectorConfig = {
    key: NodeInspectorKey,
    component: Component$1,
};

function Component({ children }) {
    const state = useViewportContext();
    const { scene, selectedKey } = usePluginState();
    const matrix = useViewportMatrix();
    return (jsx(OverlayWrapper, { onPointerDown: event => {
            if (event.button !== MouseButton.Left || event.shiftKey)
                return;
            if (!scene.value)
                return;
            event.stopPropagation();
            const position = new Vector2(event.x - state.rect.x, event.y - state.rect.y).transformAsPoint(matrix.inverse());
            selectedKey.value = scene.value.inspectPosition(position.x, position.y);
        }, children: children }));
}
function drawHook() {
    const { selectedKey, hoveredKey, afterRender, scene } = usePluginState();
    selectedKey.value;
    hoveredKey.value;
    afterRender.value;
    return (ctx, matrix) => {
        const currentScene = scene.peek();
        if (!currentScene)
            return;
        let node = currentScene.getNode(selectedKey.value);
        if (node) {
            currentScene.drawOverlay(node.key, matrix, ctx);
        }
        node = currentScene.getNode(hoveredKey.value);
        if (node && hoveredKey.value !== selectedKey.value) {
            ctx.globalAlpha = 0.5;
            currentScene.drawOverlay(hoveredKey.value, matrix, ctx);
        }
    };
}
const PreviewOverlayConfig = {
    drawHook,
    component: Component,
};

function CircleIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M10,5C12.76,5 15,7.24 15,10C15,12.76 12.76,15 10,15C7.24,15 5,12.76 5,10C5,7.24 7.24,5 10,5ZM10,7C8.344,7 7,8.344 7,10C7,11.656 8.344,13 10,13C11.656,13 13,11.656 13,10C13,8.344 11.656,7 10,7Z" }) }));
}

function CodeBlockIcon() {
    return (jsxs("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: [jsx("path", { d: "M5,9L5,6.999C5,6.469 5.211,5.96 5.585,5.586C5.96,5.211 6.469,5 6.999,5L9,5L9,7L6.999,7L7,9L7,11L7,13L8.985,13L9,15L7,15C5.895,15 5,14.105 5,13L5,11L4,11L4,9L5,9Z" }), jsx("path", { d: "M15,11L15,13.001C15,13.531 14.789,14.04 14.415,14.414C14.04,14.789 13.531,15 13.001,15L11,15L11,13L13,13L13,11L13,9L13,7L11.015,7L11,5L13,5C14.105,5 15,5.895 15,7L15,9L16,9L16,11L15,11Z" })] }));
}

function CurveIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M12.19,6.47L13.595,5.047C15.519,6.947 15.187,8.932 14.229,9.951C13.675,10.541 12.879,10.861 12.016,10.767C11.261,10.685 10.426,10.278 9.708,9.348C9.292,8.809 8.878,8.441 8.471,8.249C8.217,8.13 7.979,8.084 7.77,8.154C7.565,8.222 7.409,8.394 7.287,8.621C7.097,8.975 7.001,9.444 7,10.003C6.996,11.584 7.848,12.746 8.91,12.946C9.535,13.064 10.185,12.783 10.687,12.082L12.313,13.247C11,15.079 9.118,15.344 7.581,14.591C6.161,13.896 4.994,12.246 5,9.997C5.005,7.945 5.963,6.649 7.136,6.257C8.281,5.874 9.866,6.278 11.292,8.126C11.81,8.799 12.421,8.954 12.772,8.581C13.196,8.13 13.042,7.312 12.19,6.47Z" }) }));
}

function GridIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M6,8L4,8L4,6L6,6L6,4L8,4L8,6L12,6L12,4L14,4L14,6L16,6L16,8L14,8L14,12L16,12L16,14L14,14L14,16L12,16L12,14L8,14L8,16L6,16L6,14L4,14L4,12L6,12L6,8ZM8,12L12,12L12,8L8,8L8,12Z" }) }));
}

function ImgIcon() {
    return (jsxs("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: [jsx("path", { d: "M5,15L15,15L15,10L13,8L8,13L5,10L5,15Z" }), jsx("circle", { cx: "8", cy: "7", r: "2" })] }));
}

function LayoutIcon() {
    return (jsxs("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: [jsx("path", { d: "M14,5C14.552,5 15,5.448 15,6C15,7.916 15,12.084 15,14C15,14.552 14.552,15 14,15C12.815,15 11,15 11,15L11,5L14,5Z" }), jsx("path", { d: "M9,5L9,9L5,9L5,6C5,5.448 5.448,5 6,5L9,5Z" }), jsx("path", { d: "M9,11L9,15L6,15C5.448,15 5,14.552 5,14L5,11L9,11Z" })] }));
}

function LineIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M9.906,4.589L11.411,5.906L8.529,9.2L13.859,8.439C14.273,8.379 14.68,8.584 14.879,8.952C15.078,9.319 15.028,9.772 14.753,10.087L10.094,15.411L8.589,14.094L11.471,10.8L6.141,11.561C5.727,11.621 5.32,11.416 5.121,11.048C4.922,10.681 4.972,10.228 5.247,9.913L9.906,4.589Z" }) }));
}

function NodeIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M7,9L5,9L5,7L7,7L7,5L9,5L9,7L12,7L12,5L15,8L12,11L12,9L9,9L9,12L11,12L8,15L5,12L7,12L7,9Z" }) }));
}

function RayIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M12,9.414L6.707,14.707L5.293,13.293L10.586,8L8,8L8,6L13,6C13.552,6 14,6.448 14,7L14,12L12,12L12,9.414Z" }) }));
}

function RectIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M15,6L15,14C15,14.552 14.552,15 14,15L6,15C5.448,15 5,14.552 5,14L5,6C5,5.448 5.448,5 6,5L14,5C14.552,5 15,5.448 15,6ZM13,7L7,7L7,13L13,13L13,7Z" }) }));
}

function ShapeIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M11.746,10.93C12.637,12.664 11.973,14.504 10.611,15.244C9.692,15.743 8.385,15.804 6.94,14.829C5.555,13.893 4.689,12.16 4.544,10.388C4.395,8.572 5,6.752 6.399,5.701C8.069,4.445 10.793,4.271 12.765,4.921C14.324,5.436 15.374,6.473 15.495,7.691C15.651,9.262 14.613,10.061 13.26,10.5C12.847,10.634 12.41,10.735 12.02,10.841C11.936,10.864 11.838,10.897 11.746,10.93ZM7.601,7.299C6.737,7.949 6.445,9.103 6.537,10.224C6.633,11.389 7.149,12.556 8.06,13.171C8.696,13.601 9.251,13.706 9.656,13.486C10.207,13.187 10.315,12.395 9.886,11.701C9.48,11.044 9.513,10.523 9.68,10.122C9.835,9.75 10.164,9.417 10.678,9.187C11.243,8.935 12.157,8.8 12.908,8.503C13.216,8.381 13.542,8.264 13.505,7.888C13.485,7.691 13.359,7.53 13.197,7.384C12.928,7.143 12.558,6.959 12.138,6.821C10.736,6.358 8.789,6.406 7.601,7.299Z" }) }));
}

function TxtIcon() {
    return (jsxs("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: [jsx("path", { d: "M9,13L9,6L11,6L11,13L12,13L12,15L8,15L8,13L9,13Z" }), jsx("path", { d: "M7,8L5,8L5,6C5,5.448 5.448,5 6,5L14,5C14.552,5 15,5.448 15,6L15,8L13,8L13,7L7,7L7,8Z" })] }));
}

function VideoIcon() {
    return (jsx("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: jsx("path", { d: "M14,10.866L7.25,14.763C6.941,14.942 6.559,14.942 6.25,14.763C5.941,14.585 5.75,14.254 5.75,13.897L5.75,6.103C5.75,5.746 5.941,5.415 6.25,5.237C6.559,5.058 6.941,5.058 7.25,5.237L14,9.134C14.309,9.313 14.5,9.643 14.5,10C14.5,10.357 14.309,10.687 14,10.866ZM11.5,10L7.75,7.835L7.75,12.165L11.5,10Z" }) }));
}

function View2DIcon() {
    return (jsxs("svg", { viewBox: "0 0 20 20", fill: "currentColor", children: [jsx("path", { d: "M9,5L9,7L7,7L7,9L5,9L5,6C5,5.448 5.448,5 6,5L9,5Z" }), jsx("path", { d: "M5,11L7,11L7,13L9,13L9,15L6,15C5.448,15 5,14.552 5,14L5,11Z" }), jsx("path", { d: "M11,15L11,13L13,13L13,11L15,11L15,14C15,14.552 14.552,15 14,15L11,15Z" }), jsx("path", { d: "M15,9L13,9L13,7L11,7L11,5L14,5C14.552,5 15,5.448 15,6L15,9Z" })] }));
}

/* eslint-disable @typescript-eslint/naming-convention */
const IconMap = {
    Circle: CircleIcon,
    CodeBlock: CodeBlockIcon,
    Curve: CurveIcon,
    Grid: GridIcon,
    Img: ImgIcon,
    Layout: LayoutIcon,
    Line: LineIcon,
    Node: NodeIcon,
    Ray: RayIcon,
    Rect: RectIcon,
    Shape: ShapeIcon,
    Txt: TxtIcon,
    TxtLeaf: TxtIcon,
    Video: VideoIcon,
    View2D: View2DIcon,
};

function r(e){var t,f,n="";if("string"==typeof e||"number"==typeof e)n+=e;else if("object"==typeof e)if(Array.isArray(e)){var o=e.length;for(t=0;t<o;t++)e[t]&&(f=r(e[t]))&&(n&&(n+=" "),n+=f);}else for(f in e)e[f]&&(n&&(n+=" "),n+=f);return n}function clsx(){for(var e,t,f=0,n="",o=arguments.length;f<o;f++)(e=arguments[f])&&(t=r(e))&&(n&&(n+=" "),n+=t);return n}

var styles = {"root":"index-module_root__omEd0","label":"index-module_label__9BJvW","active":"index-module_active__KevXv","parent":"index-module_parent__5nc9I"};

const DEPTH_VAR = '--depth';
function TreeElement({ label, children, selected, depth = 0, open, icon, forwardRef, ...props }) {
    const hasChildren = !!children;
    return (jsxs(Fragment, { children: [jsxs("div", { ref: forwardRef, className: clsx(styles.label, selected && styles.active, hasChildren && styles.parent), onDblClick: () => {
                    if (hasChildren) {
                        open.value = !open.value;
                    }
                }, ...props, style: { [DEPTH_VAR]: `${depth}` }, children: [hasChildren && (jsx(Toggle, { animated: false, open: open.value, onToggle: value => {
                            open.value = value;
                        }, onDblClick: e => {
                            e.stopPropagation();
                        } })), icon, label] }), hasChildren && (jsx(Collapse, { open: open.value, animated: false, children: children }))] }));
}

function NodeElement({ node, depth = 0 }) {
    const { selectedKey, hoveredKey, openNodes, selectedChain, afterRender } = usePluginState();
    const ref = useRef(null);
    const open = useSignal(selectedChain.peek().has(node.key) || (openNodes.get(node.key) ?? false));
    const nodeSignal = useSignal(node);
    nodeSignal.value = node;
    const children = useComputed(() => {
        afterRender.value;
        return nodeSignal.value.peekChildren();
    });
    useSignalEffect(() => {
        open.value = openNodes.get(nodeSignal.value.key) ?? false;
    });
    useSignalEffect(() => {
        const chain = selectedChain.value;
        if (chain.has(nodeSignal.value.key)) {
            open.value = true;
        }
    });
    useSignalEffect(() => {
        openNodes.set(nodeSignal.value.key, open.value);
    });
    useSignalEffect(() => {
        const key = selectedKey.value;
        if (node.key === key) {
            ref.current?.scrollIntoView({ block: 'nearest', behavior: 'instant' });
        }
    });
    const Icon = IconMap[node[NODE_NAME]] ?? IconMap.Node;
    return (jsx(TreeElement, { forwardRef: ref, open: open, depth: depth, icon: jsx(Icon, {}), label: node.key, selected: selectedKey.value === node.key, onClick: event => {
            selectedKey.value = node.key;
            event.stopPropagation();
        }, onPointerEnter: () => (hoveredKey.value = node.key), onPointerLeave: () => (hoveredKey.value = null), children: children.value.length > 0 &&
            children.value.map(child => (jsx(NodeElement, { node: child, depth: depth + 1 }))) }));
}

function TreeRoot({ className, ...props }) {
    return jsx("div", { className: clsx(styles.root, className), ...props });
}

function DetachedRoot() {
    const { afterRender, scene } = usePluginState();
    const open = useSignal(false);
    const currentScene = scene.value;
    const children = currentScene ? [...currentScene.getDetachedNodes()] : [];
    afterRender.value;
    return children.length > 0 ? (jsx(TreeRoot, { children: jsx(TreeElement, { open: open, label: "Detached nodes", children: children.map(child => (jsx(NodeElement, { node: child, depth: 1 }))) }) })) : null;
}

function ViewRoot() {
    const { scene } = usePluginState();
    const view = useSignal(scene.value?.getView());
    useSignalEffect(() => {
        view.value = scene.value?.getView();
        return scene.value?.onReset.subscribe(() => {
            view.value = scene.value?.getView();
        });
    });
    return view.value ? (jsx(TreeRoot, { children: jsx(NodeElement, { node: view.value }) })) : null;
}

function TabComponent({ tab }) {
    const { sidebar } = usePanels();
    const inspectorTab = useRef(null);
    const reducedMotion = useReducedMotion();
    const { selectedKey } = usePluginState();
    const { logger } = useApplication();
    useEffect(() => logger.onInspected.subscribe(key => {
        sidebar.set(tab);
        selectedKey.value = key;
    }), [tab]);
    useSignalEffect(() => {
        if (selectedKey.value &&
            sidebar.current.peek() !== tab &&
            !reducedMotion &&
            inspectorTab.current &&
            inspectorTab.current.getAnimations().length < 2) {
            inspectorTab.current.animate(emphasize(), { duration: 400 });
            inspectorTab.current.animate([{ color: 'white' }, { color: '' }], {
                duration: 800,
            });
        }
    });
    return (jsx(Tab, { forwardRef: inspectorTab, title: "Scene Graph", id: "scene-graph-tab", tab: tab, children: jsx(AccountTree, {}) }));
}
function PaneComponent() {
    const { selectedKey } = usePluginState();
    return (jsxs(Pane, { title: "Scene Graph", id: "scene-graph-pane", onClick: () => {
            selectedKey.value = null;
        }, children: [jsx(ViewRoot, {}), jsx(DetachedRoot, {})] }));
}
const SceneGraphTabConfig = {
    name: 'scene-graph',
    tabComponent: TabComponent,
    paneComponent: PaneComponent,
};

var index = makeEditorPlugin(() => {
    return {
        name: '@motion-canvas/2d',
        provider: Provider,
        previewOverlay: PreviewOverlayConfig,
        tabs: [SceneGraphTabConfig],
        inspectors: [NodeInspectorConfig],
    };
});

export { index as default };
//# sourceMappingURL=index.js.map
