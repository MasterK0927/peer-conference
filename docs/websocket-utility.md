# WebSocket Utility Documentation

## Overview

The WebSocket utility (`svelte-video-conferencing/src/utils/websocket.js`) provides a secure WebSocket connection manager for peer-to-peer video conferencing with cryptographic signatures and message handling. It integrates with Svelte stores for state management and implements secure signaling using ECDSA cryptographic signatures.

## Architecture

The utility creates a reactive WebSocket connection that:
- Manages secure signaling between peers
- Handles cryptographic key pair generation and signatures
- Processes different types of signaling messages
- Integrates with the application's state management system

## Core Components

### Main Export

```javascript
export const webSocketStore = createWebSocketConnection('ws://localhost:3030');
```

The main export creates a WebSocket connection to the local signaling server on port 3030.

### `createWebSocketConnection(url)`

The factory function that creates and manages a WebSocket connection with the following capabilities:

#### Parameters
- `url` (string): The WebSocket server URL

#### Returns
An object with methods:
- `subscribe`: Svelte store subscription method
- `connect()`: Establishes WebSocket connection
- `disconnect()`: Closes WebSocket connection
- `sendMessage(type, payload)`: Sends generic messages
- `sendSecureOffer()`: Sends cryptographically signed offers
- `sendSecureAnswer()`: Sends cryptographically signed answers

## Internal State Management

### Local State
- `socket`: WebSocket instance
- `keyPair`: ECDSA key pair for cryptographic operations
- Svelte writable store containing the current socket state

### Integration with Application Stores

#### Connection Store (`connectionStore`)
- `setSignalingStatus(status)`: Updates WebSocket connection status
- `setError(error)`: Sets error state
- `setOffer(offer)`: Stores received offers
- `setAnswer(answer)`: Stores received answers
- `setRemotePublicKey(publicKey)`: Stores remote peer's public key
- `setOfferReceived(boolean)`: Marks offer reception status
- `addIceCandidate(candidate)`: Adds ICE candidates for WebRTC

#### Chat Store (`chatStore`)
- `addMessage(message)`: Adds chat messages to the store

## Cryptographic Operations

### Key Pair Generation
```javascript
const initializeKeyPair = async() => {
    if(!keyPair) {
        keyPair = await generateKeyPair();
        console.log("keypair generated for signalling");
    }
    return keyPair;
}
```

- Uses ECDSA P-256 curve via the crypto utility
- Generates keys on first use and caches them
- Keys are used for signing offers and answers

### Secure Offer Process
```javascript
const sendSecureOffer = async() => {
    const offerJSON = JSON.stringify(offer);
    const signature = await sign(offerJSON, keyPair.privateKey);
    const nonce = crypto.getRandomValues(new Uint8Array(16));

    const securePayload = {
        offer: offer,
        public_key: Array.from(keyPair.publicKey),
        signature: Array.from(signature),
        nonce: Array.from(nonce)
    }

    sendMessage('secure-offer', securePayload);
}
```

### Secure Answer Process
Similar to secure offers, answers are cryptographically signed before transmission.

## Message Types

### Outgoing Messages

#### Generic Message Format
```javascript
{
    signal_type: string,
    payload: string (JSON-serialized),
    sender_id: string (empty),
    timestamp: number
}
```

#### Secure Offer/Answer Format
```javascript
{
    offer: RTCSessionDescription,
    public_key: Array<number>, // Uint8Array converted to array
    signature: Array<number>,  // Uint8Array converted to array
    nonce: Array<number>       // 16-byte random nonce
}
```

### Incoming Message Handling

The utility handles four types of incoming messages:

1. **secure-offer**: Cryptographically signed WebRTC offers
2. **secure-answer**: Cryptographically signed WebRTC answers
3. **ice-candidate**: ICE candidates for NAT traversal
4. **chat**: Chat messages between peers

### Message Handlers

#### `handleSecureOffer(payload)`
- Stores the received offer in connection store
- Extracts and stores remote peer's public key
- Marks offer as received

#### `handleSecureAnswer(payload)`
- Stores the received answer in connection store
- Extracts and stores remote peer's public key

#### `handleIceCandidate(payload)`
- Forwards ICE candidates to connection store for WebRTC processing

#### `handleChatMessage(payload)`
- Forwards chat messages to chat store

## Connection Management

### Connection Lifecycle

#### Connecting
```javascript
const connect = async() => {
    if (socket && socket.readyState === WebSocket.OPEN) return;

    await initializeKeyPair(); // Ensure crypto keys are ready
    socket = new WebSocket(url);

    // Set up event handlers...
}
```

#### Event Handlers
- `onopen`: Updates signaling status to 'Connected'
- `onmessage`: Parses and routes incoming messages
- `onerror`: Updates status to 'Error' and logs errors
- `onclose`: Updates status to 'Disconnected'

#### Disconnecting
```javascript
function disconnect() {
    if (socket) {
        socket.close();
    }
}
```

## Dependencies

### External Libraries
- `svelte/store`: For reactive state management
- `../stores`: Application-specific stores (connection, chat)
- `./crypto`: Cryptographic utilities for ECDSA operations

### Crypto Utility Functions
- `generateKeyPair()`: Generates ECDSA P-256 key pairs
- `sign(data, privateKey)`: Creates ECDSA signatures

## Security Features

### Cryptographic Signatures
- All offers and answers are signed using ECDSA P-256
- Signatures prevent tampering and ensure authenticity
- Public keys are transmitted with signed messages for verification

### Nonce Generation
- 16-byte random nonces are included in secure payloads
- Helps prevent replay attacks

### Error Handling
- Comprehensive error handling for WebSocket operations
- Graceful degradation when crypto operations fail
- Detailed logging for debugging

## Usage Example

```javascript
import { webSocketStore } from './utils/websocket.js';

// Subscribe to connection state
webSocketStore.subscribe(socket => {
    if (socket) {
        console.log('Connected to signaling server');
    }
});

// Connect to server
await webSocketStore.connect();

// Send a secure offer (requires offer to be defined globally)
await webSocketStore.sendSecureOffer();

// Send a generic message
webSocketStore.sendMessage('custom-type', { data: 'example' });

// Disconnect
webSocketStore.disconnect();
```

## Configuration

### Server URL
The default configuration connects to `ws://localhost:3030`. To use a different server:

```javascript
export const webSocketStore = createWebSocketConnection('ws://your-server.com:port');
```

### Message Format
Messages follow a standardized format with:
- `signal_type`: Message type identifier
- `payload`: JSON-serialized message data
- `sender_id`: Currently empty string
- `timestamp`: Message timestamp

## Error Handling

### Connection Errors
- WebSocket connection failures update the connection store error state
- Detailed error logging to console
- Automatic status updates for UI reflection

### Message Parsing Errors
- JSON parsing errors are caught and logged
- Invalid messages don't crash the connection

### Cryptographic Errors
- Key generation failures are logged
- Signature operations include error handling

## Integration Points

### With WebRTC (`connection.js`)
- Processes WebRTC offers and answers
- Handles ICE candidate exchange
- Manages peer connection state

### With Chat System (`chat.js`)
- Routes chat messages to chat store
- Maintains message history and unread counts

### With UI Components
- Reactive connection status updates
- Error state propagation
- Real-time message handling

## Limitations and Considerations

### Global Variables
- `offer` and `answer` variables are accessed globally (potential improvement area)
- Could benefit from better encapsulation

### Error Recovery
- No automatic reconnection logic
- Manual reconnection required after connection failures

### Scalability
- Single WebSocket connection per instance
- No connection pooling or load balancing

### Security Notes
- Cryptographic signatures provide authenticity but not encryption
- WebSocket traffic is not encrypted (consider WSS in production)
- Public keys are transmitted in plaintext

## Future Improvements

1. **Automatic Reconnection**: Implement exponential backoff reconnection logic
2. **Message Queuing**: Queue messages when connection is down
3. **Connection Pooling**: Support multiple concurrent connections
4. **Encryption**: Add end-to-end encryption beyond signatures
5. **Better State Management**: Eliminate global variable dependencies
6. **Connection Health**: Implement heartbeat/ping-pong for connection monitoring
7. **Message Acknowledgments**: Ensure reliable message delivery
8. **Configuration Management**: External configuration for server URLs and timeouts