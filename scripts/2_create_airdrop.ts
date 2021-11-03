// This scripts creates a trophy with the `ByMinter` minting rule and airdrop trophy instances to a
// list of recipients in one transaction.
//
// The trophy's metadata and its recipients should be specified in JSON files and supplied to the
// script using `--metadata` and `--owners` flags, respectively.
//
// Since this intended for one-time airdrops, the trophy's `max_supply` is set to exactly the number
// of owners in the JSON file, so that no more trophy instances can be minted after the initial drop.
//
// Usage:
// ts-node 2_create_airdrop.ts --network {mainnet|testnet|localterra} --hub-address <string>
//   --metadata <string> --owners <string>

import * as fs from "fs";
import yargs from "yargs/yargs";
import { Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import { getLcd, getWallet, sendTransaction } from "./helpers";
import { Metadata } from "./metadata";

const MAX_OWNERS_PER_MSG = 50;

type ContractInfoResponse = {
  nft: string; // address of the NFT contract
  trophy_count: number; // number of existing trophies
};

function createMintMessages(
  creator: Wallet,
  hubAddress: string,
  trophyId: number,
  owners: string[]
) {
  let msgs: MsgExecuteContract[] = [];
  let start = 0;
  while (start < owners.length) {
    let end = start + MAX_OWNERS_PER_MSG;
    end = end <= owners.length ? end : owners.length;
    const msg = new MsgExecuteContract(creator.key.accAddress, hubAddress, {
      mint_by_minter: {
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
  const terra = getLcd(argv.network);
  console.log("created LCD client for", argv.network);

  const creator = getWallet(terra);
  console.log("creator address:", creator.key.accAddress);

  const metadata: Metadata = JSON.parse(fs.readFileSync(argv["metadata"], "utf8"));
  console.log("metadata:", metadata);

  const owners: string[] = JSON.parse(fs.readFileSync(argv["owners"], "utf8"));
  console.log("number of eligible owners:", owners.length);

  // query the current trophy count so that we know what's ID of this new trophy
  const response: ContractInfoResponse = await terra.wasm.contractQuery(argv["hub-address"], {
    contract_info: {},
  });
  const trophyId = response.trophy_count + 1;

  const createMsg = new MsgExecuteContract(creator.key.accAddress, argv["hub-address"], {
    create_trophy: {
      rule: {
        by_minter: creator.key.accAddress,
      },
      metadata,
      expiry: undefined,
      max_supply: owners.length,
    },
  });
  const mintMsgs = createMintMessages(creator, argv["hub-address"], trophyId, owners);
  const msgs = [createMsg, ...mintMsgs];
  console.log("successfully created execute msgs!");

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    process.stdout.write("creating airdrop... ");
    const { txhash } = await sendTransaction(terra, creator, msgs);
    console.log("success! txhash:", txhash);
    process.exit(0);
  });
})();
