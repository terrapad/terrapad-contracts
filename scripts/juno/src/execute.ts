import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import "./constants";

async function withdrawFunds(
  client: SigningCosmWasmClient,
  wallet: DirectSecp256k1HdWallet,
  presale: string,
  receiver: string
) {
  const [account] = await wallet.getAccounts();
  await client.execute(
    account.address,
    presale,
    {
      withdraw_funds: {
        receiver,
      },
    },
    "auto"
  );
  await client.execute(
    account.address,
    presale,
    {
      withdraw_unsold_token: {
        receiver,
      },
    },
    "auto"
  );
}

async function setMerkleRoot(
  client: SigningCosmWasmClient,
  wallet: DirectSecp256k1HdWallet,
  presale: string,
  root: string
) {
  const [account] = await wallet.getAccounts();
  await client.execute(
    account.address,
    presale,
    {
      set_merkle_root: {
        merkle_root: root,
      },
    },
    "auto"
  );
}

async function main() {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
    process.env.MNEMONIC || "",
    { prefix: "osmo" }
  );

  const client = await SigningCosmWasmClient.connectWithSigner(
    process.env.MAIN_NETWORK || "localhost:26657",
    wallet,
    { gasPrice: GasPrice.fromString("0.025uosmo") }
  );
}

main().catch(console.error);
