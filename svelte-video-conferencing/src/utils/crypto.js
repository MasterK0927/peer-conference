export const generateKeyPair = async() => {
    const keyPair = await window.crypto.subtle.generateKey(
        {
            name: "ECDSA",
            namedCurve: "P-256",
        },
        true,
        ["sign","verify"]
    );

    // Export in jwk format to get both x and y coordinates
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

    return {
        publicKey,
        privateKey: keyPair.privateKey
    };
}

// Helper function to convert base64url to Uint8Array
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

    // ECDSA P-256 signatures from Web Crypto are in ASN.1 DER format
    // We need to extract the r and s values and concatenate them
    
    // Get the raw signature bytes
    const asn1Signature = new Uint8Array(signatureBuffer);
    
    // Extract r and s components (each 32 bytes for P-256)
    // This is a simplified extraction - in production, use a proper ASN.1 parser
    const r = new Uint8Array(32);
    const s = new Uint8Array(32);
    
    // Assuming standard DER format, extract the components
    // Skip the headers and get to the actual values
    let rLength = asn1Signature[3];
    let rStart = 4;
    let rOffset = 0;
    
    // Handle potential padding for r component
    if (rLength > 32) {
        rStart += (rLength - 32);
        rLength = 32;
    } else if (rLength < 32) {
        rOffset = 32 - rLength;
    }
    
    r.set(asn1Signature.slice(rStart, rStart + rLength), rOffset);
    
    let sLength = asn1Signature[rStart + rLength + 1];
    let sStart = rStart + rLength + 2;
    let sOffset = 0;
    
    // Handle potential padding for s component
    if (sLength > 32) {
        sStart += (sLength - 32);
        sLength = 32;
    } else if (sLength < 32) {
        sOffset = 32 - sLength;
    }
    
    s.set(asn1Signature.slice(sStart, sStart + sLength), sOffset);
    
    // Concatenate r and s to form the 64-byte signature expected by the backend
    const signature = new Uint8Array(64);
    signature.set(r);
    signature.set(s, 32);
    
    return signature;
}

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