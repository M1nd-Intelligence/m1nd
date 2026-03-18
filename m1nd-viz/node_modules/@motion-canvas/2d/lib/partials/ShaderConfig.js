import { experimentalLog, useLogger, useScene, } from '@motion-canvas/core';
export function parseShader(value) {
    let result;
    if (!value) {
        result = [];
    }
    else if (typeof value === 'string') {
        result = [{ fragment: value }];
    }
    else if (Array.isArray(value)) {
        result = value.map(item => typeof item === 'string' ? { fragment: item } : item);
    }
    else {
        result = [value];
    }
    if (!useScene().experimentalFeatures && result.length > 0) {
        result = [];
        useLogger().log({
            ...experimentalLog(`Node uses experimental shaders.`),
            inspect: this.key,
        });
    }
    return result;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiU2hhZGVyQ29uZmlnLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL2xpYi9wYXJ0aWFscy9TaGFkZXJDb25maWcudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUNMLGVBQWUsRUFFZixTQUFTLEVBQ1QsUUFBUSxHQUVULE1BQU0scUJBQXFCLENBQUM7QUF5RjdCLE1BQU0sVUFBVSxXQUFXLENBRXpCLEtBQTJCO0lBRTNCLElBQUksTUFBc0IsQ0FBQztJQUMzQixJQUFJLENBQUMsS0FBSyxFQUFFO1FBQ1YsTUFBTSxHQUFHLEVBQUUsQ0FBQztLQUNiO1NBQU0sSUFBSSxPQUFPLEtBQUssS0FBSyxRQUFRLEVBQUU7UUFDcEMsTUFBTSxHQUFHLENBQUMsRUFBQyxRQUFRLEVBQUUsS0FBSyxFQUFDLENBQUMsQ0FBQztLQUM5QjtTQUFNLElBQUksS0FBSyxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUMsRUFBRTtRQUMvQixNQUFNLEdBQUcsS0FBSyxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsRUFBRSxDQUN4QixPQUFPLElBQUksS0FBSyxRQUFRLENBQUMsQ0FBQyxDQUFDLEVBQUMsUUFBUSxFQUFFLElBQUksRUFBQyxDQUFDLENBQUMsQ0FBQyxJQUFJLENBQ25ELENBQUM7S0FDSDtTQUFNO1FBQ0wsTUFBTSxHQUFHLENBQUMsS0FBSyxDQUFDLENBQUM7S0FDbEI7SUFFRCxJQUFJLENBQUMsUUFBUSxFQUFFLENBQUMsb0JBQW9CLElBQUksTUFBTSxDQUFDLE1BQU0sR0FBRyxDQUFDLEVBQUU7UUFDekQsTUFBTSxHQUFHLEVBQUUsQ0FBQztRQUNaLFNBQVMsRUFBRSxDQUFDLEdBQUcsQ0FBQztZQUNkLEdBQUcsZUFBZSxDQUFDLGlDQUFpQyxDQUFDO1lBQ3JELE9BQU8sRUFBRSxJQUFJLENBQUMsR0FBRztTQUNsQixDQUFDLENBQUM7S0FDSjtJQUVELE9BQU8sTUFBTSxDQUFDO0FBQ2hCLENBQUMifQ==