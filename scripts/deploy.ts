import * as fs from "fs";
import * as path from "path";
import {
  LCDClient,
  Wallet,
  MnemonicKey,
  MsgStoreCode,
  MsgInstantiateContract,
} from "@terra-money/terra.js";
import dotenv from "dotenv";
import { Network, getLcd, sendTransaction } from "./helpers";

export type ContractInfo = {
  name: string;
  symbol: string;
};

async function deploy(
  terra: LCDClient,
  deployer: Wallet,
  minter: Wallet,
  contractInfo: ContractInfo
) {
  process.stdout.write("storing code... ");
  const codePath = path.join(__dirname, "../artifacts/trophy_nft.wasm");
  const code = fs.readFileSync(codePath).toString("base64");
  const storeCodeMsg = new MsgStoreCode(deployer.key.accAddress, code);
  const storeCodeResult = await sendTransaction(terra, deployer, [storeCodeMsg]);
  const codeId = parseInt(storeCodeResult.logs[0].eventsByType.store_code.code_id[0]);
  console.log("success! code id:", codeId);

  process.stdout.write("instantiating contract... ");
  const initMsg = new MsgInstantiateContract(
    deployer.key.accAddress,
    minter.key.accAddress,
    codeId,
    {
      ...contractInfo,
      minter: minter.key.accAddress,
    }
  );
  const initResult = await sendTransaction(terra, deployer, [initMsg]);
  const contractAddress = initResult.logs[0].eventsByType.instantiate_contract.contract_address[0];
  console.log("success! contract address:", contractAddress);
}

(async function main() {
  dotenv.config();
  // const terra = getLcd(Network.Mainnet);
  const terra = getLcd(Network.Testnet);
  const deployer = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
  const minter = deployer;
  console.log("deployer address :", deployer.key.accAddress);
  console.log("minter address   :", deployer.key.accAddress);

  const contractInfo = {
    name: "cw721-batch-mint",
    symbol: "n/a",
  };

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    await deploy(terra, deployer, minter, contractInfo);
    process.exit(0);
  });
})();
