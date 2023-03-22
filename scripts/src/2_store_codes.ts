import * as fs from "fs";
import * as path from "path";
import * as promptly from "promptly";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

import * as helpers from "./helpers";
import * as keystore from "./keystore";

helpers.suppressFetchAPIWarning();

const args = yargs(hideBin(process.argv))
  .option("hub-wasm", {
    type: "string",
    describe: "path to the Badge Hub contract binary",
    demandOption: false,
    default: path.resolve(__dirname, "../artifacts/badge_hub.wasm"),
  })
  .option("nft-wasm", {
    type: "string",
    describe: "path to the Badge NFT contract binary",
    demandOption: false,
    default: path.resolve(__dirname, "../artifacts/badge_nft.wasm"),
  })
  .option("network", {
    type: "string",
    describe: "the network where the codes are to be stored; must be mainnet|testnet|localhost",
    demandOption: false,
    default: "localhost",
  })
  .option("key", {
    type: "string",
    describe: "name of key to sign the txs",
    demandOption: false,
    default: "dev",
  })
  .option("key-dir", {
    type: "string",
    describe: "directories where the encrypted key files are located",
    demandOption: false,
    default: path.resolve(__dirname, "./keys"),
  })
  .wrap(100)
  .parseSync();

(async function () {
  const password = await promptly.password("enter password to decrypt the key: ");
  const key = await keystore.load(args["key"], password, args["key-dir"]);
  const { senderAddr, client } = await helpers.createSigningClient(args["network"], key);

  process.stdout.write("uploading hub code... ");
  let { codeId, transactionHash } = await client.upload(
    senderAddr,
    fs.readFileSync(path.resolve(args["hub-wasm"])),
    "auto"
  );
  console.log(`success! code id: ${codeId}, txhash: ${transactionHash}`);

  process.stdout.write("uploading nft code... ");
  ({ codeId, transactionHash } = await client.upload(
    senderAddr,
    fs.readFileSync(path.resolve(args["nft-wasm"])),
    "auto"
  ));
  console.log(`success! code id: ${codeId}, txhash: ${transactionHash}`);
})();
