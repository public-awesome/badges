import * as fs from "fs";
import * as path from "path";
import axios from "axios";
import yargs from "yargs/yargs";

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

/**
 * Return a list of delegators to a Terra validator specified by `valOperAddress`
 *
 * NOTE: this function assumes the number of validators is no more than 5000
 */
export async function fetchDelegators(valOperAddress: string) {
  interface Delegator {
    address: string;
    amount: string;
  }
  interface DelegatorsResponse {
    delegators: Delegator[];
  }
  const response: { data: DelegatorsResponse } = await axios.get(
    `https://fcd.terra.dev/v1/staking/validators/${valOperAddress}/delegators?page=1&limit=5000`
  );
  const delegators = response.data.delegators.map((delegator) => delegator.address);
  return delegators;
}

(async function () {
  const delegators = await fetchDelegators(argv.validator);
  fs.writeFileSync(path.join(__dirname, argv.output), JSON.stringify(delegators, null, 2));
})();
