<script>
    import { onMount, onDestroy } from 'svelte';
    import { 
        connectionStore, 
        mediaStore, 
        chatStore 
    } from './stores';
    import { webSocketStore } from './utils/websocket';

    import VideoStream from './components/VideoStream.svelte';
    import ChatPanel from './components/ChatPanel.svelte';
    import ConnectionControls from './components/ConnectionControls.svelte';
    import DeviceControls from './components/DeviceControls.svelte';
    import EventLog from './components/EventLog.svelte';
    import OfferModal from './components/OfferModal.svelte';

    let localVideoElement;
    let remoteVideoElement;

    onMount(async () => {
        webSocketStore.connect();

        try {
            const stream = await navigator.mediaDevices.getUserMedia({
                video: { width: 1280, height: 720 },
                audio: true
            });
            
            mediaStore.setLocalStream(stream);
            localVideoElement.srcObject = stream;
        } catch (error) {
            console.error('Media device access error:', error);
        }
    });

    onDestroy(() => {
        webSocketStore.disconnect();
        
        if ($mediaStore.localStream) {
            $mediaStore.localStream.getTracks().forEach(track => track.stop());
        }
    });

</script>

<main class="webrtc-container">
    <div class="video-section">
        <VideoStream 
            bind:localVideoElement 
            bind:remoteVideoElement 
        />
    </div>

    <div class="controls-section">
        <ConnectionControls />
        <DeviceControls />
        <ChatPanel />
    </div>

    <EventLog />

    {#if $connectionStore.isOfferReceived}
        <OfferModal />
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

    .controls-section {
        display: grid;
        gap: 1rem;
        background: var(--bg-secondary);
        padding: 1.5rem;
        border-radius: 12px;
        margin-bottom: 2rem;
    }
</style>