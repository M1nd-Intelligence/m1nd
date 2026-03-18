/**
 * Utility to redirect remote sources via Proxy
 *
 * This utility is used to rewrite a request to be routed through
 * the Proxy instead.
 */
import { useLogger } from './useScene';
/**
 * Route the given url through a local proxy.
 *
 * @example
 * This rewrites a remote url like `https://via.placeholder.com/300.png/09f/fff`
 * into a URI-Component-Encoded string like
 * `/cors-proxy/https%3A%2F%2Fvia.placeholder.com%2F300.png%2F09f%2Ffff`
 */
export function viaProxy(url) {
    if (!isProxyEnabled()) {
        // Proxy is disabled, so we just pass as-is.
        return url;
    }
    if (url.startsWith('/cors-proxy/')) {
        // Already proxied, return as-is
        return url;
    }
    // window.location.hostname is being passed here to ensure that
    // this does not throw an Error for same-origin requests
    // e.g. /some/image -> localhost:9000/some/image
    const selfUrl = new URL(window.location.toString());
    // inside a try-catch in case the URL cannot be understood
    try {
        const expandedUrl = new URL(url, selfUrl);
        if (!expandedUrl.protocol.startsWith('http')) {
            // this is probably some embedded image (e.g. image/png;base64).
            // don't touch and pass as is
            return url;
        }
        if (selfUrl.host === expandedUrl.host) {
            // This is a request to a "local" resource.
            // No need to rewrite
            return url;
        }
        // Check if the host matches the Allow List.
        // if not, no rewrite takes place.
        // will fail in the Editor if the
        // remote host does not accept anonymous
        if (!isInsideAllowList(expandedUrl.host)) {
            return url;
        }
    }
    catch (_) {
        // in case of error just silently pass as-is
        return url;
    }
    // Everything else is a "remote" resource and requires a rewrite.
    return `/cors-proxy/${encodeURIComponent(url)}`;
}
/**
 * Check the provided host is allowed to be routed
 * to the Proxy.
 */
function isInsideAllowList(host) {
    const allowList = getAllowList();
    if (allowList.length === 0) {
        // Allow List defaults to allow all if empty
        return true;
    }
    for (const entry of allowList) {
        if (entry.toLowerCase().trim() === host) {
            return true;
        }
    }
    return false;
}
/**
 * Check if the proxy is enabled via the plugin by checking
 * for `import.meta.env.VITE_MC_PROXY_ENABLED`
 *
 * @remarks The value can either be 'true' of 'false'
 * (as strings) if present, or be undefined if not run
 * from a vite context or run without the MC Plugin.
 */
export function isProxyEnabled() {
    if (import.meta.env) {
        return import.meta.env.VITE_MC_PROXY_ENABLED === 'true';
    }
    return false;
}
/**
 * Cached value so getAllowList does not
 * try to parse the Env var on every call,
 * spamming the console in the process
 */
let AllowListCache = undefined;
/**
 * Return the list of allowed hosts
 * from the Plugin Config
 */
function getAllowList() {
    // Condition should get optimized away for Prod
    if (import.meta.env.VITEST !== 'true') {
        if (AllowListCache) {
            return [...AllowListCache];
        }
    }
    // Inline function gets immediately invoked
    // and the result stored in getAllowListCache.
    // The cached value is used on subsequent requests.
    const result = (function () {
        if (!isProxyEnabled() || !import.meta.env) {
            return [];
        }
        // This value is encoded as a JSON String.
        const valueJson = import.meta.env.VITE_MC_PROXY_ALLOW_LIST ?? '[]';
        const parsedJson = JSON.parse(valueJson);
        // Do an additional check that only strings are present,
        // create a warning and ignore the value
        if (!Array.isArray(parsedJson)) {
            useLogger().error('Parsed Allow List expected to be an Array, but is ' +
                typeof parsedJson);
        }
        const validatedEntries = [];
        for (const entry of parsedJson) {
            if (typeof entry !== 'string') {
                useLogger().warn('Unexpected Value in Allow List: ' +
                    entry +
                    '. Expected a String. Skipping.');
                continue;
            }
            validatedEntries.push(entry);
        }
        return validatedEntries;
    })();
    AllowListCache = result;
    return [...AllowListCache];
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoicHJveHlVdGlscy5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy91dGlscy9wcm94eVV0aWxzLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBOzs7OztHQUtHO0FBRUgsT0FBTyxFQUFDLFNBQVMsRUFBQyxNQUFNLFlBQVksQ0FBQztBQUVyQzs7Ozs7OztHQU9HO0FBQ0gsTUFBTSxVQUFVLFFBQVEsQ0FBQyxHQUFXO0lBQ2xDLElBQUksQ0FBQyxjQUFjLEVBQUUsRUFBRTtRQUNyQiw0Q0FBNEM7UUFDNUMsT0FBTyxHQUFHLENBQUM7S0FDWjtJQUVELElBQUksR0FBRyxDQUFDLFVBQVUsQ0FBQyxjQUFjLENBQUMsRUFBRTtRQUNsQyxnQ0FBZ0M7UUFDaEMsT0FBTyxHQUFHLENBQUM7S0FDWjtJQUVELCtEQUErRDtJQUMvRCx3REFBd0Q7SUFDeEQsZ0RBQWdEO0lBQ2hELE1BQU0sT0FBTyxHQUFHLElBQUksR0FBRyxDQUFDLE1BQU0sQ0FBQyxRQUFRLENBQUMsUUFBUSxFQUFFLENBQUMsQ0FBQztJQUNwRCwwREFBMEQ7SUFDMUQsSUFBSTtRQUNGLE1BQU0sV0FBVyxHQUFHLElBQUksR0FBRyxDQUFDLEdBQUcsRUFBRSxPQUFPLENBQUMsQ0FBQztRQUMxQyxJQUFJLENBQUMsV0FBVyxDQUFDLFFBQVEsQ0FBQyxVQUFVLENBQUMsTUFBTSxDQUFDLEVBQUU7WUFDNUMsZ0VBQWdFO1lBQ2hFLDZCQUE2QjtZQUM3QixPQUFPLEdBQUcsQ0FBQztTQUNaO1FBQ0QsSUFBSSxPQUFPLENBQUMsSUFBSSxLQUFLLFdBQVcsQ0FBQyxJQUFJLEVBQUU7WUFDckMsMkNBQTJDO1lBQzNDLHFCQUFxQjtZQUNyQixPQUFPLEdBQUcsQ0FBQztTQUNaO1FBRUQsNENBQTRDO1FBQzVDLGtDQUFrQztRQUNsQyxpQ0FBaUM7UUFDakMsd0NBQXdDO1FBQ3hDLElBQUksQ0FBQyxpQkFBaUIsQ0FBQyxXQUFXLENBQUMsSUFBSSxDQUFDLEVBQUU7WUFDeEMsT0FBTyxHQUFHLENBQUM7U0FDWjtLQUNGO0lBQUMsT0FBTyxDQUFDLEVBQUU7UUFDViw0Q0FBNEM7UUFDNUMsT0FBTyxHQUFHLENBQUM7S0FDWjtJQUVELGlFQUFpRTtJQUNqRSxPQUFPLGVBQWUsa0JBQWtCLENBQUMsR0FBRyxDQUFDLEVBQUUsQ0FBQztBQUNsRCxDQUFDO0FBRUQ7OztHQUdHO0FBQ0gsU0FBUyxpQkFBaUIsQ0FBQyxJQUFZO0lBQ3JDLE1BQU0sU0FBUyxHQUFHLFlBQVksRUFBRSxDQUFDO0lBQ2pDLElBQUksU0FBUyxDQUFDLE1BQU0sS0FBSyxDQUFDLEVBQUU7UUFDMUIsNENBQTRDO1FBQzVDLE9BQU8sSUFBSSxDQUFDO0tBQ2I7SUFDRCxLQUFLLE1BQU0sS0FBSyxJQUFJLFNBQVMsRUFBRTtRQUM3QixJQUFJLEtBQUssQ0FBQyxXQUFXLEVBQUUsQ0FBQyxJQUFJLEVBQUUsS0FBSyxJQUFJLEVBQUU7WUFDdkMsT0FBTyxJQUFJLENBQUM7U0FDYjtLQUNGO0lBQ0QsT0FBTyxLQUFLLENBQUM7QUFDZixDQUFDO0FBRUQ7Ozs7Ozs7R0FPRztBQUNILE1BQU0sVUFBVSxjQUFjO0lBQzVCLElBQUksTUFBTSxDQUFDLElBQUksQ0FBQyxHQUFHLEVBQUU7UUFDbkIsT0FBTyxNQUFNLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxxQkFBcUIsS0FBSyxNQUFNLENBQUM7S0FDekQ7SUFDRCxPQUFPLEtBQUssQ0FBQztBQUNmLENBQUM7QUFFRDs7OztHQUlHO0FBQ0gsSUFBSSxjQUFjLEdBQXlCLFNBQVMsQ0FBQztBQUNyRDs7O0dBR0c7QUFDSCxTQUFTLFlBQVk7SUFDbkIsK0NBQStDO0lBQy9DLElBQUksTUFBTSxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsTUFBTSxLQUFLLE1BQU0sRUFBRTtRQUNyQyxJQUFJLGNBQWMsRUFBRTtZQUNsQixPQUFPLENBQUMsR0FBRyxjQUFjLENBQUMsQ0FBQztTQUM1QjtLQUNGO0lBRUQsMkNBQTJDO0lBQzNDLDhDQUE4QztJQUM5QyxtREFBbUQ7SUFDbkQsTUFBTSxNQUFNLEdBQUcsQ0FBQztRQUNkLElBQUksQ0FBQyxjQUFjLEVBQUUsSUFBSSxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxFQUFFO1lBQ3pDLE9BQU8sRUFBRSxDQUFDO1NBQ1g7UUFFRCwwQ0FBMEM7UUFDMUMsTUFBTSxTQUFTLEdBQUcsTUFBTSxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsd0JBQXdCLElBQUksSUFBSSxDQUFDO1FBQ25FLE1BQU0sVUFBVSxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUMsU0FBUyxDQUFDLENBQUM7UUFDekMsd0RBQXdEO1FBQ3hELHdDQUF3QztRQUN4QyxJQUFJLENBQUMsS0FBSyxDQUFDLE9BQU8sQ0FBQyxVQUFVLENBQUMsRUFBRTtZQUM5QixTQUFTLEVBQUUsQ0FBQyxLQUFLLENBQ2Ysb0RBQW9EO2dCQUNsRCxPQUFPLFVBQVUsQ0FDcEIsQ0FBQztTQUNIO1FBQ0QsTUFBTSxnQkFBZ0IsR0FBRyxFQUFFLENBQUM7UUFDNUIsS0FBSyxNQUFNLEtBQUssSUFBSSxVQUFVLEVBQUU7WUFDOUIsSUFBSSxPQUFPLEtBQUssS0FBSyxRQUFRLEVBQUU7Z0JBQzdCLFNBQVMsRUFBRSxDQUFDLElBQUksQ0FDZCxrQ0FBa0M7b0JBQ2hDLEtBQUs7b0JBQ0wsZ0NBQWdDLENBQ25DLENBQUM7Z0JBQ0YsU0FBUzthQUNWO1lBQ0QsZ0JBQWdCLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDO1NBQzlCO1FBQ0QsT0FBTyxnQkFBZ0IsQ0FBQztJQUMxQixDQUFDLENBQUMsRUFBRSxDQUFDO0lBQ0wsY0FBYyxHQUFHLE1BQU0sQ0FBQztJQUN4QixPQUFPLENBQUMsR0FBRyxjQUFjLENBQUMsQ0FBQztBQUM3QixDQUFDIn0=