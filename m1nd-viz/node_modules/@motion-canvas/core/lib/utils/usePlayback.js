const PlaybackStack = [];
/**
 * Get a reference to the playback status.
 */
export function usePlayback() {
    const playback = PlaybackStack.at(-1);
    if (!playback) {
        throw new Error('The playback is not available in the current context.');
    }
    return playback;
}
export function startPlayback(playback) {
    PlaybackStack.push(playback);
}
export function endPlayback(playback) {
    if (PlaybackStack.pop() !== playback) {
        throw new Error('startPlayback/endPlayback were called out of order.');
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidXNlUGxheWJhY2suanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvdXRpbHMvdXNlUGxheWJhY2sudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBRUEsTUFBTSxhQUFhLEdBQXFCLEVBQUUsQ0FBQztBQUUzQzs7R0FFRztBQUNILE1BQU0sVUFBVSxXQUFXO0lBQ3pCLE1BQU0sUUFBUSxHQUFHLGFBQWEsQ0FBQyxFQUFFLENBQUMsQ0FBQyxDQUFDLENBQUMsQ0FBQztJQUN0QyxJQUFJLENBQUMsUUFBUSxFQUFFO1FBQ2IsTUFBTSxJQUFJLEtBQUssQ0FBQyx1REFBdUQsQ0FBQyxDQUFDO0tBQzFFO0lBQ0QsT0FBTyxRQUFRLENBQUM7QUFDbEIsQ0FBQztBQUVELE1BQU0sVUFBVSxhQUFhLENBQUMsUUFBd0I7SUFDcEQsYUFBYSxDQUFDLElBQUksQ0FBQyxRQUFRLENBQUMsQ0FBQztBQUMvQixDQUFDO0FBRUQsTUFBTSxVQUFVLFdBQVcsQ0FBQyxRQUF3QjtJQUNsRCxJQUFJLGFBQWEsQ0FBQyxHQUFHLEVBQUUsS0FBSyxRQUFRLEVBQUU7UUFDcEMsTUFBTSxJQUFJLEtBQUssQ0FBQyxxREFBcUQsQ0FBQyxDQUFDO0tBQ3hFO0FBQ0gsQ0FBQyJ9