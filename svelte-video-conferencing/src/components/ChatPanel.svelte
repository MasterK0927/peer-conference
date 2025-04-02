<script>
    import { chatStore } from '../stores';
    import { webSocketStore } from '../utils/websocket';
    import { fade, slide } from 'svelte/transition';

    let message = '';

    function sendMessage() {
        if (message.trim()) {
            webSocketStore.sendMessage('chat', {
                text: message,
                timestamp: new Date().toLocaleTimeString()
            });
            
            chatStore.addMessage({
                text: message,
                sender: 'local',
                timestamp: new Date().toLocaleTimeString()
            });
            
            message = '';
        }
    }
</script>

<div class="chat-panel">
    <div class="chat-messages">
        {#each $chatStore.messages as msg, index (index)}
            <div 
                class="message" 
                class:local={msg.sender === 'local'}
                class:remote={msg.sender === 'remote'}
                transition:slide
            >
                <div class="message-content">
                    <span class="message-text">{msg.text}</span>
                    <span class="message-timestamp">{msg.timestamp}</span>
                </div>
            </div>
        {/each}
    </div>
    
    <div class="chat-input">
        <input 
            type="text" 
            bind:value={message}
            placeholder="Type your message..."
            on:keydown={(e) => e.key === 'Enter' && sendMessage()}
        />
        <button on:click={sendMessage}>
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="22" y1="2" x2="11" y2="13"></line>
                <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
            </svg>
        </button>
    </div>
</div>

<style>
    .chat-panel {
        background-color: #16213e;
        border-radius: 12px;
        display: flex;
        flex-direction: column;
        height: 400px;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
    }

    .chat-messages {
        flex-grow: 1;
        overflow-y: auto;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        scrollbar-width: thin;
        scrollbar-color: #0f3460 #1a1a2e;
    }

    .chat-messages::-webkit-scrollbar {
        width: 8px;
    }

    .chat-messages::-webkit-scrollbar-track {
        background: #1a1a2e;
    }

    .chat-messages::-webkit-scrollbar-thumb {
        background-color: #0f3460;
        border-radius: 20px;
    }

    .message {
        max-width: 80%;
        padding: 0.5rem 1rem;
        border-radius: 12px;
        position: relative;
        transition: transform 0.2s ease;
    }

    .message:hover {
        transform: scale(1.02);
    }

    .message.local {
        align-self: flex-end;
        background-color: #0f3460;
        color: #e94560;
    }

    .message.remote {
        align-self: flex-start;
        background-color: #0f3460;
        color: #00adb5;
    }

    .message-content {
        display: flex;
        flex-direction: column;
    }

    .message-text {
        font-size: 0.9rem;
    }

    .message-timestamp {
        font-size: 0.7rem;
        color: rgba(255, 255, 255, 0.5);
        margin-top: 0.25rem;
        text-align: right;
    }

    .chat-input {
        display: flex;
        padding: 1rem;
        background-color: rgba(0, 0, 0, 0.2);
        border-bottom-left-radius: 12px;
        border-bottom-right-radius: 12px;
    }

    .chat-input input {
        flex-grow: 1;
        background-color: #0f3460;
        border: none;
        padding: 0.75rem;
        color: white;
        border-radius: 8px 0 0 8px;
    }

    .chat-input button {
        background-color: #e94560;
        border: none;
        padding: 0.75rem;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 0 8px 8px 0;
        color: white;
        cursor: pointer;
        transition: background-color 0.3s ease;
    }

    .chat-input button:hover {
        background-color: #ff6b81;
    }
</style>