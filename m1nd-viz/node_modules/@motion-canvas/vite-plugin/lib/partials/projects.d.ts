import { Plugin } from 'vite';
import { PluginOptions, ProjectData } from '../plugins';
interface ProjectPluginConfig {
    buildForEditor?: boolean;
    plugins: PluginOptions[];
    projects: ProjectData[];
}
export declare function projectsPlugin({ buildForEditor, plugins, projects, }: ProjectPluginConfig): Plugin;
export {};
