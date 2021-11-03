// This contract uploads latest binaries and migrates and contracts
//
// Usage:
// ts-node 6_migrate_contracts --network {mainnet|testnet|localterra} \
//   --hub-address <string> [--hub-code-id <int>] \
//   --nft-address <string> [--nft-code-id <int>]

import yargs from "yargs/yargs";
import { MsgMigrateContract } from "@terra-money/terra.js";
import { getLcd, getWallet, sendTransaction, storeCode } from "./helpers";

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
    "nft-address": {
      type: "string",
      demandOption: true,
    },
    "hub-code-id": {
      type: "number",
      demandOption: false,
    },
    "nft-code-id": {
      type: "number",
      demandOption: false,
    },
  })
  .parseSync();

(async function main() {
  const terra = getLcd(argv.network);
  console.log("created LCD client for", argv.network);

  const deployer = getWallet(terra);
  console.log("deployer address:", deployer.key.accAddress);

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    let hubCodeId: number;
    if (argv["hub-code-id"]) {
      hubCodeId = argv["hub-code-id"];
    } else {
      process.stdout.write("hub code id not provided! storing code... ");
      hubCodeId = await storeCode(terra, deployer, "../artifacts/trophy_hub.wasm");
    }
    console.log("success! hub code id:", hubCodeId);

    process.stdout.write("migrating hub contract... ");
    const hubTxResult = await sendTransaction(terra, deployer, [
      new MsgMigrateContract(deployer.key.accAddress, argv["hub-address"], hubCodeId, {}),
    ]);
    console.log("success! txhash:", hubTxResult.txhash);

    let nftCodeId: number;
    if (argv["nft-code-id"]) {
      nftCodeId = argv["nft-code-id"];
    } else {
      process.stdout.write("nft code id not provided! storing code... ");
      nftCodeId = await storeCode(terra, deployer, "../artifacts/trophy_nft.wasm");
    }
    console.log("success! nft code id:", nftCodeId);

    process.stdout.write("migrating nft contract... ");
    const nftTxResult = await sendTransaction(terra, deployer, [
      new MsgMigrateContract(deployer.key.accAddress, argv["nft-address"], nftCodeId, {}),
    ]);
    console.log("success! txhash:", nftTxResult.txhash);

    process.exit(0);
  });
})();
