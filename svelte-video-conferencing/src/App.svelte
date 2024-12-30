<script>
    import { onMount, onDestroy } from 'svelte';
    import { slide } from 'svelte/transition';
    import * as ed from '@noble/ed25519';
    import { sha512 } from '@noble/hashes/sha512';
    
    ed.etc.sha512Sync = (...m) => sha512(ed.etc.concatBytes(...m));
    
    const SignalTypes = {
        SECURE_OFFER: 'secure-offer',
        SECURE_ANSWER: 'secure-answer',
        ICE_CANDIDATE: 'ice-candidate'
    };
    
    let localVideoElement; 
    let remoteVideoElement;
    let peerConnection = null;
    let localStream = null;
    let signalingSocket = null;
    let connectionStatus = 'Not Connected';
    let peerConnectionStatus = 'Disconnected';
    let isScreenSharing = false;
    let eventLogs = [];
    let isInitialized = false;
    
    let keyPair = null;
    let clientId = null;
    let isVerified = false;
    
    let isCalling = false;
    let canCreateOffer = false;
    
    let deviceState = {
        isVideoMuted: false,
        isAudioMuted: false
    };
    
    const SIGNALING_SERVER = "ws://127.0.0.1:3030";
    const mediaConstraints = {
        video: { width: 1280, height: 720 },
        audio: true
    };
    
    const peerConfig = {
        iceServers: [
            { urls: 'stun:stun.l.google.com:19302' },
            { urls: 'stun:stun1.l.google.com:19302' }
        ]
    };
    
    function log(message, type = 'info') {
        const timestamp = new Date().toLocaleTimeString();
        eventLogs = [...eventLogs.slice(-10), { timestamp, message, type }];
        console.log(`[${type.toUpperCase()}] ${message}`);
    }
    
    async function generateKeyPair() {
        try {
            const privateKey = ed.utils.randomPrivateKey();
            const publicKey = await ed.getPublicKey(privateKey);
            
            keyPair = {
                privateKey,
                publicKey
            };
    
            const testMessage = new TextEncoder().encode('test');
            const testSignature = await ed.sign(testMessage, privateKey);
            const isValid = await ed.verify(testSignature, testMessage, publicKey);
            
            if (!isValid) {
                throw new Error('Key pair verification failed');
            }
            
            log('Ed25519 key pair generated and verified successfully', 'success');
            return keyPair;
        } catch (error) {
            log(`Key pair generation error: ${error.message}`, 'error');
            return null;
        }
    }
    
    async function signPayload(payload) {
        try {
            const message = new TextEncoder().encode(JSON.stringify(payload));
            const signature = await ed.sign(message, keyPair.privateKey);
            return Array.from(signature);
        } catch (error) {
            log(`Signing error: ${error.message}`, 'error');
            return null;
        }
    }
    
    async function initializeWebRTC() {
        try {
            const generatedKeyPair = await generateKeyPair();
            if (!generatedKeyPair) {
                throw new Error('Failed to generate key pair');
            }
            keyPair = generatedKeyPair;
    
            localStream = await navigator.mediaDevices.getUserMedia(mediaConstraints);
            if (!localVideoElement) {
                throw new Error('Local video element not bound');
            }
            localVideoElement.srcObject = localStream;
            log('Local media stream acquired', 'success');
    
            peerConnection = new RTCPeerConnection(peerConfig);
            localStream.getTracks().forEach(track => peerConnection.addTrack(track, localStream));
            
            peerConnection.onicecandidate = handleICECandidate;
            peerConnection.ontrack = handleRemoteTrack;
            peerConnection.onconnectionstatechange = handleConnectionStateChange;
    
            clientId = crypto.randomUUID();
            await connectSignalingServer();
            isInitialized = true;
            canCreateOffer = true;
            log('WebRTC system fully initialized', 'success');
            
            return true;
        } catch (error) {
            log(`WebRTC initialization error: ${error.message}`, 'error');
            isInitialized = false;
            canCreateOffer = false;
            return false;
        }
    }
    
    function connectSignalingServer() {
        return new Promise((resolve, reject) => {
            try {
                signalingSocket = new WebSocket(SIGNALING_SERVER);
    
                signalingSocket.onopen = () => {
                    connectionStatus = 'Connected';
                    log('Signaling server connection established', 'success');
                    resolve(true);
                };
    
                signalingSocket.onerror = (error) => {
                    log(`Signaling server error: ${error.message}`, 'error');
                    reject(error);
                };
    
                signalingSocket.onmessage = async (event) => {
                    const message = JSON.parse(event.data);
                    await handleSignalingMessage(message);
                };
    
                signalingSocket.onclose = () => {
                    connectionStatus = 'Disconnected';
                    isVerified = false;
                    isInitialized = false;
                    canCreateOffer = false;
                    log('Signaling server connection closed', 'warning');
                };
    
                setTimeout(() => {
                    if (signalingSocket.readyState !== WebSocket.OPEN) {
                        reject(new Error('Connection timeout'));
                    }
                }, 5000);
            } catch (error) {
                reject(error);
            }
        });
    }
    
    function handleICECandidate(event) {
        if (event.candidate) {
            sendSignalingMessage(SignalTypes.ICE_CANDIDATE, event.candidate);
        }
    }
    
    function handleRemoteTrack(event) {
        if (remoteVideoElement && event.streams[0]) {
            remoteVideoElement.srcObject = event.streams[0];
        }
    }
    
    function handleConnectionStateChange() {
        if (peerConnection) {
            peerConnectionStatus = peerConnection.connectionState;
            log(`Peer connection state: ${peerConnectionStatus}`);
            
            if (peerConnectionStatus === 'connected') {
                isCalling = true;
            } else if (peerConnectionStatus === 'disconnected' || 
                       peerConnectionStatus === 'failed' || 
                       peerConnectionStatus === 'closed') {
                isCalling = false;
            }
        }
    }
    
    async function createOffer() {
        try {
            if (!isInitialized) {
                throw new Error('WebRTC system not initialized. Please initialize first.');
            }
            if (!keyPair || !keyPair.privateKey || !keyPair.publicKey) {
                throw new Error('Cryptographic keys not properly initialized');
            }
            if (signalingSocket?.readyState !== WebSocket.OPEN) {
                throw new Error('Signaling server connection not established');
            }
    
            isCalling = true;
            const offer = await peerConnection.createOffer();
            await peerConnection.setLocalDescription(offer);
            const secureConnectionPayload = {
                offer: offer,
                public_key: Array.from(keyPair.publicKey),
                nonce: Array.from(crypto.getRandomValues(new Uint8Array(32)))
            };
            const signature = await signPayload(secureConnectionPayload.offer);
            if (!signature) {
                throw new Error('Failed to sign offer payload');
            }
            secureConnectionPayload.signature = signature;
            sendSignalingMessage(SignalTypes.SECURE_OFFER, secureConnectionPayload);
            
            log('Secure offer created and sent successfully', 'success');
            return true;
        } catch (error) {
            log(`Offer creation error: ${error.message}`, 'error');
            isCalling = false;
            return false;
        }
    }
    
    async function handleSecureOffer(payload) {
        try {
            if (!peerConnection) {
                throw new Error('No peer connection established');
            }
    
            const { offer, signature, public_key } = payload;
            const message = new TextEncoder().encode(JSON.stringify(offer));
            const isValid = await ed.verify(signature, message, public_key);
            
            if (!isValid) {
                throw new Error('Invalid offer signature');
            }
    
            await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
            const answer = await peerConnection.createAnswer();
            await peerConnection.setLocalDescription(answer);
    
            const secureAnswerPayload = {
                offer: answer,
                public_key: Array.from(keyPair.publicKey),
                nonce: Array.from(crypto.getRandomValues(new Uint8Array(32)))
            };
    
            const answerSignature = await signPayload(secureAnswerPayload.offer);
            secureAnswerPayload.signature = answerSignature;
    
            sendSignalingMessage(SignalTypes.SECURE_ANSWER, secureAnswerPayload);
            isVerified = true;
            isCalling = true;
            log('Secure offer handled and answer sent', 'success');
        } catch (error) {
            log(`Secure offer handling error: ${error.message}`, 'error');
            isCalling = false;
        }
    }

    async function handleSecureAnswer(payload) {
        try {
            if (!peerConnection) {
                throw new Error('No peer connection established');
            }
    
            const { offer, signature, public_key } = payload;
            const message = new TextEncoder().encode(JSON.stringify(offer));
            const isValid = await ed.verify(signature, message, public_key);
            
            if (!isValid) {
                throw new Error('Invalid answer signature');
            }
    
            await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
            isVerified = true;
            log('Secure answer processed successfully', 'success');
        } catch (error) {
            log(`Secure answer handling error: ${error.message}`, 'error');
            isCalling = false;
        }
    }

    async function handleSignalingMessage(message) {
        try {
            const payload = JSON.parse(message.payload);
            
            if (!message.sender_id || message.sender_id === clientId) {
                return;
            }
    
            switch (message.signal_type) {
                case SignalTypes.SECURE_OFFER:
                    await handleSecureOffer(payload);
                    break;
                case SignalTypes.SECURE_ANSWER:
                    await handleSecureAnswer(payload);
                    break;
                case SignalTypes.ICE_CANDIDATE:
                    if (peerConnection && isVerified && payload) {
                        await peerConnection.addIceCandidate(new RTCIceCandidate(payload));
                    }
                    break;
            }
        } catch (error) {
            log(`Signaling message handling error: ${error.message}`, 'error');
        }
    }
    
    function sendSignalingMessage(signalType, payload) {
        if (signalingSocket?.readyState === WebSocket.OPEN) {
            const message = {
                signal_type: signalType,
                payload: JSON.stringify(payload),
                sender_id: clientId,
                timestamp: Date.now(),
                signature: null
            };
            
            signalingSocket.send(JSON.stringify(message));
        } else {
            log('Signaling socket not ready', 'error');
        }
    }
    
    function toggleTrack(kind) {
        if (!localStream || !isInitialized) {
            log(`Cannot toggle ${kind}: stream unavailable or not initialized`, 'error');
            return;
        }
        
        const tracks = localStream.getTracks().filter(track => track.kind === kind);
        tracks.forEach(track => {
            track.enabled = !track.enabled;
            deviceState[kind === 'video' ? 'isVideoMuted' : 'isAudioMuted'] = !track.enabled;
            log(`${kind} ${track.enabled ? 'unmuted' : 'muted'}`, 'info');
        });
    }
    
    async function startScreenShare() {
        if (!peerConnection || !isInitialized) {
            log('Cannot start screen share: connection not initialized', 'error');
            return;
        }
    
        try {
            if (isScreenSharing) {
                await endScreenShare();
                return;
            }
    
            const screenStream = await navigator.mediaDevices.getDisplayMedia({ 
                video: true 
            });
    
            screenStream.getVideoTracks()[0].onended = endScreenShare;
    
            const sender = peerConnection.getSenders().find(s => s.track?.kind === 'video');
            if (sender) {
                await sender.replaceTrack(screenStream.getVideoTracks()[0]);
                isScreenSharing = true;
                log('Screen sharing started', 'success');
            }
        } catch (error) {
            log(`Screen sharing error: ${error.message}`, 'error');
        }
    }
    
    async function endScreenShare() {
        try {
            const videoTrack = localStream?.getVideoTracks()[0];
            const sender = peerConnection?.getSenders().find(s => s.track?.kind === 'video');
            
            if (sender && videoTrack) {
                await sender.replaceTrack(videoTrack);
                isScreenSharing = false;
                log('Screen sharing ended', 'info');
            }
        } catch (error) {
            log(`Error ending screen share: ${error.message}`, 'error');
        }
    }
    
    function endCall() {
        if (peerConnection) {
            peerConnection.close();
            isCalling = false;
            isVerified = false;
            log('Call ended', 'info');
        }
    }

    onMount(async () => {
        await initializeWebRTC();
    });

    onDestroy(() => {
        peerConnection?.close();
        signalingSocket?.close();
        localStream?.getTracks().forEach(track => track.stop());
    });

</script>

<!-- Main container with dark theme -->
<main class="webrtc-container dark-theme">
    <div class="video-section">
        <div class="video-grid">
            <!-- Local Stream Video Container -->
            <div class="video-wrapper local-stream">
                <div class="video-header">
                    <h3>Local Stream</h3>
                    <div class="stream-indicators">
                        <span class:active={!deviceState.isVideoMuted} 
                              class:inactive={deviceState.isVideoMuted}>
                            {deviceState.isVideoMuted ? '🚫 Video' : '📹 Video'}
                        </span>
                        <span class:active={!deviceState.isAudioMuted} 
                              class:inactive={deviceState.isAudioMuted}>
                            {deviceState.isAudioMuted ? '🔇 Audio' : '🎤 Audio'}
                        </span>
                    </div>
                </div>
                <!-- Local video element with bindings -->
                <video bind:this={localVideoElement} 
                       autoplay 
                       muted 
                       playsinline>
                </video>
            </div>

            <!-- Remote Stream Video Container -->
            <div class="video-wrapper remote-stream">
                <div class="video-header">
                    <h3>Remote Stream</h3>
                    <div class="connection-status" 
                         class:connected={peerConnectionStatus === 'Connected'} 
                         class:disconnected={peerConnectionStatus === 'Disconnected'}>
                        {peerConnectionStatus}
                    </div>
                </div>
                <!-- Remote video element with bindings -->
                <!-- svelte-ignore a11y-media-has-caption -->
                <video bind:this={remoteVideoElement} 
                       autoplay 
                       playsinline>
                </video>
            </div>
        </div>
    </div>

    <!-- Control Section -->
    <div class="controls-section">
        <!-- Connection Controls -->
        <div class="control-group">
            <button on:click={createOffer} 
                    class="primary-btn" 
                    disabled={!peerConnection || connectionStatus !== 'Connected'}>
                <span>🤝 Create Offer</span>
            </button>
            <button on:click={startScreenShare} 
                    class="secondary-btn {isScreenSharing ? 'active' : ''}"
                    disabled={!peerConnection}>
                <span>{isScreenSharing ? '⏹️ Stop Screen Share' : '📺 Share Screen'}</span>
            </button>
        </div>

        <!-- Media Controls -->
        <div class="control-group toggle-controls">
            <button on:click={() => toggleTrack('video')} 
                    class="toggle-btn {deviceState.isVideoMuted ? 'muted' : ''}">
                <span>{deviceState.isVideoMuted ? '📹 Enable Video' : '🚫 Disable Video'}</span>
            </button>
            <button on:click={() => toggleTrack('audio')} 
                    class="toggle-btn {deviceState.isAudioMuted ? 'muted' : ''}">
                <span>{deviceState.isAudioMuted ? '🎤 Enable Audio' : '🔇 Disable Audio'}</span>
            </button>
        </div>
    </div>

    <!-- Event Logging Section -->
    <div class="event-log">
        <h3>Event Log</h3>
        <ul>
            {#each eventLogs as log (log.timestamp)}
                <li class="log-entry {log.type}">
                    <span class="timestamp">{log.timestamp}</span>
                    <span class="message">{log.message}</span>
                </li>
            {/each}
        </ul>
    </div>

    <!-- Connection Status Toast -->
    {#if connectionStatus === 'Connected'}
        <div class="status-toast success" transition:slide>
            Connected to signaling server
        </div>
    {:else if connectionStatus === 'Disconnected'}
        <div class="status-toast error" transition:slide>
            Disconnected from signaling server
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