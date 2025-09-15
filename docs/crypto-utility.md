# Crypto Utility Documentation

## Overview

The Crypto Utility (`svelte-video-conferencing/src/utils/crypto.js`) provides ECDSA P-256 cryptographic operations for secure signaling in the peer-to-peer video conferencing application. It implements digital signatures to ensure authenticity and integrity of WebRTC signaling messages.

## Architecture

The utility leverages the Web Crypto API to provide:

- **Key Generation**: ECDSA P-256 key pair generation
- **Digital Signatures**: Message signing with private keys
- **Signature Verification**: Public key verification of signed messages
- **Format Conversion**: Conversion between Web Crypto and raw byte formats

## Core Functions

### Key Generation

#### `generateKeyPair()`
```javascript
export const generateKeyPair = async() => {
    const keyPair = await window.crypto.subtle.generateKey(
        {
            name: "ECDSA",
            namedCurve: "P-256",
        },
        true,
        ["sign","verify"]
    );

    // Export and convert to raw format...
    return {
        publicKey: Uint8Array(65), // Uncompressed EC point
        privateKey: CryptoKey       // Web Crypto private key
    };
}
```

**Purpose**: Generates ECDSA P-256 key pairs for cryptographic operations

**Algorithm Details**:
- **Curve**: P-256 (secp256r1) - NIST recommended elliptic curve
- **Key Usage**: Signing and verification operations
- **Extractability**: Keys are extractable for serialization

**Return Format**:
- **`publicKey`**: 65-byte Uint8Array in uncompressed EC point format
  - Byte 0: `0x04` (uncompressed point indicator)
  - Bytes 1-32: X coordinate (32 bytes)
  - Bytes 33-64: Y coordinate (32 bytes)
- **`privateKey`**: Web Crypto API CryptoKey object (non-extractable)

#### Public Key Format Conversion
```javascript
// Export in JWK format to get both x and y coordinates
const publicKeyJwk = await window.crypto.subtle.exportKey(
    "jwk",
    keyPair.publicKey
);

// Convert JWK components to raw bytes
const x = base64UrlToUint8Array(publicKeyJwk.x);
const y = base64UrlToUint8Array(publicKeyJwk.y);

// Create uncompressed EC point format: 0x04 (uncompressed) + x + y coordinates
const publicKey = new Uint8Array(65);
publicKey[0] = 0x04; // uncompressed point format
publicKey.set(x, 1);
publicKey.set(y, 33);
```

**Process**:
1. Export key in JSON Web Key (JWK) format
2. Extract X and Y coordinates from base64url
3. Convert coordinates to raw bytes
4. Assemble uncompressed EC point format

### Digital Signatures

#### `sign(data, privateKey)`
```javascript
export const sign = async(data, privateKey) => {
    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);

    const signatureBuffer = await window.crypto.subtle.sign(
        {
            name: "ECDSA",
            hash: {name: "SHA-256"},
        },
        privateKey,
        dataBuffer
    );

    // Convert ASN.1 DER to raw r||s format...
    return signature; // 64-byte Uint8Array
}
```

**Purpose**: Creates ECDSA signatures for message authenticity

**Parameters**:
- **`data`** (string): Message to be signed
- **`privateKey`** (CryptoKey): Private key for signing

**Process**:
1. Encode message as UTF-8 bytes
2. Create ECDSA signature using SHA-256 hash
3. Convert from ASN.1 DER format to raw r||s format
4. Return 64-byte signature (32 bytes r + 32 bytes s)

**Hash Function**: SHA-256 (recommended for P-256 curve)

#### ASN.1 DER to Raw Conversion
```javascript
// ECDSA P-256 signatures from Web Crypto are in ASN.1 DER format
// We need to extract the r and s values and concatenate them

const asn1Signature = new Uint8Array(signatureBuffer);

// Extract r and s components (each 32 bytes for P-256)
const r = new Uint8Array(32);
const s = new Uint8Array(32);

// Parse DER structure to extract r and s values
let rLength = asn1Signature[3];
let rStart = 4;
// ... parsing logic ...

// Concatenate r and s to form the 64-byte signature
const signature = new Uint8Array(64);
signature.set(r);
signature.set(s, 32);
```

**DER Format Parsing**:
- Handles variable-length DER encoding
- Manages potential zero-padding in components
- Normalizes to fixed 32-byte r and s values

### Signature Verification

#### `verify(data, signature, publicKey)`
```javascript
export const verify = async(data, signature, publicKey) => {
    try {
        const cryptoKey = await window.crypto.subtle.importKey(
            "raw",
            publicKey,
            {
                name: "ECDSA",
                namedCurve: "p-256",
            },
            false,
            ["verify"]
        );

        const encoder = new TextEncoder();
        const dataBuffer = encoder.encode(data);

        return await window.crypto.subtle.verify(
            {
                name: "ECDSA",
                hash: {name: "SHA-256"},
            },
            cryptoKey,
            signature,
            dataBuffer
        );
    } catch(err) {
        console.error(err);
        return false;
    }
}
```

**Purpose**: Verifies ECDSA signatures using public keys

**Parameters**:
- **`data`** (string): Original message that was signed
- **`signature`** (Uint8Array): 64-byte signature to verify
- **`publicKey`** (Uint8Array): 65-byte public key for verification

**Process**:
1. Import raw public key as CryptoKey
2. Encode message as UTF-8 bytes
3. Verify signature using Web Crypto API
4. Return boolean verification result

**Error Handling**: Returns `false` for any verification errors

## Helper Functions

### `base64UrlToUint8Array(base64Url)`
```javascript
function base64UrlToUint8Array(base64Url) {
    // Convert base64url to base64
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');

    // Add padding if needed
    const padding = '='.repeat((4 - base64.length % 4) % 4);
    const base64Padded = base64 + padding;

    // Decode base64 to string
    const rawData = atob(base64Padded);

    // Convert to Uint8Array
    const result = new Uint8Array(rawData.length);
    for (let i = 0; i < rawData.length; i++) {
        result[i] = rawData.charCodeAt(i);
    }
    return result;
}
```

**Purpose**: Converts base64url-encoded strings to Uint8Array

**Process**:
1. Replace base64url characters with base64 equivalents
2. Add appropriate padding
3. Decode base64 to binary string
4. Convert to byte array

## Security Properties

### Cryptographic Strength
- **Algorithm**: ECDSA with P-256 curve (equivalent to 128-bit security)
- **Hash Function**: SHA-256 (256-bit hash output)
- **Key Size**: 256-bit private keys, 512-bit public keys
- **Standards Compliance**: NIST FIPS 186-4 compliant

### Attack Resistance
- **Signature Forgery**: Computationally infeasible without private key
- **Key Recovery**: Private key cannot be derived from public key or signatures
- **Hash Collisions**: SHA-256 provides strong collision resistance
- **Side-Channel**: Web Crypto API provides timing attack protection

### Randomness Sources
- **Key Generation**: Uses secure random number generator from Web Crypto API
- **Signature Generation**: Includes secure random nonce generation
- **Browser Implementation**: Relies on browser's cryptographic implementation

## Data Formats

### Public Key Format (65 bytes)
```
Byte 0:    0x04 (uncompressed point indicator)
Bytes 1-32:  X coordinate (big-endian)
Bytes 33-64: Y coordinate (big-endian)
```

### Private Key Format
- Stored as Web Crypto API CryptoKey object
- Non-extractable for security
- 256-bit scalar value (not directly accessible)

### Signature Format (64 bytes)
```
Bytes 0-31:  r component (big-endian, zero-padded)
Bytes 32-63: s component (big-endian, zero-padded)
```

## Integration with WebSocket Utility

### Key Pair Initialization
```javascript
// In websocket.js
const keyPair = await generateKeyPair();
```

### Message Signing
```javascript
// In websocket.js - sendSecureOffer()
const offerJSON = JSON.stringify(offer);
const signature = await sign(offerJSON, keyPair.privateKey);

const securePayload = {
    offer: offer,
    public_key: Array.from(keyPair.publicKey),
    signature: Array.from(signature),
    nonce: Array.from(nonce)
}
```

### Signature Transmission
- Public keys transmitted as number arrays for JSON compatibility
- Signatures transmitted as number arrays
- Original data included for verification

## Error Handling

### Key Generation Errors
```javascript
try {
    const keyPair = await generateKeyPair();
} catch (error) {
    console.error('Key generation failed:', error);
    // Handle cryptographic failure
}
```

### Signing Errors
```javascript
try {
    const signature = await sign(data, privateKey);
} catch (error) {
    console.error('Signing failed:', error);
    // Handle signing failure
}
```

### Verification Errors
```javascript
const isValid = await verify(data, signature, publicKey);
if (!isValid) {
    console.warn('Signature verification failed');
    // Handle invalid signature
}
```

## Performance Considerations

### Key Generation Performance
- **Cost**: ~1-5ms on modern browsers
- **Caching**: Keys should be generated once and reused
- **Background Generation**: Can be performed during idle time

### Signing Performance
- **Cost**: ~1-3ms per signature
- **Batching**: Not applicable for individual messages
- **Optimization**: Minimal overhead for small messages

### Verification Performance
- **Cost**: ~2-4ms per verification
- **Public Key Import**: Additional overhead for key import
- **Caching**: Imported keys can be cached for multiple verifications

## Browser Compatibility

### Web Crypto API Support
- **Chrome**: Full support (version 37+)
- **Firefox**: Full support (version 34+)
- **Safari**: Full support (version 7+)
- **Edge**: Full support (version 79+)

### ECDSA P-256 Support
- **Universal**: Supported across all major browsers
- **Mobile**: Full support on mobile browsers
- **Fallbacks**: No fallback implementation (requires modern browser)

## Usage Examples

### Basic Key Generation and Signing
```javascript
import { generateKeyPair, sign, verify } from './crypto.js';

// Generate key pair
const keyPair = await generateKeyPair();

// Sign a message
const message = "Hello, secure world!";
const signature = await sign(message, keyPair.privateKey);

// Verify signature
const isValid = await verify(message, signature, keyPair.publicKey);
console.log('Signature valid:', isValid); // true
```

### WebRTC Offer Signing
```javascript
// Sign WebRTC offer for secure transmission
const offer = await peerConnection.createOffer();
const offerJSON = JSON.stringify(offer);
const signature = await sign(offerJSON, keyPair.privateKey);

// Prepare secure payload
const secureOffer = {
    offer: offer,
    publicKey: Array.from(keyPair.publicKey),
    signature: Array.from(signature),
    timestamp: Date.now()
};
```

### Signature Verification on Reception
```javascript
// Verify received signed offer
const receivedOffer = /* from network */;
const publicKeyBytes = new Uint8Array(receivedOffer.publicKey);
const signatureBytes = new Uint8Array(receivedOffer.signature);
const offerJSON = JSON.stringify(receivedOffer.offer);

const isValid = await verify(offerJSON, signatureBytes, publicKeyBytes);
if (isValid) {
    // Process valid offer
    await processOffer(receivedOffer.offer);
} else {
    console.error('Invalid offer signature - potential security threat');
}
```

## Security Best Practices

### Key Management
1. **Generate Fresh Keys**: Create new key pairs for each session
2. **Secure Storage**: Don't persist private keys beyond session
3. **Key Rotation**: Consider periodic key renewal for long sessions

### Signature Verification
1. **Always Verify**: Never process unsigned messages
2. **Timing Attacks**: Use constant-time comparison for critical applications
3. **Replay Protection**: Include timestamps or nonces in signed data

### Error Handling
1. **Fail Securely**: Reject invalid signatures completely
2. **Log Security Events**: Monitor for verification failures
3. **Rate Limiting**: Prevent signature verification DoS attacks

## Limitations and Considerations

### Current Limitations
1. **Browser Dependency**: Requires Web Crypto API support
2. **No Key Persistence**: Keys lost on page refresh
3. **Single Algorithm**: Only supports ECDSA P-256
4. **Format Conversion**: Manual ASN.1 DER parsing

### Production Considerations
1. **Key Escrow**: No key recovery mechanism
2. **Certificate Authority**: No PKI integration
3. **Key Distribution**: Relies on secure channels for public key exchange
4. **Forward Secrecy**: No perfect forward secrecy

## Future Improvements

### Enhanced Security
1. **Key Derivation**: HKDF-based key derivation from master secrets
2. **Perfect Forward Secrecy**: Ephemeral key exchange protocols
3. **Certificate Support**: X.509 certificate integration
4. **Multi-Algorithm**: Support for additional signature algorithms

### Performance Optimization
1. **WebAssembly**: High-performance crypto implementation
2. **Worker Threads**: Background cryptographic operations
3. **Key Caching**: Intelligent key and signature caching
4. **Batch Operations**: Batch signature verification

### Standards Compliance
1. **JOSE Integration**: JSON Web Signature (JWS) support
2. **WebRTC Identity**: Integration with WebRTC Identity framework
3. **FIDO2 Support**: Hardware security key integration
4. **Post-Quantum**: Preparation for post-quantum cryptography

## Testing Strategies

### Unit Testing
```javascript
// Test key generation
const keyPair = await generateKeyPair();
expect(keyPair.publicKey).toHaveLength(65);
expect(keyPair.publicKey[0]).toBe(0x04);

// Test signing and verification
const message = "test message";
const signature = await sign(message, keyPair.privateKey);
const isValid = await verify(message, signature, keyPair.publicKey);
expect(isValid).toBe(true);
```

### Integration Testing
- Cross-browser compatibility testing
- Performance benchmarking
- Memory usage analysis
- Error condition simulation

### Security Testing
- Invalid signature rejection
- Malformed key handling
- Timing attack resistance
- Cryptographic test vectors