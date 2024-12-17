export function connectToSocket(url, onMessage) {
    const socket = new WebSocket(url);
  
    socket.onopen = () => {
      console.log("Connected to WebSocket server");
    };
  
    socket.onmessage = (event) => {
      const data = JSON.parse(event.data);
      onMessage(data);
    };
  
    socket.onerror = (error) => {
      console.error("WebSocket error:", error);
    };
  
    socket.onclose = () => {
      console.log("WebSocket connection closed");
    };
  
    return socket;
  }
  