import { writable } from 'svelte/store';

function createChatStore() {
    const { subscribe, update } = writable({
        messages: [],
        currentMessage: '',
        unreadCount: 0
    });

    return {
        subscribe,
        addMessage: (message) => update(store => ({
            ...store, 
            messages: [...store.messages, message],
            unreadCount: store.unreadCount + 1
        })),
        setCurrentMessage: (message) => update(store => ({ ...store, currentMessage: message })),
        clearCurrentMessage: () => update(store => ({ ...store, currentMessage: '' })),
        resetUnreadCount: () => update(store => ({ ...store, unreadCount: 0 })),
        reset: () => set({
            messages: [],
            currentMessage: '',
            unreadCount: 0
        })
    };
}

export default createChatStore();