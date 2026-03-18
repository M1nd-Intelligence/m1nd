"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isPlugin = exports.PLUGIN_OPTIONS = void 0;
exports.PLUGIN_OPTIONS = Symbol.for('@motion-canvas/vite-plugin/PLUGIN_OPTIONS');
function isPlugin(value) {
    return value && typeof value === 'object' && exports.PLUGIN_OPTIONS in value;
}
exports.isPlugin = isPlugin;
//# sourceMappingURL=plugins.js.map