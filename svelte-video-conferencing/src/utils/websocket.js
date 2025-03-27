import { writable } from 'svelte/store';
import { connectionStore } from '../stores';

export function createWebSocketConnection(url) {
    let socket = null;

    const { subscribe, set } = writable(null);

    function connect() {
        if (socket && socket.readyState === WebSocket.OPEN) return;

        socket = new WebSocket(url);

        socket.onopen = () => {
            connectionStore.setSignalingStatus('Connected');
            set(socket);
        };

        socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                handleSignalingMessage(message);
            } catch (error) {
                console.error('WebSocket message parsing error:', error);
            }
        };

        socket.onerror = (error) => {
            connectionStore.setSignalingStatus('Error');
            connectionStore.setError(error);
            console.error('WebSocket error:', error);
        };

        socket.onclose = () => {
            connectionStore.setSignalingStatus('Disconnected');
            set(null);
        };
    }

    function disconnect() {
        if (socket) {
            socket.close();
        }
    }

    function sendMessage(type, payload) {
        if (socket && socket.readyState === WebSocket.OPEN) {
            const message = JSON.stringify({
                signal_type: type,
                payload: JSON.stringify(payload),
                timestamp: Date.now()
            });
            socket.send(message);
        }
    }

    function handleSignalingMessage(message) {
        // Implement message handling logic
        switch (message.signal_type) {
            case 'offer-with-challenge':
                handleOfferWithChallenge(message.payload);
                break;
            case 'chat':
                handleChatMessage(message.payload);
                break;
            // Add more message type handlers
        }
    }

    function handleOfferWithChallenge(payload) {
        // Implementation of offer challenge handling
        connectionStore.setOfferReceived(true);
    }

    function handleChatMessage(payload) {
        // Implementation of chat message handling
        chatStore.addMessage(payload);
    }

    return {
        subscribe,
        connect,
        disconnect,
        sendMessage
    };
}

export const webSocketStore = createWebSocketConnection('ws://localhost:3030');