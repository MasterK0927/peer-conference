<script>
    import { connectionStore } from '../stores';
    import { webSocketStore } from '../utils/websocket';
    import { scale } from 'svelte/transition';

    function acceptOffer() {
        webSocketStore.sendMessage('accept-offer', {});
        connectionStore.setOfferReceived(false);
    }

    function rejectOffer() {
        webSocketStore.sendMessage('reject-offer', {});
        connectionStore.setOfferReceived(false);
    }
</script>

<div 
    class="offer-modal" 
    transition:scale
>
    <div class="modal-content">
        <div class="modal-header">
            <h2>Incoming Connection Request</h2>
        </div>
        <div class="modal-body">
            <p>A peer wants to establish a secure WebRTC connection.</p>
        </div>
        <div class="modal-footer">
            <button 
                class="btn accept" 
                on:click={acceptOffer}
            >
                Accept Connection
            </button>
            <button 
                class="btn reject" 
                on:click={rejectOffer}
            >
                Reject Connection
            </button>
        </div>
    </div>
</div>

<style>
    .offer-modal {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: rgba(0, 0, 0, 0.7);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .modal-content {
        background-color: #16213e;
        border-radius: 16px;
        max-width: 500px;
        width: 100%;
        padding: 2rem;
        box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
    }

    .modal-header {
        text-align: center;
        margin-bottom: 1.5rem;
    }

    .modal-header h2 {
        color: #e94560;
        margin: 0;
    }

    .modal-body {
        text-align: center;
        margin-bottom: 1.5rem;
        color: #00adb5;
    }

    .modal-footer {
        display: flex;
        justify-content: center;
        gap: 1rem;
    }

    .btn {
        padding: 0.75rem 1.5rem;
        border: none;
        border-radius: 8px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
    }

    .btn.accept {
        background-color: #0f3460;
        color: #4CAF50;
    }

    .btn.accept:hover {
        background-color: #4CAF50;
        color: white;
    }

    .btn.reject {
        background-color: #0f3460;
        color: #e94560;
    }

    .btn.reject:hover {
        background-color: #e94560;
        color: white;
    }
</style>