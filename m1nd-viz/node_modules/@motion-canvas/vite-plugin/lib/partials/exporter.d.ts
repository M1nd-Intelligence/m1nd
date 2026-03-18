import { Plugin } from 'vite';
interface ExporterPluginConfig {
    outputPath: string;
}
export declare function exporterPlugin({ outputPath }: ExporterPluginConfig): Plugin;
export {};
