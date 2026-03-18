import { Plugin } from 'vite';
import { ProjectData } from '../plugins';
interface EditorPluginConfig {
    editor: string;
    projects: ProjectData[];
}
export declare function editorPlugin({ editor, projects }: EditorPluginConfig): Plugin;
export {};
