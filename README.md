# SecServ
The simple secret service

# Protocol
Alice (secret service): keypair (apk, ask)
Bob (client): keypair (bpk, bsk)

KEY {id: string, keyid: string, nonce: b64, tag: signature}
// alice checks access rights of id for keyid
EKY {nonce1:b64, enc(key, nonce:64, bpk), tag: signature}
