import { SHA256 } from "jscrypto/SHA256";
import { Word32Array } from "jscrypto";

export function sha256(data: Buffer): Buffer {
  return Buffer.from(SHA256.hash(new Word32Array(data)).toUint8Array());
}
