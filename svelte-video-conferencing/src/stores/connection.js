import { writable } from 'svelte/store';

const createConnectionStore = () => {
    const { subscribe, set, update } = writable({
        signalingStatus: 'Disconnected',
        peerConnectionStatus: 'Disconnected',
        isOfferReceived: false,
        connectionId: null,
        error: null,
        peerConnection: null,
        offer: null,
        answer: null,
        remotePublicKey: null,
        localPublicKey: null,
        iceCandidates: [],
        dataChannel: null
    });

    const createPeerConnection = () => {
        const peerConnection = new RTCPeerConnection({
            iceServers: [
                { urls: 'stun:stun.l.google.com:19302' }
            ]
        });

        peerConnection.onicecandidate = handleIceCandidate;
        peerConnection.onconnectionstatechange = handleConnectionStateChange;
        peerConnection.oniceconnectionstatechange = handleIceConnectionStateChange;
        peerConnection.ondatachannel = handleDataChannel;

        update(store => ({ ...store, peerConnection }));
        return peerConnection;
    }

    const handleIceCandidate = (e) => {
        if (e.candidate) {
            const candidate = {
                candidate: e.candidate.candidate,
                sdpMid: e.candidate.sdpMid,
                sdpMLineIndex: e.candidate.sdpMLineIndex
            };
            
            update(store => ({ 
                ...store, 
                iceCandidates: [...store.iceCandidates, candidate] 
            }));
        }
    }

    const handleConnectionStateChange = (e) => {
        update(store => {
            const state = store.peerConnection?.connectionState || 'unknown';
            console.log('Connection state changed:', state);
            return { ...store, peerConnectionStatus: state };
        });
    }

    // Handle ICE connection state changes
    const handleIceConnectionStateChange = (e) => {
        update(store => {
            const state = store.peerConnection?.iceConnectionState || 'unknown';
            console.log('ICE connection state changed:', state);
            return { ...store };
        });
    }

    // Handle incoming data channels
    const handleDataChannel = (e) => {
        const dataChannel = e.channel;
        setupDataChannel(dataChannel);
    }

    // Configure data channel
    const setupDataChannel = (dataChannel) => {
        dataChannel.onopen = () => console.log('Data channel opened');
        dataChannel.onclose = () => console.log('Data channel closed');
        dataChannel.onmessage = (e) => {
            console.log('Data channel message received:', e.data);
            // Handle incoming messages
        };

        update(store => ({ ...store, dataChannel }));
    }

    // Create and send an offer
    const createOffer = async() => {
        const peerConnection = update(store => {
            if (!store.peerConnection) {
                return createPeerConnection();
            }
            return store.peerConnection;
        });

        try {
            // Create data channel for communication
            const dataChannel = peerConnection.createDataChannel('secure-channel');
            setupDataChannel(dataChannel);

            // Create offer
            const offer = await peerConnection.createOffer({
                offerToReceiveAudio: true,
                offerToReceiveVideo: true
            });

            // Set local description
            await peerConnection.setLocalDescription(offer);

            // Store the offer
            update(store => ({ ...store, offer: offer }));
            
            return offer;
        } catch (error) {
            console.error('Error creating offer:', error);
            update(store => ({ ...store, error }));
            throw error;
        }
    }

    // Process received offer
    const processOffer = async(offer) => {
        try {
            const peerConnection = update(store => {
                if (!store.peerConnection) {
                    return createPeerConnection();
                }
                return store.peerConnection;
            });

            // Set remote description
            await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
            
            // Create answer
            const answer = await peerConnection.createAnswer();
            
            // Set local description
            await peerConnection.setLocalDescription(answer);
            
            // Store the answer
            update(store => ({ ...store, answer }));
            
            return answer;
        } catch (error) {
            console.error('Error processing offer:', error);
            update(store => ({ ...store, error }));
            throw error;
        }
    }

    // Process received answer
    async function processAnswer(answer) {
        try {
            update(store => {
                if (store.peerConnection && store.peerConnection.signalingState !== 'stable') {
                    store.peerConnection.setRemoteDescription(new RTCSessionDescription(answer));
                }
                return store;
            });
        } catch (error) {
            console.error('Error processing answer:', error);
            update(store => ({ ...store, error }));
            throw error;
        }
    }

    // Add ICE candidate
    async function addIceCandidate(candidate) {
        try {
            update(store => {
                if (store.peerConnection) {
                    store.peerConnection.addIceCandidate(new RTCIceCandidate(candidate));
                }
                return store;
            });
        } catch (error) {
            console.error('Error adding ICE candidate:', error);
            update(store => ({ ...store, error }));
            throw error;
        }
    }

    // Close connection
    function closeConnection() {
        update(store => {
            if (store.dataChannel) {
                store.dataChannel.close();
            }
            
            if (store.peerConnection) {
                store.peerConnection.close();
            }
            
            return {
                ...store,
                peerConnection: null,
                dataChannel: null,
                peerConnectionStatus: 'Disconnected'
            };
        });
    }

    return {
        subscribe,
        setSignalingStatus: (status) => update(store => ({ ...store, signalingStatus: status })),
        setPeerConnectionStatus: (status) => update(store => ({ ...store, peerConnectionStatus: status })),
        setOfferReceived: (isReceived) => update(store => ({ ...store, isOfferReceived: isReceived })),
        setConnectionId: (id) => update(store => ({ ...store, connectionId: id })),
        setError: (error) => update(store => ({ ...store, error })),
        setOffer: (offer) => update(store => ({ ...store, offer })),
        setAnswer: (answer) => update(store => ({ ...store, answer })),
        setRemotePublicKey: (remotePublicKey) => update(store => ({ ...store, remotePublicKey })),
        setLocalPublicKey: (localPublicKey) => update(store => ({ ...store, localPublicKey })),
        createOffer,
        processOffer,
        processAnswer,
        addIceCandidate,
        closeConnection,
        reset: () => set({
            signalingStatus: 'Disconnected',
            peerConnectionStatus: 'Disconnected',
            isOfferReceived: false,
            connectionId: null,
            error: null,
            peerConnection: null,
            offer: null,
            answer: null,
            remotePublicKey: null,
            localPublicKey: null,
            iceCandidates: [],
            dataChannel: null
        })
    };
}

export default createConnectionStore();