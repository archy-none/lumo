import { readFileSync } from "node:fs";
import { createInterface } from "readline";
import { lumo } from "./docs/runtime/node.mjs";

const args = process.argv.slice(2);
if (args.length === 0) {
    let code = "";
    const rl = createInterface({
        input: process.stdin,
        output: process.stdout,
        prompt: "> ",
    });

    console.log("Lumo REPL");
    rl.prompt();

    rl.on("line", (input) => {
        if (input.trim() !== "")
            lumo(`${code};${input}`)
                .then((result) => {
                    code += `;${input}`;
                    if (result !== undefined) console.log(result);
                })
                .catch((e) => console.log("\u0007Error!", e))
                .then(() => rl.prompt());
        else rl.prompt();
    });

    rl.on("close", () => {
        console.log("Bye");
        process.exit(0);
    });
} else if (args.length === 2 && args[0] === "-e") {
    console.log(await lumo(args[1]));
} else {
    const filePath = args[0];
    const source = readFileSync(filePath, "utf8");
    try {
        const result = await lumo(source.toString());
        if (result !== undefined) console.log(result);
    } catch (e) {
        console.log("Error!", e);
    }
}
