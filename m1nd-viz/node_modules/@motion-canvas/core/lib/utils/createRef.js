export function createRef() {
    let value;
    const ref = (newValue) => {
        if (newValue !== undefined) {
            value = newValue;
        }
        else {
            return value;
        }
    };
    return ref;
}
export function makeRef(object, key) {
    return newValue => {
        object[key] = newValue;
    };
}
export function makeRefs() {
    return {};
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY3JlYXRlUmVmLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vc3JjL3V0aWxzL2NyZWF0ZVJlZi50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFRQSxNQUFNLFVBQVUsU0FBUztJQUN2QixJQUFJLEtBQVEsQ0FBQztJQUNiLE1BQU0sR0FBRyxHQUFHLENBQUMsUUFBWSxFQUFFLEVBQUU7UUFDM0IsSUFBSSxRQUFRLEtBQUssU0FBUyxFQUFFO1lBQzFCLEtBQUssR0FBRyxRQUFRLENBQUM7U0FDbEI7YUFBTTtZQUNMLE9BQU8sS0FBSyxDQUFDO1NBQ2Q7SUFDSCxDQUFDLENBQUM7SUFFRixPQUFPLEdBQW1CLENBQUM7QUFDN0IsQ0FBQztBQUVELE1BQU0sVUFBVSxPQUFPLENBQ3JCLE1BQWUsRUFDZixHQUFTO0lBRVQsT0FBTyxRQUFRLENBQUMsRUFBRTtRQUNoQixNQUFNLENBQUMsR0FBRyxDQUFDLEdBQUcsUUFBUSxDQUFDO0lBQ3pCLENBQUMsQ0FBQztBQUNKLENBQUM7QUFRRCxNQUFNLFVBQVUsUUFBUTtJQUd0QixPQUFPLEVBQXFCLENBQUM7QUFDL0IsQ0FBQyJ9