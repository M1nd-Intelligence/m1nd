"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.projectsPlugin = void 0;
const path_1 = __importDefault(require("path"));
const utils_1 = require("../utils");
const versions_1 = require("../versions");
const PROJECT_QUERY_REGEX = /[?&]project\b/;
function projectsPlugin({ buildForEditor, plugins, projects, }) {
    const versions = JSON.stringify((0, versions_1.getVersions)());
    let config;
    return {
        name: 'motion-canvas:project',
        configResolved(resolvedConfig) {
            config = resolvedConfig;
        },
        async load(id) {
            if (!PROJECT_QUERY_REGEX.test(id)) {
                return;
            }
            const [base] = id.split('?');
            const { name, dir } = path_1.default.posix.parse(base);
            const runsInEditor = buildForEditor || config.command === 'serve';
            const metaFile = `${name}.meta`;
            await (0, utils_1.createMeta)(path_1.default.join(dir, metaFile));
            const imports = [];
            const pluginNames = [];
            let index = 0;
            for (const plugin of plugins) {
                if (plugin.entryPoint) {
                    const pluginName = `plugin${index}`;
                    let options = (await plugin.runtimeConfig?.()) ?? '';
                    if (typeof options !== 'string') {
                        options = JSON.stringify(options);
                    }
                    imports.push(`import ${pluginName} from '${plugin.entryPoint}'`);
                    pluginNames.push(`${pluginName}(${options})`);
                    index++;
                }
            }
            /* language=typescript */
            return `\
${imports.join('\n')}
import {${runsInEditor ? 'editorBootstrap' : 'bootstrap'}} from '@motion-canvas/core';
import {MetaFile} from '@motion-canvas/core';
        import metaFile from './${metaFile}';
        import config from './${name}';
        import settings from 'virtual:settings.meta';
        export default ${runsInEditor ? 'await editorBootstrap' : 'bootstrap'}(
          '${name}',
          ${versions},
          [${pluginNames.join(', ')}],
          config,
          metaFile,
          settings,
        );`;
        },
        config(config) {
            return {
                build: {
                    target: buildForEditor ? 'esnext' : 'modules',
                    assetsDir: './',
                    rollupOptions: {
                        preserveEntrySignatures: 'strict',
                        input: Object.fromEntries(projects.map(project => [
                            project.name,
                            project.filePath + '?project',
                        ])),
                    },
                },
                server: {
                    port: config?.server?.port ?? 9000,
                },
                esbuild: {
                    jsx: 'automatic',
                    jsxImportSource: '@motion-canvas/2d/lib',
                },
                optimizeDeps: {
                    entries: projects.map(project => project.filePath),
                    exclude: ['preact', 'preact/*', '@preact/signals'],
                },
            };
        },
    };
}
exports.projectsPlugin = projectsPlugin;
//# sourceMappingURL=projects.js.map