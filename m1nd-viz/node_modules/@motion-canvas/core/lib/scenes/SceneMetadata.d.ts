import { MetaField, ObjectMetaField } from '../meta';
import { SerializedTimeEvent } from './timeEvents';
/**
 * Create a runtime representation of the scene metadata.
 */
export declare function createSceneMetadata(): ObjectMetaField<{
    version: MetaField<any, number>;
    timeEvents: MetaField<SerializedTimeEvent[], SerializedTimeEvent[]>;
    seed: MetaField<any, number>;
}>;
/**
 * A runtime representation of the scene metadata.
 */
export type SceneMetadata = ReturnType<typeof createSceneMetadata>;
//# sourceMappingURL=SceneMetadata.d.ts.map