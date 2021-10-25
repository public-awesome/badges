import dotenv from "dotenv";
import { LCDClient, Wallet, MnemonicKey } from "@terra-money/terra.js";
import { Network, getLcd, storeCode, instantiateContract } from "./helpers";

export type ContractInfo = {
  name: string;
  symbol: string;
};

async function deploy(terra: LCDClient, deployer: Wallet) {
  process.stdout.write("storing nft code... ");
  const nftCodeId = await storeCode(terra, deployer, "../artifacts/trophy_nft.wasm");
  console.log("success! code id:", nftCodeId);

  process.stdout.write("storing hub code... ");
  const hubCodeId = await storeCode(terra, deployer, "../artifacts/trophy_hub.wasm");
  console.log("success! code id:", hubCodeId);

  process.stdout.write("instantiating hub contract... ");
  const hubAddress = await instantiateContract(terra, deployer, hubCodeId, {
    nft_code_id: nftCodeId,
  });
  console.log("success! contract address:", hubAddress);
}

(async function main() {
  dotenv.config();
  const network = Network.Testnet;
  const terra = getLcd(network);
  const deployer = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
  console.log("deployer address:", deployer.key.accAddress);

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    await deploy(terra, deployer);
    process.exit(0);
  });
})();
