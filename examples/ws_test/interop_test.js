// Node.js 24 native WebSocket interop test
const ws = new WebSocket("ws://127.0.0.1:8999");

let step = 0;

ws.addEventListener("open", () => {
    console.log("[Node.js Client] Connected to Flame WebSocket Server!");
});

ws.addEventListener("message", async (event) => {
    if (typeof event.data === "string") {
        console.log(`[Node.js Client] Received text: "${event.data}"`);
        if (event.data === "WELCOME") {
            step = 1;
            console.log("[Node.js Client] Step 1 passed (WELCOME). Sending text echo...");
            ws.send("Hello Flame");
        } else if (event.data === "ECHO: Hello Flame") {
            step = 2;
            console.log("[Node.js Client] Step 2 passed (Text Echo). Sending binary frame...");
            const binaryData = new Uint8Array([0xCA, 0xFE, 0xBA, 0xBE, 42]);
            ws.send(binaryData);
        } else if (event.data === "BROADCAST_OK") {
            step = 4;
            console.log("[Node.js Client] Step 4 passed (Broadcast). Closing connection...");
            ws.close(1000, "Done");
        }
    } else {
        // Binary blob/arrayBuffer received
        const buffer = await event.data.arrayBuffer();
        const bytes = new Uint8Array(buffer);
        console.log(`[Node.js Client] Received binary: length=${bytes.length}, bytes=[${Array.from(bytes).join(", ")}]`);
        const expected = [0xCA, 0xFE, 0xBA, 0xBE, 42];
        const match = bytes.length === expected.length && bytes.every((v, i) => v === expected[i]);
        if (match) {
            step = 3;
            console.log("[Node.js Client] Step 3 passed (Binary Echo). Sending broadcast test...");
            ws.send("BROADCAST_TEST");
        } else {
            console.error("[Node.js Client] Binary mismatch!");
            process.exit(1);
        }
    }
});

ws.addEventListener("close", (event) => {
    console.log(`[Node.js Client] Closed: code=${event.code}`);
    if (step === 4) {
        console.log("\n==========================================");
        console.log("  ALL INTEROP TESTS PASSED SUCCESSFULLY!  ");
        console.log("==========================================\n");
        process.exit(0);
    } else {
        console.error(`[Node.js Client] Closed prematurely at step ${step}`);
        process.exit(1);
    }
});

ws.addEventListener("error", (err) => {
    console.error("[Node.js Client] WebSocket error:", err);
    process.exit(1);
});

// Timeout watchdog
setTimeout(() => {
    console.error("[Node.js Client] Test timed out after 10s");
    process.exit(1);
}, 10000);
