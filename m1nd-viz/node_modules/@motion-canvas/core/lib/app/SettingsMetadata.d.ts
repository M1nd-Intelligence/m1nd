import { BoolMetaField, ColorMetaField, MetaField, ObjectMetaField, Vector2MetaField } from '../meta';
/**
 * Create a runtime representation of the settings metadata.
 */
export declare function createSettingsMetadata(): ObjectMetaField<{
    version: MetaField<any, number>;
    appearance: ObjectMetaField<{
        color: ColorMetaField;
        font: BoolMetaField;
        coordinates: BoolMetaField;
    }>;
    defaults: ObjectMetaField<{
        background: ColorMetaField;
        size: Vector2MetaField;
    }>;
}>;
/**
 * A runtime representation of the settings metadata.
 */
export type SettingsMetadata = ReturnType<typeof createSettingsMetadata>;
//# sourceMappingURL=SettingsMetadata.d.ts.map