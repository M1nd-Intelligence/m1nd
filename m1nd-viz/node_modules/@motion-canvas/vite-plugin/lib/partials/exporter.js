"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.exporterPlugin = void 0;
const fs_1 = __importDefault(require("fs"));
const mime_types_1 = __importDefault(require("mime-types"));
const path_1 = __importDefault(require("path"));
const openInExplorer_1 = require("../openInExplorer");
function exporterPlugin({ outputPath }) {
    return {
        name: 'motion-canvas:exporter',
        configureServer(server) {
            server.middlewares.use((req, res, next) => {
                if (req.url === '/__open-output-path') {
                    if (!fs_1.default.existsSync(outputPath)) {
                        fs_1.default.mkdirSync(outputPath, { recursive: true });
                    }
                    (0, openInExplorer_1.openInExplorer)(outputPath);
                    res.end();
                    return;
                }
                next();
            });
            server.ws.on('motion-canvas:export', async ({ data, frame, name, subDirectories, mimeType }, client) => {
                const extension = mime_types_1.default.extension(mimeType);
                const outputFilePath = path_1.default.join(outputPath, ...subDirectories, `${name}.${extension}`);
                const outputDirectory = path_1.default.dirname(outputFilePath);
                if (!fs_1.default.existsSync(outputDirectory)) {
                    fs_1.default.mkdirSync(outputDirectory, { recursive: true });
                }
                const base64Data = data.slice(data.indexOf(',') + 1);
                await writeBase64(outputFilePath, base64Data);
                client.send('motion-canvas:export-ack', { frame });
            });
        },
    };
}
exports.exporterPlugin = exporterPlugin;
function writeBase64(filePath, base64) {
    return new Promise((resolve, reject) => {
        fs_1.default.createWriteStream(filePath)
            .on('finish', resolve)
            .on('error', reject)
            .end(Buffer.from(base64, 'base64'));
    });
}
//# sourceMappingURL=exporter.js.map