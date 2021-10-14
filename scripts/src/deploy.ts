import * as fs from "fs";
import * as path from "path";
import {
  LCDClient,
  LocalTerra,
  Wallet,
  MsgStoreCode,
  MsgInstantiateContract,
  MsgExecuteContract,
} from "@terra-money/terra.js";
import { getLcd, getGasPrice, sendTransaction } from "./helpers";
import { Network, ContractInfo, BatchInfo } from "./types";

async function main(
  terra: LCDClient,
  deployer: Wallet,
  minter: Wallet,
  contractInfo: ContractInfo,
  batchInfo: BatchInfo,
  owners: string[]
) {
  process.stdout.write("storing code... ");
  const codePath = path.join(__dirname, "../../artifacts/trophy_nft.wasm");
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
  console.log("success! code id:", contractAddress);

  process.stdout.write("minting... ");
  const createBatchMsg = new MsgExecuteContract(minter.key.accAddress, contractAddress, {
    create_batch: {
      ...batchInfo,
    },
  });
  const mintMsg = new MsgExecuteContract(minter.key.accAddress, contractAddress, {
    mint: {
      batch_id: 1,
      owners,
    },
  });
  const executeResult = await sendTransaction(terra, minter, [createBatchMsg, mintMsg]);
  console.log("success! txhash", executeResult.txhash);
}

const terra = new LocalTerra();
const deployer = terra.wallets.test1;
const minter = terra.wallets.test2;

const contractInfo = {
  name: "Terra Trophies",
  symbol: "n/a",
};
const batchInfo = {
  name: "Test batch",
  description: "Test description",
  image: "There is no image",
};

const owners = [
  terra.wallets.test3.key.accAddress,
  terra.wallets.test4.key.accAddress,
  terra.wallets.test5.key.accAddress,
  terra.wallets.test6.key.accAddress,
  terra.wallets.test7.key.accAddress,
  terra.wallets.test8.key.accAddress,
  terra.wallets.test9.key.accAddress,
  terra.wallets.test10.key.accAddress,
];

// https://www.codegrepper.com/code-examples/javascript/repeat+array+n+times+javascript
function repeatArray<T>(arr: T[], repeats: number) {
  return Array.from({ length: repeats }, () => arr).flat();
}

// let's try creating a large number of NFTs in one transaction and see how much gas it would cost
// after some experimentation, it looks like you can create ~80 NFTs in one tx maximum
// I'm guessing the limiting factor is the size of the execute_msg
const ownersRepeated = repeatArray(owners, 10);

main(terra, deployer, minter, contractInfo, batchInfo, ownersRepeated);
