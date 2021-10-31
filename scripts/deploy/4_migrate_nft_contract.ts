import dotenv from "dotenv";
import yargs from "yargs/yargs";
import { MnemonicKey, MsgMigrateContract } from "@terra-money/terra.js";
import { Network, getLcd, sendTransaction, storeCode } from "./helpers";

const argv = yargs(process.argv)
  .options({
    network: {
      type: "string",
      demandOption: true,
    },
    "nft-address": {
      type: "string",
      demandOption: true,
    },
    "code-id": {
      type: "number",
      demandOption: false,
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
  const deployer = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
  console.log("deployer address:", deployer.key.accAddress);

  const nftAddress = argv["nft-address"];
  console.log("nft address:", nftAddress);

  let codeId: number;
  if (argv["code-id"]) {
    codeId = argv["code-id"];
  } else {
    process.stdout.write("code id not provided! storing code... ");
    codeId = await storeCode(terra, deployer, "../../artifacts/trophy_nft.wasm");
  }
  console.log("code id:", codeId);

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    process.stdout.write("migrating nft contract... ");
    const { txhash } = await sendTransaction(terra, deployer, [
      new MsgMigrateContract(deployer.key.accAddress, nftAddress, codeId, {}),
    ]);
    console.log("success! txhash:", txhash);
    process.exit(0);
  });
})();
