import dotenv from "dotenv";
import yargs from "yargs/yargs";
import { Wallet, MnemonicKey, MsgExecuteContract } from "@terra-money/terra.js";
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
  })
  .parseSync();

(async function main() {
  if (argv.network !== "mainnet" && argv.network !== "testnet") {
    throw new Error("invalid network! must be `mainnet` or `testnet`");
  }
  const terra = getLcd(argv.network === "mainnet" ? Network.Mainnet : Network.Testnet);

  dotenv.config();
  const minter = terra.wallet(new MnemonicKey({ mnemonic: process.env.MNEMONIC }));
  console.log("minter address:", minter.key.accAddress);

  const hubAddress = argv["hub-address"];
  console.log("hub address:", hubAddress);

  const metadata: Metadata = {
    name: "Test",
    description: "This is a test",
    image: "ipfs://Qmbywr7uHvupdbD6h9tx6vPu3zWY4iskPjYBhSVNreKWFy",
    animation_url: "ipfs://QmVovSXPF4WVKeJJZ4hQ5xxWME1EybVw216JZuXma6tWmF",
  };
  console.log("metadata:", metadata);

  // option 1. drop NFT to all delegators of a validator. this was used in my "thank you" NFT drop
  // const owners = await fetchDelegators("terravaloper1d3fv2cjukt0e6lrzd8d857jatlkht7wcp85zar");
  // option 2. drop to 10 random addresses, for testing. these are the 10 test accounts in LocalTerra
  const owners = [
    "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
    "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp",
    "terra1757tkx08n0cqrw7p86ny9lnxsqeth0wgp0em95",
    "terra199vw7724lzkwz6lf2hsx04lrxfkz09tg8dlp6r",
    "terra18wlvftxzj6zt0xugy2lr9nxzu402690ltaf4ss",
    "terra1e8ryd9ezefuucd4mje33zdms9m2s90m57878v9",
    "terra17tv2hvwpg0ukqgd2y5ct2w54fyan7z0zxrm2f9",
    "terra1lkccuqgj6sjwjn8gsa9xlklqv4pmrqg9dx2fxc",
    "terra1333veey879eeqcff8j3gfcgwt8cfrg9mq20v6f",
    "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n",
  ];
  console.log("number of eligible owners:", owners.length);

  const createMsg = new MsgExecuteContract(minter.key.accAddress, hubAddress, {
    create_trophy: {
      ...metadata,
    },
  });
  const mintMsgs = createMintMessages(minter, hubAddress, 1, owners);
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
