import { ColorMetaField, EnumMetaField, ExporterMetaField, MetaField, NumberMetaField, ObjectMetaField, RangeMetaField, Vector2MetaField } from '../meta';
import { CanvasColorSpace, Color, Vector2 } from '../types';
import type { Project } from './Project';
declare function createProjectMetadata(project: Project): {
    version: MetaField<unknown, number>;
    shared: ObjectMetaField<{
        background: ColorMetaField;
        range: RangeMetaField;
        size: Vector2MetaField;
        audioOffset: NumberMetaField;
    }>;
    preview: ObjectMetaField<{
        fps: NumberMetaField;
        resolutionScale: EnumMetaField<number>;
    }>;
    rendering: ObjectMetaField<{
        fps: NumberMetaField;
        resolutionScale: EnumMetaField<number>;
        colorSpace: EnumMetaField<CanvasColorSpace>;
        exporter: ExporterMetaField;
    }>;
};
export declare class ProjectMetadata extends ObjectMetaField<ReturnType<typeof createProjectMetadata>> {
    constructor(project: Project);
    getFullPreviewSettings(): {
        fps: number;
        resolutionScale: number;
        background: Color | null;
        range: [number, number];
        size: Vector2;
        audioOffset: number;
    };
    getFullRenderingSettings(): {
        fps: number;
        resolutionScale: number;
        colorSpace: CanvasColorSpace;
        background: Color | null;
        range: [number, number];
        size: Vector2;
        audioOffset: number;
        exporter: {
            name: string;
            options: unknown;
        };
    };
}
export {};
//# sourceMappingURL=ProjectMetadata.d.ts.map