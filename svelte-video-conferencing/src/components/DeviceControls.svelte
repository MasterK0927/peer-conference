<script>
    import { mediaStore } from '../stores';
</script>

<div class="device-controls">
    <div class="control-grid">
        <button 
            class="device-btn" 
            class:muted={$mediaStore.devices.video.muted}
            on:click={() => mediaStore.toggleVideo()}
        >
            <span class="btn-icon">
                {$mediaStore.devices.video.muted ? '🚫' : '📹'}
            </span>
            <span class="btn-text">
                {$mediaStore.devices.video.muted 
                    ? 'Unmute Video' 
                    : 'Mute Video'}
            </span>
        </button>

        <button 
            class="device-btn" 
            class:muted={$mediaStore.devices.audio.muted}
            on:click={() => mediaStore.toggleAudio()}
        >
            <span class="btn-icon">
                {$mediaStore.devices.audio.muted ? '🔇' : '🎤'}
            </span>
            <span class="btn-text">
                {$mediaStore.devices.audio.muted 
                    ? 'Unmute Audio' 
                    : 'Mute Audio'}
            </span>
        </button>
    </div>
</div>

<style>
    .device-controls {
        background-color: #16213e;
        border-radius: 12px;
        padding: 1rem;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
    }

    .control-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
    }

    .device-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.75rem;
        padding: 0.75rem 1.25rem;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.3s ease;
        font-weight: 600;
        background-color: #0f3460;
        color: #00adb5;
    }

    .device-btn .btn-icon {
        font-size: 1.2rem;
    }

    .device-btn:hover {
        background-color: #00adb5;
        color: white;
    }

    .device-btn.muted {
        background-color: #e94560;
        color: white;
    }

    .device-btn.muted:hover {
        background-color: #ff6b81;
    }
</style>