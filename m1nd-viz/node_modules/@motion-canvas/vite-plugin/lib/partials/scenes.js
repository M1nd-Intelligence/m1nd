"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.scenesPlugin = void 0;
const path_1 = __importDefault(require("path"));
const utils_1 = require("../utils");
const SCENE_QUERY_REGEX = /[?&]scene\b/;
function scenesPlugin() {
    return {
        name: 'motion-canvas:scene',
        async load(id) {
            if (!SCENE_QUERY_REGEX.test(id)) {
                return;
            }
            const [base] = id.split('?');
            const { name, dir } = path_1.default.posix.parse(base);
            const metaFile = `${name}.meta`;
            await (0, utils_1.createMeta)(path_1.default.join(dir, metaFile));
            const sceneFile = `${name}`;
            /* language=typescript */
            return `\
import {ValueDispatcher} from '@motion-canvas/core';
import metaFile from './${metaFile}';
import description from './${sceneFile}';
description.name = '${name}';
metaFile.attach(description.meta);
if (import.meta.hot) {
  description.onReplaced = import.meta.hot.data.onReplaced;
}
description.onReplaced ??= new ValueDispatcher(description.config);
if (import.meta.hot) {
  import.meta.hot.accept();
  if (import.meta.hot.data.onReplaced) {
    description.onReplaced.current = description;
  } else {
    import.meta.hot.data.onReplaced = description.onReplaced;
  }
}
export default description;
`;
        },
    };
}
exports.scenesPlugin = scenesPlugin;
//# sourceMappingURL=scenes.js.map