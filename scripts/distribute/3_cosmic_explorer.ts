import * as fs from "fs";
import * as path from "path";
import { MongoClient } from "mongodb";
import { decodeBase64 } from "./helpers";
import { MsgExtended, IbcOutboundTransferData, ResultItem } from "./types";

const START_TIME = 1634786670; // 2021-10-21 03:24:30 UTC, the time the 1st IBC-related tx was confirmed
const DURATION = 30 * 24 * 60 * 60;
const END_TIME = START_TIME + DURATION;
// const SUPPLY = 500;
const SUPPLY = 999999999;

// if set DURATION = 7 days, there are 1748 eligible users
// if set DURATION = 5 days, there are 1177 eligible users
// if set DURATION = 3 days, there are 748 eligible users

function criteria(msg: MsgExtended) {
  // tx must have happened within the 1st week IBC is activated
  const isWithinTimeWindow = START_TIME <= msg.time && msg.time <= END_TIME;

  // msg must be of `/ibc.applications/transfer.v1.MsgTransfer` type
  const isOutboundIbcTransfer = msg.type === "/ibc.applications.transfer.v1.MsgTransfer";

  return isWithinTimeWindow && isOutboundIbcTransfer;
}

(async function () {
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

  process.stdout.write("creating cursor... ");
  const cursor = col.find(); // if we don't provide any argument, then it finds all docs in the collection
  console.log("done!");

  const total = await cursor.count();
  let result: ResultItem[] = [];
  for (let i = 1; i <= total; i++) {
    const msg = (await cursor.next()) as MsgExtended;
    if (criteria(msg)) {
      const data: IbcOutboundTransferData = decodeBase64(msg.data);
      const { sender, token } = data;
      console.log(`[${i}/${total}] sender=${sender} denom=${token.denom}`);

      const index = result.findIndex((item) => item.account === sender);
      if (index == -1) {
        result.push({
          account: sender,
          txs: [msg.txhash],
        });
      } else {
        result[index].txs.push(msg.txhash);
      }

      // stop if the total amount of airdrops hit supply cap
      // NOTE: this assumes tx in the db are ordered by time (earliest first)
      if (result.length >= SUPPLY) break;
    }
  }

  process.stdout.write("closing client... ");
  await client.close();
  console.log("done!");

  const numberOfTransfer = result.reduce((count, item) => {
    return count + item.txs.length;
  }, 0);
  console.log("number of transfers:", numberOfTransfer);
  console.log("number of unique senders:", result.length);

  const owners = result.map((item) => item.account);
  fs.writeFileSync(
    path.join(__dirname, "../data/cosmic_adventurer_owners.json"),
    JSON.stringify(owners, null, 2)
  );
  fs.writeFileSync(
    path.join(__dirname, "../data/cosmic_adventurer_txs_by_owner.json"),
    JSON.stringify(result, null, 2)
  );
})();
