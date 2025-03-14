import { describe, expect, it } from "vitest";
import {
    EphemeralStore,
    EphemeralStoreWasm,
    EphemeralListener,
    setDebug,
    EphemeralStoreEvent,
} from "../bundler/index";

describe("EphemeralStore", () => {
    it("set and get", () => {
        const store = new EphemeralStoreWasm(30_000);
        store.set("key1", { foo: "bar" });
        expect(store.get("key1")).toEqual({ foo: "bar" });
        expect(store.getAllStates()).toEqual({ "key1": { foo: "bar" } });
    });

    it("sync", () => {
        const store = new EphemeralStoreWasm(30_000);
        store.set("key1", { foo: "bar" });
        let changed: EphemeralStoreEvent = { by: "local", added: [], updated: [], removed: [] };

        const storeB = new EphemeralStoreWasm(30_000);
        storeB.subscribe((e) => {
            changed = e;
        });
        storeB.apply(store.encode("key1"));

        expect(changed).toStrictEqual({ by: "import", added: ["key1"], updated: [], removed: [] });
        expect(storeB.get("key1")).toEqual({ foo: "bar" });
        expect(storeB.getAllStates()).toEqual({ "key1": { foo: "bar" } });
    });

    it("should remove outdated", async () => {
        setDebug();
        let outdated: string[] = [];
        const store = new EphemeralStoreWasm(5);
        store.subscribe((e) => {
            if (e.removed.length > 0) {
                outdated = e.removed;
            }
        })
        store.set("key1", { foo: "bar" });
        await new Promise((r) => setTimeout(r, 10));
        store.removeOutdated();
        expect(outdated).toEqual(["key1"]);
        expect(store.getAllStates()).toEqual({});
    });

    it("wrapped", async () => {
        const store = new EphemeralStore(10);
        let i = 0;
        const listener = ((e) => {
            if (i === 0) {
                expect(e).toStrictEqual({
                    by: "local",
                    removed: [],
                    updated: [],
                    added: ["key1"],
                });
            }
            if (i === 1) {
                expect(e).toStrictEqual({
                    by: "import",
                    removed: [],
                    updated: [],
                    added: ["key2"],
                });
            }
            if (i >= 2) {
                expect(e.by).toBe("timeout");
                for (const r of e.removed) {
                    expect(["key1", "key2"]).toContain(r);
                }
            }

            i += 1;
        }) as EphemeralListener;
        store.subscribe(listener);
        store.set("key1", "123");
        const b = new EphemeralStore(10);
        b.set("key2", "223");
        const bytes = b.encode("key2");
        store.apply(bytes);
        expect(store.getAllStates()).toEqual({ "key1": "123", "key2": "223" });
        await new Promise((r) => setTimeout(r, 20));
        expect(store.getAllStates()).toEqual({});
        expect(i).toBeGreaterThanOrEqual(3);
    });

    it("consistency", () => {
        const a = new EphemeralStoreWasm(10);
        const b = new EphemeralStoreWasm(10);
        a.set("key1", 0);
        const oldBytes = a.encode("key1");
        a.set("key1", 1);
        const newBytes = a.encode("key1");
        b.apply(newBytes);
        b.apply(oldBytes);
        expect(a.get("key1")).toBe(1);
        expect(b.get("key1")).toBe(1);
    });

    it("encode binary", () => {
        const a = new EphemeralStoreWasm(10);
        const b = new EphemeralStoreWasm(10);
        a.set("key1", {
            a: Uint8Array.from([1, 2, 3, 4]),
            b: Uint8Array.from([5, 6, 7, 8]),
        });
        const bytes = a.encodeAll();
        b.apply(bytes);
        expect(b.get("key1")).toEqual({
            a: Uint8Array.from([1, 2, 3, 4]),
            b: Uint8Array.from([5, 6, 7, 8]),
        });
    });
}); 