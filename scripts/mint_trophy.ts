import { Wallet, MnemonicKey, MsgExecuteContract } from "@terra-money/terra.js";
import dotenv from "dotenv";
import { Network, getLcd, sendTransaction, fetchDelegators } from "./helpers";
import { Metadata } from "./metadata";

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

(async function main() {
  dotenv.config();
  const network = Network.Testnet;
  const terra = getLcd(network);
  const minter = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
  console.log("minter address:", minter.key.accAddress);

  const hubAddress = process.env.CONTRACT_ADDR;
  if (!hubAddress) {
    throw new Error("ERR: contract address not provided!");
  } else {
    console.log("contract address:", hubAddress);
  }

  const metadata: Metadata = {
    name: "Test",
    description: "This is a test",
    image: "Qmbywr7uHvupdbD6h9tx6vPu3zWY4iskPjYBhSVNreKWFy",
    animation_url: "QmVovSXPF4WVKeJJZ4hQ5xxWME1EybVw216JZuXma6tWmF",
  };
  console.log("metadata:", metadata);

  const owners = await fetchDelegators("terravaloper1d3fv2cjukt0e6lrzd8d857jatlkht7wcp85zar");
  console.log("number of eligible owners:", owners.length);

  const createMsg = new MsgExecuteContract(minter.key.accAddress, hubAddress, {
    create_trophy: {
      ...metadata,
    },
  });
  const mintMsgs = createMintMessages(minter, hubAddress, 1, owners);
  const msgs = [createMsg, ...mintMsgs];
  console.log("successfully created execute msgs!");
  // console.log(msgs);

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    process.stdout.write("minting trophy... ");
    const { txhash } = await sendTransaction(terra, minter, msgs);
    console.log("success! txhash:", txhash);
    process.exit(0);
  });
})();
