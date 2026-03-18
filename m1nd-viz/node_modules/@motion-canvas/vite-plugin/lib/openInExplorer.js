"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.openInExplorer = void 0;
const child_process_1 = require("child_process");
const os_1 = require("os");
function openInExplorer(file) {
    let command = null;
    let args = [file];
    const os = (0, os_1.platform)();
    switch (os) {
        case 'win32':
            command = 'explorer';
            break;
        case 'linux':
            if (isRunningOnWSL()) {
                command = 'bash';
                args = ['-c', `cd ${file} && explorer.exe .`];
            }
            else {
                command = 'xdg-open';
            }
            break;
        case 'darwin':
            command = 'open';
            break;
    }
    if (command) {
        (0, child_process_1.spawn)(command, args, { detached: true }).unref();
    }
    else {
        console.warn(`Unsupported OS: ${os}`);
    }
}
exports.openInExplorer = openInExplorer;
function isRunningOnWSL() {
    try {
        const uname = (0, child_process_1.execSync)('uname -a').toString().toLowerCase();
        return uname.includes('microsoft');
    }
    catch (error) {
        console.error(`exec error: ${error}`);
        return false;
    }
}
//# sourceMappingURL=openInExplorer.js.map