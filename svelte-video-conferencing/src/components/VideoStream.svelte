<script>
    import { mediaStore, connectionStore } from '../stores';
    
    export let localVideoElement = null;
    export let remoteVideoElement = null;

    $: localStreamAvailable = $mediaStore.localStream !== null;
    $: remoteStreamAvailable = $mediaStore.remoteStream !== null;
    $: isVideoMuted = $mediaStore.devices.video.muted;
    $: isAudioMuted = $mediaStore.devices.audio.muted;
</script>

<div class="video-stream-container">
    <div class="video-grid">
        <!-- Local Stream -->
        <div class="video-wrapper local-stream" class:inactive={!localStreamAvailable}>
            <div class="video-header">
                <h3>Local Stream</h3>
                <div class="stream-indicators">
                    <span class:active={!isVideoMuted} class:inactive={isVideoMuted}>
                        {!isVideoMuted ? '📹 Video' : '🚫 Video'}
                    </span>
                    <span class:active={!isAudioMuted} class:inactive={isAudioMuted}>
                        {!isAudioMuted ? '🎤 Audio' : '🔇 Audio'}
                    </span>
                </div>
            </div>
            <video 
                bind:this={localVideoElement} 
                autoplay 
                muted 
                playsinline
            ></video>
        </div>

        <!-- Remote Stream -->
        <div class="video-wrapper remote-stream" class:inactive={!remoteStreamAvailable}>
            <div class="video-header">
                <h3>Remote Stream</h3>
                <div class="connection-status">
                    {$connectionStore.peerConnectionStatus}
                </div>
            </div>
            <!-- svelte-ignore a11y-media-has-caption -->
            <video 
                bind:this={remoteVideoElement} 
                autoplay 
                playsinline
            ></video>
        </div>
    </div>
</div>

<style>
    .video-stream-container {
        background-color: #1a1a2e;
        border-radius: 12px;
        overflow: hidden;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    }

    .video-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 1rem;
        padding: 1rem;
    }

    .video-wrapper {
        background-color: #16213e;
        border-radius: 10px;
        overflow: hidden;
        transition: all 0.3s ease;
        transform: scale(1);
    }

    .video-wrapper.inactive {
        opacity: 0.6;
        transform: scale(0.95);
    }

    .video-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem;
        background-color: rgba(0, 0, 0, 0.2);
    }

    .video-header h3 {
        margin: 0;
        color: #e94560;
        font-weight: 600;
    }

    video {
        width: 100%;
        aspect-ratio: 16/9;
        background-color: #000;
        object-fit: cover;
    }

    .stream-indicators, .connection-status {
        display: flex;
        gap: 0.5rem;
        font-size: 0.8rem;
    }

    .stream-indicators span, .connection-status {
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-weight: 500;
    }

    .stream-indicators .active {
        background-color: rgba(76, 175, 80, 0.2);
        color: #4CAF50;
    }

    .stream-indicators .inactive {
        background-color: rgba(244, 67, 54, 0.2);
        color: #f44336;
    }

    .connection-status {
        background-color: rgba(33, 150, 243, 0.2);
        color: #2196F3;
    }
</style>