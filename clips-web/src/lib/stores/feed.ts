import { writable } from 'svelte/store';

export const feedEvents = writable<any[]>([]);

export function addFeedEvent(event: any) {
  feedEvents.update(items => [event, ...items].slice(0, 50));
}
