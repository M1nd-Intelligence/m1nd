import { jsx as _jsx } from "@motion-canvas/2d/src/lib/jsx-runtime";
/**
 * Create a higher order component with default props.
 *
 * @example
 * ```tsx
 * const MyTxt = withDefaults(Txt, {
 *   fill: '#f3303f',
 * });
 *
 * // ...
 *
 * view.add(<MyTxt>Hello, World!</MyTxt>);
 * ```
 *
 * @param component - The base class or function component to wrap.
 * @param defaults - The default props to apply.
 */
export function withDefaults(component, defaults) {
    const Node = component;
    return (props) => _jsx(Node, { ...defaults, ...props });
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid2l0aERlZmF1bHRzLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi91dGlscy93aXRoRGVmYXVsdHMudHN4Il0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7QUFFQTs7Ozs7Ozs7Ozs7Ozs7OztHQWdCRztBQUNILE1BQU0sVUFBVSxZQUFZLENBQzFCLFNBQVksRUFDWixRQUFvQjtJQUVwQixNQUFNLElBQUksR0FBRyxTQUFTLENBQUM7SUFDdkIsT0FBTyxDQUFDLEtBQWlCLEVBQUUsRUFBRSxDQUFDLEtBQUMsSUFBSSxPQUFLLFFBQVEsS0FBTSxLQUFLLEdBQUksQ0FBQztBQUNsRSxDQUFDIn0=