import { writable } from 'svelte/store';

function createConnectionStore() {
    const { subscribe, set, update } = writable({
        signalingStatus: 'Disconnected',
        peerConnectionStatus: 'Disconnected',
        isOfferReceived: false,
        connectionId: null,
        error: null
    });

    return {
        subscribe,
        setSignalingStatus: (status) => update(store => ({ ...store, signalingStatus: status })),
        setPeerConnectionStatus: (status) => update(store => ({ ...store, peerConnectionStatus: status })),
        setOfferReceived: (isReceived) => update(store => ({ ...store, isOfferReceived: isReceived })),
        setConnectionId: (id) => update(store => ({ ...store, connectionId: id })),
        setError: (error) => update(store => ({ ...store, error })),
        reset: () => set({
            signalingStatus: 'Disconnected',
            peerConnectionStatus: 'Disconnected',
            isOfferReceived: false,
            connectionId: null,
            error: null
        })
    };
}

export default createConnectionStore();