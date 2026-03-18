import { Plugin } from 'vite';
declare module 'source-map' {
    interface SourceNode {
        add(value: string | SourceNode): void;
    }
    interface SourceMapGenerator {
        toJSON(): any;
    }
}
export declare function webglPlugin(): Plugin;
