import dotenv from "dotenv";
import yargs from "yargs/yargs";
import { MnemonicKey } from "@terra-money/terra.js";
import { Network, getLcd, storeCode, instantiateContract } from "./helpers";

const argv = yargs(process.argv)
  .options({
    network: {
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
  dotenv.config();
  const network = Network.Testnet;
  const terra = getLcd(network);
  const deployer = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
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
    console.log("hub code id:", hubCodeId);

    let nftCodeId: number;
    if (argv["nft-code-id"]) {
      nftCodeId = argv["nft-code-id"];
    } else {
      process.stdout.write("hub code id not provided! storing code... ");
      nftCodeId = await storeCode(terra, deployer, "../artifacts/trophy_nft.wasm");
    }
    console.log("nft code id:", nftCodeId);

    process.stdout.write("instantiating hub contract... ");
    const hubAddress = await instantiateContract(terra, deployer, hubCodeId, {
      nft_code_id: nftCodeId,
    });
    console.log("success! contract address:", hubAddress);

    process.exit(0);
  });
})();
