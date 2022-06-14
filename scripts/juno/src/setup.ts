import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import "./constants";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { GasPrice } from "@cosmjs/stargate";
import { instantiateContract } from "./util";
import { lockingCodeId, presaleCodeId, vestingCodeId } from "./constants";

async function setVestingWorker(
  client: SigningCosmWasmClient,
  wallet: DirectSecp256k1HdWallet,
  vesting: string,
  worker: string
) {
  const [account] = await wallet.getAccounts();
  await client.execute(
    account.address,
    vesting,
    {
      set_worker: {
        worker,
      },
    },
    "auto"
  );
}

async function setVestingStartTime(
  client: SigningCosmWasmClient,
  wallet: DirectSecp256k1HdWallet,
  vesting: string,
  startTime: number
) {
  const [account] = await wallet.getAccounts();
  await client.execute(
    account.address,
    vesting,
    {
      set_start_time: {
        new_start_time: startTime,
      },
    },
    "auto"
  );
}

async function deployContracts(
  client: SigningCosmWasmClient,
  wallet: DirectSecp256k1HdWallet
) {
  const [deployer] = await wallet.getAccounts();
  console.log(`Deployer is ${deployer.address}`);

  /******************************************************/
  // Params need to care
  const rewardToken =
    "juno1dvzgy9tz200pstzem0tf0w24nsfmjxhzmyf9xqqafjpx4thrjapq6v572r";
  const merkleRoot =
    "b1e5f5709783df6791e6327458961c81ac685cc89e87803e4197d91a964254ee";
  const totalRewardsAmount = 125000000000;
  const privateStart = Math.floor(
    new Date("2022-06-13T03:30:00.000Z").getTime() / 1000
  );
  const publicStart = Math.floor(
    new Date("2022-06-16T03:00:00.000Z").getTime() / 1000
  );
  const vestingStart = Math.floor(
    new Date("2022-06-16T04:00:00.000Z").getTime() / 1000
  );
  /******************************************************/

  const vestingParams = {
    reward_token: rewardToken,
    lock_period: 0,
    release_interval: 60,
    release_rate: 1,
    vesting_period: 1000000,
    initial_unlock: 10,
    distribution_amount: totalRewardsAmount,
  };

  const vesting = await instantiateContract(
    client,
    wallet,
    wallet,
    vestingCodeId,
    vestingParams
  );
  console.log("Vesting:", vesting.contractAddress);

  const presaleParams = {
    fund_denom: "ujunox",
    reward_token: rewardToken,
    vesting: vesting.contractAddress,
    whitelist_merkle_root: merkleRoot,
    exchange_rate: "800000", // ACCURACY: 100000000u128
    private_start_time: privateStart,
    public_start_time: publicStart,
    presale_period: 3600, // 1 hour
    total_rewards_amount: totalRewardsAmount.toString(),
  };
  const presale = await instantiateContract(
    client,
    wallet,
    wallet,
    presaleCodeId,
    presaleParams
  );
  console.log("Presale:", presale.contractAddress);

  await setVestingWorker(
    client,
    wallet,
    vesting.contractAddress,
    presale.contractAddress
  );
  console.log("Presale set to worker of vesting");

  await setVestingStartTime(
    client,
    wallet,
    vesting.contractAddress,
    vestingStart
  );
  console.log("Vesting start time is set");
}

async function deployLocking(
  client: SigningCosmWasmClient,
  wallet: DirectSecp256k1HdWallet
) {
  const [deployer] = await wallet.getAccounts();
  console.log(`Deployer is ${deployer.address}`);

  const lockingParams = {
    owner: deployer.address,
    token: "juno1uza23z5l4vzwxemswpxzq4zzln0qrk06n74dnv2zqg6fufz43ajq0klsc8",
    penalty_period: 86400 * 30,
    dead: deployer.address,
  };

  const locking = await instantiateContract(
    client,
    wallet,
    wallet,
    lockingCodeId,
    lockingParams
  );
  console.log("Locking:", locking.contractAddress);
}

async function main() {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
    process.env.MNEMONIC || "",
    { prefix: "juno" }
  );

  const client = await SigningCosmWasmClient.connectWithSigner(
    process.env.MAIN_NETWORK || "localhost:26657",
    wallet,
    { gasPrice: GasPrice.fromString(process.env.GAS_PRICE || "0.001ujuno") }
  );

  await deployContracts(client, wallet);
}

main().catch(console.error);
