import { GeneratorScene, SceneRenderEvent, Vector2, useLogger, } from '@motion-canvas/core';
import { Camera, View2D } from '../components';
import { is } from '../utils';
export class Scene2D extends GeneratorScene {
    constructor(description) {
        super(description);
        this.view = null;
        this.registeredNodes = new Map();
        this.nodeCounters = new Map();
        this.assetHash = Date.now().toString();
        this.recreateView();
        if (import.meta.hot) {
            import.meta.hot.on('motion-canvas:assets', () => {
                this.assetHash = Date.now().toString();
                this.getView().assetHash(this.assetHash);
            });
        }
    }
    getView() {
        return this.view;
    }
    next() {
        this.getView()
            ?.playbackState(this.playback.state)
            .globalTime(this.playback.time);
        return super.next();
    }
    draw(context) {
        context.save();
        this.renderLifecycle.dispatch([SceneRenderEvent.BeforeRender, context]);
        context.save();
        this.renderLifecycle.dispatch([SceneRenderEvent.BeginRender, context]);
        this.getView()
            .playbackState(this.playback.state)
            .globalTime(this.playback.time);
        this.getView().render(context);
        this.renderLifecycle.dispatch([SceneRenderEvent.FinishRender, context]);
        context.restore();
        this.renderLifecycle.dispatch([SceneRenderEvent.AfterRender, context]);
        context.restore();
    }
    reset(previousScene) {
        for (const key of this.registeredNodes.keys()) {
            try {
                this.registeredNodes.get(key).dispose();
            }
            catch (e) {
                this.logger.error(e);
            }
        }
        this.registeredNodes.clear();
        this.registeredNodes = new Map();
        this.nodeCounters.clear();
        this.recreateView();
        return super.reset(previousScene);
    }
    inspectPosition(x, y) {
        return this.execute(() => this.getView().hit(new Vector2(x, y))?.key ?? null);
    }
    validateInspection(element) {
        return this.getNode(element)?.key ?? null;
    }
    inspectAttributes(element) {
        const node = this.getNode(element);
        if (!node)
            return null;
        const attributes = {
            stack: node.creationStack,
            key: node.key,
        };
        for (const { key, meta, signal } of node) {
            if (!meta.inspectable)
                continue;
            attributes[key] = signal();
        }
        return attributes;
    }
    drawOverlay(element, matrix, context) {
        const node = this.getNode(element);
        if (node) {
            this.execute(() => {
                const cameras = this.getView().findAll(is(Camera));
                const parentCameras = [];
                for (const camera of cameras) {
                    const scene = camera.scene();
                    if (!scene)
                        continue;
                    if (scene === node || scene.findFirst(n => n === node)) {
                        parentCameras.push(camera);
                    }
                }
                if (parentCameras.length > 0) {
                    for (const camera of parentCameras) {
                        const cameraParentToWorld = camera.parentToWorld();
                        const cameraLocalToParent = camera.localToParent().inverse();
                        const nodeLocalToWorld = node.localToWorld();
                        node.drawOverlay(context, matrix
                            .multiply(cameraParentToWorld)
                            .multiply(cameraLocalToParent)
                            .multiply(nodeLocalToWorld));
                    }
                }
                else {
                    node.drawOverlay(context, matrix.multiply(node.localToWorld()));
                }
            });
        }
    }
    transformMousePosition(x, y) {
        return new Vector2(x, y).transformAsPoint(this.getView().localToParent().inverse());
    }
    registerNode(node, key) {
        const className = node.constructor?.name ?? 'unknown';
        const counter = (this.nodeCounters.get(className) ?? 0) + 1;
        this.nodeCounters.set(className, counter);
        if (key && this.registeredNodes.has(key)) {
            useLogger().error({
                message: `Duplicated node key: "${key}".`,
                inspect: key,
                stack: new Error().stack,
            });
            key = undefined;
        }
        key ?? (key = `${this.name}/${className}[${counter}]`);
        this.registeredNodes.set(key, node);
        const currentNodeMap = this.registeredNodes;
        return [key, () => currentNodeMap.delete(key)];
    }
    getNode(key) {
        if (typeof key !== 'string')
            return null;
        return this.registeredNodes.get(key) ?? null;
    }
    *getDetachedNodes() {
        for (const node of this.registeredNodes.values()) {
            if (!node.parent() && node !== this.view)
                yield node;
        }
    }
    recreateView() {
        this.execute(() => {
            const size = this.getSize();
            this.view = new View2D({
                position: size.scale(this.resolutionScale / 2),
                scale: this.resolutionScale,
                assetHash: this.assetHash,
                size,
            });
        });
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiU2NlbmUyRC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvc2NlbmVzL1NjZW5lMkQudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUVMLGNBQWMsRUFLZCxnQkFBZ0IsRUFFaEIsT0FBTyxFQUNQLFNBQVMsR0FDVixNQUFNLHFCQUFxQixDQUFDO0FBQzdCLE9BQU8sRUFBQyxNQUFNLEVBQVEsTUFBTSxFQUFDLE1BQU0sZUFBZSxDQUFDO0FBQ25ELE9BQU8sRUFBQyxFQUFFLEVBQUMsTUFBTSxVQUFVLENBQUM7QUFFNUIsTUFBTSxPQUFPLE9BQVEsU0FBUSxjQUFzQjtJQU1qRCxZQUNFLFdBQWlFO1FBRWpFLEtBQUssQ0FBQyxXQUFXLENBQUMsQ0FBQztRQVJiLFNBQUksR0FBa0IsSUFBSSxDQUFDO1FBQzNCLG9CQUFlLEdBQUcsSUFBSSxHQUFHLEVBQWdCLENBQUM7UUFDakMsaUJBQVksR0FBRyxJQUFJLEdBQUcsRUFBa0IsQ0FBQztRQUNsRCxjQUFTLEdBQUcsSUFBSSxDQUFDLEdBQUcsRUFBRSxDQUFDLFFBQVEsRUFBRSxDQUFDO1FBTXhDLElBQUksQ0FBQyxZQUFZLEVBQUUsQ0FBQztRQUNwQixJQUFJLE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxFQUFFO1lBQ25CLE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxDQUFDLEVBQUUsQ0FBQyxzQkFBc0IsRUFBRSxHQUFHLEVBQUU7Z0JBQzlDLElBQUksQ0FBQyxTQUFTLEdBQUcsSUFBSSxDQUFDLEdBQUcsRUFBRSxDQUFDLFFBQVEsRUFBRSxDQUFDO2dCQUN2QyxJQUFJLENBQUMsT0FBTyxFQUFFLENBQUMsU0FBUyxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUMsQ0FBQztZQUMzQyxDQUFDLENBQUMsQ0FBQztTQUNKO0lBQ0gsQ0FBQztJQUVNLE9BQU87UUFDWixPQUFPLElBQUksQ0FBQyxJQUFLLENBQUM7SUFDcEIsQ0FBQztJQUVlLElBQUk7UUFDbEIs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