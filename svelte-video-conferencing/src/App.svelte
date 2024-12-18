<script>
    import { onMount, onDestroy } from 'svelte';

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
    let currentChallenge = null;
    let keyPair = null;
    
    let deviceState = {
        isVideoMuted: false,
        isAudioMuted: false,
        isRecording: false
    };

    const SIGNALING_SERVER = "ws://127.0.0.1:3030";
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

    async function generateChallenge() {
        const challenge = crypto.getRandomValues(new Uint8Array(32));
        return Array.from(challenge);
    }

    async function generateKeyPair() {
        try {
            // Generate ECDSA key pair for signing
            const signingKeyPair = await crypto.subtle.generateKey(
                {
                    name: 'ECDSA',
                    namedCurve: 'P-256'
                },
                true,
                ['sign', 'verify']
            );

            // Generate RSA key pair for encryption
            const encryptionKeyPair = await crypto.subtle.generateKey(
                {
                    name: 'RSA-OAEP',
                    modulusLength: 2048,
                    publicExponent: new Uint8Array([1, 0, 1]),
                    hash: 'SHA-256'
                },
                true,
                ['encrypt', 'decrypt']
            );

            // Export the public keys in the correct format
            const signingPublicKey = await crypto.subtle.exportKey(
                'raw',  // Changed from 'spki' to 'raw' for ECDSA
                signingKeyPair.publicKey
            );

            const encryptionPublicKey = await crypto.subtle.exportKey(
                'spki',
                encryptionKeyPair.publicKey
            );

            keyPair = {
                signing: signingKeyPair,
                encryption: encryptionKeyPair
            };

            log('Key pairs generated successfully', 'success');
            log(`Signing public key length: ${signingPublicKey.byteLength}`, 'info');
            log(`Encryption public key length: ${encryptionPublicKey.byteLength}`, 'info');

            return {
                signingPublicKey: Array.from(new Uint8Array(signingPublicKey)),
                encryptionPublicKey: Array.from(new Uint8Array(encryptionPublicKey)),
                keyPair: keyPair
            };
        } catch (error) {
            log(`Key pair generation error: ${error.message}`, 'error');
            console.error('Full key pair generation error:', error);
            return null;
        }
    }

    async function signPayload(payload) {
        try {
            if (!keyPair || !keyPair.signing) {
                throw new Error('Signing key pair not initialized');
            }

            const encoder = new TextEncoder();
            const verificationPayload = {
                challenge: payload.challenge,
                offer: payload.offer,
                encrypted_data: payload.encrypted_data,
                connection_message: payload.connection_message
            };

            const data = encoder.encode(JSON.stringify(verificationPayload));
            
            const signature = await crypto.subtle.sign(
                {
                    name: 'ECDSA',
                    hash: { name: 'SHA-256' }
                },
                keyPair.signing.privateKey,
                data
            );

            // Return the signature as a byte array
            return Array.from(new Uint8Array(signature));
        } catch (error) {
            log(`Payload signing error: ${error.message}`, 'error');
            console.error('Full signing error:', error);
            return null;
        }
    }

    async function verifyPayloadSignature(payload, signature, publicKey) {
        try {
            log('Starting payload signature verification', 'info');
            
            // Log input details
            log(`Payload details: ${JSON.stringify({
                challenge: payload.challenge ? Array.from(payload.challenge) : null,
                offer: payload.offer ? 'Present' : 'Not Present',
                encrypted_data: payload.encrypted_data ? 'Present' : 'Not Present',
                connection_message: payload.connection_message
            })}`, 'info');
            
            log(`Public key length: ${publicKey.length}`, 'info');
            log(`Signature length: ${signature.length}`, 'info');

            const importedPublicKey = await crypto.subtle.importKey(
                'spki',
                new Uint8Array(publicKey),
                {
                    name: 'ECDSA',
                    namedCurve: 'P-256'
                },
                true,
                ['verify']
            );

            const verificationPayload = {
                challenge: payload.challenge,
                offer: payload.offer,
                encrypted_data: payload.encrypted_data,
                connection_message: payload.connection_message
            };

            const encoder = new TextEncoder();
            const data = encoder.encode(JSON.stringify(verificationPayload));

            log(`Verification payload JSON: ${JSON.stringify(verificationPayload)}`, 'info');
            log(`Verification data bytes length: ${data.length}`, 'info');

            const verificationResult = await crypto.subtle.verify(
                {
                    name: 'ECDSA',
                    hash: 'SHA-256'
                },
                importedPublicKey,
                new Uint8Array(signature),
                data
            );

            log(`Signature verification result: ${verificationResult}`, 'info');
            return verificationResult;
        } catch (error) {
            log(`Signature verification error: ${error.message}`, 'error');
            console.error('Full verification error:', error);
            return false;
        }
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

    
    async function createOffer() {
        try {
            if (!keyPair) {
                const keyGenResult = await generateKeyPair();
                if (!keyGenResult) {
                    throw new Error('Failed to generate key pair');
                }
                
                // Log key pair generation details
                log(`Key pair generated successfully`, 'success');
                log(`Signing public key exported length: ${keyGenResult.signingPublicKey.length}`, 'info');
            }

            const challenge = await generateChallenge();
            currentChallenge = challenge;
            log(`Generated challenge: ${Array.from(challenge)}`, 'info');

            const offer = await peerConnection.createOffer();
            await peerConnection.setLocalDescription(offer);
            log(`Created local offer`, 'success');

            const publicKey = await crypto.subtle.exportKey(
                'spki', 
                keyPair.encryption.publicKey
            );

            const connectionMessage = offerMessage || 'Secure connection request';

            const encryptedData = await encryptWithPublicKey(
                connectionMessage, 
                new Uint8Array(publicKey)
            );

            if (!encryptedData) {
                throw new Error('Encryption failed: Unable to encrypt connection message');
            }

            const signature = await signPayload({
                challenge: Array.from(challenge),
                offer: offer,
                encrypted_data: encryptedData,
                connection_message: connectionMessage
            });

            if (!signature) {
                throw new Error('Failed to create payload signature');
            }

            log(`Signature created successfully`, 'success');

            const connectionVerificationPayload = {
                challenge: Array.from(challenge),
                offer: offer,
                encrypted_data: encryptedData,
                connection_message: connectionMessage,
                public_key: Array.from(new Uint8Array(publicKey)),
                signature: signature
            };

            // Log the full payload before sending
            log(`Connection verification payload details:
                Challenge length: ${connectionVerificationPayload.challenge.length}
                Public key length: ${connectionVerificationPayload.public_key.length}
                Signature length: ${connectionVerificationPayload.signature.length}
                Offer present: ${!!connectionVerificationPayload.offer}
                Encrypted data present: ${!!connectionVerificationPayload.encrypted_data}
            `, 'info');

            sendSignalingMessage('offer-with-challenge', connectionVerificationPayload);
            log('Offer sent with challenge and signature', 'success');
        } catch (error) {
            log(`Offer creation error: ${error.message}`, 'error');
            console.error('Full offer creation error:', error);
        }
    }

    async function handleOfferWithChallenge(payload) {
        try {
            const { 
                challenge, 
                offer, 
                connection_message, 
                public_key, 
                signature 
            } = payload;
            
            const signatureVerified = await verifyPayloadSignature(
                { challenge, offer, connection_message }, 
                signature, 
                public_key
            );

            if (!signatureVerified) {
                log('Payload signature verification failed', 'error');
                return;
            }

            const decryptedMessage = await decryptWithPublicKey(
                connection_message, 
                public_key
            );

            if (decryptedMessage) {
                log('Connection message verified successfully', 'success');
                isOfferReceived = true;
                
                await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
                const answer = await peerConnection.createAnswer();
                await peerConnection.setLocalDescription(answer);

                const responsePayload = {
                    challenge: challenge,
                    challenge_response: await encryptWithPublicKey(
                        JSON.stringify(challenge), 
                        public_key
                    )
                };

                sendSignalingMessage('challenge-response', responsePayload);
            }
        } catch (error) {
            log(`Offer with challenge error: ${error.message}`, 'error');
        }
    }

    async function encryptWithPublicKey(message, publicKeyData) {
        try {
            // Ensure publicKeyData is a valid array or Uint8Array
            const keyData = publicKeyData instanceof Uint8Array 
                ? publicKeyData 
                : new Uint8Array(publicKeyData);

            // Validate key data
            if (!keyData || keyData.length === 0) {
                throw new Error('Invalid public key data');
            }

            // Generate a random IV (even though RSA-OAEP doesn't use it, we need it for the payload structure)
            const iv = crypto.getRandomValues(new Uint8Array(16));

            const publicKey = await crypto.subtle.importKey(
                'spki',
                keyData,
                {
                    name: 'RSA-OAEP',
                    hash: 'SHA-256'
                },
                false,
                ['encrypt']
            );

            const encoder = new TextEncoder();
            const data = encoder.encode(message);

            const encryptedData = await crypto.subtle.encrypt(
                { name: 'RSA-OAEP' },
                publicKey,
                data
            );

            // Create signature for the encrypted data
            const signature = crypto.getRandomValues(new Uint8Array(64)); // Placeholder signature
            
            return {
                encrypted: Array.from(new Uint8Array(encryptedData)),
                iv: Array.from(iv), // Include the IV in the payload
                sender_ip: window.location.hostname, // Add sender IP
                signature: Array.from(signature) // Include the signature
            };
        } catch (error) {
            log(`Public key encryption error: ${error.message}`, 'error');
            console.error('Full encryption error:', error);
            return null;
        }
    }

    async function decryptWithPublicKey(encryptedMessage, publicKeyData) {
        try {
            const publicKey = await crypto.subtle.importKey(
                'spki',
                new Uint8Array(publicKeyData),
                {
                    name: 'RSA-OAEP',
                    hash: 'SHA-256'
                },
                false,
                ['decrypt']
            );

            const decryptedData = await crypto.subtle.decrypt(
                {
                    name: 'RSA-OAEP'
                },
                publicKey,
                new Uint8Array(encryptedMessage)
            );

            return new TextDecoder().decode(decryptedData);
        } catch (error) {
            log(`Public key decryption error: ${error.message}`, 'error');
            return null;
        }
    }

    async function handleSignalingMessage(message) {
        try {
            const parsedPayload = JSON.parse(message.payload);
            switch (message.signal_type) {
                case 'offer-with-challenge': 
                    await handleOfferWithChallenge(parsedPayload); 
                    break;
                case 'challenge-response':
                    await validateChallengeResponse(parsedPayload);
                    break;
                case 'connection-verified':
                    handleConnectionVerification();
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

    function connectSignalingServer() {
        try {
            signalingSocket = new WebSocket(SIGNALING_SERVER);

            signalingSocket.onopen = () => {
                connectionStatus = 'Connected';
                log('Signaling server connection established', 'success');
            };

            signalingSocket.onmessage = async (event) => {
                try {
                    const message = JSON.parse(event.data);
                    await handleSignalingMessage(message);
                } catch (error) {
                    log(`Signaling message error: ${error.message}`, 'error');
                }
            };

            signalingSocket.onclose = () => {
                connectionStatus = 'Disconnected';
                log('Signaling server connection closed', 'warning');
            };

            signalingSocket.onerror = (error) => {
                log(`Signaling server connection error: ${error}`, 'error');
            };
        } catch (error) {
            log(`WebSocket connection error: ${error.message}`, 'error');
        }
    }

    function sendSignalingMessage(signalType, payload) {
        if (signalingSocket && signalingSocket.readyState === WebSocket.OPEN) {
            const message = JSON.stringify({
                signal_type: signalType,
                payload: JSON.stringify(payload),
                timestamp: Date.now()
            });
            signalingSocket.send(message);
        } else {
            log('Signaling socket not open', 'error');
        }
    }

    async function validateChallengeResponse(challengePayload) {
        try {
            if (currentChallenge && 
                JSON.stringify(challengePayload) === JSON.stringify(currentChallenge)) {
                log('Challenge validated successfully', 'success');
                
                sendSignalingMessage('connection-verified', {});
            } else {
                log('Challenge validation failed', 'error');
            }
        } catch (error) {
            log(`Challenge validation error: ${error.message}`, 'error');
        }
    }

    function handleConnectionVerification() {
        peerConnectionStatus = 'Verified';
        log('Peer connection fully verified', 'success');
    }

    async function handleAnswer(answer) {
        try {
            await peerConnection.setRemoteDescription(new RTCSessionDescription(answer));
            log('Remote answer set successfully', 'success');
        } catch (error) {
            log(`Answer handling error: ${error.message}`, 'error');
        }
    }

    function handleICECandidate(event) {
        if (event.candidate) {
            sendSignalingMessage('ice-candidate', event.candidate);
            log('ICE candidate generated', 'info');
        }
    }

    function handleRemoteTrack(event) {
        if (event.streams && event.streams[0]) {
            remoteVideoElement.srcObject = event.streams[0];
            log('Remote track received', 'success');
        }
    }

    function handleConnectionStateChange() {
        if (peerConnection) {
            peerConnectionStatus = peerConnection.connectionState;
            log(`Connection state changed: ${peerConnectionStatus}`, 'info');
        }
    }
    
    function sendChatMessage() {
        if (chatMessage.trim()) {
            const message = {
                text: chatMessage,
                sender: 'Me',
                timestamp: new Date().toLocaleTimeString()
            };

            sendSignalingMessage('chat', message);
            chatMessages = [...chatMessages, message];
            chatMessage = '';
        }
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

    function handleOfferRejected() {
        log('Offer was rejected by peer', 'warning');
    }

    function rejectOffer() {
        isOfferReceived = false;
        sendSignalingMessage('offer-rejected', {});
        log('Offer rejected', 'warning');
    }

    onMount(async () => {
        await initializeWebRTC();
    });

    onDestroy(() => {
        if (peerConnection) {
            peerConnection.close();
        }
        if (signalingSocket) {
            signalingSocket.close();
        }
        if (localStream) {
            localStream.getTracks().forEach(track => track.stop());
        }
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