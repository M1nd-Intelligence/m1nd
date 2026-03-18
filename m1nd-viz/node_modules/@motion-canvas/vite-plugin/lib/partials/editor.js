"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.editorPlugin = void 0;
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
function editorPlugin({ editor, projects }) {
    const editorPath = path_1.default.dirname(require.resolve(editor));
    const editorFile = fs_1.default.readFileSync(path_1.default.resolve(editorPath, 'editor.html'));
    const htmlParts = editorFile
        .toString()
        .replace('{{style}}', `/@fs/${path_1.default.resolve(editorPath, 'style.css')}`)
        .split('{{source}}');
    const createHtml = (src) => htmlParts[0] + src + htmlParts[1];
    const resolvedEditorId = '\0virtual:editor';
    const lookup = new Map();
    for (const project of projects) {
        lookup.set(project.url, project);
    }
    return {
        name: 'motion-canvas:editor',
        async load(id) {
            const [, query] = id.split('?');
            if (id.startsWith(resolvedEditorId)) {
                if (projects.length === 1) {
                    /* language=typescript */
                    return `\
import {editor} from '${editor}';
import project from '${projects[0].filePath}?project';
editor(project);
`;
                }
                if (query) {
                    const params = new URLSearchParams(query);
                    const name = params.get('project');
                    if (name && lookup.has(name)) {
                        /* language=typescript */
                        return `\
import {editor} from '${editor}';
import project from '${lookup.get(name).filePath}?project';
editor(project);
`;
                    }
                }
                /* language=typescript */
                return `\
import {index} from '${editor}';
index(${JSON.stringify(projects)});
`;
            }
        },
        configureServer(server) {
            server.middlewares.use((req, res, next) => {
                if (req.url) {
                    const url = new URL(req.url, `http://${req.headers.host}`);
                    if (url.pathname === '/') {
                        res.setHeader('Content-Type', 'text/html');
                        res.end(createHtml('/@id/__x00__virtual:editor'));
                        return;
                    }
                    const name = url.pathname.slice(1);
                    if (name && lookup.has(name)) {
                        res.setHeader('Content-Type', 'text/html');
                        res.end(createHtml(`/@id/__x00__virtual:editor?project=${encodeURIComponent(name)}`));
                        return;
                    }
                }
                next();
            });
        },
    };
}
exports.editorPlugin = editorPlugin;
//# sourceMappingURL=editor.js.map