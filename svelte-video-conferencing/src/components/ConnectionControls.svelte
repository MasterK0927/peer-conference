<script>
    import { connectionStore, mediaStore } from '../stores';
    import { webSocketStore } from '../utils/websocket';

    async function createOffer() {
        try {
            // Implement offer creation logic
            webSocketStore.sendMessage('create-offer', {
                // Include necessary offer details
            });
        } catch (error) {
            connectionStore.setError(error.message);
        }
    }

    function startScreenShare() {
        mediaStore.toggleScreenShare();
        // Additional screen share logic
    }
</script>

<div class="connection-controls">
    <div class="control-grid">
        <button 
            class="control-btn primary" 
            on:click={createOffer}
            disabled={$connectionStore.peerConnectionStatus === 'Connected'}
        >
            <span class="btn-icon">🤝</span>
            <span class="btn-text">Create Offer</span>
        </button>

        <button 
            class="control-btn secondary" 
            on:click={startScreenShare}
            class:active={$mediaStore.devices.screen.sharing}
        >
            <span class="btn-icon">
                {$mediaStore.devices.screen.sharing ? '⏹️' : '📺'}
            </span>
            <span class="btn-text">
                {$mediaStore.devices.screen.sharing 
                    ? 'End Screen Share' 
                    : 'Share Screen'}
            </span>
        </button>
    </div>
</div>

<style>
    .connection-controls {
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

    .control-btn {
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
    }

    .control-btn .btn-icon {
        font-size: 1.2rem;
    }

    .control-btn.primary {
        background-color: #0f3460;
        color: #e94560;
    }

    .control-btn.primary:hover {
        background-color: #e94560;
        color: white;
    }

    .control-btn.primary:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .control-btn.secondary {
        background-color: #0f3460;
        color: #00adb5;
    }

    .control-btn.secondary:hover,
    .control-btn.secondary.active {
        background-color: #00adb5;
        color: white;
    }
</style>