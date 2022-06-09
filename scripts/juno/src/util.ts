/* eslint-disable @typescript-eslint/ban-types */
import { is } from "ramda";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import fs from "fs";

export const omitEmpty = (object: object): object =>
  Object.entries(object).reduce((acc, [key, value]) => {
    const next = is(Object, value) ? omitEmpty(value) : value;
    const valid = Number.isFinite(value) || value;
    return Object.assign({}, acc, valid && { [key]: next });
  }, {});

export const toBase64 = (object: object) => {
  try {
    return Buffer.from(JSON.stringify(omitEmpty(object))).toString("base64");
  } catch (error) {
    return "";
  }
};

export const fromBase64 = <T>(string: string): T => {
  try {
    return JSON.parse(Buffer.from(string, "base64").toString());
  } catch (error) {
    return {} as T;
  }
};

/**
 * @notice Upload contract code to LocalJuno. Return code ID.
 */
export async function storeCode(
  client: SigningCosmWasmClient,
  deployer: DirectSecp256k1HdWallet,
  filepath: string
): Promise<number> {
  const code = fs.readFileSync(filepath);
  const [account] = await deployer.getAccounts();

  const result = await client.upload(account.address, code, "auto");
  return result.codeId;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function instantiateContract(
  client: SigningCosmWasmClient,
  deployer: DirectSecp256k1HdWallet,
  admin: DirectSecp256k1HdWallet, // leave this emtpy then contract is not migratable
  codeId: number,
  instantiateMsg: Record<string, unknown>
) {
  const [account] = await deployer.getAccounts();
  const result = await client.instantiate(
    account.address,
    codeId,
    instantiateMsg,
    "instantiate",
    "auto"
  );
  return result;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function migrateContract(
  client: SigningCosmWasmClient,
  sender: DirectSecp256k1HdWallet,
  admin: DirectSecp256k1HdWallet,
  contract: string,
  new_code_id: number,
  migrateMsg: Record<string, unknown>
) {
  const [account] = await sender.getAccounts();
  const result = await client.migrate(
    account.address,
    contract,
    new_code_id,
    migrateMsg,
    "auto"
  );
  return result;
}
