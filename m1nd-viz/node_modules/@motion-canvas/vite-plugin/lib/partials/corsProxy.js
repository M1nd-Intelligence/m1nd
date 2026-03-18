"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.corsProxyPlugin = void 0;
const follow_redirects_1 = __importDefault(require("follow-redirects"));
/**
 * This module provides the proxy located at
 * /cors-proxy/...
 *
 * It is needed when accessing remote resources.
 * Trying to access remote resources works while
 * in preview, but will fail when you try to
 * output the image (= "read" the canvas)
 *
 * See https://developer.mozilla.org/en-US/docs/Web/HTML/CORS_enabled_image
 * for reasons
 *
 * Using the proxy circumvents CORS-issues because
 * this way all remote resources are served from the
 * same host as the main app.
 */
function corsProxyPlugin(config) {
    setupEnvVarsForProxy(config);
    return {
        name: 'motion-canvas:cors-proxy',
        configureServer(server) {
            if (config !== false && config !== undefined) {
                motionCanvasCorsProxy(server.middlewares, config === true ? {} : config);
            }
        },
    };
}
exports.corsProxyPlugin = corsProxyPlugin;
function setupEnvVarsForProxy(config) {
    // Define Keys for Env Var
    const prefix = 'VITE_MC_PROXY_';
    const isEnabledKey = prefix + 'ENABLED';
    const allowList = prefix + 'ALLOW_LIST';
    if (config === true) {
        config = {}; // Use Default values
    }
    process.env[isEnabledKey] = String(!!config); // 'true' or 'false'
    if (config) {
        // These values are only configured if the Proxy is enabled
        // You cannot access them via import.meta.env if the Proxy
        // is set to false
        process.env[allowList] = JSON.stringify(config.allowListHosts ?? []);
    }
}
function motionCanvasCorsProxy(middleware, config) {
    // Set the default config if no config was provided
    config.allowedMimeTypes ??= ['image/*', 'video/*'];
    config.allowListHosts ??= [];
    // Check the Mime Types to have a correct structure (left/right)
    // not having them in the correct format would crash the Middleware
    // further down below
    if ((config.allowedMimeTypes ?? []).some(e => e.split('/').length !== 2)) {
        throw new Error("Invalid config for Proxy:\nAll Entries must have the following format:\n 'left/right' where left may be '*'");
    }
    middleware.use((req, res, next) => {
        if (!req.url || !req.url.startsWith('/cors-proxy/')) {
            // url does not start with /cors-proxy/, so this
            // middleware does not care about it
            return next();
        }
        // For now, only allow GET Requests
        if (req.method !== 'GET') {
            return writeError('Only GET Requests are allowed', res, 405);
        }
        let sourceUrl;
        try {
            sourceUrl = extractDestination(req.url);
        }
        catch (err) {
            return writeError(err, res);
        }
        if (!isReceivedUrlInAllowedHosts(sourceUrl.hostname, config.allowListHosts)) {
            return writeError(`Blocked by Proxy: ${sourceUrl.hostname} is not on Hosts Allowlist`, res);
        }
        // Get the resource, do some checks. Throws an Error
        // if the checks fail. The catch then writes an Error
        return tryGetResource(res, sourceUrl, config).catch(error => {
            writeError(error, res);
        });
    });
}
/**
 * Unwrap the destination from the URL.
 *
 * @remarks
 * Throws an Error if the value could not be unwrapped.
 *
 * @param url - the entire URL with the `/cors-proxy/` prefix and containing the
 * url-Encoded Path.
 *
 * @returns The URL that needs to be called.
 */
function extractDestination(url) {
    const withoutPrefix = url.replace('/cors-proxy/', '');
    const asUrl = new URL(decodeURIComponent(withoutPrefix));
    if (asUrl.protocol !== 'http:' && asUrl.protocol !== 'https:') {
        throw new Error('Only supported protocols are http and https');
    }
    return asUrl;
}
/**
 * A simple Error Helper that will write an Error and close the response.
 */
function writeError(message, res, statusCode = 400) {
    res.writeHead(statusCode, message);
    res.end();
}
/**
 * Check if the Proxy is allowed to get the requested resource based on the
 * host.
 */
function isReceivedUrlInAllowedHosts(hostname, allowListHosts) {
    if (!allowListHosts || allowListHosts.length === 0) {
        // if the allowListHosts is just the predefinedAllowlist, the user has not
        // set any additional hosts. In this case, allow any hostname
        return true;
    }
    // Check if the hostname is any of the values set in allowListHosts
    return allowListHosts.some(e => e.toLowerCase().trim() === hostname.toLowerCase().trim());
}
/**
 * Check if the Proxy is allowed to get the requested resource based on the
 * MIME-Type.
 *
 * @remarks
 * Also handles catch-All Declarations like `image/*`.
 */
function isResultOfAllowedResourceType(foundMimeType, allowedMimeTypes) {
    if (!allowedMimeTypes || allowedMimeTypes.length === 0) {
        return true; // no filters set
    }
    if (foundMimeType.split('/').length !== 2) {
        return false; // invalid mime structure
    }
    const [leftSegment, rightSegment] = foundMimeType
        .split('/')
        .map(e => e.trim().toLowerCase());
    // Get all Segments where the left Part is identical between foundMimeType and
    // allowedMimeType.
    const leftSegmentMatches = allowedMimeTypes.filter(e => e.trim().toLowerCase().split('/')[0] === leftSegment);
    if (leftSegmentMatches.length === 0) {
        // No matches at all, not even catchall - resource is rejected.
        return false;
    }
    // This just gets the right part of the MIME Types from the
    // configured allowList, e.g. "image/png" -> png
    const rightSegmentOfLeftSegmentMatches = leftSegmentMatches.map(e => e.split('/')[1]);
    // if an exact match or a catchall is found, the resource is allowed to be
    // proxied.
    return rightSegmentOfLeftSegmentMatches.some(e => e === '*' || e === rightSegment);
}
/**
 * Requests a remote resource with the help of axios
 * May throw a string in case of a bad mime-type or missing headers
 */
async function tryGetResource(res, sourceUrl, config) {
    // Turn this callback into a Promise to avoid additional nesting
    const result = await new Promise((res, rej) => {
        try {
            // We check what protocol was used and decide if we use http or https
            const request = (sourceUrl.protocol.startsWith('https')
                ? follow_redirects_1.default.https
                : follow_redirects_1.default.http).get(sourceUrl, data => {
                res(data);
            });
            request.on('error', (err) => {
                if (err.code && err.code === 'ENOTFOUND') {
                    // This is a bit hacky, but this basically returns as a
                    // 404 instead of crashing the Node Server with ENOTFOUND
                    res({ statusCode: 404 });
                }
                else {
                    rej(err);
                }
            });
        }
        catch (err) {
            rej(err);
        }
    });
    if (!result.statusCode || result.statusCode >= 300) {
        throw 'Unexpected Status: ' + result.statusCode ?? 'NO_STATUS';
    }
    const contentType = result.headers['content-type'];
    const contentLength = result.headers['content-length'];
    if (!contentType) {
        throw 'Proxied Response does not contain a Content Type';
    }
    if (!contentLength) {
        throw 'Proxied Response does not contain a Content Length';
    }
    if (!isResultOfAllowedResourceType(contentType.toString(), config.allowedMimeTypes ?? [])) {
        throw 'Proxied response has blocked content-type: ' + contentType;
    }
    // Prepare Response
    for (const key in result.headers) {
        const header = result.headers[key];
        if (header === undefined) {
            console.warn('Proxy: Received Header is empty. Skipping…');
            continue;
        }
        res.setHeader(key, header);
    }
    res.addListener('error', () => {
        console.log('Proxy: Connection Reset');
    });
    res.setHeader('x-proxy-destination', sourceUrl.toString());
    // Don't store on the server, just immediately pass on the
    // received chunks
    result.pipe(res);
}
//# sourceMappingURL=corsProxy.js.map