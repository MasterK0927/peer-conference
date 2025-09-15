# Integration Guide

## Overview

This comprehensive integration guide walks you through building a complete peer-to-peer video conferencing application using the provided utilities and stores. It covers everything from basic setup to advanced features and production deployment.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Architecture Overview](#architecture-overview)
3. [Step-by-Step Integration](#step-by-step-integration)
4. [UI Component Examples](#ui-component-examples)
5. [Advanced Features](#advanced-features)
6. [Error Handling](#error-handling)
7. [Testing](#testing)
8. [Production Deployment](#production-deployment)
9. [Troubleshooting](#troubleshooting)

## Quick Start

### Prerequisites

- Modern web browser with WebRTC support
- Node.js and npm/yarn for development
- WebSocket signaling server running on port 3030

### Basic Setup

1. **Install Dependencies**
```bash
npm install svelte
# or
yarn add svelte
```

2. **Import Core Modules**
```javascript
// In your main Svelte component
import { onMount } from 'svelte';
import { webSocketStore, connectionStore, mediaStore, chatStore } from './stores';
```

3. **Initialize Connection**
```javascript
onMount(async () => {
    try {
        // Request user media permissions
        const stream = await navigator.mediaDevices.getUserMedia({
            video: true,
            audio: true
        });
        mediaStore.setLocalStream(stream);

        // Connect to signaling server
        await webSocketStore.connect();

        console.log('Application initialized successfully');
    } catch (error) {
        console.error('Initialization failed:', error);
    }
});
```

## Architecture Overview

### Component Hierarchy

```
App.svelte
├── ConnectionManager.svelte    (Handles connection logic)
├── VideoInterface.svelte       (Video display and controls)
├── ChatInterface.svelte        (Chat functionality)
└── ControlPanel.svelte         (Media controls)
```

### Data Flow

```
User Action → UI Component → Store Update → WebSocket/WebRTC → Remote Peer
                ↓
    Store Subscription → Reactive UI Update
```

### Store Interactions

```
WebSocket Store ← → Connection Store ← → Media Store
                         ↓
                   Chat Store
```

## Step-by-Step Integration

### Step 1: Project Structure Setup

Create the following directory structure:
```
src/
├── components/
│   ├── ConnectionManager.svelte
│   ├── VideoInterface.svelte
│   ├── ChatInterface.svelte
│   └── ControlPanel.svelte
├── stores/
│   ├── index.js
│   ├── connection.js
│   ├── media.js
│   ├── chat.js
│   └── auth.js
├── utils/
│   ├── websocket.js
│   └── crypto.js
└── App.svelte
```

### Step 2: Main Application Component

```svelte
<!-- App.svelte -->
<script>
    import { onMount, onDestroy } from 'svelte';
    import ConnectionManager from './components/ConnectionManager.svelte';
    import VideoInterface from './components/VideoInterface.svelte';
    import ChatInterface from './components/ChatInterface.svelte';
    import ControlPanel from './components/ControlPanel.svelte';

    import { connectionStore, mediaStore } from './stores';

    let connectionStatus = 'Disconnected';
    let isConnected = false;

    // Subscribe to connection status
    const unsubscribeConnection = connectionStore.subscribe(state => {
        connectionStatus = state.signalingStatus;
        isConnected = state.signalingStatus === 'Connected';
    });

    onDestroy(() => {
        unsubscribeConnection();
        // Cleanup media streams
        mediaStore.reset();
        connectionStore.closeConnection();
    });
</script>

<main class="app">
    <header class="app-header">
        <h1>Peer Conference</h1>
        <div class="connection-status" class:connected={isConnected}>
            Status: {connectionStatus}
        </div>
    </header>

    <div class="app-content">
        <div class="video-section">
            <VideoInterface />
            <ControlPanel />
        </div>

        <div class="chat-section">
            <ChatInterface />
        </div>
    </div>

    <ConnectionManager />
</main>

<style>
    .app {
        display: flex;
        flex-direction: column;
        height: 100vh;
        font-family: Arial, sans-serif;
    }

    .app-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        background: #2c3e50;
        color: white;
    }

    .connection-status {
        padding: 0.5rem 1rem;
        border-radius: 4px;
        background: #e74c3c;
        color: white;
    }

    .connection-status.connected {
        background: #27ae60;
    }

    .app-content {
        flex: 1;
        display: flex;
        gap: 1rem;
        padding: 1rem;
    }

    .video-section {
        flex: 2;
        display: flex;
        flex-direction: column;
    }

    .chat-section {
        flex: 1;
        min-width: 300px;
    }
</style>
```

### Step 3: Connection Manager Component

```svelte
<!-- components/ConnectionManager.svelte -->
<script>
    import { onMount } from 'svelte';
    import { webSocketStore, connectionStore, mediaStore } from '../stores';

    let isInitializing = false;
    let connectionError = null;

    async function initializeApplication() {
        if (isInitializing) return;

        isInitializing = true;
        connectionError = null;

        try {
            // Step 1: Request media permissions
            console.log('Requesting media permissions...');
            const stream = await navigator.mediaDevices.getUserMedia({
                video: { width: 1280, height: 720 },
                audio: { echoCancellation: true, noiseSuppression: true }
            });

            mediaStore.setLocalStream(stream);
            console.log('Media stream acquired');

            // Step 2: Connect to signaling server
            console.log('Connecting to signaling server...');
            await webSocketStore.connect();
            console.log('Connected to signaling server');

            // Step 3: Set up connection event handlers
            setupConnectionHandlers();

        } catch (error) {
            console.error('Application initialization failed:', error);
            connectionError = error.message;
        } finally {
            isInitializing = false;
        }
    }

    function setupConnectionHandlers() {
        // Handle incoming offers
        connectionStore.subscribe(state => {
            if (state.isOfferReceived && state.offer && !state.answer) {
                handleIncomingOffer(state.offer);
            }
        });
    }

    async function handleIncomingOffer(offer) {
        try {
            console.log('Processing incoming offer...');
            const answer = await connectionStore.processOffer(offer);
            await webSocketStore.sendSecureAnswer();
            console.log('Answer sent successfully');
        } catch (error) {
            console.error('Failed to process offer:', error);
        }
    }

    async function initiateConnection() {
        try {
            console.log('Creating offer...');
            const offer = await connectionStore.createOffer();
            await webSocketStore.sendSecureOffer();
            console.log('Offer sent successfully');
        } catch (error) {
            console.error('Failed to create offer:', error);
        }
    }

    onMount(() => {
        initializeApplication();
    });
</script>

<div class="connection-manager">
    {#if isInitializing}
        <div class="status-message">
            <div class="spinner"></div>
            Initializing application...
        </div>
    {/if}

    {#if connectionError}
        <div class="error-message">
            <p>Connection Error: {connectionError}</p>
            <button on:click={initializeApplication}>Retry</button>
        </div>
    {/if}

    <div class="connection-controls">
        <button on:click={initiateConnection}>Start Call</button>
    </div>
</div>

<style>
    .connection-manager {
        position: fixed;
        top: 1rem;
        right: 1rem;
        z-index: 1000;
    }

    .status-message, .error-message {
        padding: 1rem;
        border-radius: 4px;
        margin-bottom: 1rem;
    }

    .status-message {
        background: #3498db;
        color: white;
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .error-message {
        background: #e74c3c;
        color: white;
    }

    .spinner {
        width: 20px;
        height: 20px;
        border: 2px solid transparent;
        border-top: 2px solid white;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    .connection-controls button {
        padding: 0.75rem 1.5rem;
        border: none;
        border-radius: 4px;
        background: #27ae60;
        color: white;
        cursor: pointer;
    }

    .connection-controls button:hover {
        background: #2ecc71;
    }
</style>
```

### Step 4: Video Interface Component

```svelte
<!-- components/VideoInterface.svelte -->
<script>
    import { onMount, onDestroy } from 'svelte';
    import { mediaStore } from '../stores';

    let localVideo;
    let remoteVideo;
    let localStream = null;
    let remoteStream = null;
    let devices = {
        audio: { muted: false, available: false },
        video: { muted: false, available: false },
        screen: { sharing: false }
    };

    // Subscribe to media store
    const unsubscribeMedia = mediaStore.subscribe(state => {
        localStream = state.localStream;
        remoteStream = state.remoteStream;
        devices = state.devices;

        // Update video elements
        updateVideoElements();
    });

    function updateVideoElements() {
        if (localVideo && localStream !== localVideo.srcObject) {
            localVideo.srcObject = localStream;
        }
        if (remoteVideo && remoteStream !== remoteVideo.srcObject) {
            remoteVideo.srcObject = remoteStream;
        }
    }

    onMount(() => {
        updateVideoElements();
    });

    onDestroy(() => {
        unsubscribeMedia();
    });
</script>

<div class="video-interface">
    <div class="video-container">
        <div class="video-wrapper remote-video">
            <video
                bind:this={remoteVideo}
                autoplay
                playsinline
                class="video-element"
                class:hidden={!remoteStream}
            />
            {#if !remoteStream}
                <div class="video-placeholder">
                    <p>Waiting for remote video...</p>
                </div>
            {/if}
        </div>

        <div class="video-wrapper local-video">
            <video
                bind:this={localVideo}
                autoplay
                muted
                playsinline
                class="video-element"
                class:hidden={devices.video.muted}
            />
            {#if devices.video.muted}
                <div class="video-placeholder">
                    <p>Video disabled</p>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .video-interface {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    .video-container {
        position: relative;
        flex: 1;
        background: #000;
        border-radius: 8px;
        overflow: hidden;
    }

    .video-wrapper {
        position: absolute;
        border-radius: 8px;
        overflow: hidden;
    }

    .remote-video {
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
    }

    .local-video {
        bottom: 20px;
        right: 20px;
        width: 200px;
        height: 150px;
        border: 2px solid white;
        z-index: 10;
    }

    .video-element {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .video-element.hidden {
        display: none;
    }

    .video-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #34495e;
        color: white;
    }
</style>
```

### Step 5: Chat Interface Component

```svelte
<!-- components/ChatInterface.svelte -->
<script>
    import { onMount, onDestroy } from 'svelte';
    import { chatStore, webSocketStore } from '../stores';

    let messages = [];
    let currentMessage = '';
    let unreadCount = 0;
    let chatContainer;
    let messageInput;

    // Subscribe to chat store
    const unsubscribeChat = chatStore.subscribe(state => {
        messages = state.messages;
        currentMessage = state.currentMessage;
        unreadCount = state.unreadCount;

        // Auto-scroll to bottom
        if (chatContainer) {
            setTimeout(() => {
                chatContainer.scrollTop = chatContainer.scrollHeight;
            }, 0);
        }
    });

    function handleSubmit(event) {
        event.preventDefault();
        sendMessage();
    }

    function sendMessage() {
        const text = currentMessage.trim();
        if (!text) return;

        const message = {
            id: Date.now().toString(),
            text: text,
            timestamp: Date.now(),
            sender: 'me',
            type: 'text'
        };

        // Send through WebSocket
        webSocketStore.sendMessage('chat', message);

        // Add to local store for immediate UI update
        chatStore.addMessage({ ...message, sender: 'me' });

        // Clear input
        chatStore.clearCurrentMessage();
    }

    function handleInputChange(event) {
        chatStore.setCurrentMessage(event.target.value);
    }

    function handleKeyPress(event) {
        if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            sendMessage();
        }
    }

    function formatTime(timestamp) {
        return new Date(timestamp).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit'
        });
    }

    onMount(() => {
        // Reset unread count when chat is visible
        chatStore.resetUnreadCount();
    });

    onDestroy(() => {
        unsubscribeChat();
    });
</script>

<div class="chat-interface">
    <div class="chat-header">
        <h3>Chat</h3>
        {#if unreadCount > 0}
            <span class="unread-badge">{unreadCount}</span>
        {/if}
    </div>

    <div class="messages-container" bind:this={chatContainer}>
        {#each messages as message (message.id)}
            <div class="message" class:own-message={message.sender === 'me'}>
                <div class="message-content">
                    <div class="message-text">{message.text}</div>
                    <div class="message-time">{formatTime(message.timestamp)}</div>
                </div>
            </div>
        {/each}
        {#if messages.length === 0}
            <div class="empty-state">
                <p>No messages yet. Start the conversation!</p>
            </div>
        {/if}
    </div>

    <form class="message-form" on:submit={handleSubmit}>
        <div class="input-container">
            <textarea
                bind:this={messageInput}
                value={currentMessage}
                on:input={handleInputChange}
                on:keypress={handleKeyPress}
                placeholder="Type your message..."
                rows="3"
            ></textarea>
            <button
                type="submit"
                disabled={!currentMessage.trim()}
                class="send-button"
            >
                Send
            </button>
        </div>
    </form>
</div>

<style>
    .chat-interface {
        display: flex;
        flex-direction: column;
        height: 100%;
        border: 1px solid #ddd;
        border-radius: 8px;
        background: white;
    }

    .chat-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        border-bottom: 1px solid #ddd;
        background: #f8f9fa;
    }

    .chat-header h3 {
        margin: 0;
        color: #2c3e50;
    }

    .unread-badge {
        background: #e74c3c;
        color: white;
        border-radius: 50%;
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.8rem;
        font-weight: bold;
    }

    .messages-container {
        flex: 1;
        overflow-y: auto;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .message {
        display: flex;
        margin-bottom: 0.5rem;
    }

    .message.own-message {
        justify-content: flex-end;
    }

    .message-content {
        max-width: 70%;
        padding: 0.75rem;
        border-radius: 8px;
        background: #e9ecef;
    }

    .own-message .message-content {
        background: #007bff;
        color: white;
    }

    .message-text {
        word-wrap: break-word;
        margin-bottom: 0.25rem;
    }

    .message-time {
        font-size: 0.8rem;
        opacity: 0.7;
    }

    .empty-state {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #6c757d;
        font-style: italic;
    }

    .message-form {
        border-top: 1px solid #ddd;
        padding: 1rem;
    }

    .input-container {
        display: flex;
        gap: 0.5rem;
    }

    .input-container textarea {
        flex: 1;
        resize: none;
        border: 1px solid #ddd;
        border-radius: 4px;
        padding: 0.75rem;
        font-family: inherit;
    }

    .input-container textarea:focus {
        outline: none;
        border-color: #007bff;
    }

    .send-button {
        padding: 0.75rem 1.5rem;
        border: none;
        border-radius: 4px;
        background: #007bff;
        color: white;
        cursor: pointer;
        align-self: flex-end;
    }

    .send-button:hover:not(:disabled) {
        background: #0056b3;
    }

    .send-button:disabled {
        background: #6c757d;
        cursor: not-allowed;
    }
</style>
```

### Step 6: Control Panel Component

```svelte
<!-- components/ControlPanel.svelte -->
<script>
    import { onDestroy } from 'svelte';
    import { mediaStore, connectionStore } from '../stores';

    let devices = {
        audio: { muted: false, available: false },
        video: { muted: false, available: false },
        screen: { sharing: false }
    };

    let localStream = null;
    let peerConnection = null;

    // Subscribe to stores
    const unsubscribeMedia = mediaStore.subscribe(state => {
        devices = state.devices;
        localStream = state.localStream;
    });

    const unsubscribeConnection = connectionStore.subscribe(state => {
        peerConnection = state.peerConnection;
    });

    async function toggleAudio() {
        mediaStore.toggleAudio();

        // Actually control the audio track
        if (localStream) {
            const audioTrack = localStream.getAudioTracks()[0];
            if (audioTrack) {
                audioTrack.enabled = !devices.audio.muted;
            }
        }
    }

    async function toggleVideo() {
        mediaStore.toggleVideo();

        // Actually control the video track
        if (localStream) {
            const videoTrack = localStream.getVideoTracks()[0];
            if (videoTrack) {
                videoTrack.enabled = !devices.video.muted;
            }
        }
    }

    async function toggleScreenShare() {
        if (!devices.screen.sharing) {
            await startScreenShare();
        } else {
            await stopScreenShare();
        }
    }

    async function startScreenShare() {
        try {
            const screenStream = await navigator.mediaDevices.getDisplayMedia({
                video: true,
                audio: true
            });

            // Replace video track in peer connection
            if (peerConnection) {
                const sender = peerConnection.getSenders().find(s =>
                    s.track && s.track.kind === 'video'
                );
                if (sender && screenStream.getVideoTracks().length > 0) {
                    await sender.replaceTrack(screenStream.getVideoTracks()[0]);
                }
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
    }

    async function stopScreenShare() {
        try {
            // Get camera stream back
            const cameraStream = await navigator.mediaDevices.getUserMedia({
                video: true,
                audio: true
            });

            // Replace screen track with camera track
            if (peerConnection) {
                const sender = peerConnection.getSenders().find(s =>
                    s.track && s.track.kind === 'video'
                );
                if (sender && cameraStream.getVideoTracks().length > 0) {
                    await sender.replaceTrack(cameraStream.getVideoTracks()[0]);
                }
            }

            // Update store
            mediaStore.setLocalStream(cameraStream);

            // Only toggle if currently sharing
            if (devices.screen.sharing) {
                mediaStore.toggleScreenShare();
            }
        } catch (error) {
            console.error('Error stopping screen share:', error);
        }
    }

    function endCall() {
        // Stop all media tracks
        if (localStream) {
            localStream.getTracks().forEach(track => track.stop());
        }

        // Close connection
        connectionStore.closeConnection();

        // Reset stores
        mediaStore.reset();
        connectionStore.reset();
    }

    onDestroy(() => {
        unsubscribeMedia();
        unsubscribeConnection();
    });
</script>

<div class="control-panel">
    <div class="controls">
        <button
            class="control-button audio"
            class:muted={devices.audio.muted}
            class:unavailable={!devices.audio.available}
            on:click={toggleAudio}
            title={devices.audio.muted ? 'Unmute microphone' : 'Mute microphone'}
        >
            {#if devices.audio.muted}
                🔇
            {:else}
                🎤
            {/if}
        </button>

        <button
            class="control-button video"
            class:muted={devices.video.muted}
            class:unavailable={!devices.video.available}
            on:click={toggleVideo}
            title={devices.video.muted ? 'Turn on camera' : 'Turn off camera'}
        >
            {#if devices.video.muted}
                📹
            {:else}
                📷
            {/if}
        </button>

        <button
            class="control-button screen-share"
            class:active={devices.screen.sharing}
            on:click={toggleScreenShare}
            title={devices.screen.sharing ? 'Stop screen sharing' : 'Share screen'}
        >
            {#if devices.screen.sharing}
                🛑
            {:else}
                🖥️
            {/if}
        </button>

        <button
            class="control-button end-call"
            on:click={endCall}
            title="End call"
        >
            📞
        </button>
    </div>
</div>

<style>
    .control-panel {
        padding: 1rem;
        background: #f8f9fa;
        border-radius: 8px;
        margin-top: 1rem;
    }

    .controls {
        display: flex;
        justify-content: center;
        gap: 1rem;
    }

    .control-button {
        width: 50px;
        height: 50px;
        border: none;
        border-radius: 50%;
        font-size: 1.5rem;
        cursor: pointer;
        transition: all 0.2s ease;
        background: white;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .control-button:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 8px rgba(0,0,0,0.15);
    }

    .control-button.muted,
    .control-button.active {
        background: #e74c3c;
        color: white;
    }

    .control-button.unavailable {
        background: #95a5a6;
        cursor: not-allowed;
    }

    .control-button.unavailable:hover {
        transform: none;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .control-button.end-call {
        background: #e74c3c;
        color: white;
    }

    .control-button.end-call:hover {
        background: #c0392b;
    }
</style>
```

## Advanced Features

### Screen Sharing Enhancement

```javascript
// Enhanced screen sharing with audio
async function startScreenShareWithAudio() {
    try {
        const screenStream = await navigator.mediaDevices.getDisplayMedia({
            video: true,
            audio: {
                echoCancellation: true,
                noiseSuppression: true,
                autoGainControl: false
            }
        });

        // Combine screen video with microphone audio
        const audioStream = await navigator.mediaDevices.getUserMedia({
            audio: true,
            video: false
        });

        const combinedStream = new MediaStream([
            ...screenStream.getVideoTracks(),
            ...audioStream.getAudioTracks()
        ]);

        // Replace tracks in peer connection
        if (peerConnection) {
            const senders = peerConnection.getSenders();

            // Replace video track
            const videoSender = senders.find(s => s.track?.kind === 'video');
            if (videoSender && combinedStream.getVideoTracks().length > 0) {
                await videoSender.replaceTrack(combinedStream.getVideoTracks()[0]);
            }

            // Replace audio track
            const audioSender = senders.find(s => s.track?.kind === 'audio');
            if (audioSender && combinedStream.getAudioTracks().length > 0) {
                await audioSender.replaceTrack(combinedStream.getAudioTracks()[0]);
            }
        }

        mediaStore.setLocalStream(combinedStream);
        mediaStore.toggleScreenShare();

        // Handle screen share end
        screenStream.getVideoTracks()[0].onended = stopScreenShare;

    } catch (error) {
        console.error('Enhanced screen share failed:', error);
    }
}
```

### File Sharing via Data Channel

```javascript
// File sharing implementation
class FileSharing {
    constructor(dataChannel) {
        this.dataChannel = dataChannel;
        this.pendingFiles = new Map();
        this.receivedChunks = new Map();
    }

    async shareFile(file) {
        const fileId = crypto.randomUUID();
        const chunkSize = 16384; // 16KB chunks
        const totalChunks = Math.ceil(file.size / chunkSize);

        // Send file metadata
        this.sendMessage({
            type: 'file-offer',
            fileId,
            name: file.name,
            size: file.size,
            type: file.type,
            totalChunks
        });

        // Send file in chunks
        for (let i = 0; i < totalChunks; i++) {
            const start = i * chunkSize;
            const end = Math.min(start + chunkSize, file.size);
            const chunk = file.slice(start, end);
            const arrayBuffer = await chunk.arrayBuffer();

            this.sendMessage({
                type: 'file-chunk',
                fileId,
                chunkIndex: i,
                data: Array.from(new Uint8Array(arrayBuffer))
            });
        }
    }

    handleMessage(message) {
        switch (message.type) {
            case 'file-offer':
                this.handleFileOffer(message);
                break;
            case 'file-chunk':
                this.handleFileChunk(message);
                break;
        }
    }

    handleFileOffer(offer) {
        this.receivedChunks.set(offer.fileId, {
            name: offer.name,
            size: offer.size,
            type: offer.type,
            totalChunks: offer.totalChunks,
            chunks: new Array(offer.totalChunks)
        });

        // Notify UI of incoming file
        chatStore.addMessage({
            id: Date.now().toString(),
            type: 'file-transfer',
            text: `Receiving file: ${offer.name}`,
            timestamp: Date.now(),
            sender: 'remote'
        });
    }

    handleFileChunk(chunk) {
        const fileData = this.receivedChunks.get(chunk.fileId);
        if (!fileData) return;

        fileData.chunks[chunk.chunkIndex] = new Uint8Array(chunk.data);

        // Check if all chunks received
        const receivedCount = fileData.chunks.filter(c => c).length;
        if (receivedCount === fileData.totalChunks) {
            this.assembleFile(chunk.fileId, fileData);
        }
    }

    assembleFile(fileId, fileData) {
        // Combine all chunks
        const totalSize = fileData.chunks.reduce((sum, chunk) => sum + chunk.length, 0);
        const fileBytes = new Uint8Array(totalSize);

        let offset = 0;
        fileData.chunks.forEach(chunk => {
            fileBytes.set(chunk, offset);
            offset += chunk.length;
        });

        // Create blob and download
        const blob = new Blob([fileBytes], { type: fileData.type });
        const url = URL.createObjectURL(blob);

        const a = document.createElement('a');
        a.href = url;
        a.download = fileData.name;
        a.click();

        URL.revokeObjectURL(url);
        this.receivedChunks.delete(fileId);

        // Notify completion
        chatStore.addMessage({
            id: Date.now().toString(),
            type: 'system',
            text: `File received: ${fileData.name}`,
            timestamp: Date.now(),
            sender: 'system'
        });
    }

    sendMessage(message) {
        if (this.dataChannel && this.dataChannel.readyState === 'open') {
            this.dataChannel.send(JSON.stringify(message));
        }
    }
}
```

### Connection Quality Monitoring

```javascript
class ConnectionQualityMonitor {
    constructor(peerConnection) {
        this.peerConnection = peerConnection;
        this.stats = {
            bandwidth: 0,
            packetLoss: 0,
            latency: 0,
            jitter: 0
        };
        this.isMonitoring = false;
    }

    startMonitoring() {
        if (this.isMonitoring) return;
        this.isMonitoring = true;
        this.monitorLoop();
    }

    stopMonitoring() {
        this.isMonitoring = false;
    }

    async monitorLoop() {
        while (this.isMonitoring) {
            try {
                await this.gatherStats();
                await new Promise(resolve => setTimeout(resolve, 1000));
            } catch (error) {
                console.error('Stats gathering error:', error);
            }
        }
    }

    async gatherStats() {
        const stats = await this.peerConnection.getStats();
        const videoSender = this.peerConnection.getSenders().find(s => s.track?.kind === 'video');

        if (videoSender) {
            const senderStats = await videoSender.getStats();

            senderStats.forEach(stat => {
                if (stat.type === 'outbound-rtp' && stat.kind === 'video') {
                    this.stats.bandwidth = stat.bytesSent;
                    this.stats.packetLoss = stat.packetsLost / (stat.packetsSent || 1);
                }

                if (stat.type === 'remote-inbound-rtp') {
                    this.stats.latency = stat.roundTripTime;
                    this.stats.jitter = stat.jitter;
                }
            });
        }

        // Emit stats update
        window.dispatchEvent(new CustomEvent('connection-stats', {
            detail: this.stats
        }));
    }
}

// Usage in connection store
let qualityMonitor;

connectionStore.subscribe(state => {
    if (state.peerConnection && !qualityMonitor) {
        qualityMonitor = new ConnectionQualityMonitor(state.peerConnection);
        qualityMonitor.startMonitoring();
    } else if (!state.peerConnection && qualityMonitor) {
        qualityMonitor.stopMonitoring();
        qualityMonitor = null;
    }
});
```

## Error Handling

### Comprehensive Error Handler

```javascript
class ErrorHandler {
    constructor() {
        this.errors = new Map();
        this.setupGlobalHandlers();
    }

    setupGlobalHandlers() {
        // WebRTC errors
        window.addEventListener('unhandledrejection', this.handleRejection.bind(this));

        // Media errors
        window.addEventListener('error', this.handleError.bind(this));
    }

    handleRejection(event) {
        console.error('Unhandled promise rejection:', event.reason);
        this.logError('promise-rejection', event.reason);
    }

    handleError(event) {
        console.error('Global error:', event.error);
        this.logError('global-error', event.error);
    }

    handleMediaError(error) {
        let userMessage = 'Media access failed';

        switch (error.name) {
            case 'NotAllowedError':
                userMessage = 'Please grant camera and microphone permissions';
                break;
            case 'NotFoundError':
                userMessage = 'No camera or microphone found';
                break;
            case 'NotReadableError':
                userMessage = 'Camera or microphone is already in use';
                break;
            case 'OverconstrainedError':
                userMessage = 'Camera or microphone constraints cannot be satisfied';
                break;
        }

        this.showUserError(userMessage);
        this.logError('media-error', error);
    }

    handleWebRTCError(error) {
        let userMessage = 'Connection failed';

        if (error.message.includes('ice')) {
            userMessage = 'Network connection failed. Please check your internet connection.';
        } else if (error.message.includes('offer') || error.message.includes('answer')) {
            userMessage = 'Failed to establish connection with peer';
        }

        this.showUserError(userMessage);
        this.logError('webrtc-error', error);
    }

    handleWebSocketError(error) {
        const userMessage = 'Failed to connect to signaling server';
        this.showUserError(userMessage);
        this.logError('websocket-error', error);
    }

    logError(type, error) {
        const errorEntry = {
            type,
            error: error.message || error,
            stack: error.stack,
            timestamp: Date.now(),
            userAgent: navigator.userAgent,
            url: window.location.href
        };

        this.errors.set(Date.now(), errorEntry);

        // Send to error reporting service
        this.reportError(errorEntry);
    }

    showUserError(message) {
        // Update UI with error message
        const errorEvent = new CustomEvent('user-error', {
            detail: { message }
        });
        window.dispatchEvent(errorEvent);
    }

    reportError(errorEntry) {
        // Send to your error reporting service
        // fetch('/api/errors', {
        //     method: 'POST',
        //     headers: { 'Content-Type': 'application/json' },
        //     body: JSON.stringify(errorEntry)
        // });
    }

    getErrorHistory() {
        return Array.from(this.errors.values());
    }
}

// Global error handler instance
const errorHandler = new ErrorHandler();

// Export for use in stores
export { errorHandler };
```

## Testing

### Unit Test Examples

```javascript
// tests/stores/chat.test.js
import { get } from 'svelte/store';
import { describe, it, expect, beforeEach } from 'vitest';
import createChatStore from '../src/stores/chat.js';

describe('Chat Store', () => {
    let chatStore;

    beforeEach(() => {
        chatStore = createChatStore();
    });

    it('should add message correctly', () => {
        const message = {
            id: '1',
            text: 'Hello',
            timestamp: Date.now(),
            sender: 'user1'
        };

        chatStore.addMessage(message);

        const state = get(chatStore);
        expect(state.messages).toHaveLength(1);
        expect(state.messages[0]).toEqual(message);
        expect(state.unreadCount).toBe(1);
    });

    it('should handle current message updates', () => {
        const testMessage = 'Hello world';

        chatStore.setCurrentMessage(testMessage);

        const state = get(chatStore);
        expect(state.currentMessage).toBe(testMessage);
    });

    it('should reset unread count', () => {
        // Add some messages first
        chatStore.addMessage({ id: '1', text: 'Message 1' });
        chatStore.addMessage({ id: '2', text: 'Message 2' });

        expect(get(chatStore).unreadCount).toBe(2);

        chatStore.resetUnreadCount();

        expect(get(chatStore).unreadCount).toBe(0);
    });
});
```

### Integration Test Examples

```javascript
// tests/integration/websocket.test.js
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createWebSocketConnection } from '../src/utils/websocket.js';

describe('WebSocket Integration', () => {
    let wsConnection;
    let mockServer;

    beforeEach(async () => {
        // Setup mock WebSocket server
        mockServer = new MockWebSocketServer('ws://localhost:8080');
        wsConnection = createWebSocketConnection('ws://localhost:8080');
    });

    afterEach(() => {
        wsConnection.disconnect();
        mockServer.close();
    });

    it('should connect and send messages', async () => {
        await wsConnection.connect();

        const testMessage = { type: 'test', data: 'hello' };
        wsConnection.sendMessage('test-type', testMessage);

        const receivedMessage = await mockServer.waitForMessage();
        expect(JSON.parse(receivedMessage).signal_type).toBe('test-type');
    });
});
```

### E2E Test Examples

```javascript
// tests/e2e/video-conference.spec.js
import { test, expect } from '@playwright/test';

test.describe('Video Conference Application', () => {
    test('should establish peer connection', async ({ browser }) => {
        // Create two browser contexts (peers)
        const context1 = await browser.newContext();
        const context2 = await browser.newContext();

        const page1 = await context1.newPage();
        const page2 = await context2.newPage();

        // Grant permissions
        await context1.grantPermissions(['microphone', 'camera']);
        await context2.grantPermissions(['microphone', 'camera']);

        // Navigate to application
        await page1.goto('http://localhost:5173');
        await page2.goto('http://localhost:5173');

        // Start call from first peer
        await page1.click('[data-testid="start-call"]');

        // Accept call on second peer
        await page2.waitForSelector('[data-testid="incoming-call"]');
        await page2.click('[data-testid="accept-call"]');

        // Verify connection established
        await expect(page1.locator('[data-testid="connection-status"]')).toHaveText('Connected');
        await expect(page2.locator('[data-testid="connection-status"]')).toHaveText('Connected');

        // Verify video elements are present
        await expect(page1.locator('[data-testid="remote-video"]')).toBeVisible();
        await expect(page2.locator('[data-testid="remote-video"]')).toBeVisible();

        await context1.close();
        await context2.close();
    });

    test('should send and receive chat messages', async ({ page }) => {
        await page.goto('http://localhost:5173');

        // Send a test message
        await page.fill('[data-testid="message-input"]', 'Hello, world!');
        await page.click('[data-testid="send-message"]');

        // Verify message appears in chat
        await expect(page.locator('[data-testid="chat-messages"]')).toContainText('Hello, world!');
    });
});
```

## Production Deployment

### Environment Configuration

```javascript
// src/config/environment.js
const config = {
    development: {
        signalingServer: 'ws://localhost:3030',
        iceServers: [
            { urls: 'stun:stun.l.google.com:19302' }
        ],
        enableLogging: true
    },
    production: {
        signalingServer: 'wss://your-signaling-server.com',
        iceServers: [
            { urls: 'stun:stun.l.google.com:19302' },
            {
                urls: 'turn:your-turn-server.com:3478',
                username: 'turnuser',
                credential: 'turnpass'
            }
        ],
        enableLogging: false
    }
};

const env = import.meta.env.MODE || 'development';
export default config[env];
```

### Build Configuration

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
    plugins: [svelte()],
    build: {
        outDir: 'dist',
        sourcemap: true,
        rollupOptions: {
            output: {
                manualChunks: {
                    vendor: ['svelte'],
                    webrtc: ['./src/stores/connection.js', './src/stores/media.js'],
                    crypto: ['./src/utils/crypto.js']
                }
            }
        }
    },
    server: {
        host: '0.0.0.0',
        port: 5173,
        https: {
            key: './certs/key.pem',
            cert: './certs/cert.pem'
        }
    }
});
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM node:18-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80 443
CMD ["nginx", "-g", "daemon off;"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  web:
    build: .
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./certs:/etc/nginx/certs

  signaling:
    image: your-signaling-server:latest
    ports:
      - "3030:3030"
    environment:
      - NODE_ENV=production
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Media Access Denied
**Problem:** User denies camera/microphone permissions
```javascript
// Solution: Graceful permission handling
async function requestPermissions() {
    try {
        const stream = await navigator.mediaDevices.getUserMedia({
            video: true,
            audio: true
        });
        return stream;
    } catch (error) {
        if (error.name === 'NotAllowedError') {
            // Show instructions for enabling permissions
            showPermissionInstructions();
        }
        throw error;
    }
}
```

#### 2. WebRTC Connection Fails
**Problem:** Peer connection fails to establish
```javascript
// Solution: Add TURN servers and connection retry
const config = {
    iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        {
            urls: 'turn:openrelay.metered.ca:80',
            username: 'openrelayproject',
            credential: 'openrelayproject'
        }
    ],
    iceCandidatePoolSize: 10
};
```

#### 3. WebSocket Connection Drops
**Problem:** Signaling connection unstable
```javascript
// Solution: Auto-reconnection logic
class ReliableWebSocket {
    constructor(url) {
        this.url = url;
        this.reconnectInterval = 1000;
        this.maxReconnectAttempts = 5;
        this.reconnectAttempts = 0;
    }

    connect() {
        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(this.url);

            this.ws.onopen = () => {
                this.reconnectAttempts = 0;
                resolve(this.ws);
            };

            this.ws.onclose = () => {
                this.handleReconnection();
            };

            this.ws.onerror = reject;
        });
    }

    handleReconnection() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            setTimeout(() => this.connect(),
                this.reconnectInterval * this.reconnectAttempts
            );
        }
    }
}
```

#### 4. Audio/Video Sync Issues
**Problem:** Audio and video out of sync
```javascript
// Solution: Proper stream handling
function handleStreamSync(stream) {
    const audioTracks = stream.getAudioTracks();
    const videoTracks = stream.getVideoTracks();

    // Ensure tracks have same timestamp base
    if (audioTracks.length > 0 && videoTracks.length > 0) {
        const syncStream = new MediaStream();

        // Add tracks in specific order
        videoTracks.forEach(track => syncStream.addTrack(track));
        audioTracks.forEach(track => syncStream.addTrack(track));

        return syncStream;
    }

    return stream;
}
```

### Debug Utilities

```javascript
// Debug helper for WebRTC connection state
function debugConnection(peerConnection) {
    const interval = setInterval(async () => {
        if (!peerConnection) {
            clearInterval(interval);
            return;
        }

        const stats = await peerConnection.getStats();
        const connection = {
            state: peerConnection.connectionState,
            iceState: peerConnection.iceConnectionState,
            signalingState: peerConnection.signalingState
        };

        console.log('Connection Debug:', connection);

        // Log ICE candidates
        stats.forEach(report => {
            if (report.type === 'candidate-pair' && report.state === 'succeeded') {
                console.log('Active ICE pair:', report);
            }
        });

        if (connection.state === 'closed' || connection.state === 'failed') {
            clearInterval(interval);
        }
    }, 2000);
}
```

This integration guide provides a complete foundation for building a robust peer-to-peer video conferencing application. The modular architecture allows for easy extension and customization while maintaining security and performance best practices.