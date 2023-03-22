import * as path from "path";
import * as promptly from "promptly";
import * as secp256k1 from "secp256k1";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

import { contracts } from "@steak-enjoyers/badges.js";

import { sha256 } from "./hash";
import * as helpers from "./helpers";
import * as keystore from "./keystore";

helpers.suppressFetchAPIWarning();

const args = yargs(hideBin(process.argv))
  .option("hub-addr", {
    type: "string",
    describe: "address of the badge hub contract",
    demandOption: true,
  })
  .option("id", {
    type: "number",
    describe: "badge id",
    demandOption: true,
  })
  .option("privkey", {
    type: "string",
    describe: "hex-encoded private key",
    demandOption: true,
  })
  .option("single-use", {
    type: "boolean",
    describe: "whether this is a single-use or a multi-use key",
    demandOption: true,
  })
  .option("owner", {
    type: "string",
    describe: "address to receive the badge",
    demandOption: true,
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

  const hubClient = new contracts.Hub.HubClient(client, senderAddr, args["hub-addr"]);

  const privKey = Buffer.from(args["privkey"], "hex");
  const pubKey = Buffer.from(secp256k1.publicKeyCreate(privKey, true));

  const message = `claim badge ${args["id"]} for user ${args["owner"]}`;
  const msgBytes = Buffer.from(message, "utf8");
  const msgHashBytes = sha256(msgBytes);
  const { signature } = secp256k1.ecdsaSign(msgHashBytes, privKey);
  console.log("signed message!");

  const promise = (() => {
    if (args["single-use"]) {
      const msg = {
        id: args["id"],
        owner: args["owner"],
        pubkey: pubKey.toString("hex"),
        signature: Buffer.from(signature).toString("hex"),
      };
      console.log("msg:", { mint_by_keys: msg });

      return hubClient.mintByKeys(msg, "auto", "", []);
    } else {
      const msg = {
        id: args["id"],
        owner: args["owner"],
        signature: Buffer.from(signature).toString("hex"),
      };
      console.log("msg:", { mint_by_key: msg });

      return hubClient.mintByKey(msg, "auto", "", []);
    }
  })();

  await promptly.confirm("proceed to mint the badge? [y/N] ");

  console.log("broadcasting tx...");
  const { transactionHash } = await promise;
  console.log(`success! txhash: ${transactionHash}`);
})();
