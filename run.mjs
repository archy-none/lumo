import { readFileSync } from "node:fs";
import { lumo } from "./docs/runtime/node.mjs";

const args = process.argv.slice(2);
if (args.length === 0) {
    console.log("Lumo Runtime 2025 Node.js Edition");
    process.exit(1);
}

const filePath = args[0];
const source = readFileSync(filePath, "utf8");
try {
    const result = await lumo(source.toString());
    if (result !== undefined) console.log(result);
} catch (e) {
    console.log("Error!", e);
}
