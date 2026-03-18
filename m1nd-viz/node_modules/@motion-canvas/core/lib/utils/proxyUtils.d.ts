/**
 * Utility to redirect remote sources via Proxy
 *
 * This utility is used to rewrite a request to be routed through
 * the Proxy instead.
 */
/**
 * Route the given url through a local proxy.
 *
 * @example
 * This rewrites a remote url like `https://via.placeholder.com/300.png/09f/fff`
 * into a URI-Component-Encoded string like
 * `/cors-proxy/https%3A%2F%2Fvia.placeholder.com%2F300.png%2F09f%2Ffff`
 */
export declare function viaProxy(url: string): string;
/**
 * Check if the proxy is enabled via the plugin by checking
 * for `import.meta.env.VITE_MC_PROXY_ENABLED`
 *
 * @remarks The value can either be 'true' of 'false'
 * (as strings) if present, or be undefined if not run
 * from a vite context or run without the MC Plugin.
 */
export declare function isProxyEnabled(): boolean;
//# sourceMappingURL=proxyUtils.d.ts.map