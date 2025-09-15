# Peer Conference Documentation

A comprehensive peer-to-peer video conferencing application with secure signaling, WebRTC integration, and real-time messaging capabilities.

## Documentation Index

### Core Documentation
- **[WebSocket Utility](./websocket-utility.md)** - Secure WebSocket connection management and signaling
- **[Connection Store](./connection-store.md)** - WebRTC peer connection state management
- **[Crypto Utility](./crypto-utility.md)** - ECDSA cryptographic operations for message signing
- **[Chat Store](./chat-store.md)** - Real-time messaging functionality
- **[Media Store](./media-store.md)** - Audio/video stream and device management

### Reference Guides
- **[API Reference](./api-reference.md)** - Complete API documentation for all modules
- **[Integration Guide](./integration-guide.md)** - Step-by-step implementation guide

## Quick Start

1. **Prerequisites**
   - Modern browser with WebRTC support
   - WebSocket signaling server running on port 3030
   - HTTPS environment (required for media access)

2. **Basic Setup**
   ```javascript
   import { webSocketStore, connectionStore, mediaStore, chatStore } from './stores';

   // Initialize connection
   await webSocketStore.connect();

   // Get user media
   const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
   mediaStore.setLocalStream(stream);

   // Create offer
   const offer = await connectionStore.createOffer();
   await webSocketStore.sendSecureOffer();
   ```

3. **UI Integration**
   ```svelte
   <script>
     import { mediaStore, chatStore } from './stores';

     let localStream, messages;

     mediaStore.subscribe(state => {
       localStream = state.localStream;
     });

     chatStore.subscribe(state => {
       messages = state.messages;
     });
   </script>
   ```

## Architecture Overview

### System Components

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   UI Components │───▶│   Svelte Stores  │───▶│  WebRTC/WebSocket│
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ User Interactions│    │  State Management│    │ Network Protocol│
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Data Flow

1. **Signaling Flow**
   ```
   Peer A ──[Offer]──▶ Signaling Server ──[Offer]──▶ Peer B
   Peer A ◀─[Answer]── Signaling Server ◀─[Answer]── Peer B
   Peer A ◀──[ICE]───▶ Signaling Server ◀───[ICE]───▶ Peer B
   ```

2. **Media Flow**
   ```
   getUserMedia() → MediaStore → PeerConnection → Remote Peer
                                      │
                                      ▼
                               ontrack event → MediaStore → UI
   ```

3. **Chat Flow**
   ```
   User Input → ChatStore → WebSocket → Remote Peer
                              │
                              ▼
                        DataChannel → ChatStore → UI
   ```

## Security Features

### Cryptographic Signing
- **Algorithm**: ECDSA P-256 with SHA-256
- **Key Management**: Session-based key pairs
- **Signature Format**: 64-byte r||s concatenation
- **Protection**: Against offer/answer tampering

### Secure Channels
- **WebRTC**: DTLS encryption for data channels
- **Media**: SRTP encryption for audio/video
- **Signaling**: Cryptographic signatures for authenticity

## Core Features

### 🎥 Video Conferencing
- WebRTC peer-to-peer connections
- Camera and microphone control
- Screen sharing capabilities
- Real-time audio/video streaming

### 💬 Chat System
- Real-time messaging via data channels
- Message history management
- Unread message notifications
- File sharing support (planned)

### 🔐 Security
- ECDSA digital signatures
- Secure offer/answer exchange
- Cryptographic nonce protection
- Web Crypto API integration

### 📱 Device Management
- Media device enumeration
- Audio/video quality controls
- Device switching capabilities
- Permission handling

## Browser Support

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| WebRTC | ✅ 37+ | ✅ 34+ | ✅ 11+ | ✅ 79+ |
| Web Crypto API | ✅ 37+ | ✅ 34+ | ✅ 7+ | ✅ 79+ |
| getDisplayMedia | ✅ 72+ | ✅ 66+ | ✅ 13+ | ✅ 79+ |
| MediaDevices | ✅ 47+ | ✅ 36+ | ✅ 11+ | ✅ 79+ |

## Performance Considerations

### Memory Usage
- Efficient stream management
- Automatic resource cleanup
- Reactive state updates only when needed

### Network Optimization
- ICE candidate optimization
- Adaptive bitrate (planned)
- Connection quality monitoring

### CPU Usage
- Hardware acceleration where available
- Efficient video encoding/decoding
- Background processing optimization

## Development Workflow

### 1. Setup Development Environment
```bash
git clone <repository>
cd peer-conference
npm install
npm run dev
```

### 2. Start Signaling Server
```bash
# In separate terminal
cd video_conference_backend
cargo run
```

### 3. Enable HTTPS (Required)
```bash
# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

### 4. Testing
```bash
# Unit tests
npm run test

# E2E tests
npm run test:e2e

# Integration tests
npm run test:integration
```

## Production Deployment

### Requirements
- HTTPS/WSS endpoints
- TURN servers for NAT traversal
- Monitoring and logging
- Error reporting service

### Configuration
```javascript
// Production config
const config = {
    signalingServer: 'wss://your-domain.com/signaling',
    iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        {
            urls: 'turn:your-turn-server.com:3478',
            username: 'user',
            credential: 'pass'
        }
    ]
};
```

## API Summary

### WebSocket Store
- `connect()` - Establish signaling connection
- `disconnect()` - Close connection
- `sendMessage(type, payload)` - Send generic message
- `sendSecureOffer()` - Send signed WebRTC offer
- `sendSecureAnswer()` - Send signed WebRTC answer

### Connection Store
- `createOffer()` - Create WebRTC offer
- `processOffer(offer)` - Handle incoming offer
- `processAnswer(answer)` - Handle incoming answer
- `addIceCandidate(candidate)` - Add ICE candidate
- `closeConnection()` - Close peer connection

### Media Store
- `setLocalStream(stream)` - Set user's media stream
- `setRemoteStream(stream)` - Set peer's media stream
- `toggleAudio()` - Toggle microphone mute
- `toggleVideo()` - Toggle camera mute
- `toggleScreenShare()` - Toggle screen sharing

### Chat Store
- `addMessage(message)` - Add chat message
- `setCurrentMessage(text)` - Set input text
- `resetUnreadCount()` - Clear notifications

### Crypto Utility
- `generateKeyPair()` - Generate ECDSA key pair
- `sign(data, privateKey)` - Create digital signature
- `verify(data, signature, publicKey)` - Verify signature

## Troubleshooting

### Common Issues

1. **Permission Denied**
   - Ensure HTTPS is enabled
   - Check browser permission settings
   - Verify media device availability

2. **Connection Fails**
   - Verify signaling server is running
   - Check firewall/NAT configuration
   - Add TURN servers for restricted networks

3. **Audio/Video Issues**
   - Check device constraints
   - Verify codec compatibility
   - Monitor connection quality

### Debug Tools
- Browser DevTools WebRTC internals
- Connection state monitoring
- ICE candidate inspection
- Media stream analysis

## Contributing

### Development Guidelines
1. Follow existing code patterns
2. Add comprehensive tests
3. Update documentation
4. Use semantic commit messages

### Testing Requirements
- Unit tests for all utilities
- Integration tests for store interactions
- E2E tests for user workflows
- Performance benchmarks

## License

[Specify your license here]

## Support

For issues and questions:
- Check troubleshooting guide
- Review API documentation
- Open GitHub issue
- Contact development team

---

*This documentation covers version 1.0 of the Peer Conference application. For the latest updates, check the project repository.*