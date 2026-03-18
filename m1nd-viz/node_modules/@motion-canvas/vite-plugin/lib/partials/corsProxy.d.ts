import { Plugin } from 'vite';
/**
 * Configuration used by the Proxy plugin
 */
export interface CorsProxyPluginConfig {
    /**
     * Set which types of resources are allowed by default.
     *
     * @remarks
     * Catchall on the right side is supported.
     * Pass an empty Array to allow all types of resources, although this is not
     * recommended.
     *
     * @defaultValue ["image/*", "video/*"]
     */
    allowedMimeTypes?: string[];
    /**
     * Set which hosts are allowed
     *
     * @remarks
     * Note that the host is everything to the left of the first `/`, and to the
     * right of the protocol `https://`. AllowList is not used by default,
     * although you should consider setting up just the relevant hosts.
     */
    allowListHosts?: string[];
}
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
export declare function corsProxyPlugin(config?: CorsProxyPluginConfig | boolean): Plugin;
