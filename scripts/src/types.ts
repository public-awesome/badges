export enum Network {
  Mainnet,
  Testnet,
  LocalTerra,
}

export type ContractInfo = {
  name: string;
  symbol: string;
};

export type BatchInfo = {
  name: string;
  description: string;
  image: string;
};
