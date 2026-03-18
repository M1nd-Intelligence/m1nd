"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getProjects = exports.createMeta = void 0;
const fast_glob_1 = __importDefault(require("fast-glob"));
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
async function createMeta(metaPath) {
    if (!fs_1.default.existsSync(metaPath)) {
        await fs_1.default.promises.writeFile(metaPath, JSON.stringify({ version: 0 }, undefined, 2), 'utf8');
    }
}
exports.createMeta = createMeta;
function getProjects(project) {
    const list = [];
    const projectList = expandFilePaths(project);
    for (const filePath of projectList) {
        const { name, dir } = path_1.default.posix.parse(filePath);
        const metaFile = `${name}.meta`;
        const metaData = getMeta(path_1.default.join(dir, metaFile));
        const url = path_1.default.posix.join(dir, name);
        const data = {
            name: metaData?.name ?? url,
            fileName: name,
            url,
            filePath,
        };
        list.push(data);
    }
    return list;
}
exports.getProjects = getProjects;
function expandFilePaths(filePaths) {
    const expandedFilePaths = [];
    for (const filePath of typeof filePaths === 'string'
        ? [filePaths]
        : filePaths) {
        if (fast_glob_1.default.isDynamicPattern(filePath)) {
            const matchingFilePaths = fast_glob_1.default.sync(filePath, { onlyFiles: true });
            expandedFilePaths.push(...matchingFilePaths);
        }
        else {
            expandedFilePaths.push(filePath);
        }
    }
    return expandedFilePaths;
}
function getMeta(metaPath) {
    if (fs_1.default.existsSync(metaPath)) {
        return JSON.parse(fs_1.default.readFileSync(metaPath, 'utf8'));
    }
}
//# sourceMappingURL=utils.js.map