<script>
    import { onMount, onDestroy } from 'svelte';

    // State management
    let localVideoElement;
    let remoteVideoElement;
    let peerConnection = null;
    let localStream = null;
    let signalingSocket = null;
    let connectionStatus = 'Not Connected';
    let peerConnectionStatus = 'Disconnected';
    let chatMessage = '';
    let chatMessages = [];
    let offerMessage = '';
    let isOfferReceived = false;
    let isScreenSharing = false;
    let eventLogs = [];
    
    // Device state tracking
    let deviceState = {
        isVideoMuted: false,
        isAudioMuted: false,
        isRecording: false
    };

    // Configuration
    const SIGNALING_SERVER = "wss://localhost:3030/ws";
    const mediaConstraints = {
        video: { width: 1280, height: 720 },
        audio: true
    };
    const iceServers = [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun1.l.google.com:19302' },
        { 
            urls: 'turn:your-turn-server.com', 
            credential: 'your-turn-credential', 
            username: 'your-turn-username' 
        }
    ];
    const peerConfig = { 
        iceServers, 
        sdpSemantics: 'unified-plan', 
        bundlePolicy: 'max-bundle' 
    };

    function log(message, type = 'info') {
        const timestamp = new Date().toLocaleTimeString();
        eventLogs = [...eventLogs.slice(-10), { timestamp, message, type }];
        console.log(`[${type.toUpperCase()}] ${message}`);
    }

    async function initializeWebRTC() {
        try {
            localStream = await navigator.mediaDevices.getUserMedia(mediaConstraints);
            localVideoElement.srcObject = localStream;
            log('Local media stream acquired', 'success');

            peerConnection = new RTCPeerConnection(peerConfig);
            localStream.getTracks().forEach(track => peerConnection.addTrack(track, localStream));
            
            peerConnection.onicecandidate = handleICECandidate;
            peerConnection.ontrack = handleRemoteTrack;
            peerConnection.onconnectionstatechange = handleConnectionStateChange;

            connectSignalingServer();
        } catch (error) {
            log(`WebRTC initialization error: ${error.message}`, 'error');
        }
    }

    function connectSignalingServer() {
        signalingSocket = new WebSocket(SIGNALING_SERVER);

        signalingSocket.onopen = () => {
            log('Signaling server connected', 'success');
            connectionStatus = 'Connected to Signaling Server';
        };

        signalingSocket.onmessage = async (event) => {
            const message = JSON.parse(event.data);
            await handleSignalingMessage(message);
        };

        signalingSocket.onerror = error => log(`Signaling error: ${error}`, 'error');
        signalingSocket.onclose = () => {
            log('Signaling server disconnected', 'warning');
            connectionStatus = 'Disconnected';
        };
    }

    function sendSignalingMessage(type, payload) {
        if (signalingSocket?.readyState === WebSocket.OPEN) {
            signalingSocket.send(JSON.stringify({ 
                signal_type: type, 
                payload: JSON.stringify(payload)
            }));
        }
    }

    async function handleSignalingMessage(message) {
        try {
            const parsedPayload = JSON.parse(message.payload);
            switch (message.signal_type) {
                case 'offer': 
                    isOfferReceived = true;
                    await handleOffer(parsedPayload); 
                    break;
                case 'answer': 
                    await handleAnswer(parsedPayload); 
                    break;
                case 'ice-candidate': 
                    await peerConnection.addIceCandidate(new RTCIceCandidate(parsedPayload)); 
                    break;
                case 'chat':
                    handleIncomingChat(parsedPayload);
                    break;
                case 'offer-rejected':
                    handleOfferRejected();
                    break;
            }
        } catch (error) {
            log(`Signaling message handling error: ${error.message}`, 'error');
        }
    }

    async function getPublicIP() {
        try {
            const response = await fetch('https://api.ipify.org?format=json');
            const data = await response.json();
            return data.ip;
        } catch (error) {
            log('Failed to get public IP', 'error');
            return null;
        }
    }

    async function generateKeyFromIP(ip) {
        const encoder = new TextEncoder();
        const data = encoder.encode(ip);
        const hash = await crypto.subtle.digest('SHA-256', data);
        return crypto.subtle.importKey(
            'raw',
            hash,
            { name: 'AES-GCM', length: 256 },
            false,
            ['encrypt', 'decrypt']
        );
    }

    async function encryptWithIP(message) {
        const senderIP = await getPublicIP();
        if (!senderIP) return null;

        const key = await generateKeyFromIP(senderIP);
        const iv = crypto.getRandomValues(new Uint8Array(12));
        const encoder = new TextEncoder();
        const encodedMessage = encoder.encode(message);

        const encryptedData = await crypto.subtle.encrypt(
            {
                name: 'AES-GCM',
                iv: iv
            },
            key,
            encodedMessage
        );

        return {
            encrypted: Array.from(new Uint8Array(encryptedData)),
            iv: Array.from(iv),
            senderIP
        };
    }

    async function decryptWithIP(encryptedData, iv, senderIP) {
        try {
            const key = await generateKeyFromIP(senderIP);
            const decrypted = await crypto.subtle.decrypt(
                {
                    name: 'AES-GCM',
                    iv: new Uint8Array(iv)
                },
                key,
                new Uint8Array(encryptedData)
            );

            return new TextDecoder().decode(decrypted);
        } catch (error) {
            log('Decryption failed: Invalid IP signature', 'error');
            return null;
        }
    }

    async function createOffer() {
        try {
            const offer = await peerConnection.createOffer();
            await peerConnection.setLocalDescription(offer);

            const encryptedData = await encryptWithIP(
                offerMessage || 'Secure connection request'
            );

            if (encryptedData) {
                sendSignalingMessage('offer', {
                    offer,
                    encryptedData
                });
                log('Offer sent with IP-signed encrypted message', 'success');
            } else {
                throw new Error('Failed to encrypt offer message');
            }
        } catch (error) {
            log(`Offer creation error: ${error.message}`, 'error');
        }
    }

    async function handleOffer(data) {
        try {
            const { offer, encryptedData } = data;
            await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));

            if (encryptedData) {
                const decryptedMessage = await decryptWithIP(
                    encryptedData.encrypted,
                    encryptedData.iv,
                    encryptedData.senderIP
                );

                if (decryptedMessage) {
                    log(`Verified offer from IP ${encryptedData.senderIP}`, 'success');
                    log(`Decrypted message: ${decryptedMessage}`, 'info');
                } else {
                    throw new Error('Failed to verify offer signature');
                }
            }

            const answer = await peerConnection.createAnswer();
            await peerConnection.setLocalDescription(answer);
            sendSignalingMessage('answer', answer);
        } catch (error) {
            log(`Offer handling error: ${error.message}`, 'error');
            rejectOffer();
        }
    }

    function sendChatMessage() {
        if (!chatMessage.trim()) return;
        
        const messageObj = {
            text: chatMessage,
            sender: 'local',
            timestamp: new Date().toLocaleTimeString()
        };
        
        chatMessages = [...chatMessages, messageObj];
        sendSignalingMessage('chat', messageObj);
        chatMessage = '';
    }

    function handleIncomingChat(message) {
        chatMessages = [...chatMessages, {
            ...message,
            sender: 'remote'
        }];
    }

    async function startScreenShare() {
        if (isScreenSharing) {
            endScreenShare();
            return;
        }

        try {
            const screenStream = await navigator.mediaDevices.getDisplayMedia({ 
                video: true,
                audio: false 
            });

            screenStream.getVideoTracks()[0].onended = endScreenShare;

            const videoTrack = screenStream.getVideoTracks()[0];
            const sender = peerConnection.getSenders().find(s => s.track.kind === 'video');
            
            if (sender) {
                await sender.replaceTrack(videoTrack);
                isScreenSharing = true;
                log('Screen sharing started', 'success');
            }
        } catch (error) {
            log(`Screen share error: ${error.message}`, 'error');
        }
    }

    function endScreenShare() {
        if (!isScreenSharing) return;

        const videoTrack = localStream.getVideoTracks()[0];
        const sender = peerConnection.getSenders().find(s => s.track.kind === 'video');
        
        if (sender) {
            sender.replaceTrack(videoTrack);
            isScreenSharing = false;
            log('Screen sharing ended', 'info');
        }
    }

    function toggleTrack(kind) {
        if (!localStream) return;
        
        const tracks = localStream.getTracks().filter(track => track.kind === kind);
        tracks.forEach(track => {
            track.enabled = !track.enabled;
            if (kind === 'video') {
                deviceState.isVideoMuted = !track.enabled;
            } else if (kind === 'audio') {
                deviceState.isAudioMuted = !track.enabled;
            }
        });
        
        log(`${kind.charAt(0).toUpperCase() + kind.slice(1)} ${tracks[0].enabled ? 'Unmuted' : 'Muted'}`, 'info');
    }

    function handleICECandidate(event) {
        if (event.candidate) {
            sendSignalingMessage('ice-candidate', event.candidate);
        }
    }

    function handleRemoteTrack(event) {
        if (event.streams[0]) {
            remoteVideoElement.srcObject = event.streams[0];
            log('Remote track received', 'success');
        }
    }

    function handleConnectionStateChange() {
        if (peerConnection) {
            peerConnectionStatus = peerConnection.connectionState;
            log(`Connection state: ${peerConnectionStatus}`, 'info');
        }
    }

    function handleOfferRejected() {
        log('Offer was rejected by peer', 'warning');
    }

    function rejectOffer() {
        isOfferReceived = false;
        sendSignalingMessage('offer-rejected', {});
        log('Offer rejected', 'warning');
    }

    onMount(() => {
        initializeWebRTC();
    });

    onDestroy(() => {
        localStream?.getTracks().forEach(track => track.stop());
        peerConnection?.close();
        signalingSocket?.close();
    });
</script>

<main class="webrtc-container dark-theme">
    <div class="video-section">
        <div class="video-grid">
            <!-- Local Stream -->
            <div class="video-wrapper local-stream">
                <div class="video-header">
                    <h3>Local Stream</h3>
                    <div class="stream-indicators">
                        <span class:active={!deviceState.isVideoMuted} 
                              class:inactive={deviceState.isVideoMuted}>
                            {!deviceState.isVideoMuted ? '📹 Video' : '🚫 Video'}
                        </span>
                        <span class:active={!deviceState.isAudioMuted} 
                              class:inactive={deviceState.isAudioMuted}>
                            {!deviceState.isAudioMuted ? '🎤 Audio' : '🔇 Audio'}
                        </span>
                    </div>
                </div>
                <video bind:this={localVideoElement} autoplay muted playsinline></video>
            </div>

            <!-- Remote Stream -->
            <div class="video-wrapper remote-stream">
                <div class="video-header">
                    <h3>Remote Stream</h3>
                    <div class="connection-status">
                        {peerConnectionStatus}
                    </div>
                </div>
                <video bind:this={remoteVideoElement} autoplay playsinline></video>
            </div>
        </div>
    </div>

    <!-- Connection Controls -->
    <div class="controls-section">
        <div class="control-group">
            <button on:click={() => createOffer()} class="primary-btn">
                <span>🤝 Create Offer</span>
            </button>
            <button 
                on:click={() => startScreenShare()} 
                class="secondary-btn {isScreenSharing ? 'active' : ''}"
            >
                <span>{isScreenSharing ? '⏹️ End Screen Share' : '📺 Share Screen'}</span>
            </button>
        </div>

        <!-- Device Controls -->
        <div class="control-group toggle-controls">
            <button 
                on:click={() => toggleTrack('video')} 
                class="toggle-btn {deviceState.isVideoMuted ? 'muted' : ''}"
            >
                <span>{deviceState.isVideoMuted ? '📹 Unmute Video' : '🚫 Mute Video'}</span>
            </button>
            <button 
                on:click={() => toggleTrack('audio')} 
                class="toggle-btn {deviceState.isAudioMuted ? 'muted' : ''}"
            >
                <span>{deviceState.isAudioMuted ? '🎤 Unmute Audio' : '🔇 Mute Audio'}</span>
            </button>
        </div>

        <!-- Chat Section -->
        <div class="chat-section">
            <div class="chat-messages">
                {#each chatMessages as msg}
                    <div class="chat-message {msg.sender}">
                        <span class="timestamp">{msg.timestamp}</span>
                        <span class="text">{msg.text}</span>
                    </div>
                {/each}
            </div>
            <div class="chat-input">
                <input 
                    type="text" 
                    bind:value={chatMessage} 
                    placeholder="Type a message..."
                    on:keydown={(e) => e.key === 'Enter' && sendChatMessage()}
                >
                <button on:click={sendChatMessage}>Send</button>
            </div>
        </div>
    </div>

    <!-- Event Log -->
    <div class="event-log">
        <h3>Event Log</h3>
        <ul>
            {#each eventLogs as log}
                <li class="log-entry {log.type}">
                    <span class="timestamp">{log.timestamp}</span>
                    <span class="message">{log.message}</span>
                </li>
            {/each}
        </ul>
    </div>

    <!-- Offer Modal -->
    {#if isOfferReceived}
        <div class="offer-modal">
            <div class="offer-content">
                <h3>Incoming Connection Request</h3>
                <p>A peer wants to establish a connection.</p>
                <div class="offer-actions">
                    <button on:click={() => createOffer()} class="accept-btn">Accept</button>
                    <button on:click={() => rejectOffer()} class="reject-btn">Reject</button>
                </div>
            </div>
        </div>
    {/if}
</main>

<style>
    .webrtc-container {
        --bg-primary: #1a1a1a;
        --bg-secondary: #2d2d2d;
        --text-primary: #ffffff;
        --text-secondary: #b3b3b3;
        --accent-color: #4CAF50;
        --error-color: #f44336;
        --warning-color: #ff9800;
        
        background-color: var(--bg-primary);
        color: var(--text-primary);
        min-height: 100vh;
        padding: 2rem;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen-Sans, Ubuntu, Cantarell, sans-serif;
    }

    .video-section {
        display: grid;
        gap: 2rem;
        margin-bottom: 2rem;
    }

    .video-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
        gap: 2rem;
    }

    .video-wrapper {
        background: var(--bg-secondary);
        border-radius: 12px;
        overflow: hidden;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }

    .video-header {
        padding: 1rem;
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: rgba(0, 0, 0, 0.2);
    }

    .video-header h3 {
        margin: 0;
        font-size: 1.1rem;
    }

    video {
        width: 100%;
        background: #000;
        aspect-ratio: 16/9;
    }

    .controls-section {
        display: grid;
        gap: 1rem;
        background: var(--bg-secondary);
        padding: 1.5rem;
        border-radius: 12px;
        margin-bottom: 2rem;
    }

    .control-group {
        display: flex;
        gap: 1rem;
        flex-wrap: wrap;
    }

    button {
        padding: 0.75rem 1.5rem;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        font-weight: 500;
        transition: all 0.3s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
    }

    .primary-btn {
        background: var(--accent-color);
        color: white;
    }

    .primary-btn:hover {
        background: #43A047;
    }

    .secondary-btn {
        background: #424242;
        color: var(--text-primary);
    }

    .secondary-btn:hover {
        background: #616161;
    }

    .toggle-btn {
        background: #424242;
        color: var(--text-primary);
    }

    .toggle-btn.muted {
        background: var(--error-color);
    }

    .chat-section {
        background: var(--bg-secondary);
        border-radius: 12px;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        height: 300px;
    }

    .chat-messages {
        flex-grow: 1;
        overflow-y: auto;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .chat-message {
        padding: 0.5rem 1rem;
        border-radius: 8px;
        max-width: 80%;
    }

    .chat-message.local {
        background: var(--accent-color);
        align-self: flex-end;
    }

    .chat-message.remote {
        background: #424242;
        align-self: flex-start;
    }

    .chat-input {
        display: flex;
        padding: 1rem;
        gap: 0.5rem;
        background: rgba(0, 0, 0, 0.2);
    }

    .chat-input input {
        flex-grow: 1;
        padding: 0.75rem;
        border: none;
        border-radius: 6px;
        background: #424242;
        color: var(--text-primary);
    }

    .offer-modal {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.8);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .offer-content {
        background: var(--bg-secondary);
        padding: 2rem;
        border-radius: 12px;
        text-align: center;
    }

    .offer-actions {
        display: flex;
        gap: 1rem;
        margin-top: 1.5rem;
        justify-content: center;
    }

    .accept-btn {
        background: var(--accent-color);
        color: white;
    }

    .reject-btn {
        background: var(--error-color);
        color: white;
    }

    .event-log {
        background: var(--bg-secondary);
        border-radius: 12px;
        padding: 1rem;
    }

    .log-entry {
        padding: 0.5rem;
        margin: 0.25rem 0;
        border-radius: 4px;
        display: flex;
        gap: 1rem;
    }

    .log-entry.error {
        background: rgba(244, 67, 54, 0.1);
        color: var(--error-color);
    }

    .log-entry.warning {
        background: rgba(255, 152, 0, 0.1);
        color: var(--warning-color);
    }

    .log-entry.success {
        background: rgba(76, 175, 80, 0.1);
        color: var(--accent-color);
    }

    .timestamp {
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    .stream-indicators {
        display: flex;
        gap: 1rem;
    }

    .stream-indicators span {
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.9rem;
    }

    .stream-indicators .active {
        background: rgba(76, 175, 80, 0.2);
        color: var(--accent-color);
    }

    .stream-indicators .inactive {
        background: rgba(244, 67, 54, 0.2);
        color: var(--error-color);
    }

    .connection-status {
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.9rem;
        background: rgba(0, 0, 0, 0.2);
    }
</style>