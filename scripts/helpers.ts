import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { stringToPath } from "@cosmjs/crypto";
import { GasPrice } from "@cosmjs/stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import * as dotenv from "dotenv";

const RPC: { [key: string]: string } = {
  mainnet: "https://rpc.stargaze-apis.com:443",
  testnet: "https://rpc.elgafar-1.stargaze-apis.com:443",
  local: "http://localhost:26657",
};

// NoteJS gives the following annoying warning when creating the signing client:
//
// > ExperimentalWarning: The Fetch API is an experimental feature.
// > This feature could change at any time
// > Use `node --trace-warnings ...` to show where the warning was created
//
// This function disables this particular warning.
//
// Copied from: https://stackoverflow.com/a/73525885
export function suppressFetchAPIWarning() {
  const originalEmit = process.emit;
  // @ts-expect-error - TS complains about the return type of originalEmit.apply
  process.emit = function (name, data, ...args) {
    if (
      name === "warning" &&
      typeof data === "object" &&
      // @ts-expect-error - No error plz
      data.name === "ExperimentalWarning" &&
      // @ts-expect-error - No error plz
      data.message.includes("Fetch API")
    ) {
      return false;
    }
    return originalEmit.apply(process, args as unknown as Parameters<typeof process.emit>);
  };
}

export async function createSigningClient() {
  process.stdout.write("reading environment variables... ");
  dotenv.config();
  const network = process.env["NETWORK"]!;
  const mnemonic = process.env["MNEMONIC"]!;
  const coinType = process.env["BIP44_COIN_TYPE"] ?? "118";
  console.log("success!");

  const rpcUrl = RPC[network]!;
  console.log("using RPC URL", rpcUrl);

  process.stdout.write("creating signer... ");
  const signer = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: "stars",
    hdPaths: [stringToPath(`m/44'/${coinType}'/0'/0/0`)],
  });
  const signerAddr = (await signer.getAccounts())[0]!.address;
  console.log("success! signer address:", signerAddr);

  process.stdout.write("creating signing client... ");
  const client = await SigningCosmWasmClient.connectWithSigner(rpcUrl, signer, {
    gasPrice: GasPrice.fromString("0ustars"),
  });
  console.log("success!");

  return { signer, signerAddr, client };
}
