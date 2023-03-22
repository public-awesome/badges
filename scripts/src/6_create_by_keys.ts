import * as fs from "fs";
import * as path from "path";
import * as promptly from "promptly";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

import { coins } from "@cosmjs/proto-signing";
import { contracts } from "@steak-enjoyers/badges.js";

import * as helpers from "./helpers";
import * as keystore from "./keystore";

const KEYS_PER_MSG = 30;

helpers.suppressFetchAPIWarning();

const args = yargs(hideBin(process.argv))
  .option("hub-addr", {
    type: "string",
    describe: "address of the badge hub contract",
    demandOption: true,
  })
  .option("metadata", {
    type: "string",
    describe: "path to a JSON file containing the badge's metadata",
    demandOption: true,
  })
  .option("pubkeys", {
    type: "string",
    describe: "path to a text file containing hex-encoded pubkeys, one per line",
    demandOption: true,
  })
  .option("transferrable", {
    type: "boolean",
    describe: "whether the badge is transferrable",
    demandOption: false,
    default: true,
  })
  .option("expiry", {
    type: "number",
    describe: "the deadline only before which this badge can be minted, in unix timestamp",
    demandOption: false,
  })
  .option("max-supply", {
    type: "number",
    describe: "the maximum number of this badge that can be minted",
    demandOption: false,
  })
  .option("data-fee-amount", {
    type: "number",
    describe: "the amount of data fee to pay",
    demandOption: false,
    default: 0,
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
    default: path.resolve(__dirname, "./keys"),
  })
  .wrap(100)
  .parseSync();

(async function () {
  const password = await promptly.password("enter password to decrypt the key: ");
  const key = await keystore.load(args["key"], password, args["key-dir"]);
  const { senderAddr, client } = await helpers.createSigningClient(args["network"], key);

  const hubClient = new contracts.Hub.HubClient(client, senderAddr, args["hub-addr"]);

  const metadata = JSON.parse(fs.readFileSync(args["metadata"], "utf8"));
  console.log("loaded metadata!");

  const pubkeys = fs
    .readFileSync(args["pubkeys"], "utf8")
    .split("\n")
    .filter((owner) => owner.length > 0);
  console.log(`loaded ${pubkeys.length} pubkeys!`);

  const msg = {
    manager: senderAddr,
    metadata,
    transferrable: args["transferrable"],
    rule: "by_keys",
    expiry: args["expiry"],
    maxSupply: args["max-supply"],
  };
  console.log("msg:", JSON.stringify({ create_badge: msg }, null, 2));

  await promptly.confirm("proceed to create the badge? [y/N] ");

  console.log("broadcasting tx...");
  const res = await hubClient.createBadge(
    // @ts-expect-error - ??
    msg,
    "auto",
    "",
    args["data-fee-amount"] > 0 ? coins(args["data-fee-amount"], "ustars") : []
  );
  console.log(`success! txhash: ${res.transactionHash}`);

  // parse tx result to find out the badge id
  const event = res.logs
    .map((log) => log.events)
    .flat()
    .find(
      (event) =>
        event.attributes.findIndex(
          (attr) => attr.key === "action" && attr.value === "badges/hub/create_badge"
        ) > 0
    )!;
  const id = Number(event.attributes.find((attr) => attr.key === "id")!.value);
  console.log("id of the badge created is:", id);

  // batch keys into batches
  const batches: string[][] = [];
  for (let start = 0; start < pubkeys.length; start += KEYS_PER_MSG) {
    let end = start + KEYS_PER_MSG;
    end = end > pubkeys.length ? pubkeys.length : end;

    batches.push(pubkeys.slice(start, end));

    console.log(`composed batch ${batches.length} for pubkeys ${start + 1} - ${end}`);
  }

  for (const [idx, batch] of batches.entries()) {
    const msg = {
      id,
      keys: batch,
    };
    console.log("msg:", JSON.stringify({ add_keys: msg }, null, 2));

    const bytes = Buffer.from(JSON.stringify(batch), "utf8");
    // TODO: this should be a flag?
    const feePerByte = 10000;
    const dataFeeAmount = bytes.length * feePerByte;

    await promptly.confirm(`proceed to add keys? batch ${idx + 1} of ${batches.length} [y/N] `);

    console.log("broacasting tx...");
    const { transactionHash } = await hubClient.addKeys(
      msg,
      "auto",
      "",
      coins(dataFeeAmount, "ustars")
    );
    console.log("success! txhash:", transactionHash);
  }
})();
