// Fetch a list of current delegators to a validator. Used in the airdrop this trophy:
// https://twitter.com/larry0x/status/1448749196018962437
//
// NOTE: This script assumes the number of validators is no more than 5000
//
// Usage:
// ts-node 7_fetch_delegators.ts --validator <string> [--output <string>]

import * as fs from "fs";
import * as path from "path";
import axios from "axios";
import yargs from "yargs/yargs";

interface Delegator {
  address: string;
  amount: string;
}

interface DelegatorsResponse {
  delegators: Delegator[];
}

const argv = yargs(process.argv)
  .options({
    validator: {
      alias: "v",
      type: "string",
      demandOption: true,
    },
    output: {
      alias: "o",
      type: "string",
      default: "../data/delegators.json",
      demandOption: false,
    },
  })
  .parseSync();

(async function () {
  const response: { data: DelegatorsResponse } = await axios.get(
    `https://fcd.terra.dev/v1/staking/validators/${argv.validator}/delegators?page=1&limit=5000`
  );
  const delegators = response.data.delegators.map((delegator) => delegator.address);
  fs.writeFileSync(path.join(__dirname, argv.output), JSON.stringify(delegators, null, 2));
})();
