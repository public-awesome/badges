import * as fs from "fs";
import * as path from "path";
import { stringToPath } from "@cosmjs/crypto";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

export const DEFAULT_KEY_DIR = path.join(__dirname, "../keys");

/**
 * Create a new signing key from mnemonic and save it to an encrypted file
 */
export async function create(
  name: string,
  mnemonic: string,
  password: string,
  coinType = 118,
  keyDir = DEFAULT_KEY_DIR
) {
  const filePath = path.join(keyDir, `${name}.json`);
  if (fs.existsSync(filePath)) {
    throw new Error(`file ${filePath} already exists!`);
  }

  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    // for this project we only work with Stargaze, so we hardcode the prefix as `stars` here
    prefix: "stars",
    hdPaths: [stringToPath(`m/44'/${coinType}'/0'/0/0`)],
  });

  const address = (await wallet.getAccounts())[0]!.address;
  const serialization = await wallet.serialize(password);

  fs.writeFileSync(filePath, serialization);

  return address;
}

/**
 * Load an existing signing key from file and encrypted it with the provided password
 */
export async function load(
  name: string,
  password: string,
  keyDir = DEFAULT_KEY_DIR
): Promise<DirectSecp256k1HdWallet> {
  const filePath = path.join(keyDir, `${name}.json`);
  if (!fs.existsSync(filePath)) {
    throw new Error(`file ${filePath} does not exist!`);
  }

  const serialization = fs.readFileSync(filePath, "utf8");

  return DirectSecp256k1HdWallet.deserialize(serialization, password);
}

export function remove(name: string, keyDir = DEFAULT_KEY_DIR) {
  const filePath = path.join(keyDir, `${name}.json`);
  if (!fs.existsSync(filePath)) {
    throw new Error(`file ${filePath} does not exist!`);
  }

  fs.unlinkSync(filePath);
}
