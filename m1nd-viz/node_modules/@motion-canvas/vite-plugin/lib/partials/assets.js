"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.assetsPlugin = void 0;
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const stream_1 = require("stream");
const AUDIO_EXTENSION_REGEX = /\.(mp3|wav|ogg|aac|flac)(?:$|\?)/;
const AUDIO_HMR_DELAY = 1000;
function assetsPlugin({ bufferedAssets }) {
    let config;
    return {
        name: 'motion-canvas:assets',
        configResolved(resolvedConfig) {
            config = resolvedConfig;
        },
        configureServer(server) {
            server.middlewares.use((req, res, next) => {
                if (req.url && bufferedAssets && bufferedAssets.test(req.url)) {
                    const file = fs_1.default.readFileSync(path_1.default.resolve(config.root, req.url.slice(1)));
                    stream_1.Readable.from(file).pipe(res);
                    return;
                }
                next();
            });
        },
        async handleHotUpdate(ctx) {
            const urls = [];
            const modules = [];
            for (const module of ctx.modules) {
                urls.push(module.url);
                if (!AUDIO_EXTENSION_REGEX.test(module.url)) {
                    modules.push(module);
                }
                else {
                    await new Promise(resolve => {
                        setTimeout(resolve, AUDIO_HMR_DELAY);
                    });
                }
            }
            if (urls.length > 0) {
                ctx.server.ws.send('motion-canvas:assets', { urls });
            }
            return modules;
        },
    };
}
exports.assetsPlugin = assetsPlugin;
//# sourceMappingURL=assets.js.map