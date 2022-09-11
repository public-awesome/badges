import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";

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

const RPC_URLS: { [key: string]: string } = {
  mainnet: "https://rpc.stargaze-apis.com:443",
  testnet: "https://rpc.elgafar-1.stargaze-apis.com:443",
  localhost: "http://localhost:26657",
};

export async function createSigningClient(network: string, wallet: DirectSecp256k1HdWallet) {
  const rpcUrl = RPC_URLS[network]!;
  console.log("using RPC URL", rpcUrl);

  const senderAddr = (await wallet.getAccounts())[0]!.address;
  console.log("using sender address:", senderAddr);

  process.stdout.write("creating signing client... ");
  const client = await SigningCosmWasmClient.connectWithSigner(rpcUrl, wallet, {
    gasPrice: GasPrice.fromString("0ustars"),
  });
  console.log("success!");

  return { senderAddr, client };
}
