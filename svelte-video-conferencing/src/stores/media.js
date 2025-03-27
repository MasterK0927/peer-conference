import { writable } from 'svelte/store';

function createMediaStore() {
    const { subscribe, update } = writable({
        localStream: null,
        remoteStream: null,
        devices: {
            audio: { muted: false, available: false },
            video: { muted: false, available: false },
            screen: { sharing: false }
        }
    });

    return {
        subscribe,
        setLocalStream: (stream) => update(store => ({ ...store, localStream: stream })),
        setRemoteStream: (stream) => update(store => ({ ...store, remoteStream: stream })),
        toggleAudio: () => update(store => ({
            ...store, 
            devices: { 
                ...store.devices, 
                audio: { 
                    ...store.devices.audio, 
                    muted: !store.devices.audio.muted 
                } 
            }
        })),
        toggleVideo: () => update(store => ({
            ...store, 
            devices: { 
                ...store.devices, 
                video: { 
                    ...store.devices.video, 
                    muted: !store.devices.video.muted 
                } 
            }
        })),
        toggleScreenShare: () => update(store => ({
            ...store, 
            devices: { 
                ...store.devices, 
                screen: { 
                    ...store.devices.screen, 
                    sharing: !store.devices.screen.sharing 
                } 
            }
        })),
        reset: () => set({
            localStream: null,
            remoteStream: null,
            devices: {
                audio: { muted: false, available: false },
                video: { muted: false, available: false },
                screen: { sharing: false }
            }
        })
    };
}

export default createMediaStore();