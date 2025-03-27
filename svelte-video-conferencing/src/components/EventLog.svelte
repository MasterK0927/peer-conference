<script>
    import { fade } from 'svelte/transition';
</script>

<div class="event-log">
    <div class="log-header">
        <h3>Event Log</h3>
    </div>
    <div class="log-content">
        <!-- Replace with actual event logs from your store -->
        <div class="log-item success" transition:fade>
            <span class="log-timestamp">12:34:56</span>
            <span class="log-message">WebSocket connection established</span>
        </div>
        <div class="log-item warning" transition:fade>
            <span class="log-timestamp">12:35:01</span>
            <span class="log-message">Media device access requested</span>
        </div>
        <div class="log-item error" transition:fade>
            <span class="log-timestamp">12:35:15</span>
            <span class="log-message">ICE candidate generation failed</span>
        </div>
    </div>
</div>

<style>
    .event-log {
        background-color: #16213e;
        border-radius: 12px;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
        max-height: 300px;
        overflow: hidden;
    }

    .log-header {
        background-color: rgba(0, 0, 0, 0.2);
        padding: 0.75rem;
        border-top-left-radius: 12px;
        border-top-right-radius: 12px;
    }

    .log-header h3 {
        margin: 0;
        color: #e94560;
        font-weight: 600;
    }

    .log-content {
        max-height: 250px;
        overflow-y: auto;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .log-item {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.5rem;
        border-radius: 6px;
    }

    .log-timestamp {
        color: rgba(255, 255, 255, 0.5);
        font-size: 0.8rem;
        min-width: 70px;
    }

    .log-message {
        flex-grow: 1;
        font-size: 0.9rem;
    }

    .log-item.success {
        background-color: rgba(76, 175, 80, 0.1);
        color: #4CAF50;
    }

    .log-item.warning {
        background-color: rgba(255, 152, 0, 0.1);
        color: #FF9800;
    }

    .log-item.error {
        background-color: rgba(244, 67, 54, 0.1);
        color: #F44336;
    }
</style>