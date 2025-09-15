# Media Store Documentation

## Overview

The Media Store (`svelte-video-conferencing/src/stores/media.js`) manages audio and video stream handling, device controls, and screen sharing functionality in the peer-to-peer video conferencing application. It provides reactive state management for media devices and streams.

## Architecture

The media store implements comprehensive media management:

- **Stream Management**: Local and remote media stream handling
- **Device Control**: Audio/video device muting and availability tracking
- **Screen Sharing**: Screen capture and sharing state management
- **State Reactivity**: Svelte store integration for real-time UI updates

## State Schema

### Core State Structure
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

### State Properties

#### Stream Management
- **`localStream`**: User's own audio/video stream from camera and microphone
- **`remoteStream`**: Remote peer's audio/video stream received through WebRTC

#### Device State Tracking
- **`devices.audio`**: Audio device state and controls
  - `muted`: Whether local audio is muted
  - `available`: Whether audio device is accessible
- **`devices.video`**: Video device state and controls
  - `muted`: Whether local video is disabled
  - `available`: Whether video device is accessible
- **`devices.screen`**: Screen sharing state
  - `sharing`: Whether screen sharing is currently active

## Core Functions

### Stream Management

#### `setLocalStream(stream)`
```javascript
setLocalStream: (stream) => update(store => ({
    ...store,
    localStream: stream
}))
```

**Purpose**: Sets the user's local media stream
- Updates local stream reference for UI display
- Typically called after getUserMedia() success
- Triggers reactive updates for local video elements

**Parameters**:
- `stream` (MediaStream | null): Local media stream from user's devices

**Usage**:
```javascript
// After getting user media
const stream = await navigator.mediaDevices.getUserMedia({
    video: true,
    audio: true
});
mediaStore.setLocalStream(stream);
```

#### `setRemoteStream(stream)`
```javascript
setRemoteStream: (stream) => update(store => ({
    ...store,
    remoteStream: stream
}))
```

**Purpose**: Sets the remote peer's media stream
- Updates remote stream reference for UI display
- Called when receiving stream from WebRTC peer connection
- Triggers reactive updates for remote video elements

**Parameters**:
- `stream` (MediaStream | null): Remote media stream from peer

**Integration with Connection Store**:
```javascript
// In WebRTC event handler
peerConnection.ontrack = (event) => {
    const [remoteStream] = event.streams;
    mediaStore.setRemoteStream(remoteStream);
};
```

### Device Control Functions

#### `toggleAudio()`
```javascript
toggleAudio: () => update(store => ({
    ...store,
    devices: {
        ...store.devices,
        audio: {
            ...store.devices.audio,
            muted: !store.devices.audio.muted
        }
    }
}))
```

**Purpose**: Toggles local audio mute state
- Inverts current audio muted status
- Updates UI controls and indicators
- Should be paired with actual track manipulation

**Implementation with Stream Control**:
```javascript
function handleToggleAudio() {
    mediaStore.toggleAudio();

    // Actually mute/unmute the audio track
    mediaStore.subscribe(state => {
        if (state.localStream) {
            const audioTrack = state.localStream.getAudioTracks()[0];
            if (audioTrack) {
                audioTrack.enabled = !state.devices.audio.muted;
            }
        }
    });
}
```

#### `toggleVideo()`
```javascript
toggleVideo: () => update(store => ({
    ...store,
    devices: {
        ...store.devices,
        video: {
            ...store.devices.video,
            muted: !store.devices.video.muted
        }
    }
}))
```

**Purpose**: Toggles local video mute state
- Inverts current video muted status
- Updates UI controls and video display
- Should be paired with actual track manipulation

**Implementation with Stream Control**:
```javascript
function handleToggleVideo() {
    mediaStore.toggleVideo();

    // Actually enable/disable the video track
    mediaStore.subscribe(state => {
        if (state.localStream) {
            const videoTrack = state.localStream.getVideoTracks()[0];
            if (videoTrack) {
                videoTrack.enabled = !state.devices.video.muted;
            }
        }
    });
}
```

#### `toggleScreenShare()`
```javascript
toggleScreenShare: () => update(store => ({
    ...store,
    devices: {
        ...store.devices,
        screen: {
            ...store.devices.screen,
            sharing: !store.devices.screen.sharing
        }
    }
}))
```

**Purpose**: Toggles screen sharing state
- Inverts current screen sharing status
- Updates UI indicators and controls
- Should be paired with actual screen capture

**Complete Screen Sharing Implementation**:
```javascript
async function handleToggleScreenShare() {
    const currentState = get(mediaStore);

    if (!currentState.devices.screen.sharing) {
        try {
            // Start screen sharing
            const screenStream = await navigator.mediaDevices.getDisplayMedia({
                video: true,
                audio: true
            });

            // Replace video track in peer connection
            const sender = peerConnection.getSenders().find(s =>
                s.track && s.track.kind === 'video'
            );
            if (sender) {
                await sender.replaceTrack(screenStream.getVideoTracks()[0]);
            }

            // Update local stream
            mediaStore.setLocalStream(screenStream);
            mediaStore.toggleScreenShare();

            // Handle screen share end
            screenStream.getVideoTracks()[0].onended = () => {
                stopScreenShare();
            };
        } catch (error) {
            console.error('Error starting screen share:', error);
        }
    } else {
        stopScreenShare();
    }
}

async function stopScreenShare() {
    try {
        // Get camera stream back
        const cameraStream = await navigator.mediaDevices.getUserMedia({
            video: true,
            audio: true
        });

        // Replace screen track with camera track
        const sender = peerConnection.getSenders().find(s =>
            s.track && s.track.kind === 'video'
        );
        if (sender) {
            await sender.replaceTrack(cameraStream.getVideoTracks()[0]);
        }

        // Update store
        mediaStore.setLocalStream(cameraStream);
        mediaStore.toggleScreenShare();
    } catch (error) {
        console.error('Error stopping screen share:', error);
    }
}
```

### State Reset

#### `reset()`
```javascript
reset: () => set({
    localStream: null,
    remoteStream: null,
    devices: {
        audio: { muted: false, available: false },
        video: { muted: false, available: false },
        screen: { sharing: false }
    }
})
```

**Purpose**: Resets all media state to initial values
- Clears stream references
- Resets device states to defaults
- Used for session cleanup or disconnection

**Complete Cleanup Implementation**:
```javascript
function cleanupMedia() {
    const state = get(mediaStore);

    // Stop local stream tracks
    if (state.localStream) {
        state.localStream.getTracks().forEach(track => track.stop());
    }

    // Stop remote stream tracks (if needed)
    if (state.remoteStream) {
        state.remoteStream.getTracks().forEach(track => track.stop());
    }

    // Reset store state
    mediaStore.reset();
}
```

## Integration Points

### With Connection Store

#### Stream Attachment to Peer Connection
```javascript
// When local stream is obtained
mediaStore.subscribe(state => {
    if (state.localStream && peerConnection) {
        state.localStream.getTracks().forEach(track => {
            peerConnection.addTrack(track, state.localStream);
        });
    }
});
```

#### Remote Stream Handling
```javascript
// In connection store - peer connection setup
peerConnection.ontrack = (event) => {
    const [remoteStream] = event.streams;
    mediaStore.setRemoteStream(remoteStream);
};
```

### With UI Components

#### Video Element Binding
```svelte
<script>
    import { mediaStore } from '../stores/media.js';

    let localVideo, remoteVideo;
    let localStream, remoteStream, devices;

    mediaStore.subscribe(state => {
        localStream = state.localStream;
        remoteStream = state.remoteStream;
        devices = state.devices;

        // Update video elements
        if (localVideo && localStream) {
            localVideo.srcObject = localStream;
        }
        if (remoteVideo && remoteStream) {
            remoteVideo.srcObject = remoteStream;
        }
    });
</script>

<!-- Local video -->
<video
    bind:this={localVideo}
    autoplay
    muted
    playsinline
    class:hidden={devices.video.muted}
/>

<!-- Remote video -->
<video
    bind:this={remoteVideo}
    autoplay
    playsinline
/>
```

#### Control Buttons
```svelte
<script>
    import { mediaStore } from '../stores/media.js';

    let devices;

    mediaStore.subscribe(state => {
        devices = state.devices;
    });
</script>

<!-- Audio control -->
<button
    on:click={() => mediaStore.toggleAudio()}
    class:muted={devices.audio.muted}
    disabled={!devices.audio.available}
>
    {devices.audio.muted ? '🔇' : '🔊'}
</button>

<!-- Video control -->
<button
    on:click={() => mediaStore.toggleVideo()}
    class:muted={devices.video.muted}
    disabled={!devices.video.available}
>
    {devices.video.muted ? '📹' : '📷'}
</button>

<!-- Screen share control -->
<button
    on:click={() => mediaStore.toggleScreenShare()}
    class:active={devices.screen.sharing}
>
    {devices.screen.sharing ? 'Stop Sharing' : 'Share Screen'}
</button>
```

## Media Device Management

### Device Enumeration and Availability
```javascript
async function checkDeviceAvailability() {
    try {
        const devices = await navigator.mediaDevices.enumerateDevices();

        const hasAudio = devices.some(device => device.kind === 'audioinput');
        const hasVideo = devices.some(device => device.kind === 'videoinput');

        // Update availability in store (would need additional setters)
        mediaStore.update(store => ({
            ...store,
            devices: {
                ...store.devices,
                audio: { ...store.devices.audio, available: hasAudio },
                video: { ...store.devices.video, available: hasVideo }
            }
        }));
    } catch (error) {
        console.error('Error checking device availability:', error);
    }
}
```

### Permission Handling
```javascript
async function requestMediaPermissions() {
    try {
        const stream = await navigator.mediaDevices.getUserMedia({
            video: true,
            audio: true
        });

        mediaStore.setLocalStream(stream);

        // Update availability
        mediaStore.update(store => ({
            ...store,
            devices: {
                ...store.devices,
                audio: { ...store.devices.audio, available: true },
                video: { ...store.devices.video, available: true }
            }
        }));

        return stream;
    } catch (error) {
        console.error('Error requesting media permissions:', error);

        // Handle specific errors
        if (error.name === 'NotAllowedError') {
            // User denied permission
        } else if (error.name === 'NotFoundError') {
            // No devices found
        }

        throw error;
    }
}
```

## Advanced Features

### Device Selection
```javascript
// Extended store for device selection
function createEnhancedMediaStore() {
    const { subscribe, update, set } = writable({
        localStream: null,
        remoteStream: null,
        devices: {
            audio: { muted: false, available: false, deviceId: null },
            video: { muted: false, available: false, deviceId: null },
            screen: { sharing: false }
        },
        availableDevices: {
            audioInputs: [],
            videoInputs: [],
            audioOutputs: []
        }
    });

    const selectAudioDevice = async (deviceId) => {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({
                audio: { deviceId: { exact: deviceId } },
                video: true
            });

            update(store => ({
                ...store,
                localStream: stream,
                devices: {
                    ...store.devices,
                    audio: { ...store.devices.audio, deviceId }
                }
            }));
        } catch (error) {
            console.error('Error selecting audio device:', error);
        }
    };

    return {
        subscribe,
        selectAudioDevice,
        // ... other methods
    };
}
```

### Stream Quality Management
```javascript
async function adjustVideoQuality(constraints) {
    try {
        const currentState = get(mediaStore);
        if (!currentState.localStream) return;

        const videoTrack = currentState.localStream.getVideoTracks()[0];
        if (videoTrack) {
            await videoTrack.applyConstraints(constraints);
        }
    } catch (error) {
        console.error('Error adjusting video quality:', error);
    }
}

// Usage examples
adjustVideoQuality({
    width: { ideal: 1280 },
    height: { ideal: 720 },
    frameRate: { ideal: 30 }
});

adjustVideoQuality({
    width: { ideal: 640 },
    height: { ideal: 480 },
    frameRate: { ideal: 15 }
});
```

## Error Handling

### Stream Acquisition Errors
```javascript
async function safeGetUserMedia(constraints) {
    try {
        const stream = await navigator.mediaDevices.getUserMedia(constraints);
        mediaStore.setLocalStream(stream);
        return stream;
    } catch (error) {
        console.error('getUserMedia error:', error);

        switch (error.name) {
            case 'NotAllowedError':
                console.error('Camera/microphone access denied');
                break;
            case 'NotFoundError':
                console.error('No camera/microphone found');
                break;
            case 'NotReadableError':
                console.error('Camera/microphone already in use');
                break;
            case 'OverconstrainedError':
                console.error('Constraints cannot be satisfied');
                break;
            default:
                console.error('Unknown getUserMedia error');
        }

        throw error;
    }
}
```

### Device Change Handling
```javascript
// Listen for device changes
navigator.mediaDevices.addEventListener('devicechange', async () => {
    console.log('Media devices changed');
    await checkDeviceAvailability();
    // Optionally restart streams with new devices
});
```

### Track Ended Handling
```javascript
function setupTrackEventHandlers(stream) {
    stream.getTracks().forEach(track => {
        track.addEventListener('ended', () => {
            console.log(`${track.kind} track ended`);

            if (track.kind === 'video' && get(mediaStore).devices.screen.sharing) {
                // Screen sharing ended by user
                mediaStore.toggleScreenShare();
            }
        });
    });
}
```

## Performance Considerations

### Stream Resource Management
```javascript
function replaceVideoTrack(newTrack) {
    const state = get(mediaStore);

    if (state.localStream) {
        // Stop old video track
        const oldVideoTrack = state.localStream.getVideoTracks()[0];
        if (oldVideoTrack) {
            oldVideoTrack.stop();
            state.localStream.removeTrack(oldVideoTrack);
        }

        // Add new track
        state.localStream.addTrack(newTrack);

        // Update UI
        mediaStore.setLocalStream(state.localStream);
    }
}
```

### Efficient State Updates
```javascript
// Batch multiple device state updates
function updateMultipleDeviceStates(updates) {
    mediaStore.update(store => ({
        ...store,
        devices: {
            audio: { ...store.devices.audio, ...updates.audio },
            video: { ...store.devices.video, ...updates.video },
            screen: { ...store.devices.screen, ...updates.screen }
        }
    }));
}
```

## Browser Compatibility

### getUserMedia Support
```javascript
function checkMediaSupport() {
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        console.error('getUserMedia not supported');
        return false;
    }

    if (!navigator.mediaDevices.getDisplayMedia) {
        console.warn('getDisplayMedia not supported - screen sharing unavailable');
    }

    return true;
}
```

### Polyfills and Fallbacks
```javascript
// Legacy browser support
if (!navigator.mediaDevices) {
    navigator.mediaDevices = {};
}

if (!navigator.mediaDevices.getUserMedia) {
    navigator.mediaDevices.getUserMedia = function(constraints) {
        const getUserMedia = navigator.webkitGetUserMedia || navigator.mozGetUserMedia;

        if (!getUserMedia) {
            return Promise.reject(new Error('getUserMedia is not implemented'));
        }

        return new Promise((resolve, reject) => {
            getUserMedia.call(navigator, constraints, resolve, reject);
        });
    };
}
```

## Security Considerations

### Permission Management
- Always request minimal necessary permissions
- Provide clear user feedback about permission requirements
- Handle permission revocation gracefully

### Stream Privacy
- Stop streams when not needed to preserve privacy
- Clear visual indicators when camera/microphone are active
- Respect user privacy preferences

## Future Enhancements

### Advanced Features
1. **Bandwidth Adaptation**: Automatic quality adjustment based on network conditions
2. **Background Blur**: Video background effects and virtual backgrounds
3. **Noise Suppression**: Advanced audio processing for better quality
4. **Multi-Camera Support**: Support for multiple video sources
5. **Recording Capabilities**: Local recording of streams

### User Experience
1. **Device Presets**: Save and restore user device preferences
2. **Quick Settings**: Easy access to common device configurations
3. **Visual Feedback**: Better visual indicators for device states
4. **Accessibility**: Enhanced accessibility features for device controls

### Performance
1. **Stream Pooling**: Efficient reuse of media streams
2. **Lazy Loading**: On-demand device enumeration and access
3. **Memory Optimization**: Better cleanup of unused resources
4. **Battery Optimization**: Power-efficient stream management

## Testing Strategies

### Unit Testing
```javascript
// Test device state management
test('toggleAudio inverts muted state', () => {
    const store = createMediaStore();

    // Initially not muted
    store.subscribe(state => {
        expect(state.devices.audio.muted).toBe(false);
    });

    // Toggle to muted
    store.toggleAudio();
    store.subscribe(state => {
        expect(state.devices.audio.muted).toBe(true);
    });
});

// Test stream management
test('setLocalStream updates local stream', () => {
    const store = createMediaStore();
    const mockStream = new MediaStream();

    store.setLocalStream(mockStream);
    store.subscribe(state => {
        expect(state.localStream).toBe(mockStream);
    });
});
```

### Integration Testing
- WebRTC integration testing
- Cross-browser compatibility testing
- Device permission flow testing
- Stream quality and performance testing

### Manual Testing
- Real device testing across different hardware
- Network condition simulation
- User experience testing for device controls
- Accessibility testing for device interfaces