import { createSceneMetadata, } from '@motion-canvas/core';
import { Scene2D } from './Scene2D';
export function makeScene2D(runner) {
    return {
        klass: Scene2D,
        config: runner,
        stack: new Error().stack,
        meta: createSceneMetadata(),
        plugins: ['@motion-canvas/2d/editor'],
    };
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibWFrZVNjZW5lMkQuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvbGliL3NjZW5lcy9tYWtlU2NlbmUyRC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQ0wsbUJBQW1CLEdBR3BCLE1BQU0scUJBQXFCLENBQUM7QUFFN0IsT0FBTyxFQUFDLE9BQU8sRUFBQyxNQUFNLFdBQVcsQ0FBQztBQUVsQyxNQUFNLFVBQVUsV0FBVyxDQUN6QixNQUFzQztJQUV0QyxPQUFPO1FBQ0wsS0FBSyxFQUFFLE9BQU87UUFDZCxNQUFNLEVBQUUsTUFBTTtRQUNkLEtBQUssRUFBRSxJQUFJLEtBQUssRUFBRSxDQUFDLEtBQUs7UUFDeEIsSUFBSSxFQUFFLG1CQUFtQixFQUFFO1FBQzNCLE9BQU8sRUFBRSxDQUFDLDBCQUEwQixDQUFDO0tBQ3RDLENBQUM7QUFDSixDQUFDIn0=