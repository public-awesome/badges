// This scripts creates a trophy with `BySignature` minting rule.
//
// The trophy's metadata should be specified in JSON files whose path is to supplied to the script
// using `--metadata` flag.
//
// Usage:
// ts-node 3_create_with_key.ts --network {mainnet|testnet|localterra} --hub-address <string>
//   --metadata <string> --expiry <int> --max-supply <int>

import * as fs from "fs";
import secp256k1 from "secp256k1";
import yargs from "yargs/yargs";
import { MsgExecuteContract } from "@terra-money/terra.js";
import { randomBytes } from "crypto";
import { getLcd, getWallet, sendTransaction, bytesToBase64 } from "./helpers";
import { Metadata } from "./metadata";

function generatePrivateKey() {
  let sk: Uint8Array;
  do {
    sk = new Uint8Array(randomBytes(32));
  } while (!secp256k1.privateKeyVerify(sk));
  return sk;
}

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
    metadata: {
      type: "string",
      demandOption: true,
    },
    expiry: {
      type: "number",
      demandOption: false,
    },
    "max-supply": {
      type: "number",
      demandOption: false,
    },
  })
  .parseSync();

(async function main() {
  const terra = getLcd(argv.network);
  console.log("created LCD client for", argv.network);

  const creator = getWallet(terra);
  console.log("creator address:", creator.key.accAddress);

  const metadata: Metadata = JSON.parse(fs.readFileSync(argv["metadata"], "utf8"));
  console.log("metadata:", metadata);

  // generate keys
  process.stdout.write("generating keys... ");
  const sk = generatePrivateKey();
  const pk = secp256k1.publicKeyCreate(sk);
  console.log("done!", { sk: bytesToBase64(sk), pk: bytesToBase64(pk) });

  const msg = new MsgExecuteContract(creator.key.accAddress, argv["hub-address"], {
    create_trophy: {
      rule: {
        by_signature: bytesToBase64(pk),
      },
      metadata,
      expiry: argv.expiry ? { at_time: argv.expiry.toString() } : undefined,
      max_supply: argv["max-supply"],
    },
  });
  console.log("successfully created execute msg!");

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    process.stdout.write("creating trophy... ");
    const { txhash } = await sendTransaction(terra, creator, [msg]);
    console.log("success! txhash:", txhash);
    process.exit(0);
  });
})();
