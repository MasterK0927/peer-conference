# API Reference Documentation

## Overview

This document provides a comprehensive API reference for the peer-to-peer video conferencing application's core modules. It includes detailed function signatures, parameters, return values, and usage examples for all public APIs.

## WebSocket Utility API

### `createWebSocketConnection(url)`

Creates a WebSocket connection manager with secure signaling capabilities.

**Parameters:**
- `url` (string): WebSocket server URL (e.g., 'ws://localhost:3030')

**Returns:**
```javascript
{
    subscribe: (callback: (socket: WebSocket | null) => void) => () => void,
    connect: () => Promise<void>,
    disconnect: () => void,
    sendMessage: (type: string, payload: any) => void,
    sendSecureOffer: () => Promise<void>,
    sendSecureAnswer: () => Promise<void>
}
```

**Example:**
```javascript
import { createWebSocketConnection } from './utils/websocket.js';

const wsConnection = createWebSocketConnection('ws://localhost:3030');

// Subscribe to connection state
const unsubscribe = wsConnection.subscribe(socket => {
    console.log('Socket state:', socket ? 'Connected' : 'Disconnected');
});

// Connect
await wsConnection.connect();

// Send message
wsConnection.sendMessage('custom-type', { data: 'example' });

// Cleanup
unsubscribe();
wsConnection.disconnect();
```

#### `connect()`

Establishes WebSocket connection with cryptographic key initialization.

**Returns:** `Promise<void>`

**Throws:** WebSocket connection errors

**Side Effects:**
- Initializes ECDSA key pair
- Updates `connectionStore.signalingStatus`
- Registers WebSocket event handlers

#### `disconnect()`

Closes the WebSocket connection.

**Returns:** `void`

**Side Effects:**
- Closes WebSocket connection
- Updates `connectionStore.signalingStatus` to 'Disconnected'

#### `sendMessage(type, payload)`

Sends a generic message through the WebSocket.

**Parameters:**
- `type` (string): Message type identifier
- `payload` (any): Message payload (will be JSON-serialized)

**Returns:** `void`

**Message Format:**
```javascript
{
    signal_type: string,
    payload: string, // JSON-serialized
    sender_id: string, // Currently empty
    timestamp: number
}
```

#### `sendSecureOffer()`

Sends a cryptographically signed WebRTC offer.

**Returns:** `Promise<void>`

**Requires:** Global `offer` variable to be defined

**Throws:**
- Key generation errors
- Signing errors
- Missing offer error

**Payload Format:**
```javascript
{
    offer: RTCSessionDescription,
    public_key: number[], // Uint8Array as array
    signature: number[],  // Uint8Array as array
    nonce: number[]       // 16-byte random nonce
}
```

#### `sendSecureAnswer()`

Sends a cryptographically signed WebRTC answer.

**Returns:** `Promise<void>`

**Requires:** Global `answer` variable to be defined

**Payload Format:** Same as `sendSecureOffer` but with `offer` field containing answer

## Crypto Utility API

### `generateKeyPair()`

Generates ECDSA P-256 key pair for cryptographic operations.

**Returns:**
```javascript
Promise<{
    publicKey: Uint8Array,  // 65 bytes, uncompressed EC point
    privateKey: CryptoKey   // Web Crypto API key object
}>
```

**Example:**
```javascript
import { generateKeyPair } from './utils/crypto.js';

const keyPair = await generateKeyPair();
console.log('Public key length:', keyPair.publicKey.length); // 65
console.log('Public key format:', keyPair.publicKey[0]);     // 0x04
```

**Key Format:**
- **Public Key**: Uncompressed EC point (0x04 + X + Y coordinates)
- **Private Key**: Non-extractable Web Crypto CryptoKey

### `sign(data, privateKey)`

Creates ECDSA signature for message authenticity.

**Parameters:**
- `data` (string): Message to sign
- `privateKey` (CryptoKey): ECDSA private key

**Returns:** `Promise<Uint8Array>` - 64-byte signature (r||s format)

**Throws:** Signing operation errors

**Example:**
```javascript
import { sign } from './utils/crypto.js';

const message = "Hello, secure world!";
const signature = await sign(message, privateKey);
console.log('Signature length:', signature.length); // 64
```

### `verify(data, signature, publicKey)`

Verifies ECDSA signature using public key.

**Parameters:**
- `data` (string): Original message that was signed
- `signature` (Uint8Array): 64-byte signature to verify
- `publicKey` (Uint8Array): 65-byte public key

**Returns:** `Promise<boolean>` - Verification result

**Example:**
```javascript
import { verify } from './utils/crypto.js';

const isValid = await verify(message, signature, publicKey);
if (isValid) {
    console.log('Signature is valid');
} else {
    console.log('Signature verification failed');
}
```

### `base64UrlToUint8Array(base64Url)`

Converts base64url-encoded string to Uint8Array.

**Parameters:**
- `base64Url` (string): Base64url-encoded string

**Returns:** `Uint8Array` - Decoded byte array

**Example:**
```javascript
const encoded = "SGVsbG8gV29ybGQ";
const decoded = base64UrlToUint8Array(encoded);
console.log(new TextDecoder().decode(decoded)); // "Hello World"
```

## Connection Store API

### Store State

```javascript
{
    signalingStatus: 'Disconnected' | 'Connected' | 'Error',
    peerConnectionStatus: string,
    isOfferReceived: boolean,
    connectionId: string | null,
    error: Error | null,
    peerConnection: RTCPeerConnection | null,
    offer: RTCSessionDescription | null,
    answer: RTCSessionDescription | null,
    remotePublicKey: Uint8Array | null,
    localPublicKey: Uint8Array | null,
    iceCandidates: RTCIceCandidate[],
    dataChannel: RTCDataChannel | null
}
```

### State Setters

#### `setSignalingStatus(status)`

**Parameters:** `status` (string) - WebSocket connection status

#### `setPeerConnectionStatus(status)`

**Parameters:** `status` (string) - WebRTC connection status

#### `setOfferReceived(isReceived)`

**Parameters:** `isReceived` (boolean) - Offer reception flag

#### `setConnectionId(id)`

**Parameters:** `id` (string | null) - Connection identifier

#### `setError(error)`

**Parameters:** `error` (Error | null) - Error object

#### `setOffer(offer)`

**Parameters:** `offer` (RTCSessionDescription | null) - WebRTC offer

#### `setAnswer(answer)`

**Parameters:** `answer` (RTCSessionDescription | null) - WebRTC answer

#### `setRemotePublicKey(publicKey)`

**Parameters:** `publicKey` (Uint8Array | null) - Remote peer's public key

#### `setLocalPublicKey(publicKey)`

**Parameters:** `publicKey` (Uint8Array | null) - Local public key

### Connection Management

#### `createOffer()`

Creates WebRTC offer for initiating peer connection.

**Returns:** `Promise<RTCSessionDescription>`

**Throws:** WebRTC operation errors

**Side Effects:**
- Creates RTCPeerConnection if needed
- Creates data channel named 'secure-channel'
- Sets local description
- Updates store with offer

**Example:**
```javascript
try {
    const offer = await connectionStore.createOffer();
    // Send offer through WebSocket
    webSocketStore.sendSecureOffer();
} catch (error) {
    console.error('Failed to create offer:', error);
}
```

#### `processOffer(offer)`

Processes incoming WebRTC offer and creates answer.

**Parameters:**
- `offer` (RTCSessionDescription): Received WebRTC offer

**Returns:** `Promise<RTCSessionDescription>`

**Throws:** WebRTC operation errors

**Side Effects:**
- Creates RTCPeerConnection if needed
- Sets remote description
- Creates and sets local answer
- Updates store with answer

#### `processAnswer(answer)`

Processes incoming WebRTC answer.

**Parameters:**
- `answer` (RTCSessionDescription): Received WebRTC answer

**Returns:** `Promise<void>`

**Throws:** WebRTC operation errors

**Side Effects:**
- Sets remote description on peer connection

#### `addIceCandidate(candidate)`

Adds ICE candidate to peer connection.

**Parameters:**
- `candidate` (RTCIceCandidate): ICE candidate to add

**Returns:** `Promise<void>`

**Throws:** ICE candidate errors

#### `closeConnection()`

Closes all connection resources.

**Returns:** `void`

**Side Effects:**
- Closes data channel
- Closes peer connection
- Resets connection state

#### `reset()`

Resets store to initial state.

**Returns:** `void`

## Chat Store API

### Store State

```javascript
{
    messages: Array<Message>,
    currentMessage: string,
    unreadCount: number
}
```

### Message Management

#### `addMessage(message)`

Adds message to chat history.

**Parameters:**
- `message` (object): Message object to add

**Side Effects:**
- Appends to messages array
- Increments unread count

**Example:**
```javascript
chatStore.addMessage({
    id: Date.now().toString(),
    text: 'Hello!',
    timestamp: Date.now(),
    sender: 'user123'
});
```

### Input Management

#### `setCurrentMessage(message)`

Updates current message input.

**Parameters:**
- `message` (string): Current input text

#### `clearCurrentMessage()`

Clears current message input.

**Returns:** `void`

### Notification Management

#### `resetUnreadCount()`

Resets unread message counter to zero.

**Returns:** `void`

#### `reset()`

Resets store to initial state.

**Returns:** `void`

## Media Store API

### Store State

```javascript
{
    localStream: MediaStream | null,
    remoteStream: MediaStream | null,
    devices: {
        audio: { muted: boolean, available: boolean },
        video: { muted: boolean, available: boolean },
        screen: { sharing: boolean }
    }
}
```

### Stream Management

#### `setLocalStream(stream)`

Sets local media stream.

**Parameters:**
- `stream` (MediaStream | null): Local media stream

**Example:**
```javascript
const stream = await navigator.mediaDevices.getUserMedia({
    video: true,
    audio: true
});
mediaStore.setLocalStream(stream);
```

#### `setRemoteStream(stream)`

Sets remote media stream.

**Parameters:**
- `stream` (MediaStream | null): Remote media stream

### Device Controls

#### `toggleAudio()`

Toggles local audio mute state.

**Returns:** `void`

**Side Effects:** Inverts `devices.audio.muted`

#### `toggleVideo()`

Toggles local video mute state.

**Returns:** `void`

**Side Effects:** Inverts `devices.video.muted`

#### `toggleScreenShare()`

Toggles screen sharing state.

**Returns:** `void`

**Side Effects:** Inverts `devices.screen.sharing`

#### `reset()`

Resets store to initial state.

**Returns:** `void`

## Store Exports

### `svelte-video-conferencing/src/stores/index.js`

```javascript
export { default as authStore } from './auth';
export { default as chatStore } from './chat';
export { default as connectionStore } from './connection';
export { default as mediaStore } from './media';
```

## Error Handling Patterns

### Async Function Error Handling

```javascript
try {
    const result = await someAsyncFunction();
    // Handle success
} catch (error) {
    console.error('Operation failed:', error);
    // Update error state
    store.setError(error);
    // Optionally re-throw
    throw error;
}
```

### WebSocket Error Handling

```javascript
socket.onerror = (error) => {
    connectionStore.setSignalingStatus('Error');
    connectionStore.setError(error);
    console.error('WebSocket error:', error);
};
```

### Media Device Error Handling

```javascript
try {
    const stream = await navigator.mediaDevices.getUserMedia(constraints);
    mediaStore.setLocalStream(stream);
} catch (error) {
    switch (error.name) {
        case 'NotAllowedError':
            console.error('Permission denied');
            break;
        case 'NotFoundError':
            console.error('No devices found');
            break;
        default:
            console.error('Unknown error:', error);
    }
    throw error;
}
```

## Common Usage Patterns

### Complete Connection Setup

```javascript
import { webSocketStore, connectionStore, mediaStore } from './stores';

async function initializeConnection() {
    // 1. Get user media
    const stream = await navigator.mediaDevices.getUserMedia({
        video: true,
        audio: true
    });
    mediaStore.setLocalStream(stream);

    // 2. Connect to signaling server
    await webSocketStore.connect();

    // 3. Create and send offer
    const offer = await connectionStore.createOffer();
    await webSocketStore.sendSecureOffer();
}
```

### Message Sending

```javascript
function sendChatMessage(text) {
    const message = {
        id: Date.now().toString(),
        text: text,
        timestamp: Date.now(),
        sender: 'local'
    };

    // Send through WebSocket
    webSocketStore.sendMessage('chat', message);

    // Add to local store for immediate UI update
    chatStore.addMessage(message);

    // Clear input
    chatStore.clearCurrentMessage();
}
```

### Media Control Implementation

```javascript
function toggleAudioWithTrack() {
    mediaStore.toggleAudio();

    // Actually control the track
    const state = get(mediaStore);
    if (state.localStream) {
        const audioTrack = state.localStream.getAudioTracks()[0];
        if (audioTrack) {
            audioTrack.enabled = !state.devices.audio.muted;
        }
    }
}
```

## Type Definitions

### Message Types

```typescript
interface ChatMessage {
    id: string;
    text: string;
    timestamp: number;
    sender: string;
    type?: 'text' | 'system' | 'file';
}

interface SignalingMessage {
    signal_type: string;
    payload: string; // JSON-serialized
    sender_id: string;
    timestamp: number;
}

interface SecurePayload {
    offer: RTCSessionDescription;
    public_key: number[];
    signature: number[];
    nonce: number[];
}
```

### Store State Types

```typescript
interface ConnectionState {
    signalingStatus: 'Disconnected' | 'Connected' | 'Error';
    peerConnectionStatus: string;
    isOfferReceived: boolean;
    connectionId: string | null;
    error: Error | null;
    peerConnection: RTCPeerConnection | null;
    offer: RTCSessionDescription | null;
    answer: RTCSessionDescription | null;
    remotePublicKey: Uint8Array | null;
    localPublicKey: Uint8Array | null;
    iceCandidates: RTCIceCandidate[];
    dataChannel: RTCDataChannel | null;
}

interface MediaState {
    localStream: MediaStream | null;
    remoteStream: MediaStream | null;
    devices: {
        audio: { muted: boolean; available: boolean };
        video: { muted: boolean; available: boolean };
        screen: { sharing: boolean };
    };
}

interface ChatState {
    messages: ChatMessage[];
    currentMessage: string;
    unreadCount: number;
}
```

## WebRTC Configuration

### ICE Servers Configuration

```javascript
const iceServers = [
    { urls: 'stun:stun.l.google.com:19302' }
    // Add TURN servers for production:
    // {
    //     urls: 'turn:turnserver.com:3478',
    //     username: 'username',
    //     credential: 'password'
    // }
];
```

### Media Constraints

```javascript
const mediaConstraints = {
    video: {
        width: { ideal: 1280 },
        height: { ideal: 720 },
        frameRate: { ideal: 30 }
    },
    audio: {
        echoCancellation: true,
        noiseSuppression: true
    }
};
```

## Security Considerations

### Input Validation

Always validate inputs before processing:

```javascript
function validateMessage(message) {
    if (!message || typeof message !== 'object') {
        throw new Error('Invalid message format');
    }
    if (!message.text || typeof message.text !== 'string') {
        throw new Error('Message text required');
    }
    if (message.text.length > MAX_MESSAGE_LENGTH) {
        throw new Error('Message too long');
    }
}
```

### Signature Verification

Always verify signatures before processing signed messages:

```javascript
const isValid = await verify(
    JSON.stringify(payload.offer),
    new Uint8Array(payload.signature),
    new Uint8Array(payload.public_key)
);

if (!isValid) {
    throw new Error('Invalid signature - potential security threat');
}
```

### Resource Cleanup

Always clean up resources to prevent memory leaks:

```javascript
function cleanup() {
    // Stop media tracks
    const state = get(mediaStore);
    if (state.localStream) {
        state.localStream.getTracks().forEach(track => track.stop());
    }

    // Close connections
    connectionStore.closeConnection();

    // Reset stores
    mediaStore.reset();
    chatStore.reset();
    connectionStore.reset();
}
```