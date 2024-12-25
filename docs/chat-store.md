# Chat Store Documentation

## Overview

The Chat Store (`svelte-video-conferencing/src/stores/chat.js`) manages real-time messaging functionality in the peer-to-peer video conferencing application. It provides reactive state management for chat messages, input handling, and unread message tracking.

## Architecture

The chat store implements a simple but effective reactive messaging system:

- **Message Storage**: Centralized message history management
- **Input Management**: Current message composition state
- **Notification System**: Unread message counting and tracking
- **State Reactivity**: Svelte store integration for real-time UI updates

## State Schema

### Core State Structure
```javascript
{
    messages: Array<Message>,
    currentMessage: string,
    unreadCount: number
}
```

### State Properties

#### Message History
- **`messages`**: Array containing all chat messages in chronological order
- Each message follows a consistent structure for display and processing

#### Input Management
- **`currentMessage`**: Current text being composed in the chat input field
- Tracks draft state for better user experience

#### Notification System
- **`unreadCount`**: Number of unread messages for notification badges
- Automatically managed based on message reception and user interactions

## Message Structure

### Standard Message Format
While the store doesn't enforce a specific message structure, typical messages include:

```javascript
{
    id: string,           // Unique message identifier
    text: string,         // Message content
    timestamp: number,    // Unix timestamp
    sender: string,       // Sender identifier
    type: 'text' | 'system' | 'file' // Message type
}
```

### Integration with WebSocket
Messages are received through the WebSocket utility and forwarded to the chat store:

```javascript
// In websocket.js - handleChatMessage()
function handleChatMessage(payload) {
    chatStore.addMessage(payload);
}
```

## Core Functions

### Message Management

#### `addMessage(message)`
```javascript
addMessage: (message) => update(store => ({
    ...store,
    messages: [...store.messages, message],
    unreadCount: store.unreadCount + 1
}))
```

**Purpose**: Adds a new message to the chat history
- Appends message to the end of the messages array
- Increments unread counter for notification purposes
- Triggers reactive updates for UI components

**Parameters**:
- `message`: Message object to be added to history

**Behavior**:
- Maintains chronological message order
- Automatically updates unread count
- Preserves immutability for Svelte reactivity

### Input Management

#### `setCurrentMessage(message)`
```javascript
setCurrentMessage: (message) => update(store => ({
    ...store,
    currentMessage: message
}))
```

**Purpose**: Updates the current message being composed
- Tracks user input in real-time
- Enables draft message persistence
- Supports UI input synchronization

**Use Cases**:
- Text input field binding
- Draft message auto-save
- Input validation feedback

#### `clearCurrentMessage()`
```javascript
clearCurrentMessage: () => update(store => ({
    ...store,
    currentMessage: ''
}))
```

**Purpose**: Clears the current message input
- Resets input field after message send
- Provides clean slate for new message composition

### Notification Management

#### `resetUnreadCount()`
```javascript
resetUnreadCount: () => update(store => ({
    ...store,
    unreadCount: 0
}))
```

**Purpose**: Resets unread message counter to zero
- Called when user views chat interface
- Clears notification badges
- Indicates user has seen new messages

**Typical Usage**:
- Chat window focus events
- Message list scrolling to bottom
- Explicit "mark as read" actions

### State Reset

#### `reset()`
```javascript
reset: () => set({
    messages: [],
    currentMessage: '',
    unreadCount: 0
})
```

**Purpose**: Resets all chat state to initial values
- Clears message history
- Resets input and notification state
- Used for session cleanup or user logout

## Integration Points

### With WebSocket Utility

#### Message Reception
```javascript
// In websocket.js
function handleChatMessage(payload) {
    chatStore.addMessage(payload);
}
```

Messages received through WebSocket are automatically added to the chat store.

#### Message Transmission
```javascript
// In UI component
import { webSocketStore } from '../utils/websocket.js';

function sendMessage(text) {
    const message = {
        type: 'chat',
        text: text,
        timestamp: Date.now(),
        sender: 'local'
    };

    webSocketStore.sendMessage('chat', message);
    chatStore.clearCurrentMessage();
}
```

### With UI Components

#### Reactive Subscriptions
```javascript
// In Svelte component
import { chatStore } from '../stores/chat.js';

let messages, currentMessage, unreadCount;

chatStore.subscribe(state => {
    messages = state.messages;
    currentMessage = state.currentMessage;
    unreadCount = state.unreadCount;
});
```

#### Input Binding
```svelte
<!-- In Svelte component -->
<script>
    import { chatStore } from '../stores/chat.js';

    let inputValue = '';

    // Bind to store
    chatStore.subscribe(state => {
        inputValue = state.currentMessage;
    });

    // Update store on input
    function handleInput(event) {
        chatStore.setCurrentMessage(event.target.value);
    }
</script>

<input
    bind:value={inputValue}
    on:input={handleInput}
    placeholder="Type a message..."
/>
```

### With Notification System

#### Unread Badge Display
```svelte
<!-- Notification badge -->
{#if unreadCount > 0}
    <span class="badge">{unreadCount}</span>
{/if}
```

#### Mark as Read Behavior
```javascript
// When chat becomes visible
function onChatVisible() {
    chatStore.resetUnreadCount();
}
```

## Usage Patterns

### Basic Message Display
```svelte
<script>
    import { chatStore } from '../stores/chat.js';

    let messages = [];

    chatStore.subscribe(state => {
        messages = state.messages;
    });
</script>

<div class="chat-messages">
    {#each messages as message (message.id)}
        <div class="message">
            <span class="sender">{message.sender}:</span>
            <span class="text">{message.text}</span>
            <span class="time">{new Date(message.timestamp).toLocaleTimeString()}</span>
        </div>
    {/each}
</div>
```

### Message Input Component
```svelte
<script>
    import { chatStore } from '../stores/chat.js';
    import { webSocketStore } from '../utils/websocket.js';

    let currentMessage = '';

    chatStore.subscribe(state => {
        currentMessage = state.currentMessage;
    });

    function handleSubmit() {
        if (currentMessage.trim()) {
            const message = {
                id: Date.now().toString(),
                text: currentMessage.trim(),
                timestamp: Date.now(),
                sender: 'me',
                type: 'text'
            };

            // Send through WebSocket
            webSocketStore.sendMessage('chat', message);

            // Add to local store (for immediate UI update)
            chatStore.addMessage(message);

            // Clear input
            chatStore.clearCurrentMessage();
        }
    }

    function handleKeyPress(event) {
        if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            handleSubmit();
        }
    }
</script>

<div class="message-input">
    <textarea
        bind:value={currentMessage}
        on:input={e => chatStore.setCurrentMessage(e.target.value)}
        on:keypress={handleKeyPress}
        placeholder="Type your message..."
        rows="3"
    ></textarea>
    <button on:click={handleSubmit} disabled={!currentMessage.trim()}>
        Send
    </button>
</div>
```

### Chat Window with Notifications
```svelte
<script>
    import { chatStore } from '../stores/chat.js';

    let messages = [];
    let unreadCount = 0;
    let isVisible = false;

    chatStore.subscribe(state => {
        messages = state.messages;
        unreadCount = state.unreadCount;
    });

    // Auto-scroll to bottom on new messages
    $: if (messages.length > 0 && isVisible) {
        setTimeout(() => {
            const chatContainer = document.getElementById('chat-messages');
            if (chatContainer) {
                chatContainer.scrollTop = chatContainer.scrollHeight;
            }
        }, 0);
    }

    function toggleVisibility() {
        isVisible = !isVisible;
        if (isVisible && unreadCount > 0) {
            chatStore.resetUnreadCount();
        }
    }
</script>

<div class="chat-window">
    <button class="chat-toggle" on:click={toggleVisibility}>
        Chat
        {#if unreadCount > 0}
            <span class="unread-badge">{unreadCount}</span>
        {/if}
    </button>

    {#if isVisible}
        <div class="chat-content">
            <div id="chat-messages" class="messages-container">
                {#each messages as message}
                    <!-- Message display -->
                {/each}
            </div>
            <!-- Message input component -->
        </div>
    {/if}
</div>
```

## Performance Considerations

### Memory Management
- **Message Limit**: Consider implementing message limit for long conversations
- **Message Cleanup**: Implement periodic cleanup for old messages
- **Efficient Updates**: Uses immutable updates for optimal Svelte reactivity

### Optimization Strategies
```javascript
// Example: Limited message history
const MAX_MESSAGES = 100;

addMessage: (message) => update(store => {
    const newMessages = [...store.messages, message];

    // Keep only recent messages
    const trimmedMessages = newMessages.length > MAX_MESSAGES
        ? newMessages.slice(-MAX_MESSAGES)
        : newMessages;

    return {
        ...store,
        messages: trimmedMessages,
        unreadCount: store.unreadCount + 1
    };
})
```

### Reactive Performance
- **Selective Subscriptions**: Subscribe only to needed state properties
- **Derived Stores**: Use derived stores for computed values
- **Component Optimization**: Implement message virtualization for large histories

## Error Handling

### Message Addition Errors
```javascript
addMessage: (message) => {
    try {
        if (!message || typeof message !== 'object') {
            console.warn('Invalid message format:', message);
            return;
        }

        update(store => ({
            ...store,
            messages: [...store.messages, message],
            unreadCount: store.unreadCount + 1
        }));
    } catch (error) {
        console.error('Error adding message:', error);
    }
}
```

### Input Validation
```javascript
setCurrentMessage: (message) => {
    try {
        if (typeof message !== 'string') {
            console.warn('Message must be a string:', message);
            return;
        }

        update(store => ({
            ...store,
            currentMessage: message.slice(0, MAX_MESSAGE_LENGTH)
        }));
    } catch (error) {
        console.error('Error setting current message:', error);
    }
}
```

## Security Considerations

### Message Sanitization
- **XSS Prevention**: Sanitize message content before display
- **Content Validation**: Validate message structure and content
- **Size Limits**: Implement message size limitations

### Privacy Features
- **Message Encryption**: Integration with crypto utilities for encrypted messaging
- **Ephemeral Messages**: Support for self-destructing messages
- **Message History Control**: User control over message retention

## Future Enhancements

### Advanced Features
1. **Message Threading**: Reply-to-message functionality
2. **Rich Content**: Support for links, images, and file attachments
3. **Message Reactions**: Emoji reactions to messages
4. **Message Search**: Full-text search through message history
5. **Message Editing**: Edit and delete sent messages

### Performance Improvements
1. **Virtual Scrolling**: Efficient rendering of large message histories
2. **Message Pagination**: Load messages in chunks
3. **Background Sync**: Offline message queuing and synchronization
4. **Compression**: Message compression for bandwidth optimization

### User Experience
1. **Typing Indicators**: Show when remote user is typing
2. **Read Receipts**: Message delivery and read confirmations
3. **Message Status**: Delivery status indicators
4. **Draft Persistence**: Save drafts across sessions

### Integration Features
1. **File Sharing**: Direct file transfer through data channels
2. **Voice Messages**: Audio message recording and playback
3. **Screen Annotations**: Collaborative screen annotation
4. **Message Translation**: Real-time message translation

## Testing Strategies

### Unit Testing
```javascript
// Test message addition
test('addMessage adds message and increments unread count', () => {
    const store = createChatStore();
    const message = { id: '1', text: 'Hello', timestamp: Date.now() };

    store.addMessage(message);

    store.subscribe(state => {
        expect(state.messages).toHaveLength(1);
        expect(state.messages[0]).toEqual(message);
        expect(state.unreadCount).toBe(1);
    });
});

// Test input management
test('setCurrentMessage updates current message', () => {
    const store = createChatStore();
    const text = 'Hello world';

    store.setCurrentMessage(text);

    store.subscribe(state => {
        expect(state.currentMessage).toBe(text);
    });
});
```

### Integration Testing
- WebSocket message integration
- UI component integration
- Notification system testing
- Cross-browser compatibility

### User Experience Testing
- Message ordering accuracy
- Real-time synchronization
- Notification reliability
- Input responsiveness