import { errorToLog, useLogger } from '../utils';
import { DependencyContext } from './DependencyContext';
export class ComputedContext extends DependencyContext {
    constructor(factory, owner) {
        super(owner);
        this.factory = factory;
        this.markDirty();
    }
    toSignal() {
        return this.invokable;
    }
    dispose() {
        super.dispose();
        this.last = undefined;
    }
    invoke(...args) {
        if (this.event.isRaised()) {
            this.clearDependencies();
            this.startCollecting();
            try {
                this.last = this.factory(...args);
            }
            catch (e) {
                useLogger().error({
                    ...errorToLog(e),
                    inspect: this.owner?.key,
                });
            }
            this.finishCollecting();
        }
        this.event.reset();
        this.collect();
        return this.last;
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29tcHV0ZWRDb250ZXh0LmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3NpZ25hbHMvQ29tcHV0ZWRDb250ZXh0LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBQyxVQUFVLEVBQUUsU0FBUyxFQUFDLE1BQU0sVUFBVSxDQUFDO0FBQy9DLE9BQU8sRUFBQyxpQkFBaUIsRUFBQyxNQUFNLHFCQUFxQixDQUFDO0FBT3RELE1BQU0sT0FBTyxlQUF3QixTQUFRLGlCQUFzQjtJQUdqRSxZQUNtQixPQUFtQyxFQUNwRCxLQUFXO1FBRVgsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBSEksWUFBTyxHQUFQLE9BQU8sQ0FBNEI7UUFJcEQsSUFBSSxDQUFDLFNBQVMsRUFBRSxDQUFDO0lBQ25CLENBQUM7SUFFTSxRQUFRO1FBQ2IsT0FBTyxJQUFJLENBQUMsU0FBUyxDQUFDO0lBQ3hCLENBQUM7SUFFZSxPQUFPO1FBQ3JCLEtBQUssQ0FBQyxPQUFPLEVBQUUsQ0FBQztRQUNoQixJQUFJLENBQUMsSUFBSSxHQUFHLFNBQVMsQ0FBQztJQUN4QixDQUFDO0lBRWtCLE1BQU0sQ0FBQyxHQUFHLElBQVc7UUFDdEMsSUFBSSxJQUFJLENBQUMsS0FBSyxDQUFDLFFBQVEsRUFBRSxFQUFFO1lBQ3pCLElBQUksQ0FBQyxpQkFBaUIsRUFBRSxDQUFDO1lBQ3pCLElBQUksQ0FBQyxlQUFlLEVBQUUsQ0FBQztZQUN2QixJQUFJO2dCQUNGLElBQUksQ0FBQyxJQUFJLEdBQUcsSUFBSSxDQUFDLE9BQU8sQ0FBQyxHQUFHLElBQUksQ0FBQyxDQUFDO2FBQ25DO1lBQUMsT0FBTyxDQUFNLEVBQUU7Z0JBQ2YsU0FBUyxFQUFFLENBQUMsS0FBSyxDQUFDO29CQUNoQixHQUFHLFVBQVUsQ0FBQyxDQUFDLENBQUM7b0JBQ2hCLE9BQU8sRUFBRSxJQUFJLENBQUMsS0FBSyxFQUFFLEdBQUc7aUJBQ3pCLENBQUMsQ0FBQzthQUNKO1lBQ0QsSUFBSSxDQUFDLGdCQUFnQixFQUFFLENBQUM7U0FDekI7UUFDRCxJQUFJLENBQUMsS0FBSyxDQUFDLEtBQUssRUFBRSxDQUFDO1FBQ25CLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQztRQUVmLE9BQU8sSUFBSSxDQUFDLElBQUssQ0FBQztJQUNwQixDQUFDO0NBQ0YifQ==