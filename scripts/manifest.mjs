import { readFile, writeFile, mkdir, copyFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve, basename } from "node:path";
import { pathToFileURL } from "node:url";

export const INSTALLER_NAME = "RevenueOS-Windows-x64-Setup.exe";
export function createManifest({ version, tag, signature, publishedAt }) {
  if (!/^\d+\.\d+\.\d+$/.test(version) || tag !== `v${version}`) throw new Error("Release tag must match the stable application version");
  if (typeof signature !== "string" || !signature.trim()) throw new Error("Missing updater signature");
  const signatureText = Buffer.from(signature.trim(), "base64").toString("utf8");
  const comment = signatureText.match(/^trusted comment: ([^\r\n]*)/m)?.[1];
  const signedVersion = comment?.split("\t").find((field) => field.startsWith("version:"))?.slice("version:".length);
  if (signedVersion !== version) throw new Error("Signature metadata and release version must agree");
  if (!Number.isFinite(Date.parse(publishedAt))) throw new Error("Invalid publication date");
  return {
    version,
    notes: `Revenue OS ${version} para Windows. Atualização com assinatura verificada e preservação dos dados de acesso.`,
    pub_date: publishedAt,
    platforms: {
      "windows-x86_64": {
        signature: signature.trim(),
        url: `https://github.com/andrefonn/revenueos-desktop/releases/download/${tag}/${INSTALLER_NAME}`,
      },
    },
  };
}

async function packageRelease() {
  const [installerPath, tag] = process.argv.slice(2);
  if (!installerPath || !tag) throw new Error("Usage: node scripts/manifest.mjs installer.exe vX.Y.Z");
  const config = JSON.parse(await readFile("src-tauri/tauri.conf.json", "utf8"));
  const signature = await readFile(`${installerPath}.sig`, "utf8");
  const manifest = createManifest({ version: config.version, tag, signature, publishedAt: new Date().toISOString() });
  const installer = await readFile(installerPath);
  const hash = createHash("sha256").update(installer).digest("hex");
  await mkdir("dist", { recursive: true });
  await copyFile(installerPath, `dist/${INSTALLER_NAME}`);
  await writeFile(`dist/${INSTALLER_NAME}.sig`, signature);
  await writeFile("dist/latest.json", JSON.stringify(manifest, null, 2) + "\n");
  await writeFile("dist/SHA256SUMS.txt", `${hash}  ${INSTALLER_NAME}\n`);
  console.log(`Packaged ${basename(installerPath)} as ${INSTALLER_NAME} (${installer.length} bytes)`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await packageRelease();
