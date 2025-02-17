import { writable } from 'svelte/store';
import { connectionStore, chatStore } from '../stores';
import { generateKeyPair, sign } from './crypto';

export function createWebSocketConnection(url) {
    let socket = null;
    let keyPair = null;

    const { subscribe, set } = writable(null);

    const initializeKeyPair = async() => {
        if(!keyPair) {
            keyPair = await generateKeyPair();
            console.log("keypair generated for signalling");
        }
        return keyPair;
    }

    const connect = async() => {
        if (socket && socket.readyState === WebSocket.OPEN) return;

        await initializeKeyPair();

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

    const sendSecureOffer = async() => {
        if(!keyPair) {
            await initializeKeyPair();
        }

        // Make sure offer is defined and available
        if (!offer) {
            console.error("Offer is undefined in sendSecureOffer!");
            return;
        }

        const offerJSON = JSON.stringify(offer);
        console.log("Offer being signed:", offerJSON);
        
        const signature = await sign(offerJSON, keyPair.privateKey);
        console.log("Generated signature length:", signature.length);
        
        const nonce = crypto.getRandomValues(new Uint8Array(16));
        
        console.log("Public key length:", keyPair.publicKey.length);
        console.log("Public key first bytes:", Array.from(keyPair.publicKey.slice(0, 5)));
        
        const securePayload = {
            offer: offer,
            public_key: Array.from(keyPair.publicKey),
            signature: Array.from(signature),
            nonce: Array.from(nonce)
        }

        console.log("Sending secure payload:", securePayload);
        sendMessage('secure-offer', securePayload);
    }

    const sendSecureAnswer = async() => {
        if (!keyPair) {
            await initializeKeyPair();
        }

        const answerJSON = JSON.stringify(answer);
        const nonce = crypto.getRandomValues(new Uint8Array(16));
        const signature = await sign(answerJSON, keyPair.privateKey);

        const securePayload = {
            offer: answer,
            public_key: Array.from(keyPair.publicKey),
            signature: Array.from(signature),
            nonce: Array.from(nonce)
        }

        sendMessage('secure-answer', securePayload);
    }

    function sendMessage(type, payload) {
        if (socket && socket.readyState === WebSocket.OPEN) {
            const message = JSON.stringify({
                signal_type: type,
                payload: JSON.stringify(payload),
                sender_id: "",
                timestamp: Date.now()
            });
            socket.send(message);
        }
    }

    function handleSignalingMessage(message) {
        switch (message.signal_type) {
            case 'secure-offer':
                handleSecureOffer(JSON.parse(message.payload));
                break;
            case 'secure-answer':
                handleSecureAnswer(JSON.parse(message.payload));
                break;
            case 'ice-candidate':
                handleIceCandidate(JSON.parse(message.payload));
                break;
            case 'chat':
                handleChatMessage(JSON.parse(message.payload));
                break;
            default:
                console.warn('Unknown signal type:', message.signal_type);
        }
    }

    function handleSecureOffer(payload) {
        connectionStore.setOffer(payload.offer);
        connectionStore.setRemotePublicKey(payload.public_key);
        connectionStore.setOfferReceived(true);
    }

    function handleSecureAnswer(payload) {
        connectionStore.setAnswer(payload.offer);
        connectionStore.setRemotePublicKey(payload.public_key);
    }

    function handleIceCandidate(payload) {
        connectionStore.addIceCandidate(payload);
    }

    function handleChatMessage(payload) {
        chatStore.addMessage(payload);
    }


    return {
        subscribe,
        connect,
        disconnect,
        sendMessage,
        sendSecureOffer,
        sendSecureAnswer
    };
}

export const webSocketStore = createWebSocketConnection('ws://localhost:3030');