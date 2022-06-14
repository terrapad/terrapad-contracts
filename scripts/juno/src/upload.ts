import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import "./constants";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { GasPrice } from "@cosmjs/stargate";
import { storeCode } from "./util";

async function storeContract(
  client: SigningCosmWasmClient,
  wallet: DirectSecp256k1HdWallet,
  path: string
) {
  const codeId = await storeCode(client, wallet, path);
  console.log(path, codeId);
}

async function main() {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
    process.env.MNEMONIC || "",
    { prefix: "juno" }
  );

  const [deployer] = await wallet.getAccounts();
  console.log(deployer.address);

  const client = await SigningCosmWasmClient.connectWithSigner(
    process.env.MAIN_NETWORK || "",
    wallet,
    { gasPrice: GasPrice.fromString(process.env.GAS_PRICE || "0.001ujuno") }
  );

  await storeContract(client, wallet, "../../artifacts/presale.wasm");
  await storeContract(client, wallet, "../../artifacts/vesting.wasm");
  await storeContract(client, wallet, "../../artifacts/locking.wasm");
  await storeContract(client, wallet, "../../artifacts/cw20_base.wasm");
}

main().catch(console.error);
