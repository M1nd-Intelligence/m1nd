/**
 * Default tokenizer function used by ownerless code signals.
 *
 * @param input - The code to tokenize.
 */
export function defaultTokenize(input) {
    const tokens = [];
    let currentToken = '';
    let whitespace = false;
    for (const char of input) {
        switch (char) {
            case ' ':
            case '\t':
            case '\n':
                if (!whitespace && currentToken !== '') {
                    tokens.push(currentToken);
                    currentToken = '';
                }
                whitespace = true;
                currentToken += char;
                break;
            case '(':
            case ')':
            case '{':
            case '}':
            case '[':
            case ']':
                if (currentToken !== '') {
                    tokens.push(currentToken);
                    currentToken = '';
                }
                whitespace = false;
                tokens.push(char);
                break;
            default:
                if (whitespace && currentToken !== '') {
                    tokens.push(currentToken);
                    currentToken = '';
                }
                whitespace = false;
                currentToken += char;
                break;
        }
    }
    if (currentToken !== '') {
        tokens.push(currentToken);
    }
    return tokens;
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiQ29kZVRva2VuaXplci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9saWIvY29kZS9Db2RlVG9rZW5pemVyLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUVBOzs7O0dBSUc7QUFDSCxNQUFNLFVBQVUsZUFBZSxDQUFDLEtBQWE7SUFDM0MsTUFBTSxNQUFNLEdBQWEsRUFBRSxDQUFDO0lBQzVCLElBQUksWUFBWSxHQUFHLEVBQUUsQ0FBQztJQUN0QixJQUFJLFVBQVUsR0FBRyxLQUFLLENBQUM7SUFFdkIsS0FBSyxNQUFNLElBQUksSUFBSSxLQUFLLEVBQUU7UUFDeEIsUUFBUSxJQUFJLEVBQUU7WUFDWixLQUFLLEdBQUcsQ0FBQztZQUNULEtBQUssSUFBSSxDQUFDO1lBQ1YsS0FBSyxJQUFJO2dCQUNQLElBQUksQ0FBQyxVQUFVLElBQUksWUFBWSxLQUFLLEVBQUUsRUFBRTtvQkFDdEMsTUFBTSxDQUFDLElBQUksQ0FBQyxZQUFZLENBQUMsQ0FBQztvQkFDMUIsWUFBWSxHQUFHLEVBQUUsQ0FBQztpQkFDbkI7Z0JBQ0QsVUFBVSxHQUFHLElBQUksQ0FBQztnQkFDbEIsWUFBWSxJQUFJLElBQUksQ0FBQztnQkFDckIsTUFBTTtZQUNSLEtBQUssR0FBRyxDQUFDO1lBQ1QsS0FBSyxHQUFHLENBQUM7WUFDVCxLQUFLLEdBQUcsQ0FBQztZQUNULEtBQUssR0FBRyxDQUFDO1lBQ1QsS0FBSyxHQUFHLENBQUM7WUFDVCxLQUFLLEdBQUc7Z0JBQ04sSUFBSSxZQUFZLEtBQUssRUFBRSxFQUFFO29CQUN2QixNQUFNLENBQUMsSUFBSSxDQUFDLFlBQVksQ0FBQyxDQUFDO29CQUMxQixZQUFZLEdBQUcsRUFBRSxDQUFDO2lCQUNuQjtnQkFDRCxVQUFVLEdBQUcsS0FBSyxDQUFDO2dCQUNuQixNQUFNLENBQUMsSUFBSSxDQUFDLElBQUksQ0FBQyxDQUFDO2dCQUNsQixNQUFNO1lBQ1I7Z0JBQ0UsSUFBSSxVQUFVLElBQUksWUFBWSxLQUFLLEVBQUUsRUFBRTtvQkFDckMsTUFBTSxDQUFDLElBQUksQ0FBQyxZQUFZLENBQUMsQ0FBQztvQkFDMUIsWUFBWSxHQUFHLEVBQUUsQ0FBQztpQkFDbkI7Z0JBQ0QsVUFBVSxHQUFHLEtBQUssQ0FBQztnQkFDbkIsWUFBWSxJQUFJLElBQUksQ0FBQztnQkFDckIsTUFBTTtTQUNUO0tBQ0Y7SUFFRCxJQUFJLFlBQVksS0FBSyxFQUFFLEVBQUU7UUFDdkIsTUFBTSxDQUFDLElBQUksQ0FBQyxZQUFZLENBQUMsQ0FBQztLQUMzQjtJQUVELE9BQU8sTUFBTSxDQUFDO0FBQ2hCLENBQUMifQ==