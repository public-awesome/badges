import * as fs from "fs";
import * as path from "path";
import * as crypto from "crypto";
import * as secp256k1 from "secp256k1";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

function randomKey() {
  let privKey: Buffer;
  do {
    privKey = crypto.randomBytes(32);
  } while (!secp256k1.privateKeyVerify(privKey));

  return {
    privKey: privKey.toString("hex"),
    pubKey: Buffer.from(secp256k1.publicKeyCreate(privKey)).toString("hex"),
  };
}

const args = yargs(hideBin(process.argv))
  .option("count", {
    type: "number",
    describe: "number of keys to generate",
    demandOption: true,
  })
  .option("out-dir", {
    type: "string",
    describe: "path to a directory where the generated keys are to be stored",
    demandOption: true,
  })
  .wrap(100)
  .parseSync();

if (fs.existsSync(args["out-dir"])) {
  throw `folder ${args["out-dir"]} already exists!`;
} else {
  fs.mkdirSync(args["out-dir"]);
}

const keys = [];
for (let i = 0; i < args["count"]; i++) {
  keys.push(randomKey());
  console.log(`generated key ${i + 1}/${args["count"]}`);
}

const pubKeys = keys.map((key) => key.pubKey);
const privKeys = keys.map((key) => key.privKey);

fs.writeFileSync(path.join(args["out-dir"], "keys.json"), JSON.stringify(keys, null, 2));
fs.writeFileSync(path.join(args["out-dir"], "pubkeys.txt"), pubKeys.join("\n"));
fs.writeFileSync(path.join(args["out-dir"], "privkeys.txt"), privKeys.join("\n"));

console.log("saved keys to file!");
