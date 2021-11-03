// This scripts fetches all IBC-related transactions occurred between two block heights and store
// them in a MongoDB collection.
//
// Usage:
// mongod --dbpath <string>
// ts-node 8_fetch_ibc_msgs.ts [--start-height <int>] [--end-height <int>]

import axios from "axios";
import yargs from "yargs/yargs";
import { MongoClient } from "mongodb";
import { sleep, dateStringToTimestamp, encodeBase64 } from "./helpers";
import { Block, Msg, MsgExtended, Tx } from "./message";

const argv = yargs(process.argv)
  .options({
    "start-height": {
      alias: "s",
      type: "number",
      default: 4985676, // the block where the first ever IBC related tx was confirmed
      demandOption: false,
    },
    "end-height": {
      alias: "e",
      type: "number",
      demandOption: false, // default to the current height
    },
  })
  .parseSync();

async function getLatestBlockHeight() {
  const response = await axios.get<Block>("https://lcd.terra.dev/blocks/latest");
  const height = response.data.block.header.height;
  return parseInt(height);
}

async function _getTxsAtHeight(height: number) {
  const response = await axios.get<Tx[]>(`https://lcd.terra.dev/index/tx/by_height/${height}`);
  return response.data;
}

async function getTxsAtHeight(height: number) {
  // we might hit LCD's rate limit at some times. in this case, we sleep for 1 minute then retry
  // once. if still fails, we abort
  try {
    return await _getTxsAtHeight(height);
  } catch {
    process.stdout.write("query failed! retry in 1 min... ");
    await sleep(60000);
    return await _getTxsAtHeight(height);
  }
}

function compileMsg(tx: Tx, msg: Msg): MsgExtended {
  const { "@type": omit, ...data } = msg; // `data` is all the other properties in the object except for `@type`
  return {
    height: parseInt(tx.height),
    time: dateStringToTimestamp(tx.timestamp),
    txhash: tx.txhash,
    type: msg["@type"],
    data: encodeBase64(data), // we encode `data` into base64 format when storing in DB
  };
}

function compileMsgs(txs: Tx[]) {
  let msgs: MsgExtended[] = [];
  for (let tx of txs) {
    // we only include messages related to IBC
    const newMsgs = tx.tx.body.messages
      .map((msg) => compileMsg(tx, msg))
      .filter((msg) => msg.type.includes("ibc"));
    msgs = msgs.concat(newMsgs);
  }
  return msgs;
}

(async function () {
  const START_HEIGHT = argv["start-height"];
  // if `end-height` parameter isn't provided, we use the latest block
  const END_HEIGHT = argv["end-height"] ? argv["end-height"] : await getLatestBlockHeight();
  console.log("start height :", START_HEIGHT);
  console.log("end height   :", END_HEIGHT);

  process.stdout.write("creating mongodb client instance... ");
  const client = new MongoClient("mongodb://localhost:27017");
  console.log("done!");

  process.stdout.write("connecting client... ");
  await client.connect();
  console.log("done!");

  process.stdout.write("connecting db... ");
  const db = client.db("TerraTrophiesDistribution");
  console.log("done!");

  process.stdout.write("connecting collection... ");
  const col = db.collection("IbcMsgs");
  console.log("done!");

  try {
    const total = END_HEIGHT - START_HEIGHT + 1;
    for (let i = 0; i < total; i++) {
      const height = START_HEIGHT + i;
      process.stdout.write(`[${i}/${total}] (${((100 * i) / total).toFixed(2)}%) `);

      process.stdout.write(`height=${height} `);
      const txs = await getTxsAtHeight(height);
      const msgs = compileMsgs(txs);
      process.stdout.write(`msgs=${msgs.length} `);

      if (msgs.length > 0) {
        process.stdout.write("inserting... ");
        await col.insertMany(msgs);
        console.log("done!");
      } else {
        console.log("");
      }
    }
  } catch (err) {
    console.log(err);
  } finally {
    process.stdout.write("closing client... ");
    await client.close();
    console.log("done!");
  }
})();
