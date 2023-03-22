import * as path from "path";
import * as promptly from "promptly";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

import * as helpers from "./helpers";
import * as keystore from "./keystore";

helpers.suppressFetchAPIWarning();

const args = yargs(hideBin(process.argv))
  .option("hub-code-id", {
    type: "number",
    describe: "code id of the badges hub contract",
    demandOption: true,
  })
  .option("nft-code-id", {
    type: "number",
    describe: "code if of the badges nft contract",
    demandOption: true,
  })
  .option("network", {
    type: "string",
    describe: "the network where the codes are to be stored; must be mainnet|testnet|localhost",
    demandOption: false,
    default: "localhost",
  })
  .option("key", {
    type: "string",
    describe: "name of key to sign the txs",
    demandOption: false,
    default: "dev",
  })
  .option("key-dir", {
    type: "string",
    describe: "directories where the encrypted key files are located",
    demandOption: false,
    default: path.resolve(__dirname, "../keys"),
  })
  .wrap(100)
  .parseSync();

(async function () {
  const password = await promptly.password("enter password to decrypt the key: ");
  const key = await keystore.load(args["key"], password, args["key-dir"]);
  const { senderAddr, client } = await helpers.createSigningClient(args["network"], key);

  process.stdout.write("instantiating hub contract... ");
  const { contractAddress: hubAddr, transactionHash: hubTxHash } = await client.instantiate(
    senderAddr,
    args["hub-code-id"],
    {
      fee_per_byte: "200000", // 0.2 STARS per byte
    },
    "badge-hub",
    "auto",
    {
      admin: senderAddr,
    }
  );
  console.log(`success! contract address: ${hubAddr}, txhash: ${hubTxHash}`);

  process.stdout.write("instantiating nft contract... ");
  const { contractAddress: nftAddr, transactionHash: nftTxHash } = await client.instantiate(
    senderAddr,
    args["nft-code-id"],
    {
      hub: hubAddr,
      api_url: "https://api.badges.fun/metadata",
      collection_info: {
        creator: senderAddr,
        description: "Badges is an NFT protocol that allows anyone to create digital badges",
        image: "https://badges.fun/logo.png",
        external_link: "https://badges.fun",
        royalty_info: {
          payment_address: senderAddr,
          share: "0.05",
        },
      },
    },
    "badge-nft",
    "auto",
    {
      admin: senderAddr,
    }
  );
  console.log(`success! contract address: ${nftAddr}, txhash: ${nftTxHash}`);

  process.stdout.write("setting nft address at the hub contract... ");
  const { transactionHash } = await client.execute(
    senderAddr,
    hubAddr,
    {
      set_nft: {
        nft: nftAddr,
      },
    },
    "auto"
  );
  console.log(`success! txhash: ${transactionHash}`);
})();
