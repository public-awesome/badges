// below are return types when querying LCD
export interface Block {
  block: {
    header: {
      height: string;
      time: string;
    };
  };
}

export interface Tx {
  height: string;
  timestamp: string;
  txhash: string;
  tx: {
    body: {
      messages: Msg[];
    };
  };
}

export interface Msg {
  "@type": string;
  [key: string]: string | object;
}

export interface IbcOutboundTransferData {
  token: {
    denom: string;
    amount: string;
  };
  sender: string;
  receiver: string;
}

export interface IbcInboundTransferData {
  packet: {
    data: string; // packet data is base64-encoded; should decode to `IbcPacketData`
  };
}

export interface IbcPacketData {
  denom: string;
  amount: string;
  sender: string;
  receiver: string;
}

// we store msgs in this format to the database
export type MsgExtended = {
  height: number;
  time: number;
  txhash: string;
  type: string;
  data: string; // we encode message data in base64 format when storing in DB
};

// we organize results (which accounts are eligible for NFT drops and the txs the make them eligible)
// in this type
export type ResultItem = {
  account: string;
  txs: string[];
};
