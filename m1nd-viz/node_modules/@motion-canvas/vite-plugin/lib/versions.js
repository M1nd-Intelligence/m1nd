"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getVersions = void 0;
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
function getVersions() {
    return {
        core: loadVersion('@motion-canvas/core'),
        two: loadVersion('@motion-canvas/2d'),
        ui: loadVersion('@motion-canvas/ui'),
        vitePlugin: loadVersion('..'),
    };
}
exports.getVersions = getVersions;
function loadVersion(module) {
    try {
        const modulePath = path_1.default.dirname(require.resolve(`${module}/package.json`));
        const packageJsonPath = path_1.default.resolve(modulePath, 'package.json');
        const packageJson = JSON.parse(fs_1.default.readFileSync(packageJsonPath).toString());
        return packageJson.version ?? null;
    }
    catch (_) {
        return null;
    }
}
//# sourceMappingURL=versions.js.map