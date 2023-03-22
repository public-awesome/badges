import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import {
  BadgesResponse,
  BadgeResponse,
  ConfigResponse,
  KeyResponse,
  OwnerResponse,
} from "@steak-enjoyers/badges.js/types/codegen/Hub.types";
import create from "zustand";
import { Network, NetworkConfig, NETWORK_CONFIGS, PUBLIC_ACCOUNTS } from "./configs";

export type State = {
  networkConfig?: NetworkConfig;

  wallet?: DirectSecp256k1HdWallet;
  senderAddr?: string;
  wasmClient?: SigningCosmWasmClient;

  badgeCount?: number;
  badges: { [key: number]: BadgeResponse };

  init: () => Promise<void>;
  getAllBadges: () => Promise<BadgesResponse>;
  getBadge: (id: number) => Promise<BadgeResponse>;
  isKeyWhitelisted: (id: number, privkeyStr: string) => Promise<boolean>;
  isOwnerEligible: (id: number, owner: string) => Promise<boolean>;
};

export const useStore = create<State>((set) => ({
  badges: {},

  init: async () => {
    const network = process.env["NETWORK"] ?? Network.Testnet;
    const networkConfig = NETWORK_CONFIGS[network as Network];

    console.log("using network:", network);
    console.log("network config:", networkConfig);

    // pick a random public account to use
    const randomIndex = Math.floor(Math.random() * PUBLIC_ACCOUNTS.length);
    const mnemonic = PUBLIC_ACCOUNTS[randomIndex]!;

    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: networkConfig.prefix,
    });

    const senderAddr = (await wallet.getAccounts())[0]!.address;

    console.log("created wallet with address", senderAddr);

    const wasmClient = await SigningCosmWasmClient.connectWithSigner(networkConfig.rpcUrl, wallet, {
      prefix: networkConfig.prefix,
      gasPrice: GasPrice.fromString(networkConfig.gasPrices),
    });

    console.log("created wasm client with RPC URL", networkConfig.rpcUrl);

    const configRes: ConfigResponse = await wasmClient.queryContractSmart(networkConfig.hub, {
      config: {},
    });

    console.log("fetched badge count:", configRes.badge_count);

    return set({
      networkConfig,
      wallet,
      senderAddr,
      wasmClient,
      badgeCount: configRes.badge_count,
    });
  },

  getAllBadges: async function () {
    const badgesResponse = (await this.wasmClient!.queryContractSmart(this.networkConfig!.hub, {
      badges: {
        limit: 30, // currently there are less than 30 badges in total so this works
      },
    })) as BadgesResponse;

    for (const badge of badgesResponse.badges) {
      this.badges[badge.id] = badge;
    }

    set({ badges: this.badges });

    return { badges: Object.values(this.badges) };
  },

  getBadge: async function (id: number) {
    if (!(id in this.badges)) {
      this.badges[id] = await this.wasmClient!.queryContractSmart(this.networkConfig!.hub, {
        badge: {
          id,
        },
      });

      set({ badges: this.badges });
    }
    return this.badges[id]!;
  },

  // NOTE: unlike with getBadge, we don't cache the result in the store, because it the user submits
  // a successful mint tx, the eligibility of the key changes
  isKeyWhitelisted: async function (id: number, privkeyStr: string) {
    const keyRes: KeyResponse = await this.wasmClient!.queryContractSmart(this.networkConfig!.hub, {
      key: {
        id,
        pubkey: privkeyStr,
      },
    });
    return keyRes.whitelisted;
  },

  // NOTE: unlike with getBadge, we don't cache the result in the store, because it the user submits
  // a successful mint tx, the eligibility of the owner address changes
  isOwnerEligible: async function (id: number, owner: string) {
    const ownerRes: OwnerResponse = await this.wasmClient!.queryContractSmart(
      this.networkConfig!.hub,
      {
        owner: {
          id,
          user: owner,
        },
      }
    );
    return !ownerRes.claimed; // the address is eligible if it has NOT claimed
  },
}));
