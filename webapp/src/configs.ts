export enum Network {
  Mainnet = "mainnet",
  Testnet = "testnet",
  Localhost = "localhost",
}

export type NetworkConfig = {
  // chain info
  name: string;
  chainId: string;
  prefix: string;
  rpcUrl: string;
  // gas settings
  gas?: number;
  gasAdjustment: number;
  gasPrices: string;
  // contract addresses
  hub: string;
  nft: string;
  // a function that takes a txhash and returns a block explorer URL showing this tx
  getExplorerUrl(txhash: string): string;
};

export const NETWORK_CONFIGS: { [key in Network]: NetworkConfig } = {
  [Network.Mainnet]: {
    name: "mainnet",
    chainId: "stargaze-1",
    prefix: "stars",
    rpcUrl: "https://rpc.stargaze-apis.com:443",
    gas: undefined,
    gasAdjustment: 1.4,
    gasPrices: "0ustars",
    hub: "stars13unm9tgtwq683wplupjlgw39nghm7xva7tmu7m29tmpxxnkhpkcq4gf3p4",
    nft: "stars1z5qcmx9frn2y92cjy3k62gzylkezkphdwrx3675mvug3fd9l26fshdd85t",
    getExplorerUrl: (txhash: string) => `https://www.mintscan.io/stargaze/txs/${txhash}`,
  },
  [Network.Testnet]: {
    name: "testnet",
    chainId: "elgafar-1",
    prefix: "stars",
    rpcUrl: "https://rpc.elgafar-1.stargaze-apis.com:443",
    gas: undefined,
    gasAdjustment: 1.4,
    gasPrices: "0ustars",
    hub: "stars1dacun0xn7z73qzdcmq27q3xn6xuprg8e2ugj364784al2v27tklqynhuqa",
    nft: "stars1vlw4y54dyzt3zg7phj8yey9fg4zj49czknssngwmgrnwymyktztstalg7t",
    getExplorerUrl: (txhash: string) =>
      `https://stargaze-testnet-explorer.pages.dev/stargaze/tx/${txhash}`,
  },
  [Network.Localhost]: {
    name: "local",
    chainId: "stars-dev-1",
    prefix: "stars",
    rpcUrl: "http://localhost:26657",
    gas: undefined,
    gasAdjustment: 1.4,
    gasPrices: "0ustars",
    hub: "stars14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9srsl6sm",
    nft: "stars1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrq096cja",
    getExplorerUrl: (_txhash: string) => "", // there's no explorer for localhost
  },
};

/**
 * Since Stargaze has zero gas price, instead of having users connect their own wallets, we can
 * simply embed a few "public" wallets in the webapp, and submit txs from them.
 *
 * We use multiple accounts, and randomly choose one for submitting the tx. This is because if
 * multiple people are submitting txs in the same block, they may have account sequence mismatch
 * errors if using the same account. Having multiple account availble minimizes the likelihood of
 * this error.
 */
export const PUBLIC_ACCOUNTS = [
  "left physical cliff pumpkin chimney sock claim asset refuse rug neutral shrug wall obey fruit punch lunar battle harvest note merit bottom later garlic",
  "coast keen penalty old tape winner pepper squeeze replace behave abandon master stay sample practice excite bright school pioneer cheese scale law census miracle",
  "impulse embrace subject subway update unfair wool uniform reject weapon diesel north duty loan mother alarm shock agree lady piece spring toilet uniform disorder",
  "bulk oil three faint hood return apart stock attract nice unfair sphere emerge obey music tray anchor vague universe bag produce limit annual father",
  "doll bus impact chair fabric shallow impose cruise scorpion episode gallery forget ask main coyote when badge volume denial material group patient waste school",
  "dance brush sentence can apology decade sing venue meat outdoor credit achieve hobby word crawl render kind expose resemble person gas gym evil slab",
  "ride umbrella easy wink bamboo room unknown coconut flash effort chest scorpion pact shaft exile picnic employ ginger state road huge city reveal teach",
  "nest inner evil tell relief base burger coral gentle wood fix shoe poem board inch list unfair more roof cry candy auto wage nerve",
  "please enemy bleak burst recall rookie quick reduce approve ethics butter shield main expand garlic crumble parrot boy wheat key arena horn ordinary insect",
  "conduct garden certain wash timber neglect wash useless flame pitch hint seek sound ability fever ribbon add actor people fame unlock audit core wall",
];
