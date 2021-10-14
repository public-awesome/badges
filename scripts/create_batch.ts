import { LCDClient, Wallet, MnemonicKey, MsgExecuteContract } from "@terra-money/terra.js";
import axios from "axios";
import dotenv from "dotenv";
import { Network, getLcd, sendTransaction } from "./helpers";

const MAX_OWNERS_PER_MSG = 50;

export type BatchInfo = {
  name: string;
  description: string;
  image: string;
};

async function createBatch(
  terra: LCDClient,
  minter: Wallet,
  contractAddress: string,
  batchInfo: BatchInfo
) {
  process.stdout.write("creating batch... ");
  const msg = new MsgExecuteContract(minter.key.accAddress, contractAddress, {
    create_batch: {
      ...batchInfo,
    },
  });
  const result = await sendTransaction(terra, minter, [msg]);
  const batchId = parseInt(result.logs[0].eventsByType.from_contract.batch_id[0]);
  console.log(`success! txhash=${result.txhash} batchId=${batchId}`);
  return batchId;
}

async function mint(
  terra: LCDClient,
  minter: Wallet,
  contractAddress: string,
  batchId: number,
  owners: string[]
) {
  console.log("preparing mint messages...");
  let msgs: MsgExecuteContract[] = [];
  let start = 0;
  while (start < owners.length) {
    let end = start + MAX_OWNERS_PER_MSG;
    end = end <= owners.length ? end : owners.length;
    const msg = new MsgExecuteContract(minter.key.accAddress, contractAddress, {
      mint: {
        batch_id: batchId,
        owners: owners.slice(start, end),
      },
    });
    msgs.push(msg);
    console.log(`msg ${msgs.length}: ${end - start} owners`);
    start = end;
  }

  process.stdout.write("minting... ");
  const result = await sendTransaction(terra, minter, msgs);
  console.log(`success! txhash=${result.txhash}`);
}

(async function main() {
  dotenv.config();
  // const terra = getLcd(Network.Mainnet);
  const terra = getLcd(Network.Testnet);
  const minter = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
  console.log("minter address  :", minter.key.accAddress);

  const contractAddress = process.env.CONTRACT_ADDR;
  if (!contractAddress) {
    throw new Error("ERR: contract address not provided!");
  } else {
    console.log("contract adress :", contractAddress);
  }

  const batchInfo = {
    name: "THANK YOU by larry_0x",
    description: "Thank you for delegating to my validator!",
    image: "",
  };

  interface Delegator {
    address: string;
    amount: string;
  }
  interface DelegatorsResponse {
    delegators: Delegator[];
  }
  const response: { data: DelegatorsResponse } = await axios.get(
    "https://fcd.terra.dev/v1/staking/validators/terravaloper1d3fv2cjukt0e6lrzd8d857jatlkht7wcp85zar/delegators?page=1&limit=5000"
  );
  const owners = response.data.delegators.map((delegator) => delegator.address);
  console.log("eligible owners:", owners);

  process.stdout.write("ready to execute; press any key to continue, CTRL+C to abort...");
  process.stdin.once("data", async function () {
    // const batchId = await createBatch(terra, minter, contractAddress, batchInfo);
    await mint(terra, minter, contractAddress, 1, owners);
    process.exit(0);
  });
})();
