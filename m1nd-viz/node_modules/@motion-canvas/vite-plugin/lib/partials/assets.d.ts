import { Plugin } from 'vite';
interface AssetsPluginConfig {
    bufferedAssets: RegExp | false;
}
export declare function assetsPlugin({ bufferedAssets }: AssetsPluginConfig): Plugin;
export {};
