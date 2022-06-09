import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import "./constants";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { GasPrice } from "@cosmjs/stargate";
import { instantiateContract } from "./util";
import { cw20CodeId } from "./constants";

async function main() {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
    process.env.MNEMONIC || "",
    { prefix: "juno" }
  );

  const [deployer] = await wallet.getAccounts();

  const client = await SigningCosmWasmClient.connectWithSigner(
    process.env.MAIN_NETWORK || "localhost:26657",
    wallet,
    { gasPrice: GasPrice.fromString("0.1ujunox") }
  );

  const cw20 = await instantiateContract(client, wallet, wallet, cw20CodeId, {
    name: "Test Token",
    symbol: "TEST",
    decimals: 6,
    initial_balances: [
      {
        address: deployer.address,
        amount: "100000000000",
      },
    ],
  });
  console.log("cw20", cw20.contractAddress);
}

main().catch(console.error);
