# Connection Store Documentation

## Overview

The Connection Store (`svelte-video-conferencing/src/stores/connection.js`) is the central state management system for WebRTC peer-to-peer connections in the video conferencing application. It handles the complete lifecycle of peer connections, from initial signaling to data channel communication.

## Architecture

The connection store implements a factory pattern that creates a reactive Svelte store with comprehensive WebRTC functionality:

- **State Management**: Centralized connection state tracking
- **WebRTC Integration**: Complete RTCPeerConnection lifecycle management
- **Signaling Support**: Integration with WebSocket signaling
- **Data Channels**: Secure peer-to-peer communication channels
- **ICE Handling**: STUN/TURN server integration for NAT traversal

## State Schema

### Core State Structure
```javascript
{
    signalingStatus: 'Disconnected' | 'Connected' | 'Error',
    peerConnectionStatus: string, // RTCPeerConnection.connectionState
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

### State Properties

#### Connection Status
- **`signalingStatus`**: WebSocket signaling server connection state
- **`peerConnectionStatus`**: WebRTC peer connection state (`new`, `connecting`, `connected`, `disconnected`, `failed`, `closed`)
- **`isOfferReceived`**: Flag indicating if a WebRTC offer has been received from remote peer

#### Identity & Security
- **`connectionId`**: Unique identifier for the connection session
- **`remotePublicKey`**: ECDSA public key from remote peer for signature verification
- **`localPublicKey`**: Local ECDSA public key for signing operations

#### WebRTC Objects
- **`peerConnection`**: The main RTCPeerConnection instance
- **`offer`**: WebRTC session description offer (SDP)
- **`answer`**: WebRTC session description answer (SDP)
- **`iceCandidates`**: Array of ICE candidates for NAT traversal
- **`dataChannel`**: RTCDataChannel for direct peer-to-peer messaging

#### Error Handling
- **`error`**: Latest error object for debugging and user feedback

## Core Functions

### Connection Creation & Management

#### `createPeerConnection()`
```javascript
const createPeerConnection = () => {
    const peerConnection = new RTCPeerConnection({
        iceServers: [
            { urls: 'stun:stun.l.google.com:19302' }
        ]
    });

    peerConnection.onicecandidate = handleIceCandidate;
    peerConnection.onconnectionstatechange = handleConnectionStateChange;
    peerConnection.oniceconnectionstatechange = handleIceConnectionStateChange;
    peerConnection.ondatachannel = handleDataChannel;

    update(store => ({ ...store, peerConnection }));
    return peerConnection;
}
```

**Purpose**: Creates and configures a new RTCPeerConnection with event handlers

**Configuration**:
- Uses Google's public STUN server for ICE candidate gathering
- Registers all necessary event handlers for connection lifecycle

#### `closeConnection()`
```javascript
function closeConnection() {
    update(store => {
        if (store.dataChannel) store.dataChannel.close();
        if (store.peerConnection) store.peerConnection.close();

        return {
            ...store,
            peerConnection: null,
            dataChannel: null,
            peerConnectionStatus: 'Disconnected'
        };
    });
}
```

**Purpose**: Cleanly closes all connection resources

### Offer/Answer Exchange

#### `createOffer()`
```javascript
const createOffer = async() => {
    const peerConnection = /* get or create peer connection */;

    try {
        // Create data channel for communication
        const dataChannel = peerConnection.createDataChannel('secure-channel');
        setupDataChannel(dataChannel);

        // Create offer
        const offer = await peerConnection.createOffer({
            offerToReceiveAudio: true,
            offerToReceiveVideo: true
        });

        await peerConnection.setLocalDescription(offer);
        update(store => ({ ...store, offer: offer }));

        return offer;
    } catch (error) {
        console.error('Error creating offer:', error);
        update(store => ({ ...store, error }));
        throw error;
    }
}
```

**Purpose**: Creates WebRTC offer for initiating peer connection
- Creates data channel for direct messaging
- Configures to receive both audio and video
- Sets local description and stores offer in state

#### `processOffer(offer)`
```javascript
const processOffer = async(offer) => {
    try {
        const peerConnection = /* get or create peer connection */;

        await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
        const answer = await peerConnection.createAnswer();
        await peerConnection.setLocalDescription(answer);

        update(store => ({ ...store, answer }));
        return answer;
    } catch (error) {
        console.error('Error processing offer:', error);
        update(store => ({ ...store, error }));
        throw error;
    }
}
```

**Purpose**: Processes incoming WebRTC offer and creates answer
- Sets remote description from received offer
- Creates and sets local answer
- Returns answer for transmission to remote peer

#### `processAnswer(answer)`
```javascript
async function processAnswer(answer) {
    try {
        update(store => {
            if (store.peerConnection && store.peerConnection.signalingState !== 'stable') {
                store.peerConnection.setRemoteDescription(new RTCSessionDescription(answer));
            }
            return store;
        });
    } catch (error) {
        console.error('Error processing answer:', error);
        update(store => ({ ...store, error }));
        throw error;
    }
}
```

**Purpose**: Processes incoming WebRTC answer
- Validates signaling state before setting remote description
- Completes the offer/answer exchange

### ICE Candidate Handling

#### `handleIceCandidate(event)`
```javascript
const handleIceCandidate = (e) => {
    if (e.candidate) {
        const candidate = {
            candidate: e.candidate.candidate,
            sdpMid: e.candidate.sdpMid,
            sdpMLineIndex: e.candidate.sdpMLineIndex
        };

        update(store => ({
            ...store,
            iceCandidates: [...store.iceCandidates, candidate]
        }));
    }
}
```

**Purpose**: Handles ICE candidate discovery events
- Extracts candidate information from RTCPeerConnectionIceEvent
- Stores candidates in state for transmission to remote peer

#### `addIceCandidate(candidate)`
```javascript
async function addIceCandidate(candidate) {
    try {
        update(store => {
            if (store.peerConnection) {
                store.peerConnection.addIceCandidate(new RTCIceCandidate(candidate));
            }
            return store;
        });
    } catch (error) {
        console.error('Error adding ICE candidate:', error);
        update(store => ({ ...store, error }));
        throw error;
    }
}
```

**Purpose**: Adds received ICE candidates to peer connection
- Creates RTCIceCandidate from received data
- Adds to peer connection for connectivity establishment

### Data Channel Management

#### `setupDataChannel(dataChannel)`
```javascript
const setupDataChannel = (dataChannel) => {
    dataChannel.onopen = () => console.log('Data channel opened');
    dataChannel.onclose = () => console.log('Data channel closed');
    dataChannel.onmessage = (e) => {
        console.log('Data channel message received:', e.data);
        // Handle incoming messages
    };

    update(store => ({ ...store, dataChannel }));
}
```

**Purpose**: Configures data channel event handlers
- Sets up open/close/message event listeners
- Stores data channel reference in state

#### `handleDataChannel(event)`
```javascript
const handleDataChannel = (e) => {
    const dataChannel = e.channel;
    setupDataChannel(dataChannel);
}
```

**Purpose**: Handles incoming data channel from remote peer
- Called when remote peer creates data channel
- Sets up the received channel with event handlers

## State Setters

### Basic Setters
```javascript
setSignalingStatus: (status) => update(store => ({ ...store, signalingStatus: status }))
setPeerConnectionStatus: (status) => update(store => ({ ...store, peerConnectionStatus: status }))
setOfferReceived: (isReceived) => update(store => ({ ...store, isOfferReceived: isReceived }))
setConnectionId: (id) => update(store => ({ ...store, connectionId: id }))
setError: (error) => update(store => ({ ...store, error }))
setOffer: (offer) => update(store => ({ ...store, offer }))
setAnswer: (answer) => update(store => ({ ...store, answer }))
setRemotePublicKey: (remotePublicKey) => update(store => ({ ...store, remotePublicKey }))
setLocalPublicKey: (localPublicKey) => update(store => ({ ...store, localPublicKey }))
```

### Reset Function
```javascript
reset: () => set({
    signalingStatus: 'Disconnected',
    peerConnectionStatus: 'Disconnected',
    isOfferReceived: false,
    connectionId: null,
    error: null,
    peerConnection: null,
    offer: null,
    answer: null,
    remotePublicKey: null,
    localPublicKey: null,
    iceCandidates: [],
    dataChannel: null
})
```

## Event Handlers

### Connection State Changes

#### `handleConnectionStateChange(event)`
```javascript
const handleConnectionStateChange = (e) => {
    update(store => {
        const state = store.peerConnection?.connectionState || 'unknown';
        console.log('Connection state changed:', state);
        return { ...store, peerConnectionStatus: state };
    });
}
```

**States**: `new`, `connecting`, `connected`, `disconnected`, `failed`, `closed`

#### `handleIceConnectionStateChange(event)`
```javascript
const handleIceConnectionStateChange = (e) => {
    update(store => {
        const state = store.peerConnection?.iceConnectionState || 'unknown';
        console.log('ICE connection state changed:', state);
        return { ...store };
    });
}
```

**States**: `new`, `checking`, `connected`, `completed`, `failed`, `disconnected`, `closed`

## Integration Points

### With WebSocket Utility
- **Signaling Status**: Updates from WebSocket connection events
- **Offer/Answer Exchange**: Processes signed offers and answers from WebSocket
- **ICE Candidates**: Receives and processes ICE candidates via WebSocket
- **Error Propagation**: WebSocket errors are reflected in connection state

### With UI Components
- **Reactive Updates**: Svelte components subscribe to connection state changes
- **Status Display**: Connection and signaling status for user feedback
- **Error Handling**: Error state propagation to UI for user notification
- **Action Triggers**: UI actions trigger connection store methods

### With Media Store
- **Stream Management**: Coordinates with media store for audio/video streams
- **Device Control**: Integration with media device management
- **Screen Sharing**: Coordinates screen sharing through data channels

## WebRTC Configuration

### ICE Servers
```javascript
{
    iceServers: [
        { urls: 'stun:stun.l.google.com:19302' }
    ]
}
```

**Current Configuration**: Uses Google's public STUN server
**Production Considerations**: Should include TURN servers for enterprise networks

### Data Channel Configuration
```javascript
const dataChannel = peerConnection.createDataChannel('secure-channel');
```

**Channel Name**: `secure-channel`
**Purpose**: Direct peer-to-peer messaging bypass server

## Security Considerations

### Cryptographic Integration
- Stores public keys for signature verification
- Integrates with ECDSA signing operations
- Supports secure offer/answer exchange

### Connection Security
- WebRTC uses DTLS for data channel encryption
- SRTP for media stream encryption
- ICE prevents certain network attacks

## Error Handling Patterns

### Async Operation Errors
```javascript
try {
    // WebRTC operation
} catch (error) {
    console.error('Operation failed:', error);
    update(store => ({ ...store, error }));
    throw error; // Re-throw for caller handling
}
```

### Connection State Errors
- Connection failures update `peerConnectionStatus`
- ICE failures logged and tracked
- Signaling errors propagated from WebSocket utility

## Usage Examples

### Basic Connection Setup
```javascript
import connectionStore from './stores/connection.js';

// Subscribe to connection state
connectionStore.subscribe(state => {
    console.log('Connection status:', state.peerConnectionStatus);
    console.log('Signaling status:', state.signalingStatus);
});

// Create and send offer
const offer = await connectionStore.createOffer();
// Send offer through WebSocket...

// Process received answer
await connectionStore.processAnswer(receivedAnswer);
```

### Data Channel Communication
```javascript
connectionStore.subscribe(state => {
    if (state.dataChannel && state.dataChannel.readyState === 'open') {
        // Send message through data channel
        state.dataChannel.send(JSON.stringify({
            type: 'chat',
            message: 'Hello peer!'
        }));
    }
});
```

### Error Handling
```javascript
connectionStore.subscribe(state => {
    if (state.error) {
        console.error('Connection error:', state.error);
        // Display error to user
        // Potentially retry connection
    }
});
```

## Performance Considerations

### State Updates
- Uses immutable updates to trigger Svelte reactivity
- Efficient array operations for ICE candidates
- Minimal re-renders through selective subscriptions

### Memory Management
- Proper cleanup in `closeConnection()`
- Event handler removal on connection close
- Resource disposal for streams and channels

## Limitations & Future Improvements

### Current Limitations
1. **Single Connection**: Only supports one peer connection at a time
2. **STUN Only**: No TURN server support for restricted networks
3. **Basic Error Recovery**: Limited automatic error recovery
4. **State Persistence**: No state persistence across page refreshes

### Recommended Improvements
1. **Multi-peer Support**: Support for multiple simultaneous connections
2. **TURN Integration**: Add TURN server support for enterprise environments
3. **Automatic Reconnection**: Implement connection recovery strategies
4. **State Persistence**: Save connection state to localStorage
5. **Bandwidth Management**: Add bandwidth monitoring and adaptation
6. **Connection Quality**: Implement connection quality metrics
7. **Advanced ICE**: Support for ICE restart and candidate pair statistics

## Testing Strategies

### Unit Testing
- Mock RTCPeerConnection for isolated testing
- Test state transitions and error handling
- Verify offer/answer exchange logic

### Integration Testing
- Test with WebSocket utility integration
- Verify data channel functionality
- Test ICE candidate exchange

### End-to-End Testing
- Real browser WebRTC connection testing
- Network condition simulation
- Multi-browser compatibility testing