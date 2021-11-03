import * as fs from "fs";
import dotenv from "dotenv";
import yargs from "yargs/yargs";
import { MnemonicKey, MsgExecuteContract } from "@terra-money/terra.js";
import { Network, getLcd, sendTransaction } from "./helpers";
import { Metadata } from "./types";

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
  })
  .parseSync();

(async function main() {
  dotenv.config();
  const network = Network.Testnet;
  const terra = getLcd(network);
  const creator = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
  console.log("creator address:", creator.key.accAddress);

  const metadata: Metadata = JSON.parse(fs.readFileSync(argv["metadata"], "utf8"));
  console.log("metadata:", metadata);

  const msg = new MsgExecuteContract(creator.key.accAddress, argv["hub-address"], {
    edit_trophy: {
      trophy_id: 1,
      metadata,
    },
  });
  console.log("msg:", JSON.stringify(msg.execute_msg, null, 2));

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    process.stdout.write("editing trophy... ");
    const { txhash } = await sendTransaction(terra, creator, [msg]);
    console.log("success! txhash:", txhash);
    process.exit(0);
  });
})();
