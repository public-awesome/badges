import * as fs from "fs";
import dotenv from "dotenv";
import yargs from "yargs/yargs";
import { Wallet, MnemonicKey, MsgExecuteContract } from "@terra-money/terra.js";
import { Network, getLcd, sendTransaction } from "./helpers";
import { Metadata } from "./types";

const MAX_OWNERS_PER_MSG = 50;

function createMintMessages(
  minter: Wallet,
  hubAddress: string,
  trophyId: number,
  owners: string[]
) {
  let msgs: MsgExecuteContract[] = [];
  let start = 0;
  while (start < owners.length) {
    let end = start + MAX_OWNERS_PER_MSG;
    end = end <= owners.length ? end : owners.length;
    const msg = new MsgExecuteContract(minter.key.accAddress, hubAddress, {
      mint_trophy: {
        trophy_id: trophyId,
        owners: owners.slice(start, end),
      },
    });
    msgs.push(msg);
    console.log(`msg ${msgs.length}: ${end - start} owners`);
    start = end;
  }
  return msgs;
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
    "trophy-id": {
      type: "number",
      demandOption: true,
    },
    metadata: {
      type: "string",
      demandOption: true,
    },
    owners: {
      type: "string",
      demandOption: true,
    },
  })
  .parseSync();

(async function main() {
  if (argv.network !== "mainnet" && argv.network !== "testnet") {
    throw new Error("invalid network! must be `mainnet` or `testnet`");
  }
  const terra = getLcd(argv.network === "mainnet" ? Network.Mainnet : Network.Testnet);

  dotenv.config();
  if (!process.env.MNEMONIC) {
    throw new Error("mnemonic phrase not provided!");
  }
  const minter = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
  console.log("minter address:", minter.key.accAddress);

  const metadata: Metadata = JSON.parse(fs.readFileSync(argv["metadata"], "utf8"));
  console.log("metadata:", metadata);

  const owners: string[] = JSON.parse(fs.readFileSync(argv["owners"], "utf8"));
  console.log("number of eligible owners:", owners.length);

  const createMsg = new MsgExecuteContract(minter.key.accAddress, argv["hub-address"], {
    create_trophy: metadata,
  });
  const mintMsgs = createMintMessages(minter, argv["hub-address"], argv["trophy-id"], owners);
  const msgs = [createMsg, ...mintMsgs];
  console.log("successfully created execute msgs!");

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    process.stdout.write("minting trophy... ");
    const { txhash } = await sendTransaction(terra, minter, msgs);
    console.log("success! txhash:", txhash);
    process.exit(0);
  });
})();
