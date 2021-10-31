export function sleep(ms: number) {
  return new Promise((res) => setTimeout(res, ms));
}

export function dateStringToTimestamp(dateStr: string) {
  return Math.floor(Date.parse(dateStr) / 1000); // Date.parse returns milliseconds
}

export function encodeBase64(obj: object) {
  return Buffer.from(JSON.stringify(obj)).toString("base64");
}

export function decodeBase64(encodedStr: string) {
  return JSON.parse(Buffer.from(encodedStr, "base64").toString());
}
