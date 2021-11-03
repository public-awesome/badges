// This script claims a trophy with `BySignature` minting rule using its associated secret key.
//
// NOTE: Flag `--secret-key` refers to the secret key associated with the trophy, NOT the sender
// wallet's private key!
//
// Usage:
// ts-node 4_claim_with_key.ts --network {mainnet|testnet|localterra} --hub-address <string>
//   --trophy-id <int> --private-key <string>

import secp256k1 from "secp256k1";
import yargs from "yargs/yargs";
import { MsgExecuteContract } from "@terra-money/terra.js";
import { getLcd, getWallet, sendTransaction, bytesToBase64, base64ToBytes } from "./helpers";
import { SHA256 } from "jscrypto/SHA256";

const argv = yargs(process.argv)
  .options({
    network: {
      type: "string",
      demandOption: true,
    },
    "hub-address": {
      type: "string",
      demandOption: true,
    },
    "trophy-id": {
      type: "number",
      demandOption: true,
    },
    "secret-key": {
      type: "string",
      demandOption: true,
    },
  })
  .parseSync();

(async function main() {
  const terra = getLcd(argv.network);
  console.log("created LCD client for", argv.network);

  const user = getWallet(terra);
  console.log("user address:", user.key.accAddress);

  process.stdout.write("signing message... ");
  const msg = user.key.accAddress;
  const msgHash = Buffer.from(SHA256.hash(msg).toString(), "hex");
  const privKey = base64ToBytes(argv["secret-key"]);
  const { signature } = secp256k1.ecdsaSign(msgHash, privKey);
  console.log("success!");

  const executeMsg = new MsgExecuteContract(user.key.accAddress, argv["hub-address"], {
    mint_by_signature: {
      trophy_id: 3,
      signature: bytesToBase64(signature),
    },
  });
  console.log("successfully created execute msg!");

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    process.stdout.write("claiming trophy... ");
    const { txhash } = await sendTransaction(terra, user, [executeMsg]);
    console.log("success! txhash:", txhash);
    process.exit(0);
  });
})();
